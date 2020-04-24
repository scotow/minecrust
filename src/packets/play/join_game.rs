use futures::AsyncWrite;
use std::fmt::{self, Display, Formatter};
use std::marker;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use crate::{impl_size, impl_send};
use crate::types::{self, Send, Size, VarInt};

#[derive(macro_derive::Size, macro_derive::Send, Debug)]
pub struct JoinGame {
    pub id: i32,
    pub game_mode: GameMode,
    pub dimension: Dimension,
    pub hash_seed: i64,
    pub max_player: u8,
    pub level_type: LevelType,
    pub view_distance: types::VarInt,
    pub reduced_debug_info: bool,
    pub enable_respawn_screen: bool,
}

impl JoinGame {
    const PACKET_ID: types::VarInt = types::VarInt(0x26);

    pub async fn send_packet<W: AsyncWrite + marker::Unpin + marker::Send>(
        &self,
        writer: &mut W,
    ) -> Result<()> {
        (Self::PACKET_ID.size() + self.size()).send(writer).await?;
        Self::PACKET_ID.send(writer).await?;
        self.send(writer).await?;
        Ok(())
    }
}

impl Default for JoinGame {
    fn default() -> Self {
        JoinGame {
            id: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i32,
            game_mode: GameMode::Survival,
            dimension: Dimension::Overworld,
            hash_seed: 0,
            max_player: 0,
            level_type: LevelType::Default,
            view_distance: VarInt::new(16),
            reduced_debug_info: true,
            enable_respawn_screen: false,
        }
    }
}

#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum GameMode {
    Survival = 0,
    Creative,
    Adventure,
    Spectator,
}
impl_size!(GameMode, 1);
impl_send!(GameMode as u8);

#[derive(Copy, Clone, Debug)]
#[repr(i8)]
pub enum Dimension {
    Nether = -1,
    Overworld,
    End,
}
impl_size!(Dimension, 1);
impl_send!(Dimension as i8);

// #[async_trait::async_trait]
// impl types::Send for Dimension {
//     async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
//         writer.write_i8(*self as i8).await
//     }
// }

#[derive(Copy, Clone, Debug)]
pub enum LevelType {
    Default,
    Flat,
    LargeBiomes,
    Amplified,
    Customized,
    Buffet,
    Default11,
}

impl Display for LevelType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use LevelType::*;

        write!(
            f,
            "{}",
            match self {
                Default => "default",
                Flat => "flat",
                LargeBiomes => "largeBiomes",
                Amplified => "amplified",
                Customized => "customized",
                Buffet => "buffet",
                Default11 => "default_1_1",
            }
        )
    }
}

impl Size for LevelType {
    fn size(&self) -> VarInt {
        types::String::new(&self.to_string()).size()
    }
}

#[async_trait::async_trait]
impl types::Send for LevelType {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        types::String::new(&self.to_string()).send(writer).await
    }
}
