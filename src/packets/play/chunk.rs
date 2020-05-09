use crate::types::{self, LengthVec, Size, VarInt, BitArray};
use anyhow::Result;
use futures::prelude::*;
use nbt::Blob;
use std::fmt::{self, Debug};
use crate::{impl_size, impl_send};
use std::ops::Add;
use crate::packets::play::block::Block;

#[derive(Debug)]
pub struct Chunk {
    pub x: i32,
    pub z: i32,
    heightmap: Heightmap,
    biomes: Biomes,
    sections: [Option<ChunkSection>; 16],
}
crate ::impl_packet!(Chunk, 0x22);

impl Chunk {
    pub fn new(x: i32, z: i32) -> Self {
        Self {
            x,
            z,
            heightmap: Heightmap::new(),
            biomes: Biomes::new(Biome::Plains),
            sections: Default::default(),
        }
    }

    pub fn clone(&self, x: i32, z: i32) -> Self {
        Self {
            x,
            z,
            heightmap: self.heightmap.clone(),
            biomes: self.biomes.clone(),
            sections: self.sections.clone(),
        }
    }

    fn bitmask(&self) -> VarInt {
        let mut bitmask = 0;
        for (y, exists) in self.sections.iter().map(Option::is_some).enumerate() {
            if exists {
                bitmask |= 1 << y as i32;
            }
        }
        VarInt::new(bitmask)
    }

    pub fn get_block(&self, x: u8, y: u16, z: u8) -> Block {
        if let Some(section) = &self.sections[y as usize / 16] {
            return section.get(x, (y % 16) as u8, z);
        }
        Block::Air
    }

    pub fn set_block(&mut self, x: u8, y: u16, z: u8, block: Block) {
        let section_index = y as usize / 16;
        let section = match (&mut self.sections[section_index], block) {
            (None, Block::Air) => return,
            (None, _) => {
                let section = ChunkSection::new();
                self.sections[section_index] = Some(section);
                self.sections[section_index].as_mut().unwrap()
            },
            (Some(s), _) => s
        };

        // Set block in section.
        section.set(x, (y % 16) as u8, z, block);

        // Update heightmap if needed.
        if block != Block::Air {
            self.heightmap.replace_if_bigger(x, z, y as u16);
        } else if y == self.heightmap.get(x, z) {
            for i in (0..y - 1).rev() {
                if self.get_block(x, i, z) != Block::Air {
                    self.heightmap.set(x, z, i);
                    return;
                }
            }
            self.heightmap.set(x, z, 0);
        }
    }
}

impl types::Size for Chunk {
    fn size(&self) -> types::VarInt {
        let sections_size = self.sections.size();

        self.x.size() + self.z.size()
            + true.size()
            + self.bitmask().size()
            + self.heightmap.size()
            + self.biomes.size()
            + sections_size.size()
            + sections_size
            + VarInt::new(0).size()
    }
}

#[async_trait::async_trait]
impl types::Send for Chunk {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        self.x.send(writer).await?;
        self.z.send(writer).await?;
        true.send(writer).await?;
        self.bitmask().send(writer).await?;
        self.heightmap.send(writer).await?;
        self.biomes.send(writer).await?;
        self.sections.size().send(writer).await?;
        self.sections.send(writer).await?;
        VarInt::new(0).send(writer).await
    }
}

#[derive(Debug, Clone)]
struct Heightmap(BitArray<Vec<u64>>);

impl Heightmap {
    const MOTION_BLOCKING_KEY: &'static str = "MOTION_BLOCKING";

    fn new() -> Self {
        Self(BitArray::<Vec<u64>>::new(36, 9))
    }

    fn get(&self, x: u8, z: u8) -> u16 {
        self.0.get((z as usize * 16) + x as usize)
    }

    fn set(&mut self, x: u8, z: u8, value: u16) {
        self.0.set((z as usize * 16) + x as usize, value);
    }

    pub fn replace_if_bigger(&mut self, x: u8, z: u8, value: u16) {
        if value > self.get(x, z) {
            self.set(x, z, value);
        }
    }
}

impl From<&Heightmap> for Blob {
    fn from(heightmap: &Heightmap) -> Self {
        let data: Vec<i64> = heightmap.0.as_slice().iter().map(|v| *v as i64).collect();
        let mut blob = nbt::Blob::new();
        blob.insert(Heightmap::MOTION_BLOCKING_KEY.to_string(), data).expect("invalid NBT array");
        blob
    }
}

impl types::Size for Heightmap {
    fn size(&self) -> types::VarInt {
        // 0A + u16 + (0C + u16 + MOTION_BLOCKING_KEY.len() + (36 as u32) + 36 * i64) + EOF
        VarInt::new(1 + 2 + (1 + 2 + Self::MOTION_BLOCKING_KEY.len() as i32 + 4 + 36 * 8) + 1)
    }
}

#[async_trait::async_trait]
impl types::Send for Heightmap {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        let mut vec = Vec::with_capacity(*self.size() as usize);
        Blob::from(self).to_writer(&mut vec)?;
        vec.send(writer).await
    }
}

#[derive(Debug, Clone, macro_derive::Size, macro_derive::Send)]
struct ChunkSection {
    block_count: i16,
    bits_per_block: u8,
    palette: Option<LengthVec<VarInt>>,
    data: BitArray<LengthVec<u64>>,
}

impl ChunkSection {
    pub fn new() -> Self {
        Self {
            block_count: 0,
            bits_per_block: 14,
            palette: None,
            data: BitArray::<LengthVec<u64>>::new(16 * 16 * 16 * 14 / 64, 14),
        }
    }

    pub fn get(&self, x: u8, y: u8, z: u8) -> Block {
        self.data.get(y as usize * 256 + z as usize * 16 + x as usize).into()
    }

    pub fn set(&mut self, x: u8, y: u8, z: u8, new: Block) {
        let old = self.get(x, y, z);
        if old == new {
            return
        }

        if new == Block::Air {
            self.block_count -= 1;
        } else if old == Block::Air {
            self.block_count += 1;
        }

        self.data.set(y as usize * 256 + z as usize * 16 + x as usize, new as u16);
    }
}

impl Size for [Option<ChunkSection>; 16] {
    fn size(&self) -> VarInt {
        self.iter().map(Size::size).fold(VarInt::new(0), Add::add)
    }
}

#[async_trait::async_trait]
impl types::Send for [Option<ChunkSection>; 16] {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        for section in self.iter().filter_map(|s| s.as_ref()) {
            section.send(writer).await?;
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Biomes([Biome; 1024]);

impl Biomes {
    fn new(biome: Biome) -> Self {
        Self([biome; 1024])
    }
}

impl Debug for Biomes {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0[..].fmt(formatter)
    }
}

impl types::Size for Biomes {
    fn size(&self) -> types::VarInt {
        VarInt::new(self.0.len() as i32 * 4)
    }
}

#[async_trait::async_trait]
impl types::Send for Biomes {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        for biome in self.0.iter() {
            (*biome as i32).send(writer).await?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(i32)]
pub enum Biome {
    Ocean = 0,
    Plains = 1,
    Void = 127,
}
impl_size!(Biome, 4);
impl_send!(Biome as i32);

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;

    const ALL_4_HEIGHTMAP: [u64; 36] = [
        72198606942111748,
        36099303471055874,
        9241421688590303745,
        4620710844295151872,
        2310355422147575936,
        1155177711073787968,
        577588855536893984,
        288794427768446992,
        144397213884223496,
        72198606942111748,
        36099303471055874,
        9241421688590303745,
        4620710844295151872,
        2310355422147575936,
        1155177711073787968,
        577588855536893984,
        288794427768446992,
        144397213884223496,
        72198606942111748,
        36099303471055874,
        9241421688590303745,
        4620710844295151872,
        2310355422147575936,
        1155177711073787968,
        577588855536893984,
        288794427768446992,
        144397213884223496,
        72198606942111748,
        36099303471055874,
        9241421688590303745,
        4620710844295151872,
        2310355422147575936,
        1155177711073787968,
        577588855536893984,
        288794427768446992,
        144397213884223496
    ];

    #[test]
    fn test_heightmap() {
        let heightmap = Heightmap::new();
        assert_eq!(heightmap.get(0, 0), 0);

        let mut heightmap = Heightmap::new();
        heightmap.set(0, 0, 5);
        assert_eq!(heightmap.get(0, 0), 5);

        let mut heightmap = Heightmap::new();
        heightmap.set(1, 0, 42);
        assert_eq!(heightmap.get(1, 0), 42);

        let mut heightmap = Heightmap::new();
        heightmap.set(7, 0, 0b111_111_111);
        assert_eq!(heightmap.get(7, 0), 0b111_111_111);

        let heightmap = Heightmap(BitArray::<Vec<_>>::from_slice(&ALL_4_HEIGHTMAP, 9));
        for z in 0..16 {
            for x in 0..16 {
                assert_eq!(heightmap.get(x, z), 4);
            }
        }

        let mut rng = rand::thread_rng();
        let mut z_range = (0..16).collect::<Vec<_>>();
        z_range.shuffle(&mut rng);

        let mut heightmap = Heightmap::new();
        for z in z_range {
            let mut x_range = (0..16).collect::<Vec<_>>();
            x_range.shuffle(&mut rng);

            for x in x_range {
                heightmap.set(x, z, 4);
                assert_eq!(heightmap.get(x, z), 4);
            }
        }

        assert_eq!(heightmap.0.as_slice(), &ALL_4_HEIGHTMAP[..]);
    }

    #[test]
    fn test_chunk_section() {
        let section = ChunkSection::new();
        assert_eq!(section.get(0, 0, 0), Block::Air);

        let mut section = ChunkSection::new();
        section.set(0, 0, 0, Block::HoneyBlock);
        assert_eq!(section.get(0, 0, 0), Block::HoneyBlock);

        let mut section = ChunkSection::new();
        section.set(0, 0, 0, Block::HoneyBlock);
        assert_eq!(section.get(0, 0, 0), Block::HoneyBlock);

        let mut section = ChunkSection::new();
        section.set(4, 0, 0, Block::HoneyBlock);
        assert_eq!(section.get(4, 0, 0), Block::HoneyBlock);

        let mut rng = rand::thread_rng();
        let mut section = ChunkSection::new();

        let mut y_range = (0..16).collect::<Vec<_>>();
        y_range.shuffle(&mut rng);
        for y in y_range {
            let mut z_range = (0..16).collect::<Vec<_>>();
            z_range.shuffle(&mut rng);
            for z in z_range {
                let mut x_range = (0..16).collect::<Vec<_>>();
                x_range.shuffle(&mut rng);

                for x in x_range {
                    section.set(x, y, z, Block::HoneyBlock);
                    assert_eq!(section.get(x, y, z), Block::HoneyBlock);
                }
            }
        }
    }
}
