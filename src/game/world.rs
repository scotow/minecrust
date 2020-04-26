use crate::game::player::Player;
use crate::packets::play::keep_alive::KeepAlive;
use crate::types;
use futures::{AsyncRead, AsyncWrite};
use futures_timer::Delay;
use piper::{Arc, Mutex, Receiver, Sender};
use std::collections::HashMap;
use std::time::Duration;

pub struct World<R: Send + Sync + Unpin, W: Send + Sync + Unpin> {
    players: HashMap<types::VarInt, Player<R, W>>,
    player_receiver: Receiver<Player<R, W>>,
}

impl<R: AsyncRead + Send + Sync + Unpin, W: AsyncWrite + Send + Sync + Unpin> World<R, W> {
    pub fn new() -> (Self, Sender<Player<R, W>>) {
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
                let keep_alive_packet = KeepAlive::new();
                for player in players.lock().values_mut() {
                    player.send_packet(&keep_alive_packet).await.unwrap();
                }
                Delay::new(heartbeat).await;
            }
        };
        let add_player_loop = async {
            let player = player_receiver.recv().await.unwrap();
            players.lock().insert(player.id(), player);
        };

        futures::join!(keep_alive_loop, add_player_loop);
    }
}
