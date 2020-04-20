use std::io::{Read, Write};

use crate::error::Result;
use crate::stream::{ReadExtension, WriteExtension};
use crate::types;

#[derive(Debug)]
pub struct LoginRequest {
    pub user_name: types::String,
}

impl LoginRequest {
    const START_PACKET_ID: types::VarInt = types::VarInt(0x00);
    const START_MAX_SIZE: types::VarInt = types::VarInt(1 + 4 * 16 + 1);

    const SUCCESS_PACKET_ID: types::VarInt = types::VarInt(0x02);
    const RANDOM_UUID: &'static str = "cbc2619b-9c6b-4171-a51d-abc281d6ff38";

    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let size = reader.read_var_int()?;
        if *size > *Self::START_MAX_SIZE {
            return Err("invalid packet size".into());
        }

        let id = reader.read_var_int()?;
        if *id != *Self::START_PACKET_ID {
            return Err("unexpected non login packet id".into());
        }

        Ok(Self {
            user_name: reader.take((*size - *id.size()) as u64).read_string()?,
        })
    }

    pub fn answer<W: Write>(&self, writer: &mut W) -> Result<()> {
        let uuid = types::String::new(Self::RANDOM_UUID);

        writer
            .write_var_int(Self::SUCCESS_PACKET_ID.size() + uuid.size() + self.user_name.size())?;
        writer.write_var_int(Self::SUCCESS_PACKET_ID)?;
        writer.write_string(&uuid)?;
        writer.write_string(&self.user_name)?;
        Ok(())
    }
}
