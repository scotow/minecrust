use std::io::{Read, Write};
use std::ops::{Deref};
use std::string::String as StdString;

use crate::stream::{ReadExtension, WriteExtension};
use crate::error::*;
use crate::types;

#[derive(Debug, Clone)]
pub struct String(StdString);

impl String {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }

    pub fn parse<R: Read>(reader: &mut R) -> Result<Self> {
        let mut result = StdString::new();
        let size = reader.read_var_int()?;
        let read = reader.take(*size as u64).read_to_string(&mut result)?;
        if *size as usize != read {
            Err("invalid string size".into())
        } else {
            Ok(Self::new(&result))
        }
    }

    pub fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_var_int(types::VarInt::new(self.0.len() as i32))?;
        writer.write_all(self.0.as_bytes())?;
        Ok(())
    }

    pub fn size(&self) -> types::VarInt {
        types::VarInt::new(self.0.len() as i32).size() + types::VarInt::new(self.0.len() as i32)
    }
}

impl Deref for String {
    type Target = StdString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}