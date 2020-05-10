use crate::packets;
use crate::packets::{LoginRequest, Packet, Ping, ServerDescription, StatusRequest};
use anyhow::Result;
use futures::prelude::*;

pub struct Fsm<'a> {
    server_description: ServerDescription,
    state: State,
    reader: &'a mut Box<dyn AsyncRead + Send + Sync + Unpin>,
    writer: &'a mut Box<dyn AsyncWrite + Send + Sync + Unpin>,
}

impl<'a> Fsm<'a> {
    pub fn from_rw(
        server_description: ServerDescription,
        reader: &'a mut Box<dyn AsyncRead + Send + Sync + Unpin>,
        writer: &'a mut Box<dyn AsyncWrite + Send + Sync + Unpin>,
    ) -> Self {
        Self {
            server_description,
            state: State::new(),
            reader,
            writer,
        }
    }

    pub async fn next(&mut self) -> Result<State> {
        let state = self
            .state
            .clone()
            .next(&self.server_description, &mut self.reader, &mut self.writer)
            .await?;
        Ok(state.clone())
    }

    pub async fn play(mut self) -> Result<Option<LoginRequest>> {
        loop {
            self.state = match self.next().await? {
                State::Finished(login) => return Ok(Some(login.clone())),
                State::StatusFinished => return Ok(None),
                state @ State::Status => {
                    // ignore what happens after a ping has been asked
                    let _ = state
                        .next(&self.server_description, &mut self.reader, &mut self.writer)
                        .await;
                    return Ok(None);
                }
                state => state,
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum State {
    Handshake,
    Status,
    StatusFinished,
    Play,
    Finished(LoginRequest),
}

impl State {
    pub fn new() -> Self {
        State::Handshake
    }

    pub async fn next(
        self,
        server_description: &ServerDescription,
        reader: &mut Box<dyn AsyncRead + Send + Sync + Unpin>,
        writer: &mut Box<dyn AsyncWrite + Send + Sync + Unpin>,
    ) -> Result<Self> {
        match self {
            State::Handshake => {
                let handshake = packets::Handshake::parse(reader).await?;

                Ok(match *handshake.next_state {
                    1 => State::Status,
                    2 => State::Play,
                    _ => unreachable!(),
                })
            }
            State::Status => {
                let status_request = StatusRequest::parse(reader).await?;
                status_request.answer(writer, &server_description).await?;
                writer.flush().await?;

                let ping = Ping::parse(reader).await?;
                ping.send_packet(writer).await?;
                writer.flush().await?;

                Ok(State::StatusFinished)
            }
            State::Play => {
                let login_start = LoginRequest::parse(reader).await?;
                login_start.answer(writer).await?;
                writer.flush().await?;
                Ok(State::Finished(login_start))
            }
            State::StatusFinished => Ok(self),
            State::Finished(_) => Ok(self),
        }
    }
}
