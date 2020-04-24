#![allow(unused_must_use, unused_imports)]

use futures::prelude::*;
use std::marker::Unpin;
use std::net::TcpListener;

use anyhow::Result;
use smol::{Async, Task};

use minecrust::packets::play::join_game::{JoinGame, Dimension};
use minecrust::packets::{Handshake, LoginRequest, PingRequest, ServerDescription, StatusRequest};
use minecrust::stream::ReadExtension;
use minecrust::types::{self, Size};

fn main() {
    let it = minecrust::packets::play::LeaveGame {
        id: 0,
        foo: types::VarInt::new(300),
        name: types::String::new("hello, world!"),
    };
    // dbg!(it.size());

    let mut server_description: ServerDescription = Default::default();
    server_description.players = (1, 0);
    server_description.description = "Rusty Minecraft Server".to_string();
    server_description.icon = std::fs::read("./examples/assets/server-icon.png").ok();

    let server_description: &'static ServerDescription = Box::leak(Box::new(server_description));
    let listener = Async::<TcpListener>::bind("127.0.0.1:25565").unwrap();
    let mut incoming = listener.incoming();
    smol::run(async {
        while let Some(stream) = incoming.next().await {
            let stream = stream.unwrap();
            Task::spawn(handle_connexion(stream, server_description))
                .unwrap()
                .detach();
        }
    });
}

async fn handle_connexion(
    mut stream: (impl AsyncRead + AsyncWrite + Unpin + Send),
    server_description: &ServerDescription,
) -> Result<()> {
    let handshake = Handshake::parse(&mut stream).await.unwrap();
    println!("{:?}", handshake);

    match *handshake.next_state {
        1 => handle_status(&mut stream, server_description).await,
        2 => {
            handle_login(&mut stream).await.unwrap();
            handle_play(&mut stream).await
        }
        _ => unreachable!(),
    }
    .unwrap();
    Ok(())
}

async fn handle_status(
    stream: &mut (impl AsyncRead + AsyncWrite + Unpin + Send),
    server_description: &ServerDescription,
) -> Result<()> {
    let status_request = StatusRequest::parse(stream).await?;
    status_request.answer(stream, server_description).await?;
    stream.flush().await?;
    println!("Status sent.");

    let ping_request = PingRequest::parse(stream).await?;
    ping_request.answer(stream).await?;
    stream.flush().await?;
    println!("Pong sent.");
    Ok(())
}

async fn handle_login(stream: &mut (impl AsyncRead + AsyncWrite + Unpin + Send)) -> Result<()> {
    let login_start = LoginRequest::parse(stream).await?;
    login_start.answer(stream).await?;
    stream.flush().await?;
    println!("{:?}", login_start);
    Ok(())
}

async fn handle_play(stream: &mut (impl AsyncRead + AsyncWrite + Unpin + Send)) -> Result<()> {
    let join_game = JoinGame::default();
    join_game.send_packet(stream).await?;
    stream.flush().await?;

    let mut buf = Vec::new();
    stream.read_to_end(&mut buf).await;
    dbg!(buf);
    Ok(())
}
