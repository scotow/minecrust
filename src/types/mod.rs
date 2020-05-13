pub mod bit_array;
pub mod chat;
pub mod position;
pub mod prefix;
pub mod receive;
pub mod send;
pub mod size;
pub mod string;
pub mod var_int;

pub use bit_array::*;
pub use position::*;
pub use prefix::*;
pub use receive::{FromReader, Receive};
pub use send::Send;
pub use size::Size;
pub use string::*;
pub use var_int::*;

use futures::prelude::*;
use std::marker;

/// This trait is equivalent to AsyncRead + Unpin + Send + Sync
/// This mean it can be shared across thread, hence the name T(hread)AsyncRead
pub trait TAsyncRead: AsyncRead + Unpin + marker::Send + Sync {}
impl<R> TAsyncRead for R where R: AsyncRead + Unpin + marker::Send + Sync {}

/// This trait is equivalent to AsyncWrite + Unpin + Send + Sync
/// This mean it can be shared across thread, hence the name T(hread)AsyncWrite
pub trait TAsyncWrite: AsyncWrite + Unpin + marker::Send + Sync {}
impl<R> TAsyncWrite for R where R: AsyncWrite + Unpin + marker::Send + Sync {}
