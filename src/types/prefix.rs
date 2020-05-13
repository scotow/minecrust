use crate::types::{self, Send, Size, TAsyncWrite};
use anyhow::Result;
use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Default)]
pub struct LengthVec<T>(pub Vec<T>);

impl<T> LengthVec<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn from(vec: Vec<T>) -> Self {
        Self(vec)
    }
}

impl<T> Index<usize> for LengthVec<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<T> IndexMut<usize> for LengthVec<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<T> std::ops::Deref for LengthVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for LengthVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Size + Sync> Size for LengthVec<T> {
    fn size(&self) -> types::VarInt {
        types::VarInt::new(self.0.len() as i32).size() + self.0.size()
    }
}

#[async_trait::async_trait]
impl<T: Send + Sync + std::marker::Send> Send for LengthVec<T> {
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
        types::VarInt::new(self.0.len() as i32).send(writer).await?;
        self.0.send(writer).await
    }
}

#[derive(Debug, Clone, Default)]
pub struct SizeVec<T>(pub Vec<T>);

impl<T> SizeVec<T> {
    pub fn new() -> Self {
        Self(Vec::new())
    }
}

impl<T> std::ops::Deref for SizeVec<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> std::ops::DerefMut for SizeVec<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Size + Sync> Size for SizeVec<T> {
    fn size(&self) -> types::VarInt {
        let inner_size = self.0.size();
        inner_size.size() + inner_size
    }
}

#[async_trait::async_trait]
impl<T: Size + Send + Sync + std::marker::Send> Send for SizeVec<T> {
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
        self.0.size().send(writer).await?;
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
impl<T> Send for BoolOption<T>
where
    T: Send + Sync + std::marker::Send,
{
    async fn send<Write: TAsyncWrite>(&self, writer: &mut Write) -> Result<()> {
        self.0.is_some().send(writer).await?;
        if let Some(item) = &self.0 {
            item.send(writer).await?;
        }
        Ok(())
    }
}
