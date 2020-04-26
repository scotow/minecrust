use crate::packets::{LoginRequest, Packet, Ping, ServerDescription, StatusRequest};
use crate::types::VarInt;
use crate::{packets, types};
use anyhow::Result;
use futures::prelude::*;
use piper::Mutex;

pub struct Player {
    read_stream: Mutex<Box<dyn AsyncRead + Send + Sync + Unpin>>,
    write_stream: Mutex<Box<dyn AsyncWrite + Send + Sync + Unpin>>,
    id: types::VarInt,
}

impl Player {
    pub async fn new(
        mut reader: impl AsyncRead + Send + Sync + Unpin + 'static,
        mut writer: impl AsyncWrite + Send + Sync + Unpin + 'static,
    ) -> Result<Option<Self>> {
        let state = Response::new();
        match state.next(&mut reader, &mut writer).await? {
            Response::Handshake => panic!("This should never happens"),
            Response::Status => return Ok(None),
            Response::Play => {
                return Ok(Some(Self {
                    read_stream: Mutex::new(Box::new(reader)),
                    write_stream: Mutex::new(Box::new(writer)),
                    id: VarInt::default(),
                }))
            }
        }
    }

    pub fn id(&self) -> VarInt {
        self.id
    }

    pub async fn send_packet(&mut self, packet: &(impl Packet + Sync)) -> Result<()> {
        packet.send_packet(&mut self.write_stream).await
    }

    pub async fn run(&mut self) {
        loop {}
    }
}

enum Response {
    Handshake,
    Status,
    Play,
}

impl Response {
    pub fn new() -> Self {
        Response::Handshake
    }

    pub async fn next(
        self,
        reader: &mut (impl AsyncRead + Send + Sync + Unpin),
        writer: &mut (impl AsyncWrite + Send + Sync + Unpin),
    ) -> Result<Self> {
        match self {
            Response::Handshake => {
                let handshake = packets::Handshake::parse(reader).await?;
                println!("{:?}", handshake);

                Ok(match *handshake.next_state {
                    1 => Response::Status,
                    2 => Response::Play,
                    _ => unreachable!(),
                })
            }
            Response::Status => {
                let mut server_description = ServerDescription::default();
                server_description.players = (1, 0);
                server_description.description = "Rusty Minecraft Server".to_string();
                server_description.icon = std::fs::read("./examples/assets/server-icon.png").ok();

                let status_request = StatusRequest::parse(reader).await?;
                status_request.answer(writer, &server_description).await?;
                writer.flush().await?;
                println!("Status sent.");

                let ping = Ping::parse(reader).await?;
                ping.send_packet(writer).await?;
                writer.flush().await?;
                println!("Pong sent.");

                Ok(Response::Status)
            }
            Response::Play => {
                let login_start = LoginRequest::parse(reader).await?;
                login_start.answer(writer).await?;
                // stream.flush().await?;
                println!("{:?}", login_start);
                Ok(Response::Play)
            }
        }
    }
}
