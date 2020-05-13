pub mod handshake;
pub mod login;
pub mod play;
pub mod status;

pub use handshake::*;
pub use login::*;
pub use status::*;

use crate::types::{self, Send, Size, TAsyncWrite};
use anyhow::Result;

#[async_trait::async_trait]
pub trait Packet: Size + Send {
    const PACKET_ID: types::VarInt;

    async fn send_packet<W: TAsyncWrite>(
        &self,
        writer: &mut W,
    ) -> Result<()> {
        (Self::PACKET_ID.size() + self.size()).send(writer).await?;
        Self::PACKET_ID.send(writer).await?;
        self.send(writer).await?;
        Ok(())
    }
}

#[macro_export]
macro_rules! impl_packet {
    ($type:ty, $id:expr) => {
        impl $crate::packets::Packet for $type {
            const PACKET_ID: $crate::types::VarInt = $crate::types::VarInt($id);
        }
    };
}
