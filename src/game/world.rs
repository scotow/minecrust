use crate::game::player::Player;
use crate::packets::play::keep_alive::KeepAlive;
use crate::types;
use futures_timer::Delay;
use piper::{Arc, Mutex, Receiver, Sender};
use std::collections::HashMap;
use std::time::Duration;

pub struct World {
    players: HashMap<types::VarInt, Player>,
    player_receiver: Receiver<Player>,
}

impl World {
    pub fn new() -> (Self, Sender<Player>) {
        let (sender, receiver) = piper::chan(1);
        (
            Self {
                players: HashMap::new(),
                player_receiver: receiver,
            },
            sender,
        )
    }

    pub async fn run(self, heartbeat: Duration) {
        let (players, player_receiver) = (self.players, self.player_receiver);
        let players = Arc::new(Mutex::new(players)); // TODO: move to a RW lock
        let keep_alive_loop = async {
            loop {
                Delay::new(heartbeat).await;
                let keep_alive_packet = KeepAlive::new();
                // TODO: remove the for loop for a join_all
                for player in players.lock().values_mut() {
                    player.send_packet(&keep_alive_packet).await.unwrap();
                }
            }
        };
        let add_player_loop = async {
            loop {
                let player = player_receiver.recv().await.unwrap();
                let id = player.id();
                players.lock().insert(id, player);
            }
        };

        // run forever
        let _ = futures::join!(keep_alive_loop, add_player_loop);
    }
}
