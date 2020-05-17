#![allow(unused_must_use, unused_imports)]
use anyhow::Result;
use futures::io::BufReader;
use futures::prelude::*;
use minecrust::game::map::generator::FlatChunkGenerator;
use minecrust::game::{Player, ServerBuilder, World};
use minecrust::packets::play::slot::{Slot, Window};
use minecrust::packets::{Handshake, LoginRequest, Packet, Ping, ServerDescription, StatusRequest};
use minecrust::types::{self, Size};
use piper::{Arc, Mutex};
use smol::{Async, Task};
use std::marker::Unpin;
use std::net::TcpListener;
use std::time::Duration;

fn main() -> ! {
    let world = ServerBuilder::new()
        .with_players((1, 0))
        .with_description("Rusty Minecraft Server".into())
        .with_icon_from_path("./examples/assets/server-icon.png")
        .unwrap()
        .build_leak(FlatChunkGenerator::new());

    let world = smol::block_on(world);
    eprintln!("World map generated.");

    let world: &'static World = Box::leak(Box::new(world));

    let listener = Async::<TcpListener>::bind("127.0.0.1:25565").unwrap();
    let mut incoming = listener.incoming();
    smol::run(async move {
        Task::spawn(world.run(Duration::from_secs(1))).detach();

        while let Some(stream) = incoming.next().await {
            let stream = Arc::new(stream.unwrap());
            let reader = futures::io::BufReader::new(stream.clone());
            let writer = futures::io::BufWriter::new(stream.clone());

            Task::spawn(async move {
                world.handle_connection(reader, writer).await;
            })
            .detach();
        }
    });
    panic!("This should never happens");
}
