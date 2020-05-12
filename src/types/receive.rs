use super::*;
use anyhow::Result;

#[async_trait::async_trait]
pub trait FromReader: Sized {
    async fn from_reader(reader: &mut impl TAsyncRead) -> Result<Self>;
}

macro_rules! _impl_read_primitives {
    ($( $type:tt => $size:expr),+) => {
        $(
            #[async_trait::async_trait]
            impl FromReader for $type {
                async fn from_reader(reader: &mut impl TAsyncRead) -> Result<Self> {
                    let mut buff = [0; $size];
                    reader.read_exact(&mut buff).await?;
                    Ok($type::from_be_bytes(buff))
                }
            }
        )+
    }
}

_impl_read_primitives! {
    u8 => 1, u16 => 2, u32 => 4, u64 => 8,
    i8 => 1, i16 => 2, i32 => 4, i64 => 8,
    f32 => 4, f64 => 8
}

#[async_trait::async_trait]
pub trait Receive {
    async fn receive<T: FromReader>(&mut self) -> Result<T>;
}

#[async_trait::async_trait]
impl<Reader> Receive for Reader
where
    Reader: TAsyncRead + Sized,
{
    async fn receive<T: FromReader>(&mut self) -> Result<T> {
        T::from_reader(self).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stream::FromReader;
    use futures::io::Cursor;
    use futures_await_test::async_test;

    #[async_test]
    async fn read_u8() -> Result<()> {
        let mut buffer = Cursor::new(vec![0xDE, 0xAD]);
        assert_eq!(u8::from_reader(&mut buffer).await?, 0xDE);
        assert_eq!(buffer.receive::<u8>().await?, 0xAD);
        assert!(buffer.receive::<u8>().await.is_err());
        Ok(())
    }

    #[async_test]
    async fn read_u16() -> Result<()> {
        let mut buffer = Cursor::new(vec![0xDE, 0xAD, 0xBE, 0xEF, 0x42]);
        assert_eq!(buffer.receive::<u16>().await?, 0xDEAD);
        assert_eq!(buffer.receive::<u16>().await?, 0xBEEF);
        assert!(buffer.receive::<u16>().await.is_err());
        Ok(())
    }

    #[async_test]
    async fn read_i8() -> Result<()> {
        let mut buffer = Cursor::new(vec![0xFF, 0x7F]);
        assert_eq!(buffer.receive::<i8>().await?, -1);
        assert_eq!(buffer.receive::<i8>().await?, 127);
        assert!(buffer.receive::<i8>().await.is_err());
        Ok(())
    }
}
