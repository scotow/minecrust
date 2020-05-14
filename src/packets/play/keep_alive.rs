use std::time::{SystemTime, UNIX_EPOCH};

use crate::impl_packet;

#[derive(macro_derive::Size, macro_derive::Send, Debug)]
pub struct KeepAlive(i64);
impl_packet!(KeepAlive, 0x21);

impl KeepAlive {
    pub fn new() -> Self {
        Default::default()
    }
}

impl Default for KeepAlive {
    fn default() -> Self {
        Self(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        )
    }
}
