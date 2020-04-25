use std::marker::Unpin;
use std::ops::Deref;
use std::string::String as StdString;

use crate::stream::ReadExtension;
use crate::types::{self, Send, Size};

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::prelude::*;
use futures::AsyncWriteExt;

#[derive(Debug, Clone)]
pub struct String(StdString);

impl String {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }

    pub async fn parse<R: AsyncRead + Unpin + std::marker::Send>(reader: &mut R) -> Result<Self> {
        let mut result = StdString::new();
        let size = reader.read_var_int().await?;
        let read = reader
            .take(*size as u64)
            .read_to_string(&mut result)
            .await?;
        if *size as usize != read {
            Err(anyhow!("invalid string size"))
        } else {
            Ok(Self::new(&result))
        }
    }
}

impl Size for String {
    fn size(&self) -> types::VarInt {
        types::VarInt::new(self.0.len() as i32).size() + types::VarInt::new(self.0.len() as i32)
    }
}

#[async_trait]
impl Send for String {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        types::VarInt::new(self.0.len() as i32).send(writer).await?;
        writer.write_all(self.0.as_bytes()).await?;
        Ok(())
    }
}

impl Deref for String {
    type Target = StdString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
