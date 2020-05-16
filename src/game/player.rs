use crate::fsm::Fsm;
use crate::game::world::World;
use crate::packets::play::block::Block;
use crate::packets::play::block_change::BlockChange;
use crate::packets::play::chat_message::{InChatMessage, OutChatMessage};
use crate::packets::play::entity_position::{
    OutEntityHeadLook, OutPosition, OutPositionRotation, OutRotation,
};
use crate::packets::play::join_game::GameMode;
use crate::packets::play::player_digging::PlayerDigging;
use crate::packets::play::player_position::{
    InPlayerPosition, InPlayerPositionRotation, InPlayerRotation, OutViewPosition,
};
use crate::packets::play::{chat_message, player_position::OutPlayerPositionLook};
use crate::packets::{Packet, ServerDescription};
use crate::types::chat::Chat;
use crate::types::{
    self, BoolOption, EntityPosition, LengthVec, Receive, TAsyncRead, TAsyncWrite, VarInt,
};
use anyhow::Result;
use futures::prelude::*;
use futures::AsyncWriteExt;
use piper::{Lock, LockGuard};
use std::cmp::min;
use std::collections::HashSet;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

pub struct Player {
    read_stream: Lock<Box<dyn TAsyncRead>>,
    write_stream: Lock<Box<dyn TAsyncWrite>>,
    world: &'static World,
    id: types::VarInt,
    info: Info,
    position: Lock<EntityPosition>,
    loaded_chunks: Lock<HashSet<(i32, i32)>>,
}

impl Player {
    const RENDER_DISTANCE: i32 = 16;

    pub async fn new(
        reader: impl TAsyncRead + 'static,
        writer: impl TAsyncWrite + 'static,
        server_description: ServerDescription,
        world: &'static World,
    ) -> Result<Option<Self>> {
        let mut reader: Box<dyn TAsyncRead> = Box::new(reader);
        let mut writer: Box<dyn TAsyncWrite> = Box::new(writer);
        let fsm = Fsm::from_rw(server_description, &mut reader, &mut writer);
        let login = fsm.play().await?;
        if login.is_none() {
            return Ok(None);
        }
        let login = login.unwrap();

        let id = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
            % 1000) as i32;

        Ok(Some(Self {
            read_stream: Lock::new(reader),
            write_stream: Lock::new(writer),
            world,
            id: VarInt(id),
            info: Info::from_name(&*login.user_name),
            position: Lock::new(EntityPosition::new(0., 5., 0., 0, 0)),
            loaded_chunks: Lock::new(HashSet::new()),
        }))
    }

    pub fn id(&self) -> VarInt {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.info.name
    }

    pub fn info(&self) -> &Info {
        &self.info
    }

    pub async fn position(&self) -> LockGuard<EntityPosition> {
        self.position.lock().await
    }

    pub async fn send_packet(&self, packet: &(impl Packet + Sync)) -> Result<()> {
        packet
            .send_packet(&mut *self.write_stream.lock().await)
            .await?;
        self.write_stream.lock().await.flush().await?;
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        self.send_chunks_around(Self::RENDER_DISTANCE).await?;

        let position = OutPlayerPositionLook::from(&*self.position.lock().await);
        position
            .send_packet(&mut *self.write_stream.lock().await)
            .await?;
        self.write_stream.lock().await.flush().await?;

        let message = OutChatMessage::new(
            Chat::new("Welcome in Minecrust!"),
            chat_message::Position::GameInfo,
        );
        self.send_packet(&message).await?;

        self.handle_packet().await
    }

    async fn handle_packet(&self) -> Result<()> {
        loop {
            let size: types::VarInt = self.read_stream.lock().await.receive().await?;
            let rest_reader = &mut *self.read_stream.lock().await;
            let rest_reader = &mut rest_reader.take(*size as u64);
            let packet_id: types::VarInt = rest_reader.receive().await?;

            match packet_id {
                InChatMessage::PACKET_ID => {
                    let in_message: InChatMessage = rest_reader.receive().await?;
                    let out_message = OutChatMessage::from_player_message(&self, in_message);
                    self.world.broadcast_packet(&out_message).await?;
                }
                InPlayerPosition::PACKET_ID => {
                    let in_position: InPlayerPosition = rest_reader.receive().await?;
                    let delta = self.position.lock().await.update_position(&in_position);

                    let out_position = OutPosition::from(&self, &delta, in_position.on_ground);
                    self.world
                        .broadcast_packet_except(&out_position, &self)
                        .await?;

                    if delta.subchunk_changed {
                        let out_view = OutViewPosition::from(&*self.position.lock().await);
                        self.send_packet(&out_view).await?;
                    }

                    self.send_needed_chunks(Self::RENDER_DISTANCE).await?;
                }
                InPlayerPositionRotation::PACKET_ID => {
                    let in_position_rotation: InPlayerPositionRotation =
                        rest_reader.receive().await?;
                    let delta = self
                        .position
                        .lock()
                        .await
                        .update_position(&in_position_rotation);
                    self.position
                        .lock()
                        .await
                        .update_angle(&in_position_rotation);

                    let out_position_rotation =
                        OutPositionRotation::from(&self, &delta, in_position_rotation.on_ground)
                            .await;
                    self.world
                        .broadcast_packet_except(&out_position_rotation, &self)
                        .await?;

                    let out_head_look = OutEntityHeadLook::from(&self).await;
                    self.world
                        .broadcast_packet_except(&out_head_look, &self)
                        .await?;

                    if delta.subchunk_changed {
                        let out_view = OutViewPosition::from(&*self.position.lock().await);
                        self.send_packet(&out_view).await?;
                    }

                    self.send_needed_chunks(Self::RENDER_DISTANCE).await?;
                }
                InPlayerRotation::PACKET_ID => {
                    let in_rotation: InPlayerRotation = rest_reader.receive().await?;
                    self.position.lock().await.update_angle(&in_rotation);

                    let out_rotation = OutRotation::from(&self, in_rotation.on_ground).await;
                    self.world
                        .broadcast_packet_except(&out_rotation, &self)
                        .await?;

                    let out_head_look = OutEntityHeadLook::from(&self).await;
                    self.world
                        .broadcast_packet_except(&out_head_look, &self)
                        .await?;
                }
                PlayerDigging::PACKET_ID => {
                    let action: PlayerDigging = rest_reader.receive().await?;
                    if let PlayerDigging::FinishedDigging(position, _face) = dbg!(action) {
                        let block_change = BlockChange::new(position.clone(), Block::Air);
                        self.world
                            .broadcast_packet_except(&block_change, &self)
                            .await?;
                    }
                }
                _ => {
                    let _error = futures::io::copy(rest_reader, &mut futures::io::sink()).await;
                    print!("{} ", packet_id);
                }
            }
        }
    }

    async fn send_chunks_around(&self, range: i32) -> Result<()> {
        let (p_x, p_z) = self.position.lock().await.chunk();
        let mut chunks = self.loaded_chunks.lock().await;

        for z in p_z - range..p_z + range {
            for x in p_x - range..p_x + range {
                if !chunks.insert((x, z)) {
                    continue;
                }

                let chunk = self.world.map.chunk(x, z).await;
                self.send_packet(&*chunk).await?;
            }
        }
        Ok(())
    }

    async fn send_needed_chunks(&self, range: i32) -> Result<()> {
        let full = {
            let (x, z) = self.position.lock().await.chunk();
            let chunks = self.loaded_chunks.lock().await;

            (-range..range).all(|r| {
                chunks.contains(&(x - range, z + r))
                    && chunks.contains(&(x + range, z + r))
                    && chunks.contains(&(x + r, z - range))
                    && chunks.contains(&(x + r, z - range))
            })
        };
        if full {
            return Ok(());
        }

        self.send_chunks_around(range).await
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
    pub fn from_name(name: &str) -> Self {
        let name = types::String::new(&name[..min(name.len(), 16)]);
        Self {
            uuid: offline_uuid(&name),
            name,
            properties: LengthVec::new(),
            game_mode: GameMode::Creative,
            ping: VarInt::new(5),
            display_name: BoolOption(None),
        }
    }

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

#[allow(dead_code)]
impl InfoProperty {
    pub fn new_texture(path: &str) -> Self {
        Self {
            name: types::String::new("textures"),
            value: types::String::new(std::str::from_utf8(&std::fs::read(path).unwrap()).unwrap()),
            signed: false,
        }
    }
}

crate::impl_size!(Uuid, 16);
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
    let mut builder = uuid::Builder::from_bytes(context.compute().into());

    builder
        .set_variant(uuid::Variant::RFC4122)
        .set_version(uuid::Version::Md5);

    builder.build()
}
