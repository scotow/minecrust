use std::collections::HashMap;
use crate::types;
use crate::game::player::Player;
use std::time::Duration;
use futures::{AsyncRead, AsyncWrite};
use piper::{Sender, Receiver, Mutex};
use futures_timer::Delay;
use crate::packets::play::keep_alive::KeepAlive;

pub struct World<S: AsyncRead + AsyncWrite + Unpin + Send> {
    players: HashMap<types::VarInt, Player<S>>,
    player_receiver: Receiver<Player<S>>,
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> World<S> {
    pub fn new() -> (Self, Sender<Player<S>>) {
        let (sender, receiver) = piper::chan(1);
        (
            Self {
                players: HashMap::new(),
                player_receiver: receiver,
            },
            sender
        )
    }

    pub async fn run(&mut self, heartbeat: Duration) {
        let mutex = Mutex::new(self);
        let keep_alive_loop = async {
            loop {
                let keep_alive_packet = KeepAlive::new();
                for player in mutex.lock().players.values_mut() {
                    player.send_packet(&keep_alive_packet).await.unwrap();
                }
                Delay::new(heartbeat).await;
            }
        };
        let add_player_loop = async {
            let player = self.player_receiver.recv().await.unwrap();
            mutex.lock().players.insert();
        };
    }
}