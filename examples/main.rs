#![allow(unused_must_use, unused_imports)]

use futures::prelude::*;
use std::marker::Unpin;
use std::net::TcpListener;

use anyhow::Result;
use smol::{Async, Task};

use futures::io::BufReader;
use minecrust::game::player::Player;
use minecrust::game::world::World;
use minecrust::packets::play::slot::{Slot, Window};
use minecrust::packets::{Handshake, LoginRequest, Packet, Ping, ServerDescription, StatusRequest};
use minecrust::types::{self, Size};
use piper::{Arc, Mutex};
use std::time::Duration;

fn main() -> ! {
    let (world, new_player) = World::new();
    let world: &'static mut World = Box::leak(Box::new(world));

    let mut server_description = ServerDescription::default();
    server_description.players = (1, 0);
    server_description.description = "Rusty Minecraft Server".to_string();
    server_description.icon = std::fs::read("./examples/assets/server-icon.png").ok();

    let listener = Async::<TcpListener>::bind("127.0.0.1:25565").unwrap();
    let mut incoming = listener.incoming();
    smol::run(async move {
        Task::spawn(world.run(Duration::from_secs(1))).detach();

        while let Some(stream) = incoming.next().await {
            let stream = Arc::new(stream.unwrap());
            let reader = futures::io::BufReader::new(stream.clone());
            let writer = futures::io::BufWriter::new(stream.clone());
            let player = Player::new(
                reader, writer,
                server_description.clone(),
                world,
            ).await.unwrap();
            if player.is_none() {
                continue;
            }
            let mut player = player.unwrap();

            let new_player = new_player.clone();
            Task::spawn(async move {
                new_player.send(player.clone()).await;
                player.run().await;
            })
            .detach();
        }
    });
    panic!("This should never happens");
}
