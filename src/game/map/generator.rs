use crate::packets::play::block::Block;
use crate::packets::play::chunk::Chunk;

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

/// When implementing this trait you `block` function will be called with all the needed blocks
/// ranging from y=0 to y=255.
/// You can return `None` once there is nothing higher than the current y
/// Here is an example of a FlatWorld:
/// ```
/// use minecrust::game::map::generator::SingleBlockGenerator;
/// use minecrust::packets::play::block::Block;
/// pub struct Flat {}
/// impl SingleBlockGenerator for Flat {
///     fn block(&self, x: i32, y: u16, z: i32) -> Option<Block> {
///         match y {
///             0 => Some(Block::Bedrock),
///             1 => Some(Block::Dirt),
///             2 => Some(Block::Dirt),
///             3 => Some(Block::Grass),
///             _ => None,
///         }
///     }
/// }
/// ```
/// In this case, for each pair (x, z), the block function will be called with y = 0, 1, 2 and 3
/// before moving to the next (x, z).
pub trait SingleBlockGenerator {
    fn block(&self, x: i32, y: u16, z: i32) -> Option<Block>;
}

impl<G> ChunkGenerator for G
where
    G: SingleBlockGenerator,
{
    fn chunk(&self, c_x: i32, c_z: i32) -> Chunk {
        let mut chunk = Chunk::new(c_x, c_z);
        for z in 0..16_u8 {
            for x in 0..16_u8 {
                for y in 0..255 {
                    if let Some(block) = self.block(c_x * 16 + x as i32, y, c_z * 16 + z as i32) {
                        chunk.set_block(x, y, z, block);
                    } else {
                        break;
                    }
                }
            }
        }

        chunk
    }
}
