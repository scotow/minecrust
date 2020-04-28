use crate::packets;



use crate::packets::{LoginRequest, Packet, Ping, ServerDescription, StatusRequest};
use anyhow::Result;


use futures::prelude::*;


pub enum State {
    Handshake(ServerDescription),
    Status(ServerDescription),
    Play,
    Finished,
}

impl State {
    pub fn new(server_description: ServerDescription) -> Self {
        State::Handshake(server_description)
    }

    pub async fn next(
        self,
        reader: &mut (impl AsyncRead + Send + Sync + Unpin),
        writer: &mut (impl AsyncWrite + Send + Sync + Unpin),
    ) -> Result<Self> {
        match self {
            State::Handshake(server_description) => {
                let handshake = packets::Handshake::parse(reader).await?;

                Ok(match *handshake.next_state {
                    1 => State::Status(server_description),
                    2 => State::Play,
                    _ => unreachable!(),
                })
            }
            State::Status(server_description) => {
                let status_request = StatusRequest::parse(reader).await?;
                status_request.answer(writer, &server_description).await?;
                writer.flush().await?;

                let ping = Ping::parse(reader).await?;
                ping.send_packet(writer).await?;
                writer.flush().await?;

                Ok(State::Finished)
            }
            State::Play => {
                let login_start = LoginRequest::parse(reader).await?;
                login_start.answer(writer).await?;
                writer.flush().await?;
                Ok(State::Finished)
            }
            State::Finished => Ok(self),
        }
    }
}
