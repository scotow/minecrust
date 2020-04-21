use futures::prelude::*;

use crate::error::Result;
use crate::stream::{ReadExtension, WriteExtension};
use crate::types;
use std::marker::Unpin;

#[derive(Debug)]
pub struct LoginRequest {
    pub user_name: types::String,
}

impl LoginRequest {
    const START_PACKET_ID: types::VarInt = types::VarInt(0x00);
    const START_MAX_SIZE: types::VarInt = types::VarInt(1 + 4 * 16 + 1);

    const SUCCESS_PACKET_ID: types::VarInt = types::VarInt(0x02);
    const RANDOM_UUID: &'static str = "cbc2619b-9c6b-4171-a51d-abc281d6ff38";

    pub async fn parse<R: AsyncRead + Unpin + Send>(reader: &mut R) -> Result<Self> {
        let size = reader.read_var_int().await?;
        if *size > *Self::START_MAX_SIZE {
            return Err("invalid packet size".into());
        }

        let id = reader.read_var_int().await?;
        if *id != *Self::START_PACKET_ID {
            return Err("unexpected non login packet id".into());
        }

        Ok(Self {
            user_name: reader
                .take((*size - *id.size()) as u64)
                .read_string()
                .await?,
        })
    }

    pub async fn answer<W: AsyncWrite + Unpin + Send>(&self, writer: &mut W) -> Result<()> {
        let uuid = types::String::new(Self::RANDOM_UUID);

        writer
            .write_var_int(Self::SUCCESS_PACKET_ID.size() + uuid.size() + self.user_name.size())
            .await?;
        writer.write_var_int(Self::SUCCESS_PACKET_ID).await?;
        writer.write_string(&uuid).await?;
        writer.write_string(&self.user_name).await?;
        Ok(())
    }
}
