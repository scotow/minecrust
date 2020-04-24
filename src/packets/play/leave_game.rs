use crate::types;

#[derive(macro_derive::Size, Debug)]
pub struct LeaveGame {
    pub id: u64,
    pub foo: types::VarInt,
    pub name: types::String,
}
