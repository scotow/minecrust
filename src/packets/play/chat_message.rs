use crate::game::player::Player;
use crate::types::{self, chat::Chat, Receive, TAsyncRead, VarInt};
use crate::{impl_packet, impl_send, impl_size};
use anyhow::Result;

pub struct InChatMessage(types::String);

impl InChatMessage {
    pub const PACKET_ID: VarInt = VarInt(0x03);
}

#[async_trait::async_trait]
impl types::FromReader for InChatMessage {
    async fn from_reader<R: TAsyncRead>(reader: &mut R) -> Result<Self> {
        let content = reader.receive().await?;
        Ok(Self(content))
    }
}

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct OutChatMessage {
    content: Chat,
    position: Position,
}
impl_packet!(OutChatMessage, 0x0F);

impl OutChatMessage {
    pub fn new(content: Chat, position: Position) -> Self {
        Self { content, position }
    }

    pub fn from_player_message(from: &Player, message: InChatMessage) -> Self {
        Self::new(Chat::user_message(&from.name(), &message.0), Position::Chat)
    }
}

impl From<InChatMessage> for OutChatMessage {
    fn from(message: InChatMessage) -> Self {
        Self::new(Chat::new(&message.0), Position::default())
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
