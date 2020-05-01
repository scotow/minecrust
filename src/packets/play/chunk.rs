use crate::types::{self, SizeVec};
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
    heightmaps: nbt::Value,
    /*
    biomes: Vec<i32>,
    data: Vec<ChunkSection>,
    block_entities: SizeVec<u8>,
    */
    _inner: Vec<u8>,
}
crate::impl_packet!(Chunk, 0x22);

impl Chunk {
    ///                              y z x
    pub fn new(x: i32, z: i32, data: &[Vec<Vec<Block>>], path: &str) -> Self {
        let mut bitmask: i32 = 0;
        let mut output_data = Vec::new();
        let mut heightmaps = vec![vec![0; 16]; 16];

        for section_index in 0..16 {
            let mut sub_section: Vec<Block> = Vec::new();
            for y in (section_index * 16)..((section_index + 1) * 16) {
                for z in 0..16 {
                    for x in 0..16 {
                        if data[y][z][x] != Block::Air {
                            if heightmaps[z][x] < y {
                                heightmaps[z][x] = y;
                            }
                            bitmask |= 1 << section_index;
                        }
                        sub_section.push(data[y][z][x]);
                    }
                }
            }
            if bitmask & (1 << section_index) != 0 {
                let chunk = ChunkSection::new(sub_section);
                output_data.push(chunk);
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
        let mut hash = HashMap::new();
        hash.insert("MOTION_BLOCKING".to_string(), heightmaps.clone());
        hash.insert("WORLD_SURFACE".to_string(), heightmaps);
        let heightmaps = nbt::Value::Compound(hash);

        let mut hash = HashMap::new();
        hash.insert("".to_string(), heightmaps);
        let heightmaps = nbt::Value::Compound(hash);
        let biomes = vec![2_i32; 1024];

        use crate::types::size::Size;
        let data = std::fs::read(path).unwrap();
        let size = *(x.size()
            + z.size()
            + true.size()
            + types::VarInt(bitmask).size()
            + heightmaps.size()
            + biomes.size()) as usize;

        // here we should get 8A 10 and it's not the case
        dbg!(&data[size..size + 2]);

        Self {
            x,
            z,
            full_chunk: true,
            primary_bit_mask: types::VarInt(bitmask),
            heightmaps,
            /*
            biomes,
            data: output_data,
            block_entities: SizeVec::new(),
            */
            _inner: data[size..].to_vec(),
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

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(u16)]
pub enum Block {
    Air = 0,
    Stone = 1,
    Dirt = 10,
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
    palette: Option<u8>,
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

        Self {
            block_count,
            bits_per_block: 14,
            palette: None,
            data_array: SizeVec(data_array),
        }
    }
}

impl types::Size for nbt::Value {
    fn size(&self) -> types::VarInt {
        let mut vec = Vec::new();
        self.to_writer(&mut vec).unwrap();
        vec.size()
    }
}

#[async_trait::async_trait]
impl types::Send for nbt::Value {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        let mut vec = Vec::new();
        self.to_writer(&mut vec)?;
        vec.send(writer).await
    }
}
