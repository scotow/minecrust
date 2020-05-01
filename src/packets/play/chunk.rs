use crate::types::{self, SizeVec, Size, VarInt};
use anyhow::Result;
use bitvec::order::{Lsb0, Msb0};
use bitvec::vec::BitVec;
use futures::prelude::*;
use std::collections::HashMap;

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Chunk {
    x: i32,
    z: i32,
    full_chunk: bool,
    primary_bit_mask: types::VarInt,
    heightmaps: nbt::Blob,
    biomes: Vec<i32>,
    data: ChunkData,
    block_entities: SizeVec<u8>,
}
crate::impl_packet!(Chunk, 0x22);

impl Chunk {
    ///                              y z x
    pub fn new(x: i32, z: i32, data: &[Vec<Vec<Block>>]) -> Self {
        let mut bitmask: i32 = 0;
        let mut sections = Vec::new();
        let mut heightmaps = vec![vec![0; 16]; 16];

        for section_index in 0..16 {
            let mut section: Vec<Block> = Vec::new();
            for y in (section_index * 16)..((section_index + 1) * 16) {
                for z in 0..16 {
                    for x in 0..16 {
                        if data[y][z][x] != Block::Air {
                            // Notchian server adds 1 here, don't know why.
                            if heightmaps[z][x] < y {
                                heightmaps[z][x] = y;
                            }
                            bitmask |= 1 << section_index;
                        }
                        section.push(data[y][z][x]);
                    }
                }
            }
            if bitmask & (1 << section_index) != 0 {
                let section = ChunkSection::new(section);
                dbg!(section.size());
                sections.push(section);
            }
        }

        let mut bit_vec_heightmaps = BitVec::<Lsb0, u64>::new();
        for z in heightmaps {
            for y in z {
                let mut truc = as_bitvec(y);
                bit_vec_heightmaps.append(&mut truc);
            }
        }

        let heightmaps: Vec<i64> = bit_vec_heightmaps
            .as_slice()
            .iter()
            .map(|el| *el as i64)
            .collect();
        let heightmaps: nbt::Value = heightmaps.into();

        let mut blob = nbt::Blob::new();
        blob.insert("MOTION_BLOCKING".to_string(), heightmaps.clone());
        blob.insert("WORLD_SURFACE".to_string(), heightmaps);

        let biomes = vec![1_i32; 1024];
        let data = ChunkData(sections);

        Self {
            x,
            z,
            full_chunk: true,
            primary_bit_mask: types::VarInt(bitmask),
            heightmaps: blob,
            biomes,
            data,
            block_entities: SizeVec::new(),
        }
    }
}

fn as_bitvec(value: usize) -> BitVec {
    let mut res = BitVec::new();
    for i in 0..9 {
        res.push((value & (1 << i)) != 0);
    }
    res
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
}

impl Block {
    pub fn as_bitvec(&self) -> BitVec {
        let value = *self as u16;
        let mut res = BitVec::new();
        for i in 0..14 {
            res.push((value & (1 << i)) != 0);
        }
        res
    }
}

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
struct ChunkSection {
    block_count: i16,
    bits_per_block: u8,
    palette: Option<SizeVec<VarInt>>,
    data_array: SizeVec<i64>,
}

impl ChunkSection {
    pub fn new(vec: Vec<Block>) -> Self {
        let mut block_count = 0;
        let mut data = BitVec::<Lsb0, u64>::new();
        for block in vec {
            if block != Block::Air {
                block_count += 1;
            }
            data.append(&mut block.as_bitvec());
        }

        let data_array: Vec<i64> = data.as_slice().iter().map(|el| *el as i64).collect();

        // if block_count == 4096 {
        //     dbg!(block_count);
        //     dbg!(data_array.len());
        //     dbg!(data);
        // } else {
        //     println!("WUT?! invalid block count: {}", block_count);
        // }

        // let palette = SizeVec(
        //     vec![
        //         VarInt(0),
        //         VarInt(33),
        //         VarInt(10),
        //         VarInt(50)
        //     ]
        // );

        Self {
            block_count,
            bits_per_block: 14,
            palette: None,
            data_array: SizeVec(data_array),
        }
    }
}

impl types::Size for nbt::Blob {
    fn size(&self) -> types::VarInt {
        let mut vec = Vec::new();
        self.to_writer(&mut vec).unwrap();
        vec.size()
    }
}

#[async_trait::async_trait]
impl types::Send for nbt::Blob {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        let mut vec = Vec::new();
        self.to_writer(&mut vec)?;
        vec.send(writer).await
    }
}

#[derive(Debug)]
struct ChunkData(Vec<ChunkSection>);

impl types::Size for ChunkData {
    fn size(&self) -> types::VarInt {
        let inner_size = self.0.size();
        inner_size.size() + inner_size
    }
}

#[async_trait::async_trait]
impl types::Send for ChunkData {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        self.0.size().send(writer).await?;
        self.0.send(writer).await
    }
}
