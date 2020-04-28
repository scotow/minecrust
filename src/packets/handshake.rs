use futures::prelude::*;
use std::marker::Unpin;

use crate::stream::ReadExtension;
use crate::types;
use anyhow::{anyhow, Result};

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Handshake {
    pub protocol_version: types::VarInt,
    pub server_address: types::String,
    pub server_port: u16,
    pub next_state: types::VarInt,
}
crate::impl_packet!(Handshake, 0x00);

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

    pub async fn parse<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self> {
        let size = reader.read_var_int().await?;
        let mut reader = reader.take(*size as u64);

        let id = reader.read_var_int().await?;
        if *id != *Self::PACKET_ID {
            return Err(anyhow!("unexpected non handshake packet id"));
        }

        let handshake = Self::new(
            reader.read_var_int().await?,
            reader.read_string().await?,
            reader.read_u16().await?,
            reader.read_var_int().await?,
        );

        if !(1..=2).contains(&*handshake.next_state) {
            return Err(anyhow!("invalid next state packet id"));
        }

        Ok(handshake)
    }
}
