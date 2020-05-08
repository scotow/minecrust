use crate::game::player::Player;
use anyhow::Result;
use crate::packets::play::keep_alive::KeepAlive;
use crate::types;
use futures_timer::Delay;
use piper::{Lock, Arc};
use std::collections::HashMap;
use std::time::Duration;
use crate::packets::Packet;
use crate::packets::play::player_info::{PlayerInfo, Action};
use crate::packets::play::spawn_player::SpawnPlayer;
use crate::packets::play::destroy_entity::DestroyEntity;

pub struct World {
    players: Lock<HashMap<types::VarInt, Arc<Player>>>,
}

impl World {
    pub fn new() -> Self {
            Self {
                players: Lock::new(HashMap::new()),
            }
    }

    pub async fn run(&self, heartbeat: Duration) {
        let keep_alive_loop = async {
            loop {
                Delay::new(heartbeat).await;
                let keep_alive_packet = KeepAlive::new();
                self.broadcast_packet(&keep_alive_packet).await;
            }
        };

        // Run forever.
        let _ = futures::join!(keep_alive_loop);
    }

    pub async fn broadcast_packet(&self, packet: &(impl Packet + Sync)) {
        // TODO: Use a async RW lock.
        let mut players = self.players.lock().await;
        let iter = players.values_mut().map(|player| {
            player.send_packet(packet)
        });
        futures::future::join_all(iter).await;
    }

    pub async fn broadcast_packet_except(&self, packet: &(impl Packet + Sync), except: &Player) {
        let mut players = self.players.lock().await;
        let iter = players.values_mut()
            .filter(|player| player.id() != except.id())
            .map(|player| { player.send_packet(packet) });

        futures::future::join_all(iter).await;
    }

    async fn add_player(&self, mut player: Arc<Player>) {
        let id = player.id();
        let player_info = player.info().clone();
        let mut players = self.players.lock().await;

        // Player info.
        let new_player_info = PlayerInfo::new(Action::Add, vec![player.info()]);
        self.broadcast_packet(&new_player_info).await;

        let mut all_info = players.values_mut().map(|p| p.info()).collect::<Vec<_>>();
        all_info.push(player.info());
        let all_players_info = PlayerInfo::new(Action::Add, all_info);
        player.send_packet(&all_players_info).await;

        // Player spawn.
        let spawn_player = SpawnPlayer::new(&player);
        self.broadcast_packet(&spawn_player).await;

        // Spawn other players in new player game.
        for other in players.values_mut() {
            if other.id() == player.id() { continue }
            let spawn_other = SpawnPlayer::new(&other);
            player.send_packet(&spawn_other).await;
        }

        // Insert player to global list.
        self.players.lock().await.insert(id, player);
    }

    pub async fn remove_player(&self, player: &Player) {
        let id = player.id();
        self.players.lock().await.remove(&id);

        let destroy = DestroyEntity::single(id);
        self.broadcast_packet(&destroy).await;

        let info = PlayerInfo::new(Action::Remove, vec![player.info()]);
        self.broadcast_packet(&info).await;
    }
}