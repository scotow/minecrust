use crate::impl_packet;
use crate::types::LengthVec;

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Recipes(LengthVec<Recipe>);
impl_packet!(Recipes, 0x5B);

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Recipe {}
