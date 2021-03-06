use crate::packets::play::block::Block;
use crate::types::{self, BitArray, LengthVec, Size, TAsyncWrite, VarInt};
use crate::{impl_send, impl_size};
use anyhow::Result;
use nbt::Blob;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::fmt::{self, Debug};
use std::ops::Add;

#[derive(Debug)]
pub struct Chunk {
    pub x: i32,
    pub z: i32,
    heightmap: Heightmap,
    biomes: Biomes,
    sections: [Option<ChunkSection>; 16],
}
crate::impl_packet!(Chunk, 0x22);

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
            }
            (Some(s), _) => s,
        };

        // Set block in section.
        section.set(x, (y % 16) as u8, z, block);
        if section.block_count == 0 {
            self.sections[section_index] = None
        }

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

        self.x.size()
            + self.z.size()
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
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
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
        blob.insert(Heightmap::MOTION_BLOCKING_KEY.to_string(), data)
            .expect("invalid NBT array");
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
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
        let mut vec = Vec::with_capacity(*self.size() as usize);
        Blob::from(self).to_writer(&mut vec)?;
        vec.send(writer).await
    }
}

#[derive(Debug, Clone)]
struct ChunkSection {
    block_count: i16,
    bits_per_block: u8,
    mapping: HashMap<Block, (u16, usize)>,
    available: Vec<usize>,
    palette: LengthVec<VarInt>,
    data: BitArray<LengthVec<u64>>,
}

impl ChunkSection {
    const WIDTH: usize = 16; // X
    const LENGTH: usize = 16; // Z
    const HEIGHT: usize = 16; // Y
    const CAPACITY: usize = Self::WIDTH * Self::LENGTH * Self::HEIGHT;

    const INDIRECT_MIN_BITS_PER_BLOCK: u8 = 4;
    const INDIRECT_MAX_BITS_PER_BLOCK: u8 = 8;
    const DIRECT_BITS_PER_BLOCK: u8 = 14;
    const DOWNSCALE_MARGIN: usize = 4;

    pub fn new() -> Self {
        let mut mapping = HashMap::with_capacity(1);
        mapping.insert(Block::Air, (Self::CAPACITY as u16, 0));

        // Remove first index from 'available' for Block::Air.
        Self {
            block_count: 0,
            bits_per_block: Self::INDIRECT_MIN_BITS_PER_BLOCK,
            mapping,
            available: (1..(1 << Self::INDIRECT_MIN_BITS_PER_BLOCK as usize))
                .rev()
                .collect(),
            palette: LengthVec::from(vec![VarInt(Block::Air as i32)]),
            data: BitArray::<LengthVec<u64>>::new(
                Self::CAPACITY * Self::INDIRECT_MIN_BITS_PER_BLOCK as usize / 64,
                Self::INDIRECT_MIN_BITS_PER_BLOCK as usize,
            ),
        }
    }

    pub fn get(&self, x: u8, y: u8, z: u8) -> Block {
        let palette_index = self
            .data
            .get(y as usize * Self::WIDTH * Self::LENGTH + z as usize * Self::WIDTH + x as usize);

        if self.bits_per_block == Self::DIRECT_BITS_PER_BLOCK {
            palette_index.into()
        } else {
            (*self.palette[palette_index as usize] as u16).into()
        }
    }

    pub fn set(&mut self, x: u8, y: u8, z: u8, new: Block) {
        let old = self.get(x, y, z);
        if old == new {
            return;
        }

        if new == Block::Air {
            self.block_count -= 1;
        } else if old == Block::Air {
            self.block_count += 1;
        }

        self.decrement_palette(old);

        let palette_index = self.increment_palette(new);
        self.data.set(
            y as usize * Self::WIDTH * Self::LENGTH + z as usize * Self::WIDTH + x as usize,
            palette_index as u16,
        );

        self.scale_down_palette_if_needed();
    }

    fn decrement_palette(&mut self, block: Block) {
        let mut entry = match self.mapping.entry(block) {
            Occupied(entry) => entry,
            Vacant(_) => unreachable!(),
        };

        if entry.get().0 == 1 {
            self.available.push(entry.remove().1);
        } else {
            entry.get_mut().0 -= 1;
        }
    }

    fn increment_palette(&mut self, block: Block) -> usize {
        if self.mapping.contains_key(&block) {
            let (count, index) = self.mapping.get_mut(&block).unwrap();
            *count += 1;
            *index
        } else {
            self.scale_up_palette_if_needed();

            // We are on a direct palette. Add block count and return its direct value.
            if self.bits_per_block == Self::DIRECT_BITS_PER_BLOCK {
                self.mapping.insert(block, (1, 0));
                return block as usize;
            }

            let palette_index = self.available.pop().unwrap();
            self.mapping.insert(block, (1, palette_index));

            // Increase palette size if it's the first time that we reach that size.
            if palette_index >= self.palette.len() {
                self.palette
                    .resize(palette_index + 1, VarInt(Block::Air as i32));
            }

            // Add block to the palette.
            self.palette[palette_index] = VarInt(block as i32);
            palette_index
        }
    }

    fn scale_up_palette_if_needed(&mut self) {
        // Only scale if the palette is full.
        if self.bits_per_block == Self::DIRECT_BITS_PER_BLOCK || !self.available.is_empty() {
            return;
        }

        let new_bits_per_block = match self.bits_per_block {
            bpb @ Self::INDIRECT_MIN_BITS_PER_BLOCK..=Self::INDIRECT_MAX_BITS_PER_BLOCK
                if bpb < Self::INDIRECT_MAX_BITS_PER_BLOCK =>
            {
                self.bits_per_block + 1
            }
            Self::INDIRECT_MAX_BITS_PER_BLOCK => Self::DIRECT_BITS_PER_BLOCK,
            _ => unreachable!(),
        };

        // Add some new available palette indexes only if we don't scale to a direct palette.
        if new_bits_per_block != Self::DIRECT_BITS_PER_BLOCK {
            self.available.append(
                &mut ((1 << self.bits_per_block as usize)..(1 << new_bits_per_block as usize))
                    .rev()
                    .collect(),
            );
        }

        // Rebuild the new data array.
        let mut new_data = BitArray::<LengthVec<u64>>::new(
            Self::CAPACITY * new_bits_per_block as usize / 64,
            new_bits_per_block as usize,
        );

        if new_bits_per_block == Self::DIRECT_BITS_PER_BLOCK {
            for i in 0..Self::CAPACITY {
                new_data.set(i, *self.palette[self.data.get(i) as usize] as u16);
            }
        } else {
            for i in 0..Self::CAPACITY {
                new_data.set(i, self.data.get(i));
            }
        }

        self.bits_per_block = new_bits_per_block;
        self.data = new_data;
    }

    fn scale_down_palette_if_needed(&mut self) {
        #[allow(overlapping_patterns)]
        let new_bits_per_block = match self.bits_per_block {
            Self::INDIRECT_MIN_BITS_PER_BLOCK => return,
            bpb @ Self::INDIRECT_MIN_BITS_PER_BLOCK..=Self::INDIRECT_MAX_BITS_PER_BLOCK
                if bpb > Self::INDIRECT_MIN_BITS_PER_BLOCK =>
            {
                self.bits_per_block - 1
            }
            Self::DIRECT_BITS_PER_BLOCK => Self::INDIRECT_MAX_BITS_PER_BLOCK,
            _ => unreachable!(),
        };

        if self.mapping.len() > (1 << new_bits_per_block as usize) - Self::DOWNSCALE_MARGIN {
            return;
        }

        // Rebuilding the available indexes list.
        self.available = (self.mapping.len()..(1 << new_bits_per_block))
            .rev()
            .collect();

        // Rebuild palette indexes and replace them in the map.
        let mut new_palette = LengthVec::from(Vec::with_capacity(self.mapping.len()));
        for (new_index, (block, (_count, palette_index))) in self.mapping.iter_mut().enumerate() {
            *palette_index = new_index;
            new_palette.push(VarInt(*block as i32));
        }

        let mut new_data = BitArray::<LengthVec<u64>>::new(
            Self::CAPACITY * new_bits_per_block as usize / 64,
            new_bits_per_block as usize,
        );

        if self.bits_per_block == Self::DIRECT_BITS_PER_BLOCK {
            for i in 0..Self::CAPACITY {
                new_data.set(i, self.mapping[&self.data.get(i).into()].1 as u16);
            }
        } else {
            for i in 0..Self::CAPACITY {
                new_data.set(
                    i,
                    self.mapping[&(*self.palette[self.data.get(i) as usize] as u16).into()].1
                        as u16,
                );
            }
        }

        self.bits_per_block = new_bits_per_block as u8;
        self.palette = new_palette;
        self.data = new_data;
    }
}

impl Size for ChunkSection {
    fn size(&self) -> VarInt {
        let size = self.block_count.size() + self.bits_per_block.size() + self.data.size();

        if self.bits_per_block == Self::DIRECT_BITS_PER_BLOCK {
            size
        } else {
            size + self.palette.size()
        }
    }
}

#[async_trait::async_trait]
impl types::Send for ChunkSection {
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
        self.block_count.send(writer).await?;
        self.bits_per_block.send(writer).await?;
        if self.bits_per_block != Self::DIRECT_BITS_PER_BLOCK {
            self.palette.send(writer).await?;
        }
        self.data.send(writer).await
    }
}

impl Size for [Option<ChunkSection>; 16] {
    fn size(&self) -> VarInt {
        self.iter().map(Size::size).fold(VarInt::new(0), Add::add)
    }
}

#[async_trait::async_trait]
impl types::Send for [Option<ChunkSection>; 16] {
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
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
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
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
        144397213884223496,
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

    #[test]
    fn test_palette() {
        // Test initial bits per size.
        let section = ChunkSection::new();
        assert_eq!(
            section.bits_per_block,
            ChunkSection::INDIRECT_MIN_BITS_PER_BLOCK
        );

        // Shouldn't scale up, we are on the limit.
        let mut section = ChunkSection::new();
        set_first_blocks(
            &mut section,
            1 << ChunkSection::INDIRECT_MIN_BITS_PER_BLOCK as u16,
        );
        assert_eq!(
            section.bits_per_block,
            ChunkSection::INDIRECT_MIN_BITS_PER_BLOCK
        );

        // Should scale up, we are on the limit + 1.
        let mut section = ChunkSection::new();
        set_first_blocks(
            &mut section,
            (1 << ChunkSection::INDIRECT_MIN_BITS_PER_BLOCK as u16) + 1,
        );
        assert_eq!(
            section.bits_per_block,
            ChunkSection::INDIRECT_MIN_BITS_PER_BLOCK + 1
        );
        check_first_blocks(&mut section, 0, 17);

        // Shouldn't scale down, we just removed 2 blocks (less than margin).
        let mut section = ChunkSection::new();
        set_first_blocks(
            &mut section,
            (1 << ChunkSection::INDIRECT_MIN_BITS_PER_BLOCK as u16) + 1,
        );
        clean_first_blocks(&mut section, 2);
        assert_eq!(
            section.bits_per_block,
            ChunkSection::INDIRECT_MIN_BITS_PER_BLOCK + 1
        );

        // Shouldn't scale down, we are just above the downscale limit.
        let mut section = ChunkSection::new();
        set_first_blocks(
            &mut section,
            (1 << ChunkSection::INDIRECT_MIN_BITS_PER_BLOCK as u16) + 1,
        );
        clean_first_blocks(&mut section, ChunkSection::DOWNSCALE_MARGIN as u16);
        assert_eq!(
            section.bits_per_block,
            ChunkSection::INDIRECT_MIN_BITS_PER_BLOCK + 1
        );

        // Should scale down, we removed as much as the margin + 1.
        let mut section = ChunkSection::new();
        set_first_blocks(
            &mut section,
            (1 << ChunkSection::INDIRECT_MIN_BITS_PER_BLOCK as u16) + 1,
        );
        clean_first_blocks(&mut section, ChunkSection::DOWNSCALE_MARGIN as u16 + 1);
        assert_eq!(
            section.bits_per_block,
            ChunkSection::INDIRECT_MIN_BITS_PER_BLOCK
        );
        check_first_blocks(&mut section, ChunkSection::DOWNSCALE_MARGIN as u16 + 2, 10);

        // Should scale up to max indirect size, we are on the direct limit - 1.
        let mut section = ChunkSection::new();
        set_first_blocks(
            &mut section,
            1 << ChunkSection::INDIRECT_MAX_BITS_PER_BLOCK as u16,
        );
        assert_eq!(
            section.bits_per_block,
            ChunkSection::INDIRECT_MAX_BITS_PER_BLOCK
        );
        check_first_blocks(&mut section, 0, 100);

        // Should scale up to a direct palette.
        let mut section = ChunkSection::new();
        set_first_blocks(
            &mut section,
            1 << (ChunkSection::INDIRECT_MAX_BITS_PER_BLOCK as u16) + 1,
        );
        assert_eq!(section.bits_per_block, ChunkSection::DIRECT_BITS_PER_BLOCK);
        check_first_blocks(&mut section, 0, 100);

        // Should scale up to a direct palette then downscale to a max size indirect palette.
        let mut section = ChunkSection::new();
        set_first_blocks(
            &mut section,
            (1 << ChunkSection::INDIRECT_MAX_BITS_PER_BLOCK as u16) + 1,
        );
        clean_first_blocks(&mut section, ChunkSection::DOWNSCALE_MARGIN as u16 + 1);
        assert_eq!(
            section.bits_per_block,
            ChunkSection::INDIRECT_MAX_BITS_PER_BLOCK
        );
        check_first_blocks(&mut section, ChunkSection::DOWNSCALE_MARGIN as u16 + 2, 100);
    }

    fn set_first_blocks(section: &mut ChunkSection, n: u16) {
        for i in 0..n {
            section.set(
                (i % 16) as u8,
                (i / 256) as u8,
                (i / 16) as u8,
                Block::from(i),
            );
        }
    }

    fn clean_first_blocks(section: &mut ChunkSection, n: u16) {
        for i in 1..(n + 1) {
            section.set((i % 16) as u8, (i / 256) as u8, (i / 16) as u8, Block::Air);
        }
    }

    fn check_first_blocks(section: &mut ChunkSection, start: u16, n: u16) {
        for i in (start)..(start + n) {
            assert_eq!(
                section.get((i % 16) as u8, (i / 256) as u8, (i / 16) as u8),
                Block::from(i)
            );
        }
    }
}
