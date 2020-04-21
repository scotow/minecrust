use crate::types;
use anyhow::Result;
use minecrust::packets::{Handshake, LoginRequest, PingRequest, ServerDescription, StatusRequest};
use std::io::{Read, Write};

pub enum State {
    Handshake(Handshake),
    Status(StatusRequest),
    Ping(PingRequest),
    Closed,
    Login(LoginRequest),
}

pub struct Fsm {
    pub state: State,
    description: ServerDescription,
}

impl Fsm {
    pub fn init(stream: &mut (impl Read + Write), description: ServerDescription) -> Result<Self> {
        Ok(Self {
            state: State::Handshake(Handshake::parse(stream), description),
        })
    }

    pub fn next(self) -> Result<Self> {
        let next_state = match self.state {
            State::Handshake(packet) => match *handshake.next_state {
                1 => State::Status(StatusRequest::parse(stream)?),
                2 => State::Login(LoginRequest::parse(stream)?),
            },
            State::Status(packet) => {
                packet.answer(stream, self.description)?;
                stream.flush()?;
                State::Ping(PingRequest::parse(stream)?)
            }
            State::Ping(packet) => {
                packet.answer(stream)?;
                stream.flush();
                State::Closed
            }
            State::Login(LoginRequest) => {
                packet.answer(stream)?;
                stream.flush();
                State::Closed
            }
        };
        Ok(Self {
            state: next_state?,
            ..self
        })
    }
}
