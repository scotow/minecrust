use crate::impl_packet;
use crate::types;
use crate::types::{EntityPosition, VarInt};
use futures::AsyncRead;
use anyhow::Result;
use crate::stream::ReadExtension;
use crate::game::player::Player;

#[derive(Debug, Default, macro_derive::Size, macro_derive::Send)]
pub struct OutPlayerPositionLook {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub x_angle: f32,
    pub z_angle: f32,
    pub relative_flag: i8,
    pub teleport_id: types::VarInt,
}
impl_packet!(OutPlayerPositionLook, 0x36);

impl From<&EntityPosition> for OutPlayerPositionLook {
    fn from(position: &EntityPosition) -> Self {
        Self {
            x: position.x,
            y: position.y,
            z: position.z,
            x_angle: position.x_angle as f32,
            z_angle: position.x_angle as f32,
            relative_flag: 0,
            teleport_id: VarInt(0),
        }
    }
}

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct OutPosition {
    id: VarInt,
    delta_x: i16,
    delta_y: i16,
    delta_z: i16,
    on_ground: bool,
}
impl_packet!(OutPosition, 0x29);

impl OutPosition {
    pub async fn from_player_position(player: &Player, new: &InPlayerPosition) -> Self {
        let current_position = player.position().await;
        Self {
            id: player.id(),
            delta_x: (new.x * 32. - current_position.x * 32.) as i16 * 128,
            delta_y: (new.y * 32. - current_position.y * 32.) as i16 * 128,
            delta_z: (new.z * 32. - current_position.z * 32.) as i16 * 128,
            on_ground: new.on_ground,
        }
    }

    pub async fn from_player_position_rotation(player: &Player, new: &InPlayerPositionRotation) -> Self {
        let current_position = player.position().await;
        Self {
            id: player.id(),
            delta_x: (new.x * 32. - current_position.x * 32.) as i16 * 128,
            delta_y: (new.y * 32. - current_position.y * 32.) as i16 * 128,
            delta_z: (new.z * 32. - current_position.z * 32.) as i16 * 128,
            on_ground: new.on_ground,
        }
    }
}

pub struct InPlayerPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub on_ground: bool,
}

impl InPlayerPosition {
    pub const PACKET_ID: VarInt = VarInt(0x11);

    pub async fn parse<R: AsyncRead + Unpin + std::marker::Send>(reader: &mut R) -> Result<Self> {
        let x = reader.read_f64().await?;
        let y = reader.read_f64().await?;
        let z = reader.read_f64().await?;
        let on_ground = reader.read_bool().await?;
        Ok(Self {
            x,
            y,
            z,
            on_ground,
        })
    }
}

pub struct InPlayerPositionRotation {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub x_angle: f32,
    pub z_angle: f32,
    pub on_ground: bool,
}

impl InPlayerPositionRotation {
    pub const PACKET_ID: VarInt = VarInt(0x12);

    pub async fn parse<R: AsyncRead + Unpin + std::marker::Send>(reader: &mut R) -> Result<Self> {
        let x = reader.read_f64().await?;
        let y = reader.read_f64().await?;
        let z = reader.read_f64().await?;
        let x_angle = reader.read_f32().await?;
        let z_angle = reader.read_f32().await?;
        let on_ground = reader.read_bool().await?;
        Ok(Self {
            x,
            y,
            z,
            x_angle,
            z_angle,
            on_ground,
        })
    }
}
