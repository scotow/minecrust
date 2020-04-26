use crate::packets::{Packet, StatusRequest, Ping, ServerDescription, LoginRequest};
use crate::{types, packets};
use crate::types::VarInt;
use anyhow::Result;
use futures::{AsyncRead, AsyncWrite};
use piper::{Arc, Mutex};
use crate::game::player::Response::Continue;

pub struct Player<R, W> {
    read_stream: Mutex<R>,
    write_stream: Mutex<W>,
    id: types::VarInt,
}

impl<R: AsyncRead, W: AsyncWrite + Send + Unpin> Player<R, W> {
    pub async fn new(mut reader: R, mut writer: W) -> Result<Option<Self>> {
        let mut state = Response::Continue(Handshake);
        loop {
            match state {
                Response::Continue(next) => {
                    let it = next.receive(&mut reader, &mut writer).await?;
                    state = it;
                },
                Response::Status => return Ok(None),
                Response::Play => return Ok(Some(
                    Self {
                        read_stream: Mutex::new(reader),
                        write_stream: Mutex::new(writer),
                        id: VarInt::default(),
                    }
                ))
            }
        }
    }

    pub fn id(&self) -> VarInt {
        self.id
    }

    pub async fn send_packet(&mut self, packet: &(impl Packet + Sync)) -> Result<()> {
        packet.send_packet(&mut self.write_stream).await
    }
}

enum Response<R: AsyncRead, W: AsyncWrite + Send + Unpin> {
    Continue(Box<dyn PacketHandler<R, W>>),
    Status,
    Play,
}

#[async_trait::async_trait]
trait PacketHandler<R: AsyncRead, W: AsyncWrite + Send + Unpin> {
    async fn receive(
        &self,
        reader: &mut (impl AsyncRead + Unpin + Send),
        writer: &mut (impl AsyncWrite + Unpin + Send),
    ) -> Result<Response<R, W>>;
}

struct Handshake<R, W>();

#[async_trait::async_trait]
impl<R: AsyncRead, W: AsyncWrite + Send + Unpin> PacketHandler<R, W> for Handshake<R, W> {
    async fn receive(
        &self,
        reader: &mut (impl AsyncRead + Unpin + Send),
        _writer: &mut (impl AsyncWrite + Unpin + Send),
    ) -> Result<Response<R, W>> {
        let handshake = packets::Handshake::parse(reader).await?;
        println!("{:?}", handshake);

        Ok(
            match *handshake.next_state {
                1 => Response::Continue(Box::new(Status::<R, W>())),
                2 => Response::Continue(Box::new(Login)),
                _ => unreachable!()
            }
        )
    }
}

struct Status<R, W>();

#[async_trait::async_trait]
impl<R: AsyncRead, W: AsyncWrite + Send + Unpin> PacketHandler<R, W> for Status<R, W> {
    async fn receive(
        &self,
        reader: &mut (impl AsyncRead + Unpin + Send),
        writer: &mut (impl AsyncWrite + Unpin + Send),
    ) -> Result<Response<R, W>> {
        let mut server_description = ServerDescription::default();
        server_description.players = (1, 0);
        server_description.description = "Rusty Minecraft Server".to_string();
        server_description.icon = std::fs::read("./examples/assets/server-icon.png").ok();

        let status_request = StatusRequest::parse(reader).await?;
        status_request.answer(writer, server_description).await?;
        // writer.flush().await?;
        println!("Status sent.");

        let ping = Ping::parse(reader).await?;
        ping.send_packet(writer).await?;
        // writer.flush().await?;
        println!("Pong sent.");

        Ok(Response::Status)
    }
}

struct Login<R, W>();

#[async_trait::async_trait]
impl<R: AsyncRead, W: AsyncWrite + Send + Unpin> PacketHandler<R, W> for Login<R, W> {
    async fn receive(
        &self,
        reader: &mut (impl AsyncRead + Unpin + Send),
        writer: &mut (impl AsyncWrite + Unpin + Send),
    ) -> Result<Response<R, W>> {
        let login_start = LoginRequest::parse(reader).await?;
        login_start.answer(writer).await?;
        // stream.flush().await?;
        println!("{:?}", login_start);
        Ok(Response::Play)
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