use crate::packets::play::player_position::{InPlayerPosition, InPlayerPositionRotation};

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

    pub fn update_from_position(&mut self, position: &InPlayerPosition) {
        self.x = position.x;
        self.y = position.y;
        self.z = position.z;
    }

    pub fn update_from_position_rotation(&mut self, position: &InPlayerPositionRotation) {
        self.x = position.x;
        self.y = position.y;
        self.z = position.z;
        self.x_angle = position.x_angle as u8;
        self.z_angle = position.z_angle as u8;
    }
}
