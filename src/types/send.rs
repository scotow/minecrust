use anyhow::Result;
use async_trait::async_trait;
use futures::prelude::*;
use futures::AsyncWriteExt;

#[async_trait]
pub trait Send {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()>;
}

macro_rules! _impl_send_primitives {
    ($( $type:tt ),+) => {
        $(
            #[async_trait]
            impl Send for $type {
                async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
                    writer.write_all(&self.to_be_bytes()).await?;
                    Ok(())
                }
            }
        )+
    }
}

_impl_send_primitives!(u8, u16, u32, u64, i8, i16, i32, i64);

#[macro_export]
macro_rules! impl_send {
    ($type:tt as $repr:tt) => {
        #[async_trait::async_trait]
        impl $crate::types::Send for $type {
            async fn send<W: futures::prelude::AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> anyhow::Result<()> {
                use $crate::types::Send;
                // <$repr>::send(*self as $repr, writer).await
                (*self as $repr).send(writer).await
            }
        }
    };
    // ($type:tt) => {
    //     #[async_trait::async_trait]
    //     impl $crate::types::Send for $type {
    //         async fn send<W: futures::prelude::AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> anyhow::Result<()> {
    //             concat_idents!(writer.write_, $type)(*self).await
    //         }
    //     }
    // };
}

#[async_trait]
impl Send for bool {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        (*self as u8).send(writer).await
    }
}