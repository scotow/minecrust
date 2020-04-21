use futures::prelude::*;
use std::marker::Unpin;

use crate::stream::{ReadExtension, WriteExtension};
use crate::types;
use anyhow::{anyhow, Result};

use serde::Serialize;
use serde_json::json;

pub struct StatusRequest {}

impl StatusRequest {
    const PACKET_ID: types::VarInt = types::VarInt(0x00);

    pub async fn parse<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self> {
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

    pub async fn answer<W: AsyncWrite + Unpin + Send>(
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

        writer
            .write_var_int(Self::PACKET_ID.size() + info.size())
            .await?;
        writer.write_var_int(Self::PACKET_ID).await?;
        writer.write_string(&info).await?;
        Ok(())
    }
}

#[derive(Default, Debug)]
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

#[derive(Serialize, Debug)]
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

pub struct PingRequest {
    payload: i64,
}

impl PingRequest {
    const PACKET_ID: types::VarInt = types::VarInt(0x01);

    pub async fn parse<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self> {
        let size = reader.read_var_int().await?;
        if *size != *Self::PACKET_ID.size() + 8 {
            return Err(anyhow!("invalid packet size"));
        }

        let id = reader.read_var_int().await?;
        if *id != *Self::PACKET_ID {
            return Err(anyhow!("unexpected non ping packet id"));
        }

        Ok(Self {
            payload: reader.read_i64().await?,
        })
    }

    pub async fn answer<W: AsyncWrite + Unpin + Send>(&self, writer: &mut W) -> Result<()> {
        writer
            .write_var_int(Self::PACKET_ID.size() + types::VarInt::new(8))
            .await?;
        writer.write_var_int(Self::PACKET_ID).await?;
        writer.write_i64(self.payload).await?;
        Ok(())
    }
}
