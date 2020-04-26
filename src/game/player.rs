use crate::types;
use futures::{AsyncRead, AsyncWrite};
use piper::{Arc, Mutex};
use crate::types::VarInt;
use crate::packets::Packet;
use anyhow::Result;

pub struct Player<S: AsyncRead + AsyncWrite + Unpin + Send> {
    read_stream: Arc<Mutex<S>>,
    write_stream: Arc<Mutex<S>>,
    id: types::VarInt,
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> Player<S> {
    pub fn new(stream: S) -> Self {
        let stream = Arc::new(Mutex::from(stream));
        Self {
            read_stream: stream.clone(),
            write_stream: stream,
            id: VarInt::default(),
        }
    }

    pub async fn send_packet(&mut self, packet: &(impl Packet + Sync)) -> Result<()> {
        packet.send_packet(&mut self.write_stream).await
    }
}

// Player::<Stdin, TcpStream>::new::<File>(f)
// pub struct Player<R: AsyncRead, W: AsyncWrite> {
//     read_stream: Arc<Mutex<R>>,
//     write_stream: Arc<Mutex<W>>,
// }
//
// impl<R: AsyncRead, W: AsyncWrite> Player<R, W> {
//     pub fn new<S: AsyncRead + AsyncWrite>(stream: S) -> Player<R, W> {
//         let stream = Arc::new(Mutex::from(stream));
//         Player {
//             read_stream: stream.clone(),
//             write_stream: stream.clone(),
//         }
//     }
// }