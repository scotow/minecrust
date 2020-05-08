use crate::types::{VarInt, PositionDelta};
use crate::game::player::Player;
use futures::AsyncRead;
use crate::stream::ReadExtension;
use anyhow::Result;
use crate::packets::play::player_position::{InPlayerPosition, InPlayerPositionRotation, InPlayerRotation};

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct OutPosition {
    id: VarInt,
    delta_x: i16,
    delta_y: i16,
    delta_z: i16,
    on_ground: bool,
}
crate ::impl_packet!(OutPosition, 0x29);

impl OutPosition {
    pub fn from(player: &Player, delta: &PositionDelta, on_ground: bool) -> Self {
        Self {
            id: player.id(),
            delta_x: delta.0,
            delta_y: delta.1,
            delta_z: delta.2,
            on_ground,
        }
    }
}

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct OutPositionRotation {
    id: VarInt,
    delta_x: i16,
    delta_y: i16,
    delta_z: i16,
    x_angle: u8,
    z_angle: u8,
    on_ground: bool,
}
crate ::impl_packet!(OutPositionRotation, 0x2A);

impl OutPositionRotation {
    pub async fn from(player: &Player, delta: &PositionDelta, on_ground: bool) -> Self {
        let current_position = player.position().await;
        Self {
            id: player.id(),
            delta_x: delta.0,
            delta_y: delta.1,
            delta_z: delta.2,
            x_angle: current_position.x_angle,
            z_angle: current_position.z_angle,
            on_ground,
        }
    }
}

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct OutRotation {
    id: VarInt,
    x_angle: u8,
    z_angle: u8,
    on_ground: bool,
}
crate ::impl_packet!(OutRotation, 0x2B);

impl OutRotation {
    pub async fn from(player: &Player, on_ground: bool) -> Self {
        let current_position = player.position().await;
        Self {
            id: player.id(),
            x_angle: current_position.x_angle,
            z_angle: current_position.z_angle,
            on_ground,
        }
    }
}

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct OutEntityHeadLook {
    id: VarInt,
    x_angle: u8,
}
crate ::impl_packet!(OutEntityHeadLook, 0x3C);

impl OutEntityHeadLook {
    pub async fn from(player: &Player) -> Self {
        let current_position = player.position().await;
        Self {
            id: player.id(),
            x_angle: current_position.x_angle,
        }
    }
}