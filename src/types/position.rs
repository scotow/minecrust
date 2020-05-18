use crate::packets::play::player_position::{PlayerPositionPacket, PlayerRotationPacket};
use crate::types::Receive;
use crate::types::{Send, TAsyncRead, TAsyncWrite};
use anyhow::Result;

#[derive(Debug, Clone, macro_derive::Size, macro_derive::Send)]
pub struct EntityPosition {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub x_angle: u8,
    pub z_angle: u8,
}

impl EntityPosition {
    pub fn new(x: f64, y: f64, z: f64, x_angle: u8, z_angle: u8) -> Self {
        Self {
            x,
            y,
            z,
            x_angle,
            z_angle,
        }
    }

    pub fn at(x: i64, y: i64, z: i64) -> Self {
        Self::new(x as f64, y as f64, z as f64, 0, 0)
    }

    pub fn rotation(&self) -> (f32, f32) {
        (
            self.x_angle as f32 * 360. / 256.,
            self.z_angle as f32 * 360. / 256.,
        )
    }

    pub fn chunk(&self) -> (i32, i32) {
        let (mut x, mut z) = (self.x as i32, self.z as i32);
        if x < 0 {
            x -= 16
        }
        if z < 0 {
            z -= 16
        }

        (x / 16, z / 16)
    }

    pub fn subchunk(&self) -> (i32, i32, i32) {
        let mut y = self.y as i32;
        if y < 0 {
            y -= 16
        }

        let (x, z) = self.chunk();
        (x, y / 16, z)
    }

    pub fn update_position(&mut self, from: &dyn PlayerPositionPacket) -> PositionDelta {
        let before_subchunk = self.subchunk();

        let mut delta = PositionDelta {
            x: ((from.x() * 32. - self.x * 32.) * 128.) as i16,
            y: ((from.y() * 32. - self.y * 32.) * 128.) as i16,
            z: ((from.z() * 32. - self.z * 32.) * 128.) as i16,
            subchunk_changed: false,
        };
        self.x = from.x();
        self.y = from.y();
        self.z = from.z();

        delta.subchunk_changed = before_subchunk != self.subchunk();
        delta
    }

    pub fn update_angle(&mut self, from: &dyn PlayerRotationPacket) {
        self.x_angle = (from.x_angle().rem_euclid(360.) * 255. / 360.) as u8;
        self.z_angle = (from.z_angle().rem_euclid(360.) * 255. / 360.) as u8;
    }
}

#[derive(Debug, Clone)]
pub struct PositionDelta {
    pub x: i16,
    pub y: i16,
    pub z: i16,
    pub subchunk_changed: bool,
}

#[derive(Debug, Clone)]
pub struct BlockPosition {
    pub x: i32,
    pub y: u16,
    pub z: i32,
}
crate::impl_size!(BlockPosition, 8);

#[async_trait::async_trait]
impl Send for BlockPosition {
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
        let n = ((self.x as u32 as u64 & 0x3FF_FFFF) << 38)
            | ((self.z as u32 as u64 & 0x3FF_FFFF) << 12)
            | (self.y as u64 & 0xFFF);
        n.send(writer).await
    }
}

impl BlockPosition {
    pub fn new(x: i32, y: u16, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn from_u64(n: u64) -> Self {
        Self::new((n >> 38) as i32, (n & 0xFFF) as u16, (n << 26 >> 38) as i32)
    }
}

#[async_trait::async_trait]
impl crate::types::FromReader for BlockPosition {
    async fn from_reader<R: TAsyncRead>(reader: &mut R) -> Result<Self> {
        Ok(reader.receive::<u64>().await?.into())
    }
}

impl From<u64> for BlockPosition {
    fn from(n: u64) -> Self {
        Self::from_u64(n)
    }
}
