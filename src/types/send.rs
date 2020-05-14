use super::TAsyncWrite;
use anyhow::Result;
use async_trait::async_trait;
use futures::prelude::*;

#[async_trait]
pub trait Send {
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()>;
}

macro_rules! _impl_send_primitives {
    ($( $type:tt ),+) => {
        $(
            #[async_trait]
            impl Send for $type {
                async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
                    writer.write_all(&self.to_be_bytes()).await?;
                    Ok(())
                }
            }
        )+
    }
}

_impl_send_primitives!(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64);

#[macro_export]
macro_rules! impl_send {
    ($type:tt $(as $repr:tt)+) => {
        #[async_trait::async_trait]
        impl $crate::types::Send for $type {
            async fn send<W: $crate::types::TAsyncWrite>(
                &self,
                writer: &mut W,
            ) -> anyhow::Result<()> {
                #[allow(unused_imports)]
                use $crate::types::Send;
                (*self $(as $repr)+).send(writer).await
            }
        }

        #[async_trait::async_trait]
        impl $crate::types::Send for &$type {
            async fn send<W: $crate::types::TAsyncWrite>(
                &self,
                writer: &mut W,
            ) -> anyhow::Result<()> {
                #[allow(unused_imports)]
                use $crate::types::Send;
                (**self $(as $repr)+).send(writer).await
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
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
        (*self as u8).send(writer).await
    }
}

#[async_trait]
impl<T: Send + Sync> Send for Option<T> {
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
        match self {
            Some(t) => t.send(writer).await,
            None => Ok(()),
        }
    }
}

#[async_trait]
impl<T: Send + Sync> Send for [T] {
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
        // TODO: use write_all instead of .iter
        for i in self.iter() {
            i.send(writer).await?;
        }
        Ok(())
    }
}
