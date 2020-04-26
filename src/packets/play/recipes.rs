use crate::impl_packet;
use crate::types::SizeVec;

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Recipes(SizeVec<Recipe>);
impl_packet!(Recipes, 0x5B);

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Recipe {}
