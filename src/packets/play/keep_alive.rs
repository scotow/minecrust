use futures::AsyncWrite;
use std::fmt::{self, Display, Formatter};
use std::marker;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;

use crate::types::{self, Send, Size, VarInt};
use crate::{impl_packet, impl_send, impl_size};

#[derive(macro_derive::Size, macro_derive::Send, Debug)]
pub struct KeepAlive(i64);
impl_packet!(KeepAlive, 0x21);

impl KeepAlive {
    pub fn new() -> Self {
        Self(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap().as_secs() as i64
        )
    }
}