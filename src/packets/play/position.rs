use crate::impl_packet;
use crate::types;

#[derive(Debug, Default, macro_derive::Size, macro_derive::Send)]
pub struct Position {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub x_angle: f32,
    pub z_angle: f32,
    pub relative_flag: i8,
    pub teleport_id: types::VarInt,
}
impl_packet!(Position, 0x36);
