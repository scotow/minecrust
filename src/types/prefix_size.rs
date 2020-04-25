use crate::types::{self, Size, Send};
use futures::AsyncWrite;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct SizeVec<T: Sized>(pub Vec<T>);

impl<T: Size + Sync> Size for SizeVec<T> {
    fn size(&self) -> types::VarInt {
        types::VarInt::new(self.0.len() as i32).size() + self.0.size()
    }
}

#[async_trait::async_trait]
impl<T: Send + Sync + std::marker::Send> Send for SizeVec<T> {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        types::VarInt::new(self.0.len() as i32).send(writer).await?;
        self.0.send(writer).await
    }
}

#[derive(Debug, Copy, Clone)]
pub struct BoolOption<T>(pub Option<T>);

impl<T: Size + Sync> Size for BoolOption<T> {
    fn size(&self) -> types::VarInt {
        bool::default().size() + self.0.size()
    }
}

#[async_trait::async_trait]
impl<T: Send + Sync + std::marker::Send> Send for BoolOption<T> {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        self.0.is_some().send(writer).await?;
        if let Some(item) = &self.0 {
            item.send(writer).await?;
        }
        Ok(())
    }
}