use crate::game::player::Player;
use crate::packets::play::keep_alive::KeepAlive;
use crate::types;
use futures_timer::Delay;
use piper::{Arc, Mutex, Receiver, Sender, Lock};
use std::collections::HashMap;
use std::time::Duration;
use crate::packets::Packet;

pub struct World {
    players: Lock<HashMap<types::VarInt, Player>>,
    player_receiver: Receiver<Player>,
}

impl World {
    pub fn new() -> (Self, Sender<Player>) {
        let (sender, receiver) = piper::chan(1);
        (
            Self {
                players: Lock::new(HashMap::new()),
                player_receiver: receiver,
            },
            sender,
        )
    }

    pub async fn run(&self, heartbeat: Duration) {
        let player_receiver = &self.player_receiver;
        let keep_alive_loop = async {
            loop {
                Delay::new(heartbeat).await;
                let keep_alive_packet = KeepAlive::new();
                self.broadcast_packet(&keep_alive_packet).await;
            }
        };
        let add_player_loop = async {
            loop {
                let player = player_receiver.recv().await.unwrap();
                let id = player.id();
                self.players.lock().await.insert(id, player);
            }
        };

        // Run forever.
        let _ = futures::join!(keep_alive_loop, add_player_loop);
    }

    pub async fn broadcast_packet(&self, packet: &(impl Packet + Sync)) {
        // TODO: Use a async RW lock.
        let mut players = self.players.lock().await;
        let iter = players.values_mut().map(|player| {
            player.send_packet(packet)
        });
        futures::future::join_all(iter).await;
    }
}