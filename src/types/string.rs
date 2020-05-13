use crate::types::{self, FromReader, Receive, Send, Size, TAsyncRead, TAsyncWrite};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use futures::prelude::*;
use futures::AsyncWriteExt;
use std::ops::Deref;
use std::string::String as StdString;

#[derive(Debug, Clone)]
pub struct String(StdString);

impl String {
    pub fn new(s: &str) -> Self {
        Self(s.to_string())
    }

    /// TODO: delete this in favor of the ReadExtension
    pub async fn parse(reader: &mut impl TAsyncRead) -> Result<Self> {
        let mut result = StdString::new();
        let size = reader.receive::<types::VarInt>().await?;
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

#[async_trait::async_trait]
impl FromReader for String {
    async fn from_reader<R: TAsyncRead>(reader: &mut R) -> Result<Self> {
        String::parse(reader).await
    }
}

impl Size for String {
    fn size(&self) -> types::VarInt {
        types::VarInt::new(self.0.len() as i32).size() + types::VarInt::new(self.0.len() as i32)
    }
}

#[async_trait]
impl Send for String {
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
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

impl From<StdString> for String {
    fn from(s: StdString) -> Self {
        Self(s)
    }
}

impl From<&str> for String {
    fn from(s: &str) -> Self {
        Self::new(s)
    }
}
