use crate::impl_packet;
use anyhow::{ensure, Result};

#[derive(Debug, Default, macro_derive::Size, macro_derive::Send)]
pub struct HeldItemSlot(i8);

impl HeldItemSlot {
    pub fn new(slot: i8) -> Result<Self> {
        ensure!((0..=8).contains(&slot), "invalid slot index");
        Ok(Self(slot))
    }
}
impl_packet!(HeldItemSlot, 0x40);
