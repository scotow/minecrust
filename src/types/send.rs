use crate::stream::WriteExtension;
use anyhow::Result;
use async_trait::async_trait;
use futures::prelude::*;

#[async_trait]
pub trait Send {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()>;
}

#[async_trait]
impl Send for u8 {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        writer.write_u8(*self).await
    }
}

#[async_trait]
impl Send for i32 {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        writer.write_i32(*self).await
    }
}

#[async_trait]
impl Send for i64 {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        writer.write_i64(*self).await
    }
}

#[async_trait]
impl Send for bool {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        writer.write_bool(*self).await
    }
}
