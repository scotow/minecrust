use super::Packet;
use crate::impl_packet;
use crate::types::{self, Receive, ServerDescription, Send, Size, TAsyncRead, TAsyncWrite};
use anyhow::{anyhow, ensure, Result};
use serde_json::json;

pub struct StatusRequest {}

impl StatusRequest {
    pub const PACKET_ID: types::VarInt = types::VarInt(0x00);

    pub async fn answer<W: TAsyncWrite>(
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

#[async_trait::async_trait]
impl types::FromReader for StatusRequest {
    async fn from_reader<R: TAsyncRead>(reader: &mut R) -> Result<Self> {
        let size: types::VarInt = reader.receive().await?;
        if *size != 1 {
            return Err(anyhow!("invalid packet size"));
        }

        let id: types::VarInt = reader.receive().await?;
        if *id != *Self::PACKET_ID {
            return Err(anyhow!("unexpected non request packet id"));
        }

        Ok(Self {})
    }
}

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Ping {
    payload: i64,
}

impl_packet!(Ping, 0x01);

#[async_trait::async_trait]
impl types::FromReader for Ping {
    async fn from_reader<R: TAsyncRead>(reader: &mut R) -> Result<Self> {
        let size = reader.receive::<types::VarInt>().await?;
        let packet_id = Self::PACKET_ID.size();
        ensure!(
            size == packet_id + 0_i64.size(),
            "invalid packet size: {}",
            *size
        );

        let _id: types::VarInt = reader.receive().await?;
        Ok(Self {
            payload: reader.receive().await?,
        })
    }
}
