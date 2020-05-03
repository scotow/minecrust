use crate::types::{self, LengthVec, Size, VarInt, SizeVec};
use anyhow::Result;
use bitvec::order::{Lsb0, Msb0};
use bitvec::vec::BitVec;
use futures::prelude::*;
use std::collections::HashMap;
use nbt::Blob;
use std::fmt::Debug;
use serde::export::Formatter;
use std::fmt;
use crate::{impl_size, impl_send};
use std::ops::Add;

// #[derive(Debug)]
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

    fn bitmask(&self) -> VarInt {
        let mut bitmask = 0;
        for (y, exists) in self.sections.iter().map(Option::is_some).enumerate() {
            if exists {
                bitmask |= 1 << y as i32;
            }
        }
        VarInt::new(bitmask)
    }

    pub fn set_block(&mut self, x: u8, y: i16, z: u8, block: Block) {
        if block != Block::Air {
            self.heightmap.replace_if_bigger(x, z, y);
        } else {
            // TODO: Find the new lowest block in the column and replace.
        }

        let chunk_index = y as usize / 16;
        let section = match (&mut self.sections[chunk_index], block) {
            (None, Block::Air) => return,
            (None, _) => {
                let section = ChunkSection::new();
                self.sections[chunk_index] = Some(section);
                self.sections[chunk_index].as_mut().unwrap()
            },
            (Some(s), _) => s
        };

        section.set(x, (y % 16) as u8, z, block);
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

#[derive(Clone)]
struct Heightmap([i64; 36]);

impl Heightmap {
    const MOTION_BLOCKING_KEY: &'static str = "MOTION_BLOCKING";

    fn new() -> Self {
        Self([0; 36])
    }

    fn get(&self, x: u8, z: u8) -> i16 {
        let x = x as usize;
        let z = z as usize;
        let start_index = (z * 16 + x) * 9 / 64;
        let end_index = ((z * 16 + x + 1) * 9 - 1) / 64;
        let start_bit = (z * 16 + x) * 9 % 64;

        if start_index == end_index {
            let buffer = self.0[start_index] as u64;
            let mask = 0b111_111_111_u64 << (start_bit as u64);

            ((buffer & mask) >> (start_bit as u64)) as i16
        } else {
            let buffer = ((self.0[start_index + 1] as u64 as u128) << 64) | (self.0[start_index] as u64 as u128);
            let mask = 0b111_111_111_u128 << (start_bit as u128);

            ((buffer & mask) >> (start_bit as u128)) as i16
        }
    }

    fn set(&mut self, x: u8, z: u8, value: i16) {
        let x = x as usize;
        let z = z as usize;
        let start_index = (z * 16 + x) * 9 / 64;
        let end_index = ((z * 16 + x + 1) * 9 - 1) / 64;
        let start_bit = (z * 16 + x) * 9 % 64;

        if start_index == end_index {
            let clear = !(0b111_111_111_u64 << (start_bit as u64));
            let mut buffer = self.0[start_index] as u64 & clear;
            buffer |= (value as u64) << (start_bit as u64);
            self.0[start_index] = buffer as i64;
        } else {
            let clear = !(0b111_111_111_u128 << (start_bit as u128));
            let mut buffer = (((self.0[start_index + 1] as u64 as u128) << 64) | (self.0[start_index] as u64 as u128)) & clear;
            buffer |= (value as u16 as u128) << (start_bit as u128);

            self.0[start_index + 1] = (buffer >> 64) as i64;
            self.0[start_index] = buffer as i64;
        }
    }

    pub fn replace_if_bigger(&mut self, x: u8, z: u8, value: i16) {
        if value > self.get(x, z) {
            self.set(x, z, value);
        }
    }
}

impl From<&Heightmap> for Blob {
    fn from(heightmap: &Heightmap) -> Self {
        let mut blob = nbt::Blob::new();
        blob.insert(Heightmap::MOTION_BLOCKING_KEY.to_string(), &heightmap.0[..]).expect("invalid NBT array");
        blob
    }
}

impl Debug for Heightmap {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0[..].fmt(formatter)
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
        Blob::from(self).to_writer(&mut vec);
        vec.send(writer).await
    }
}

// #[derive(Debug, Clone, Copy, Eq, PartialEq)]
// #[repr(u16)]
// pub enum Block {
//     Air = 0,
//     Bedrock = 1,
//     Dirt = 2,
//     Grass = 3
// }
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u16)]
pub enum Block {
    Air = 0,
    Bedrock = 33,
    Dirt = 10,
    Grass = 9,
    Water = 49,
    Lava = 50,
    WhiteConcrete = 8902,
    BlackConcrete = 8917,
    HoneyBlock = 11335,
}

impl From<u16> for Block {
    fn from(n: u16) -> Block {
        unsafe { std::mem::transmute(n) }
    }
}

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
struct ChunkSection {
    block_count: i16,
    bits_per_block: u8,
    palette: Option<LengthVec<VarInt>>,
    data: LengthVec<u64>,
}

impl ChunkSection {
    pub fn new() -> Self {
        Self {
            block_count: 0,
            bits_per_block: 14,
            palette: None,
            data: LengthVec(vec![0u64; 16 * 16 * 16 * 14 / 64]),
        }
    }

    pub fn get(&self, x: u8, y: u8, z: u8) -> Block {
        let x = x as usize;
        let y = y as usize;
        let z = z as usize;
        let bits_per_block = self.bits_per_block as usize;

        let start_index = (y * 256 + z * 16 + x) * bits_per_block / 64;
        let end_index = ((y * 256 + z * 16 + x + 1) * bits_per_block - 1) / 64;
        let start_bit = (y * 256 + z * 16 + x) * bits_per_block % 64;

        if start_index == end_index {
            let buffer = self.data[start_index] as u64;
            let mask = !(!0_u64 << bits_per_block as u64) << (start_bit as u64);

            ((buffer & mask) >> (start_bit as u64)) as u16
        } else {
            let buffer = ((self.data[start_index + 1] as u64 as u128) << 64) | (self.data[start_index] as u64 as u128);
            let mask = !(!0_u128 << bits_per_block as u128) << (start_bit as u128);

            ((buffer & mask) >> (start_bit as u128)) as u16
        }.into()
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

        let x = x as usize;
        let y = y as usize;
        let z = z as usize;
        let value = new as u16;
        let bits_per_block = self.bits_per_block as usize;

        let start_index = (y * 256 + z * 16 + x) * bits_per_block / 64;
        let end_index = ((y * 256 + z * 16 + x + 1) * bits_per_block - 1) / 64;
        let start_bit = (y * 256 + z * 16 + x) * bits_per_block % 64;

        if start_index == end_index {
            let clear = !(!(!0_u64 << bits_per_block as u64) << (start_bit as u64));
            let mut buffer = self.data[start_index] as u64 & clear;
            buffer |= (value as u64) << (start_bit as u64);
            self.data[start_index] = buffer as u64;
        } else {
            let clear = !(!(!0_u128 << bits_per_block as u128) << (start_bit as u128));
            let mut buffer = (((self.data[start_index + 1] as u64 as u128) << 64) | (self.data[start_index] as u64 as u128)) & clear;
            buffer |= (value as u16 as u128) << (start_bit as u128);

            self.data[start_index + 1] = (buffer >> 64) as u64;
            self.data[start_index] = buffer as u64;
        }
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

// impl types::Size for nbt::Blob {
//     fn size(&self) -> types::VarInt {
//         let mut vec = Vec::new();
//         self.to_writer(&mut vec).unwrap();
//         vec.size()
//     }
// }
//
// #[async_trait::async_trait]
// impl types::Send for nbt::Blob {
//     async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
//         let mut vec = Vec::new();
//         self.to_writer(&mut vec)?;
//         vec.send(writer).await
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;

    const ALL_4_HEIGHTMAP: Heightmap = Heightmap([
        72198606942111748,
        36099303471055874,
        -9205322385119247871,
        4620710844295151872,
        2310355422147575936,
        1155177711073787968,
        577588855536893984,
        288794427768446992,
        144397213884223496,
        72198606942111748,
        36099303471055874,
        -9205322385119247871,
        4620710844295151872,
        2310355422147575936,
        1155177711073787968,
        577588855536893984,
        288794427768446992,
        144397213884223496,
        72198606942111748,
        36099303471055874,
        -9205322385119247871,
        4620710844295151872,
        2310355422147575936,
        1155177711073787968,
        577588855536893984,
        288794427768446992,
        144397213884223496,
        72198606942111748,
        36099303471055874,
        -9205322385119247871,
        4620710844295151872,
        2310355422147575936,
        1155177711073787968,
        577588855536893984,
        288794427768446992,
        144397213884223496
    ]);

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

        for z in 0..16 {
            for x in 0..16 {
                assert_eq!(ALL_4_HEIGHTMAP.get(x, z), 4);
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

        assert_eq!(heightmap.0.to_vec(), ALL_4_HEIGHTMAP.0.to_vec());
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
