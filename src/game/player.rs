use crate::packets::{LoginRequest, Packet, Ping, ServerDescription, StatusRequest};
use crate::types::VarInt;
use crate::{packets, types};
use anyhow::Result;
use futures::prelude::*;
use piper::{Arc, Mutex};

/// here we use the Arc to get interior mutability
#[derive(Clone)]
pub struct Player {
    read_stream: Arc<Mutex<Box<dyn AsyncRead + Send + Sync + Unpin>>>,
    write_stream: Arc<Mutex<Box<dyn AsyncWrite + Send + Sync + Unpin>>>,
    id: types::VarInt,
}

impl Player {
    pub async fn new(
        mut reader: impl AsyncRead + Send + Sync + Unpin + 'static,
        mut writer: impl AsyncWrite + Send + Sync + Unpin + 'static,
    ) -> Result<Option<Self>> {
        let state = Response::new();
        match state.next(&mut reader, &mut writer).await? {
            Response::Handshake | Response::Finished => panic!("This should never happens"),
            state @ Response::Status => {
                // ignore what happens after a ping has been asked
                let _ = state.next(&mut reader, &mut writer).await;
                return Ok(None);
            }
            state @ Response::Play => {
                state.next(&mut reader, &mut writer).await?;
                return Ok(Some(Self {
                    read_stream: Arc::new(Mutex::new(Box::new(reader))),
                    write_stream: Arc::new(Mutex::new(Box::new(writer))),
                    id: VarInt::default(),
                }));
            }
        }
    }

    pub fn id(&self) -> VarInt {
        self.id
    }

    pub async fn send_packet(&mut self, packet: &(impl Packet + Sync)) -> Result<()> {
        packet.send_packet(&mut self.write_stream).await
    }

    pub async fn run(&mut self) -> ! {
        loop {}
    }
}

enum Response {
    Handshake,
    Status,
    Play,
    Finished,
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

                let ping = Ping::parse(reader).await?;
                ping.send_packet(writer).await?;
                writer.flush().await?;

                Ok(Response::Finished)
            }
            Response::Play => {
                let login_start = LoginRequest::parse(reader).await?;
                login_start.answer(writer).await?;
                writer.flush().await?;
                Ok(Response::Finished)
            }
            Response::Finished => Ok(self),
        }
    }
}
