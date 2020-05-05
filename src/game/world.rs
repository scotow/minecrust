use crate::game::player::Player;
use crate::packets::play::keep_alive::KeepAlive;
use crate::types;
use futures_timer::Delay;
use piper::{Arc, Mutex, Receiver, Sender, Lock};
use std::collections::HashMap;
use std::time::Duration;
use crate::packets::Packet;
use crate::packets::play::player_info::{PlayerInfo, Action};
use crate::packets::play::spawn_player::SpawnPlayer;

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
                self.add_player(
                    player_receiver.recv().await.unwrap()
                ).await;
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

    async fn add_player(&self, mut player: Player) {
        let new_player_info = PlayerInfo::new(Action::Add, vec![player.info()]);
        self.broadcast_packet(&new_player_info).await;

        let spawn_player = SpawnPlayer::new(&player);
        self.broadcast_packet(&spawn_player).await;

        // Insert player to global list.
        let id = player.id();
        self.players.lock().await.insert(id, player.clone());

        // Send everybody else player info.
        let mut players = self.players.lock().await;
        let info = players.values_mut().map(|p| p.info()).collect::<Vec<_>>();
        let all_players_info = PlayerInfo::new(Action::Add, info);
        player.clone().send_packet(&all_players_info).await;
        // player.clone().send_packet(&new_player_info).await;

        // Spawn other players in new player game.
        for other in players.values_mut() {
            if other.id() == player.id() { continue }
            let spawn_other = SpawnPlayer::new(&other);
            player.send_packet(&spawn_other).await;
        }
    }
}