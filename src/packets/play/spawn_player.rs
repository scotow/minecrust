use crate::impl_packet;
use crate::types;
use crate::types::{EntityPosition, VarInt};
use uuid::Uuid;
use crate::game::player::Player;
use piper::{Arc, Mutex};

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct SpawnPlayer {
    id: VarInt,
    uuid: Uuid,
    position: Arc<Mutex<EntityPosition>>,
}
impl_packet!(SpawnPlayer, 0x05);

impl SpawnPlayer {
    pub fn new(player: &Player) -> Self {
        dbg!(player.position().clone());
        Self {
            id: player.id(),
            uuid: player.info().uuid(),
            position: player.position().clone(),
        }
    }
}