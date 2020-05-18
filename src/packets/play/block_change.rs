use crate::packets::play::Block;
use crate::types::{BlockPosition, VarInt};

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct BlockChange {
    position: BlockPosition,
    block: VarInt,
}

impl BlockChange {
    pub fn new(position: BlockPosition, block: Block) -> Self {
        Self {
            position,
            block: VarInt(block as i32),
        }
    }
}
crate::impl_packet!(BlockChange, 0x0C);
