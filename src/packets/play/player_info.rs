use crate::types::{LengthVec, VarInt, Size, Send, TAsyncWrite};
use crate::game::player::Info;
use crate::types;
use anyhow::Result;
use crate::{impl_size, impl_packet};


#[derive(Debug)]
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

impl<'a> Size for PlayerInfo<'a> {
    fn size(&self) -> VarInt {
        self.action.size() +
            match self.action {
                Action::Add => self.info.size(),
                Action::UpdateGameMode => unimplemented!(),
                Action::UpdateLatency => unimplemented!(),
                Action::UpdateDisplayName => unimplemented!(),
                Action::Remove => {
                    let length = self.info.len() as i32;
                    VarInt(length + length * 16)
                }
            }
    }
}

#[async_trait::async_trait]
impl<'a> Send for PlayerInfo<'a> {
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
        self.action.send(writer).await?;
        match self.action {
            Action::Add => self.info.send(writer).await?,
            Action::UpdateGameMode => unimplemented!(),
            Action::UpdateLatency => unimplemented!(),
            Action::UpdateDisplayName => unimplemented!(),
            Action::Remove => {
                VarInt(self.info.len() as i32).send(writer).await?;
                for info in self.info.iter() {
                    info.uuid().send(writer).await?;
                }
            }
        }
        Ok(())
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
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
        VarInt(*self as i32).send(writer).await
    }
}
