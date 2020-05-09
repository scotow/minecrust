use crate::packets::play::chunk::Chunk;
use crate::packets::play::block::Block;

pub trait ChunkGenerator {
    fn chunk(&self, x: i32, z: i32) -> Chunk;
}

pub struct SameChunkGenerator(Chunk);

impl SameChunkGenerator {
    pub fn new(original: Chunk) -> Self {
        Self(original)
    }
}

impl ChunkGenerator for SameChunkGenerator {
    fn chunk(&self, x: i32, z: i32) -> Chunk {
        self.0.clone(x, z)
    }
}

pub struct FlatChunkGenerator(SameChunkGenerator);

impl FlatChunkGenerator {
    pub fn new() -> Self {
        let mut original = Chunk::new(0, 0);
        for z in 0..16 {
            for x in 0..16 {
                original.set_block(x, 0, z, Block::Bedrock);
                original.set_block(x, 1, z, Block::Dirt);
                original.set_block(x, 2, z, Block::Dirt);
                original.set_block(x, 3, z, Block::Grass);
            }
        }

        Self(SameChunkGenerator::new(original))
    }
}

impl ChunkGenerator for FlatChunkGenerator {
    fn chunk(&self, x: i32, z: i32) -> Chunk {
        self.0.chunk(x, z)
    }
}