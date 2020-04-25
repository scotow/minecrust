use crate::types::{self};
use crate::{impl_size, impl_send, impl_packet};

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Chunk {
    data: Vec<u8>
}
impl_packet!(Chunk, 0x22);

impl Chunk {
    pub fn new(path: &str) -> Self {
        Self {
            data: std::fs::read(path).unwrap()
        }
    }
}