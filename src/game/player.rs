use crate::packets::play::chunk::Chunk;
use crate::packets::play::held_item_slot::HeldItemSlot;
use crate::packets::play::join_game::{JoinGame, GameMode};
use crate::packets::play::{player_position::OutPlayerPositionLook, slot::{Slot, Window}, chat_message};
use crate::packets::{Packet, ServerDescription};

use crate::fsm::State;
use crate::types::{self, VarInt, LengthVec, BoolOption, EntityPosition};
use anyhow::Result;
use futures::prelude::*;
use piper::{Arc, Lock, LockGuard};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::packets::play::block::Block;
use futures_timer::Delay;
use crate::packets::play::chat_message::{OutChatMessage, InChatMessage};
use crate::game::world::World;
use crate::stream::ReadExtension;
use uuid::Uuid;
use futures::AsyncWriteExt;
use std::cmp::min;
use crate::packets::play::player_position::{InPlayerPosition, InPlayerPositionRotation, InPlayerRotation};
use crate::packets::play::entity_position::{OutPosition, OutPositionRotation, OutRotation, OutEntityHeadLook};
use std::cell::{RefCell, Cell};
use std::str::FromStr;
use crate::types::chat::Chat;

/// here we use the Arc to get interior mutability
pub struct Player {
    read_stream: Lock<Box<dyn AsyncRead + Send + Sync + Unpin>>,
    write_stream: Lock<Box<dyn AsyncWrite + Send + Sync + Unpin>>,
    world: &'static World,
    id: types::VarInt,
    info: Info,
    position: Lock<EntityPosition>,
}

impl Player {
    pub async fn new(
        mut reader: impl AsyncRead + Send + Sync + Unpin + 'static,
        mut writer: impl AsyncWrite + Send + Sync + Unpin + 'static,
        server_description: ServerDescription,
        world: &'static World
    ) -> Result<Option<Self>> {
        let state = State::new(server_description);
        match state.next(&mut reader, &mut writer).await? {
            State::Handshake(_) | State::Finished => panic!("This should never happens"),
            state @ State::Status(_) => {
                // ignore what happens after a ping has been asked
                let _ = state.next(&mut reader, &mut writer).await;
                return Ok(None);
            }
            state @ State::Play => {
                state.next(&mut reader, &mut writer).await?;
                let id = (SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() % 1000) as i32;
                return Ok(Some(Self {
                    read_stream: Lock::new(Box::new(reader)),
                    write_stream: Lock::new(Box::new(writer)),
                    world,
                    id: VarInt(id),
                    info: Info::from_id(id),
                    position: Lock::new(EntityPosition::new(0., 5., 0., 0, 0)),
                }));
            }
        }
    }

    pub fn id(&self) -> VarInt {
        self.id
    }

    pub fn info(&self) -> &Info {
        &self.info
    }

    /// return a LockGuard, drop it as soon as possible
    pub async fn position(&self) -> LockGuard<EntityPosition> {
        self.position.lock().await
    }

    pub async fn send_packet(&self, packet: &(impl Packet + Sync)) -> Result<()> {
        packet.send_packet(&mut *self.write_stream.lock().await).await?;
        self.write_stream.lock().await.flush().await?;
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        let position = OutPlayerPositionLook::from(&*self.position.lock().await);
        position.send_packet(&mut *self.write_stream.lock().await).await?;
        self.write_stream.lock().await.flush().await?;

        for i in 0..=45 {
            let slot = Slot::empty(Window::Inventory, i);
            slot.send_packet(&mut *self.write_stream.lock().await).await?;
        }
        HeldItemSlot::new(4)?
            .send_packet(&mut *self.write_stream.lock().await)
            .await?;
        self.write_stream.lock().await.flush().await?;

        let mut chunk = Chunk::new(0, 0);
        for z in 0..16 {
            for x in 0..16 {
                if (z + x) % 2 == 0 {
                    chunk.set_block(x, 4, z, Block::WhiteConcrete);
                } else {
                    chunk.set_block(x, 4, z, Block::BlackConcrete);
                }
            }
        }

        for x in -4..4 {
            for z in -4..4 {
                chunk.x = x;
                chunk.z = z;
                chunk.send_packet(&mut *self.write_stream.lock().await).await?;
                self.write_stream.lock().await.flush().await?;
            }
        }

        let message = OutChatMessage::new(
            Chat::new("Welcome in Minecrust!"),
            chat_message::Position::GameInfo,
        );
        self.send_packet(&message).await?;

        if let Some(error) = self.handle_packet().await.err() {
            self.world.remove_player(&self).await;
        }
        Ok(())
    }

    async fn handle_packet(&self) -> Result<()> {
        loop {
            let size = self.read_stream.lock().await.read_var_int().await?;
            let rest_reader = &mut *self.read_stream.lock().await;
            let rest_reader = &mut rest_reader.take(*size as u64);
            let packet_id = rest_reader.read_var_int().await?;

            match packet_id {
                InChatMessage::PACKET_ID => {
                    let in_message = InChatMessage::parse(rest_reader).await?;
                    let out_message = OutChatMessage::from_player_message(&self, in_message);
                    self.world.broadcast_packet(&out_message).await;
                },
                InPlayerPosition::PACKET_ID => {
                    let in_position = InPlayerPosition::parse(rest_reader).await?;
                    let delta = self.position.lock().await.update_position(&in_position);

                    let out_position = OutPosition::from(&self, &delta, in_position.on_ground);
                    self.world.broadcast_packet_except(&out_position, &self).await;
                },
                InPlayerPositionRotation::PACKET_ID => {
                    let in_position_rotation = InPlayerPositionRotation::parse(rest_reader).await?;
                    let delta = self.position.lock().await.update_position(&in_position_rotation);
                    self.position.lock().await.update_angle(&in_position_rotation);

                    let out_position_rotation = OutPositionRotation::from(&self, &delta, in_position_rotation.on_ground).await;
                    self.world.broadcast_packet_except(&out_position_rotation, &self).await;

                    let out_head_look = OutEntityHeadLook::from(&self).await;
                    self.world.broadcast_packet_except(&out_head_look, &self).await;
                },
                InPlayerRotation::PACKET_ID => {
                    let in_rotation = InPlayerRotation::parse(rest_reader).await?;
                    self.position.lock().await.update_angle(&in_rotation);

                    let out_rotation = OutRotation::from(&self, in_rotation.on_ground).await;
                    self.world.broadcast_packet_except(&out_rotation, &self).await;

                    let out_head_look = OutEntityHeadLook::from(&self).await;
                    self.world.broadcast_packet_except(&out_head_look, &self).await;
                },
                _ => {
                    let _error = futures::io::copy(rest_reader, &mut futures::io::sink()).await;
                    print!("{} ", packet_id);
                }
            }
        }
        Ok(())
    }
}

impl PartialEq for Player {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Info {
    uuid: Uuid,
    name: types::String,
    properties: LengthVec<InfoProperty>,
    game_mode: GameMode,
    ping: VarInt,
    display_name: BoolOption<Chat>,
}

impl Info {
    pub fn from_id(id: i32) -> Self {
        let name = format!("Player {}", id);
        Self {
            uuid: offline_uuid(&name),
            name: types::String::new(&name[..min(name.len(), 16)]),
            properties: LengthVec::new(),
            game_mode: GameMode::Creative,
            ping: VarInt::new(5),
            display_name: BoolOption(None),
        }
    }

    pub fn uuid(&self) -> Uuid {
        self.uuid
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
struct InfoProperty {
    name: types::String,
    value: types::String,
    signed: bool,
}

impl InfoProperty {
    pub fn new_texture(path: &str) -> Self {
        Self {
            name: types::String::new("textures"),
            value: types::String::new(std::str::from_utf8(&std::fs::read(path).unwrap()).unwrap()),
            signed: false,
        }
    }
}

crate ::impl_size!(Uuid, 16);
#[async_trait::async_trait]
impl types::Send for Uuid {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(&self.as_bytes()[..]).await?;
        Ok(())
    }
}

fn offline_uuid(name: &str) -> Uuid {
    let mut context = md5::Context::new();
    context.consume(format!("OfflinePlayer:{}", name).as_bytes());
    let mut builder = uuid::Builder::from_bytes(
        context.compute().into()
    );

    builder
        .set_variant(uuid::Variant::RFC4122)
        .set_version(uuid::Version::Md5);

    builder.build()
}