use crate::impl_packet;
use crate::types;
use crate::types::{EntityPosition, VarInt};
use futures::AsyncRead;
use anyhow::Result;
use crate::stream::ReadExtension;


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

pub trait PlayerPositionPacket {
    fn x(&self) -> f64;
    fn y(&self) -> f64;
    fn z(&self) -> f64;
}

pub trait PlayerRotationPacket {
    fn x_angle(&self) -> f32;
    fn z_angle(&self) -> f32;
}

#[derive(Debug)]
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

impl PlayerPositionPacket for InPlayerPosition {
    fn x(&self) -> f64 { self.x }
    fn y(&self) -> f64 { self.y }
    fn z(&self) -> f64 { self.z }
}

#[derive(Debug)]
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

impl PlayerPositionPacket for InPlayerPositionRotation {
    fn x(&self) -> f64 { self.x }
    fn y(&self) -> f64 { self.y }
    fn z(&self) -> f64 { self.z }
}

impl PlayerRotationPacket for InPlayerPositionRotation {
    fn x_angle(&self) -> f32 { self.x_angle }
    fn z_angle(&self) -> f32 { self.z_angle }
}

#[derive(Debug)]
pub struct InPlayerRotation {
    pub x_angle: f32,
    pub z_angle: f32,
    pub on_ground: bool,
}

impl InPlayerRotation {
    pub const PACKET_ID: VarInt = VarInt(0x13);

    pub async fn parse<R: AsyncRead + Unpin + std::marker::Send>(reader: &mut R) -> Result<Self> {
        let x_angle = reader.read_f32().await?;
        let z_angle = reader.read_f32().await?;
        let on_ground = reader.read_bool().await?;
        Ok(Self {
            x_angle,
            z_angle,
            on_ground,
        })
    }
}

impl PlayerRotationPacket for InPlayerRotation {
    fn x_angle(&self) -> f32 { self.x_angle }
    fn z_angle(&self) -> f32 { self.z_angle }
}