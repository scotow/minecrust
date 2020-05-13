use crate::types::{LengthVec, VarInt};

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct DestroyEntity(LengthVec<VarInt>);

impl DestroyEntity {
    pub fn single(id: VarInt) -> Self {
        Self(LengthVec::from(vec![id]))
    }
}
crate::impl_packet!(DestroyEntity, 0x38);
