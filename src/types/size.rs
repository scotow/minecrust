use crate::types;
use crate::types::VarInt;

pub trait Size {
    fn size(&self) -> types::VarInt;
}

#[macro_export]
macro_rules! impl_size {
    ($type:ty, $size:literal) => {
        impl $crate::types::Size for $type {
            fn size(&self) -> VarInt {
                VarInt::new($size)
            }
        }
    };
}

impl_size!(bool, 1);

impl_size!(u8, 1);
impl_size!(u16, 2);
impl_size!(u32, 4);
impl_size!(u64, 8);

impl_size!(i8, 1);
impl_size!(i16, 2);
impl_size!(i32, 4);
impl_size!(i64, 8);
