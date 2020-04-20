use std::io::{Read, Write};

use crate::error::Result;
use crate::stream::{ReadExtension, WriteExtension};
use crate::types;

use serde::Serialize;
use serde_json::json;

pub struct StatusRequest {}

impl StatusRequest {
    const PACKET_ID: types::VarInt = types::VarInt(0x00);

    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let size = reader.read_var_int()?;
        if *size != 1 {
            return Err("invalid packet size".into());
        }

        let id = reader.read_var_int()?;
        if *id != *Self::PACKET_ID {
            return Err("unexpected non request packet id".into());
        }

        Ok(Self {})
    }

    pub fn answer<W: Write>(&self, writer: &mut W, description: &ServerDescription) -> Result<()> {
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

        writer.write_var_int(Self::PACKET_ID.size() + info.size())?;
        writer.write_var_int(Self::PACKET_ID)?;
        writer.write_string(&info)?;
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

    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let size = reader.read_var_int()?;
        if *size != *Self::PACKET_ID.size() + 8 {
            return Err("invalid packet size".into());
        }

        let id = reader.read_var_int()?;
        if *id != *Self::PACKET_ID {
            return Err("unexpected non ping packet id".into());
        }

        Ok(Self {
            payload: reader.read_i64()?,
        })
    }

    pub fn answer<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_var_int(Self::PACKET_ID.size() + types::VarInt::new(8))?;
        writer.write_var_int(Self::PACKET_ID)?;
        writer.write_i64(self.payload)?;
        Ok(())
    }
}
