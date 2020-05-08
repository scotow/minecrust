use crate::types;
use crate::types::VarInt;
use std::ops::Add;
use piper::{Arc, Mutex};

pub trait Size {
    fn size(&self) -> types::VarInt;
}

#[macro_export]
macro_rules! impl_size {
    ($type:ty, $size:literal) => {
        impl $crate::types::Size for $type {
            fn size(&self) -> $crate::types::VarInt {
                $crate::types::VarInt::new($size)
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

impl_size!(f32, 4);
impl_size!(f64, 8);

impl<T: Size> Size for Option<T> {
    fn size(&self) -> VarInt {
        self.as_ref().map_or(VarInt::new(0), Size::size)
    }
}

impl<T: Size> Size for Vec<T> {
    fn size(&self) -> VarInt {
        self.iter().map(Size::size).fold(VarInt::new(0), Add::add)
    }
}

impl<T: Size> Size for Arc<T> {
    fn size(&self) -> VarInt {
        self.size()
    }
}

impl<T: Size> Size for Mutex<T> {
    fn size(&self) -> VarInt {
        self.lock().size()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vec_size() {
        assert_eq!(*vec![42_i64, -1_i64].size(), 16);

        assert_eq!(*vec![VarInt(0)].size(), 1);
        assert_eq!(*vec![VarInt(0), VarInt(0)].size(), 2);
        assert_eq!(*vec![VarInt(0), VarInt(128)].size(), 3);

        use crate::types::String;
        assert_eq!(*vec![String::new("hello")].size(), 6);
        assert_eq!(
            *vec![String::new("hello"), String::new(&"R".repeat(128))].size(),
            6 + 130
        );
    }
}
