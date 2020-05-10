use crate::impl_packet;
use crate::packets::Packet;
use crate::stream::ReadExtension;
use crate::types::{self, Send, Size};
use anyhow::{anyhow, ensure, Result};
use futures::prelude::*;
use serde::Serialize;
use serde_json::json;
use std::marker::Unpin;

pub struct StatusRequest {}

impl StatusRequest {
    pub const PACKET_ID: types::VarInt = types::VarInt(0x00);

    pub async fn parse<R: AsyncRead + Unpin + std::marker::Send>(reader: &mut R) -> Result<Self> {
        let size = reader.read_var_int().await?;
        if *size != 1 {
            return Err(anyhow!("invalid packet size"));
        }

        let id = reader.read_var_int().await?;
        if *id != *Self::PACKET_ID {
            return Err(anyhow!("unexpected non request packet id"));
        }

        Ok(Self {})
    }

    pub async fn answer<W: AsyncWrite + Unpin + std::marker::Send>(
        &self,
        writer: &mut W,
        description: &ServerDescription,
    ) -> Result<()> {
        let info = types::String::new(
            &json!({
                "version": description.version,
                "players": {
                    "online": description.players.0,
                    "max": description.players.1,
                    "sample": []
                },
                "description": {
                    "text": description.description
                },
                "favicon": description.icon_data()
            })
            .to_string(),
        );

        (Self::PACKET_ID.size() + info.size()).send(writer).await?;
        Self::PACKET_ID.send(writer).await?;
        info.send(writer).await?;
        Ok(())
    }
}

#[derive(Default, Debug, Clone)]
pub struct ServerDescription {
    pub version: Version,
    pub players: (u32, u32),
    pub description: String,
    pub icon: Option<Vec<u8>>,
}

impl ServerDescription {
    pub fn icon_data(&self) -> Option<String> {
        self.icon
            .as_ref()
            .map(|i| format!("data:image/png;base64,{}", base64::encode(i)))
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct Version {
    name: &'static str,
    protocol: u16,
}

impl Default for Version {
    fn default() -> Self {
        Version {
            name: "1.15.2",
            protocol: 578,
        }
    }
}

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Ping {
    payload: i64,
}

impl Ping {
    pub async fn parse<R: AsyncRead + Unpin + std::marker::Send>(reader: &mut R) -> Result<Self> {
        let size = reader.read_var_int().await?;
        ensure!(
            size == Self::PACKET_ID.size() + 0_i64.size(),
            "invalid packet size: {}",
            *size
        );

        let _id = reader.read_var_int().await?;
        Ok(Self {
            payload: reader.read_i64().await?,
        })
    }
}
impl_packet!(Ping, 0x01);
