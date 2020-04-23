use futures::AsyncWrite;
use std::fmt::{self, Display, Formatter};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use crate::impl_size;
use crate::stream::WriteExtension;
use crate::types::{self, Size, VarInt};

#[derive(size_derive::Size, Debug)]
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

    pub async fn send<W: AsyncWrite + Unpin + Send>(&self, writer: &mut W) -> Result<()> {
        let level_type = types::String::new(&LevelType::Default.to_string());

        writer
            .write_var_int(
                Self::PACKET_ID.size()
                    + self.id.size()
                    + (self.game_mode as u8).size()
                    + (self.dimension as u8).size()
                    + self.hash_seed.size()
                    + self.max_player.size()
                    + level_type.size()
                    + self.view_distance.size()
                    + self.reduced_debug_info.size()
                    + self.enable_respawn_screen.size(),
            )
            .await?;

        dbg!(self.size());

        writer.write_var_int(Self::PACKET_ID).await?;
        writer.write_i32(self.id).await?;
        writer.write_u8(self.game_mode as u8).await?;
        writer.write_i8(self.dimension as i8).await?;
        writer.write_i64(self.hash_seed).await?;
        writer.write_u8(self.max_player).await?;
        writer.write_string(&level_type).await?;
        writer.write_var_int(self.view_distance).await?;
        writer.write_bool(self.reduced_debug_info).await?;
        writer.write_bool(self.enable_respawn_screen).await?;

        Ok(())
    }
}

impl Default for JoinGame {
    fn default() -> Self {
        JoinGame {
            id: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i32,
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

#[derive(Copy, Clone, Debug)]
#[repr(i8)]
pub enum Dimension {
    Nether = -1,
    Overworld,
    End,
}
impl_size!(Dimension, 1);

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