use crate::types::{self, BoolOption};
use crate::{impl_packet, impl_send, impl_size};

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Slot {
    window: Window,
    index: u16,
    item: BoolOption<Item>,
}

impl Slot {
    pub fn empty(window: Window, index: u16) -> Self {
        Self {
            window,
            index,
            item: BoolOption(None),
        }
    }
}
impl_packet!(Slot, 0x17);

#[derive(Debug, Copy, Clone)]
#[repr(i8)]
pub enum Window {
    Inventory = 0,
}
impl_size!(Window, 1);
impl_send!(Window as i8);

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Item {
    id: types::VarInt,
    count: i8,
    nbt: Vec<u8>,
}
