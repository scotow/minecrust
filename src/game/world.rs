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
use crate::packets::play::join_game::JoinGame;
use crate::packets::play::chat_message::{OutChatMessage, Position};
use crate::types::chat::{Chat};


use crate::game::map::Map;

pub struct World {
    players: Lock<HashMap<types::VarInt, Arc<Player>>>,
    pub map: Map
}

impl World {
    pub async fn new() -> Self {
        Self {
            players: Lock::new(HashMap::new()),
            map: Map::new().await
        }
    }

    pub async fn run(&self, heartbeat: Duration) {
        let keep_alive_loop = async {
            loop {
                Delay::new(heartbeat).await;
                let keep_alive_packet = KeepAlive::new();
                let _ = self.broadcast_packet(&keep_alive_packet).await;
            }
        };

        // Run forever.
        let _ = futures::join!(keep_alive_loop);
    }

    pub async fn broadcast_packet(&self, packet: &(impl Packet + Sync)) -> Result<()> {
        // TODO: Use a async RW lock.
        let mut players = self.players.lock().await;
        let iter = players.values_mut().map(|player| {
            player.send_packet(packet)
        });
        futures::future::join_all(iter).await;
        Ok(())
    }

    pub async fn broadcast_packet_except(&self, packet: &(impl Packet + Sync), except: &Player) -> Result<()> {
        let mut players = self.players.lock().await;
        let iter = players.values_mut()
            .filter(|player| &***player != except)
            .map(|player| { player.send_packet(packet) });

        futures::future::join_all(iter).await;
        Ok(())
    }

    pub async fn add_player(&self, player: Player) {
        let player = Arc::new(player);
        let id = player.id();

        // Insert player to global map.
        self.players.lock().await.insert(id, Arc::clone(&player));

        if self._add_player(Arc::clone(&player)).await.is_err() {
            self.remove_player(&*player).await.unwrap();
        }
    }

    // TODO: Find better name.
    async fn _add_player(&self, player: Arc<Player>) -> Result<()> {
        let join_game = JoinGame::default();
        player.send_packet(&join_game).await?;

        // Send all players info to the new player.
        {
            let mut players = self.players.lock().await;
            let all_info = players.values_mut().map(|p| p.info()).collect::<Vec<_>>();
            let all_players_info = PlayerInfo::new(Action::Add, all_info);
            player.send_packet(&all_players_info).await?;
        }

        // Send the new player info to everybody else.
        let new_player_info = PlayerInfo::new(Action::Add, vec![player.info()]);
        self.broadcast_packet_except(&new_player_info, &player).await?;

        // Spawn the new player in everybody else game.
        let spawn_player = SpawnPlayer::new(&player).await;
        self.broadcast_packet_except(&spawn_player, &player).await?;

        // Spawn other players in the new player game.
        for other in self.players.lock().await.values_mut()
            .filter(|other| &***other != &*player) {
            let spawn_other = SpawnPlayer::new(&other).await;
            player.send_packet(&spawn_other).await?;
        }

        let announcement = OutChatMessage::new(
            Chat::player_joined(&player.info().name()),
            Position::SystemMessage,
        );
        self.broadcast_packet_except(&announcement, &player).await?;

        player.run().await
    }

    pub async fn remove_player(&self, player: &Player) -> Result<()> {
        let id = player.id();
        if self.players.lock().await.remove(&id).is_none() {
            return Ok(())
        }

        let destroy = DestroyEntity::single(id);
        self.broadcast_packet(&destroy).await?;

        let info = PlayerInfo::new(Action::Remove, vec![player.info()]);
        self.broadcast_packet(&info).await?;

        let announcement = OutChatMessage::new(
            Chat::player_left(&player.info().name()),
            Position::SystemMessage,
        );
        self.broadcast_packet(&announcement).await?;
        Ok(())
    }
}