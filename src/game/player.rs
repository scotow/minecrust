use crate::packets::play::chunk::Chunk;
use crate::packets::play::held_item_slot::HeldItemSlot;
use crate::packets::play::join_game::JoinGame;
use crate::packets::play::{position::Position, slot::{Slot, Window}, chat_message};
use crate::packets::{Packet, ServerDescription};

use crate::fsm::State;
use crate::types::{self, VarInt, Chat};
use anyhow::Result;
use futures::prelude::*;
use piper::{Arc, Mutex};
use std::time::Duration;
use crate::packets::play::block::Block;
use futures_timer::Delay;
use crate::packets::play::chat_message::ChatMessage;

/// here we use the Arc to get interior mutability
#[derive(Clone)]
pub struct Player {
    read_stream: Arc<Mutex<Box<dyn AsyncRead + Send + Sync + Unpin>>>,
    write_stream: Arc<Mutex<Box<dyn AsyncWrite + Send + Sync + Unpin>>>,
    id: types::VarInt,
}

impl Player {
    pub async fn new(
        mut reader: impl AsyncRead + Send + Sync + Unpin + 'static,
        mut writer: impl AsyncWrite + Send + Sync + Unpin + 'static,
        server_description: ServerDescription,
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
                return Ok(Some(Self {
                    read_stream: Arc::new(Mutex::new(Box::new(reader))),
                    write_stream: Arc::new(Mutex::new(Box::new(writer))),
                    id: VarInt::default(),
                }));
            }
        }
    }

    pub fn id(&self) -> VarInt {
        self.id
    }

    pub async fn send_packet(&mut self, packet: &(impl Packet + Sync)) -> Result<()> {
        packet.send_packet(&mut self.write_stream).await?;
        self.write_stream.flush().await?;
        Ok(())
    }

    pub async fn run(&mut self) -> Result<()> {
        let join_game = JoinGame::default();
        join_game.send_packet(&mut self.write_stream).await?;
        // early flush so the player can get in game faster
        self.write_stream.flush().await?;

        let mut position = Position::default();
        // position.x = 234.0;
        position.y = 5.0;
        // position.z = 72.0;
        position.send_packet(&mut self.write_stream).await?;

        for i in 0..=45 {
            let slot = Slot::empty(Window::Inventory, i);
            slot.send_packet(&mut self.write_stream).await?;
        }
        HeldItemSlot::new(4)?
            .send_packet(&mut self.write_stream)
            .await?;
        self.write_stream.flush().await?;


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
                chunk.send_packet(&mut self.write_stream).await?;
                self.write_stream.flush().await?;
            }
        }

        Delay::new(Duration::from_secs(3)).await;
        for t in &[chat_message::Position::Chat, chat_message::Position::GameInfo, chat_message::Position::SystemMessage] {
            let message = ChatMessage::new(
                Chat::new("Hello, World!"),
                *t,
            );
            self.send_packet(&message).await?;
            Delay::new(Duration::from_secs(1)).await;
        }

        let mut buf = Vec::new();
        self.read_stream.read_to_end(&mut buf).await?;
        dbg!(buf.len());
        Ok(())
    }
}
