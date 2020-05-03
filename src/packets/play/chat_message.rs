use crate::types;
use crate::{impl_packet, impl_size, impl_send};

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct ChatMessage {
    content: types::Chat,
    position: Position,
}
impl_packet!(ChatMessage, 0x0F);

impl ChatMessage {
    pub fn new(content: types::Chat, position: Position) -> Self {
        Self {
            content,
            position,
        }
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