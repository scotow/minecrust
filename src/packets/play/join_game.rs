use crate::types::{self, Size, TAsyncWrite, VarInt};
use crate::{impl_packet, impl_send, impl_size};
use anyhow::Result;
use std::fmt::{self, Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

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
impl_packet!(JoinGame, 0x26);

impl Default for JoinGame {
    fn default() -> Self {
        JoinGame {
            id: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i32,
            game_mode: GameMode::Creative,
            dimension: Dimension::Overworld,
            hash_seed: 0,
            max_player: 0,
            level_type: LevelType::Default,
            view_distance: VarInt::new(16),
            reduced_debug_info: false,
            enable_respawn_screen: false,
        }
    }
}

// TODO: Move to own file.
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
#[repr(i32)]
pub enum Dimension {
    Nether = -1,
    Overworld,
    End,
}
impl_size!(Dimension, 4);
impl_send!(Dimension as i32);

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
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
        types::String::new(&self.to_string()).send(writer).await
    }
}
