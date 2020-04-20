use std::io::Read;

use crate::error::Result;
use crate::stream::ReadExtension;
use crate::types;

#[derive(Debug)]
pub struct Handshake {
    pub protocol_version: types::VarInt,
    pub server_address: types::String,
    pub server_port: u16,
    pub next_state: types::VarInt,
}

impl Handshake {
    const PACKET_ID: types::VarInt = types::VarInt(0x00);

    pub fn new(
        protocol_version: types::VarInt,
        server_address: types::String,
        server_port: u16,
        next_state: types::VarInt,
    ) -> Self {
        Self {
            protocol_version,
            server_address,
            server_port,
            next_state,
        }
    }

    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let size = reader.read_var_int()?;
        let mut reader = reader.take(*size as u64);

        let id = reader.read_var_int()?;
        if *id != *Self::PACKET_ID {
            return Err("unexpected non handshake packet id".into());
        }

        let handshake = Self::new(
            reader.read_var_int()?,
            reader.read_string()?,
            reader.read_u16()?,
            reader.read_var_int()?,
        );

        if !(1..=2).contains(&*handshake.next_state) {
            return Err("invalid next state packet id".into());
        }

        Ok(handshake)
    }
}
