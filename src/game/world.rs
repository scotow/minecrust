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

    /// /!\ we should see what we are doing in case:
    /// 1. Two players have teh same ID
    /// 2. A player disconnect:
    ///     Who is supposed to remove the player from the hashmap?
    ///     Maybe we could use weakref in the HashMap Arc?
    pub async fn add_player(&self, player: Player) -> Result<()>{
        let player = Arc::new(player);
        let new_player_info = PlayerInfo::new(Action::Add, vec![player.info()]);
        self.broadcast_packet(&new_player_info).await;

        let spawn_player = SpawnPlayer::new(&player).await;
        self.broadcast_packet(&spawn_player).await;

        // Insert player to global list.
        let id = player.id();
        // insert is still unstable on the entry
        // What are we doing when two players have the same ID?
        self.players.lock().await.insert(id, player.clone());

        // Send everybody else player info.
        let mut players = self.players.lock().await;
        let info = players.values_mut().map(|p| p.info()).collect::<Vec<_>>();
        let all_players_info = PlayerInfo::new(Action::Add, info);
        player.send_packet(&all_players_info).await?;

        // Spawn other players in new player game.
        for other in players.values_mut() {
            if other.id() == player.id() { continue }
            let spawn_other = SpawnPlayer::new(&other).await;
            player.send_packet(&spawn_other).await?;
        }

        // run while the player is alive
        player.run().await
    }
}
