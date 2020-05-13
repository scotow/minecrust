use crate::types::{self, Receive, TAsyncRead};
use anyhow::{anyhow, Result};
use futures::prelude::*;

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
}

#[async_trait::async_trait]
impl types::FromReader for Handshake {
    async fn from_reader<R: TAsyncRead>(reader: &mut R) -> Result<Self> {
        let size: types::VarInt = reader.receive().await?;
        let mut reader = reader.take(*size as u64);

        let id: types::VarInt = reader.receive().await?;
        if *id != *Self::PACKET_ID {
            return Err(anyhow!("unexpected non handshake packet id"));
        }

        let handshake = Self::new(
            reader.receive().await?,
            reader.receive().await?,
            reader.receive().await?,
            reader.receive().await?,
        );

        if !(1..=2).contains(&*handshake.next_state) {
            return Err(anyhow!("invalid next state packet id"));
        }

        Ok(handshake)
    }
}
