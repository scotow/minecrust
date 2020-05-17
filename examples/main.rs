#![allow(unused_must_use, unused_imports)]
use anyhow::Result;
use futures::io::BufReader;
use futures::prelude::*;
use minecrust::game::map::generator::FlatChunkGenerator;
use minecrust::game::player::Player;
use minecrust::game::world::World;
use minecrust::packets::play::slot::{Slot, Window};
use minecrust::packets::{Handshake, LoginRequest, Packet, Ping, ServerDescription, StatusRequest};
use minecrust::types::{self, Size};
use piper::{Arc, Mutex};
use smol::{Async, Task};
use std::marker::Unpin;
use std::net::TcpListener;
use std::time::Duration;

fn main() -> ! {
    let mut server_description = ServerDescription::default();
    server_description.players = (1, 0);
    server_description.description = "Rusty Minecraft Server".to_string();
    server_description.icon = std::fs::read("./examples/assets/server-icon.png").ok();
    let server_description: &'static ServerDescription = Box::leak(Box::new(server_description));

    let generator = FlatChunkGenerator::new();
    let world = smol::block_on(World::new(server_description, generator));
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
            let player = Player::new(reader, writer, server_description, world)
                .await
                .unwrap();
            if player.is_none() {
                continue;
            }
            let player = player.unwrap();

            Task::spawn(async move {
                world.add_player(player).await;
            })
            .detach();
        }
    });
    panic!("This should never happens");
}
