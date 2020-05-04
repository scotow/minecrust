use crate::types;
use crate::{impl_packet, impl_size, impl_send};
use crate::types::{VarInt, Chat};
use futures::AsyncRead;
use anyhow::Result;
use crate::stream::ReadExtension;

pub struct InChatMessage(types::String);

impl InChatMessage {
    pub const PACKET_ID: VarInt = VarInt(0x03);

    pub async fn parse<R: AsyncRead + Unpin + std::marker::Send>(reader: &mut R) -> Result<Self> {
        let content = reader.read_string().await?;
        Ok(Self(content))
    }
}

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct OutChatMessage {
    content: types::Chat,
    position: Position,
}
impl_packet!(OutChatMessage, 0x0F);

impl OutChatMessage {
    pub fn new(content: types::Chat, position: Position) -> Self {
        Self {
            content,
            position,
        }
    }
}

impl From<InChatMessage> for OutChatMessage {
    fn from(message: InChatMessage) -> Self {
        Self::new(
            Chat::new(&message.0),
            Position::default(),
        )
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u8)]
pub enum Position {
    Chat = 0,
    SystemMessage = 1,
    GameInfo = 2,
}
impl_size!(Position, 1);
impl_send!(Position as u8);

impl Default for Position {
    fn default() -> Self {
        Position::Chat
    }
}