use crate::types::{LengthVec, VarInt};
use crate::game::player::{Player, Info};
use crate::types;
use futures::AsyncWrite;
use anyhow::Result;
use crate::{impl_size, impl_packet};

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct PlayerInfo<'a> {
    action: Action,
    info: LengthVec<&'a Info>,
}
impl_packet!(PlayerInfo<'_>, 0x34);

impl<'a> PlayerInfo<'a> {
    pub fn new(action: Action, info: Vec<&'a Info>) -> Self {
        Self {
            action,
            info: LengthVec(info),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u8)]
pub enum Action {
    Add = 0,
    UpdateGameMode,
    UpdateLatency,
    UpdateDisplayName,
    Remove,
}
impl_size!(Action, 1);

#[async_trait::async_trait]
impl types::Send for Action {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        VarInt(*self as i32).send(writer).await
    }
}