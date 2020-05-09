use async_trait::async_trait;
use futures::prelude::*;
use std::marker::Unpin;

use crate::types::*;
use anyhow::Result;

#[async_trait]
pub trait ReadExtension: AsyncRead + Unpin + Sized {
    async fn read_u8(&mut self) -> Result<u8> {
        let mut buff = [0; 1];
        self.read_exact(&mut buff).await?;
        Ok(buff[0])
    }

    async fn read_u16(&mut self) -> Result<u16> {
        let mut buff = [0; 2];
        self.read_exact(&mut buff).await?;
        Ok(u16::from_be_bytes(buff))
    }

    async fn read_u64(&mut self) -> Result<u64> {
        let mut buff = [0; 8];
        self.read_exact(&mut buff).await?;
        Ok(u64::from_be_bytes(buff))
    }

    async fn read_i8(&mut self) -> Result<i8> {
        self.read_u8().await.map(|n| n as i8)
    }

    async fn read_i16(&mut self) -> Result<i16> {
        self.read_u16().await.map(|n| n as i16)
    }

    async fn read_i64(&mut self) -> Result<i64> {
        self.read_u64().await.map(|n| n as i64)
    }

    async fn read_f32(&mut self) -> Result<f32> {
        let mut buff = [0; 4];
        self.read_exact(&mut buff).await?;
        Ok(f32::from_be_bytes(buff))
    }

    async fn read_f64(&mut self) -> Result<f64> {
        let mut buff = [0; 8];
        self.read_exact(&mut buff).await?;
        Ok(f64::from_be_bytes(buff))
    }

    async fn read_bool(&mut self) -> Result<bool> {
        let mut buff = [0; 1];
        self.read_exact(&mut buff).await?;
        Ok(buff[0] != 0)
    }

    async fn read_var_int(&mut self) -> Result<VarInt> {
        VarInt::parse(self).await
    }

    async fn read_string(&mut self) -> Result<String> {
        String::parse(self).await
    }

    async fn read_block_position(&mut self) -> Result<BlockPosition> {
        Ok(self.read_u64().await?.into())
    }
}

impl<R: AsyncRead + Unpin> ReadExtension for R {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::ReadExtension;
    use futures::io::Cursor;
    use futures_await_test::async_test;

    #[async_test]
    async fn read_u8() -> Result<()> {
        let mut buffer = Cursor::new(vec![0xDE, 0xAD]);
        assert_eq!(buffer.read_u8().await?, 0xDE);
        assert_eq!(buffer.read_u8().await?, 0xAD);
        assert!(buffer.read_u8().await.is_err());
        Ok(())
    }

    #[async_test]
    async fn read_u16() -> Result<()> {
        let mut buffer = Cursor::new(vec![0xDE, 0xAD, 0xBE, 0xEF, 0x42]);
        assert_eq!(buffer.read_u16().await?, 0xDEAD);
        assert_eq!(buffer.read_u16().await?, 0xBEEF);
        assert!(buffer.read_u16().await.is_err());
        Ok(())
    }

    #[async_test]
    async fn read_i8() -> Result<()> {
        let mut buffer = Cursor::new(vec![0xFF, 0x7F]);
        assert_eq!(buffer.read_i8().await?, -1);
        assert_eq!(buffer.read_i8().await?, 127);
        assert!(buffer.read_i8().await.is_err());
        Ok(())
    }
}
