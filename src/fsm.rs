use crate::packets::{Handshake, LoginRequest, Packet, Ping, StatusRequest};
use crate::types::{Receive, ServerDescription, TAsyncRead, TAsyncWrite};
use anyhow::Result;
use futures::prelude::*;

pub struct Fsm<'a> {
    server_description: &'a ServerDescription,
    state: State,
    reader: &'a mut dyn TAsyncRead,
    writer: &'a mut dyn TAsyncWrite,
}

impl<'a> Fsm<'a> {
    pub fn from_rw(
        server_description: &'a ServerDescription,
        reader: &'a mut dyn TAsyncRead,
        writer: &'a mut dyn TAsyncWrite,
    ) -> Self {
        Self {
            server_description,
            state: State::new(),
            reader,
            writer,
        }
    }

    pub async fn next_state(&mut self) -> Result<State> {
        let state = self
            .state
            .clone()
            .next(&self.server_description, &mut self.reader, &mut self.writer)
            .await?;
        Ok(state)
    }

    pub async fn play(mut self) -> Result<Option<LoginRequest>> {
        loop {
            self.state = match self.next_state().await? {
                State::Finished(login) => return Ok(Some(login)),
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

impl Default for State {
    fn default() -> Self {
        State::Handshake
    }
}

impl State {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn next(
        self,
        server_description: &ServerDescription,
        mut reader: &mut dyn TAsyncRead,
        mut writer: &mut dyn TAsyncWrite,
    ) -> Result<Self> {
        match self {
            State::Handshake => {
                let handshake: Handshake = reader.receive().await?;

                Ok(match *handshake.next_state {
                    1 => State::Status,
                    2 => State::Play,
                    _ => unreachable!(),
                })
            }
            State::Status => {
                let status_request: StatusRequest = reader.receive().await?;
                status_request
                    .answer(&mut writer, &server_description)
                    .await?;
                writer.flush().await?;

                let ping: Ping = reader.receive().await?;
                ping.send_packet(&mut writer).await?;
                writer.flush().await?;

                Ok(State::StatusFinished)
            }
            State::Play => {
                let login_start: LoginRequest = reader.receive().await?;
                login_start.answer(&mut writer).await?;
                writer.flush().await?;
                Ok(State::Finished(login_start))
            }
            State::StatusFinished => Ok(self),
            State::Finished(_) => Ok(self),
        }
    }
}
