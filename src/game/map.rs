use std::collections::HashMap;
use piper::{Lock, LockGuard};
use crate::packets::play::chunk::Chunk;
use crate::packets::play::block::Block;

pub struct Map {
    chunks: Lock<HashMap<(i32, i32), Lock<Chunk>>>
}

impl Map {
    pub async fn new() -> Self {
        let mut chunks = HashMap::new();
        let r = 20;
        for chunk_x in -r..r {
            for chunk_z in -r..r {
                let mut chunk = Chunk::new(chunk_x, chunk_z);
                for z in 0..16 {
                    for x in 0..16 {
                        if (z + x) % 2 == 0 {
                            chunk.set_block(x, 4, z, Block::WhiteConcrete);
                        } else {
                            chunk.set_block(x, 4, z, Block::BlackConcrete);
                        }
                        // if z % 4 < 2 && x % 4 < 2 || z % 4 >= 2 && x % 4 >= 2 {
                        //     chunk.set_block(x, 4, z, Block::SlimeBlock);
                        // } else {
                        //     chunk.set_block(x, 4, z, Block::WhiteConcrete);
                        // }
                    }
                }
                chunks.insert((chunk_x, chunk_z), Lock::new(chunk));
            }
        }
        let map = Self {
            chunks: Lock::new(chunks)
        };
        map.set_block(-1, 4, 0, Block::RedConcrete).await;
        map.set_block(0, 4, -1, Block::RedConcrete).await;

        map
    }

    pub async fn chunk(&self, x: i32, z: i32) -> LockGuard<Chunk> {
        self.chunks.lock().await[&(x, z)].lock().await
    }

    pub async fn set_block(&self, mut x: i32, y: u16, mut z: i32, block: Block) {
        if x < 0 { x -= 16 }
        if z < 0 { z -= 16 }

        let mut chunk = self.chunk(x / 16, z / 16).await;
        chunk.set_block(x.rem_euclid(16) as u8, y, z.rem_euclid(16) as u8, block);
    }
}