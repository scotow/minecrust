pub mod generator;

use crate::game::map::generator::ChunkGenerator;
use crate::packets::play::block::Block;
use crate::packets::play::chunk::Chunk;
use piper::{Lock, LockGuard};
use std::collections::HashMap;

pub struct Map {
    chunks: Lock<HashMap<(i32, i32), Lock<Chunk>>>,
    generator: Box<dyn ChunkGenerator + Sync + std::marker::Send + 'static>,
}

impl Map {
    pub async fn new(generator: impl ChunkGenerator + Sync + std::marker::Send + 'static) -> Self {
        Self {
            chunks: Lock::new(HashMap::new()),
            generator: Box::new(generator),
        }
    }

    pub async fn chunk(&self, x: i32, z: i32) -> LockGuard<Chunk> {
        let mut chunks = self.chunks.lock().await;
        chunks
            .entry((x, z))
            .or_insert_with(|| Lock::new(self.generator.chunk(x, z)))
            .lock()
            .await
    }

    pub async fn set_block(&self, mut x: i32, y: u16, mut z: i32, block: Block) {
        if x < 0 {
            x -= 16
        }
        if z < 0 {
            z -= 16
        }

        let mut chunk = self.chunk(x / 16, z / 16).await;
        chunk.set_block(x.rem_euclid(16) as u8, y, z.rem_euclid(16) as u8, block);
    }
}
