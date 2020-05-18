pub mod bit_array;
pub mod chat;
pub mod position;
pub mod prefix;
pub mod receive;
pub mod send;
pub mod server_description;
pub mod size;
pub mod string;
pub mod var_int;

pub use bit_array::*;
pub use position::*;
pub use prefix::*;
pub use receive::{FromReader, Receive};
pub use send::Send;
pub use server_description::*;
pub use size::Size;
pub use string::*;
pub use var_int::*;

use futures::prelude::*;
use piper::Arc;
use std::marker;

/// This trait is equivalent to AsyncRead + Unpin + Send + Sync
/// This mean it can be shared across thread, hence the name T(hread)AsyncRead
pub trait TAsyncRead: AsyncRead + Unpin + marker::Send + Sync {}
impl<R> TAsyncRead for R where R: AsyncRead + Unpin + marker::Send + Sync {}

/// This trait is equivalent to AsyncWrite + Unpin + Send + Sync
/// This mean it can be shared across thread, hence the name T(hread)AsyncWrite
pub trait TAsyncWrite: AsyncWrite + Unpin + marker::Send + Sync {}
impl<W> TAsyncWrite for W where W: AsyncWrite + Unpin + marker::Send + Sync {}

/// This trait is equivalent to AsyncWrite + AsyncRead + Unpin + Send + Sync
/// This mean it can be shared across thread, hence the name T(hread)AsyncStream
pub trait TAsyncStream: TAsyncRead + TAsyncWrite {}
impl<S> TAsyncStream for S where S: TAsyncRead + TAsyncWrite {}

impl dyn TAsyncStream {
    pub fn split<S>(stream: S) -> (ReadHalf<S>, WriteHalf<S>)
    where
        for<'a> &'a S: AsyncRead + AsyncWrite,
    {
        let stream = Arc::new(stream);
        let reader = ReadHalf::new(stream.clone());
        let writer = WriteHalf::new(stream);
        (reader, writer)
    }
}

use futures::io;
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct ReadHalf<R> {
    inner: Arc<R>,
}

impl<R> ReadHalf<R> {
    pub fn new(inner: Arc<R>) -> Self {
        Self { inner }
    }
}

impl<R> AsyncRead for ReadHalf<R>
where
    for<'a> &'a R: AsyncRead,
{
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.inner).poll_read(cx, buf)
    }
}

pub struct WriteHalf<W> {
    inner: Arc<W>,
}

impl<W> WriteHalf<W> {
    pub fn new(inner: Arc<W>) -> Self {
        Self { inner }
    }
}

impl<W> AsyncWrite for WriteHalf<W>
where
    for<'a> &'a W: AsyncWrite,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        Pin::new(&mut &*self.inner).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self.inner).poll_flush(cx)
    }

    fn poll_close(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut &*self.inner).poll_close(cx)
    }
}
