use crate::packets::play::player_position::{InPlayerPosition, InPlayerPositionRotation, PlayerPositionPacket, PlayerRotationPacket};

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

    pub fn update_position(&mut self, from: &dyn PlayerPositionPacket) -> PositionDelta {
        let delta = PositionDelta(
            ((from.x() * 32. - self.x * 32.) * 128.) as i16,
            ((from.y() * 32. - self.y * 32.) * 128.) as i16,
            ((from.z() * 32. - self.z * 32.) * 128.) as i16,
        );
        self.x = from.x();
        self.y = from.y();
        self.z = from.z();

        delta
    }

    pub fn update_angle(&mut self, from: &dyn PlayerRotationPacket) {
        self.x_angle = ((from.x_angle() % 360. + 360.) % 360. * 255. / 360.) as u8;
        self.z_angle = ((from.z_angle() % 360. + 360.) % 360. * 255. / 360.) as u8;
    }
}

pub struct PositionDelta(pub i16, pub i16, pub i16);
