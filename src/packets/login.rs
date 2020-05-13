use futures::prelude::*;
use crate::types::{self, Send, Size, TAsyncRead, TAsyncWrite, Receive};
use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct LoginRequest {
    pub user_name: types::String,
}

impl LoginRequest {
    pub const START_PACKET_ID: types::VarInt = types::VarInt(0x00);
    pub const SUCCESS_PACKET_ID: types::VarInt = types::VarInt(0x02);

    const START_MAX_SIZE: types::VarInt = types::VarInt(1 + 4 * 16 + 1);
    const RANDOM_UUID: &'static str = "cbc2619b-9c6b-4171-a51d-abc281d6ff38";

    pub async fn parse<R: TAsyncRead>(reader: &mut R) -> Result<Self> {
        let size: types::VarInt = reader.receive().await?;
        if *size > *Self::START_MAX_SIZE {
            return Err(anyhow!("invalid packet size"));
        }

        let id: types::VarInt = reader.receive().await?;
        if *id != *Self::START_PACKET_ID {
            return Err(anyhow!("unexpected non login packet id"));
        }

        Ok(Self {
            user_name: reader
                .take((*size - *id.size()) as u64)
                .receive()
                .await?,
        })
    }

    pub async fn answer<W: TAsyncWrite>(
        &self,
        writer: &mut W,
    ) -> Result<()> {
        let uuid = types::String::new(Self::RANDOM_UUID);

        (Self::SUCCESS_PACKET_ID.size() + uuid.size() + self.user_name.size())
            .send(writer)
            .await?;
        Self::SUCCESS_PACKET_ID.send(writer).await?;
        uuid.send(writer).await?;
        self.user_name.send(writer).await?;
        Ok(())
    }
}
