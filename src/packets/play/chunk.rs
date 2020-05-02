use crate::types::{self, LengthVec, Size, VarInt, SizeVec};
use anyhow::Result;
use bitvec::order::{Lsb0, Msb0};
use bitvec::vec::BitVec;
use futures::prelude::*;
use std::collections::HashMap;
use std::env::var;

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Chunk {
    x: i32,
    z: i32,
    // heightmaps: [i64; 36],
    // biomes: [i32; 1024],
    // sections: [Option<ChunkSection>; 16]
}
crate::impl_packet!(Chunk, 0x22);

impl Chunk {
    pub fn new(x: i32, z: i32) -> Self {
        Self {
            x,
            z,
            // heightmaps: [0; 36],
            // biomes: [127; 1024],
            // sections: [None; 16]
        }
    }

    pub fn set_block(&mut self, x: u8, y: i16, z: u8, block: Block) {
        // if block != Block::Air && y > extract_value(&self.heightmaps, x, z) {
        //
        // }


        // let mut bitmask: i32 = 0;
        // let mut sections = SizeVec::new();
        // let mut heightmaps = vec![vec![0; 16]; 16];
        //
        // for section_index in 0..16 {
        //     let mut section: Vec<Block> = Vec::new();
        //     for y in (section_index * 16)..((section_index + 1) * 16) {
        //         for z in 0..16 {
        //             for x in 0..16 {
        //                 if data[y][z][x] != Block::Air {
        //                     // Notchian server adds 1 here, don't know why.
        //                     if heightmaps[z][x] < y {
        //                         heightmaps[z][x] = y;
        //                     }
        //                     bitmask |= 1 << section_index;
        //                 }
        //                 section.push(data[y][z][x]);
        //             }
        //         }
        //     }
        //     if bitmask & (1 << section_index) != 0 {
        //         let section = ChunkSection::new(section);
        //         sections.push(section);
        //     }
        // }
        //
        // let mut bit_vec_heightmaps = BitVec::<Lsb0, u64>::new();
        // for z in heightmaps {
        //     for y in z {
        //         let mut truc = as_bitvec(y);
        //         bit_vec_heightmaps.append(&mut truc);
        //     }
        // }
        //
        // let heightmaps: Vec<i64> = bit_vec_heightmaps
        //     .as_slice()
        //     .iter()
        //     .map(|el| *el as i64)
        //     .collect();
        // let heightmaps: nbt::Value = heightmaps.into();
        //
        // let mut blob = nbt::Blob::new();
        // blob.insert("MOTION_BLOCKING".to_string(), heightmaps.clone());
        // blob.insert("WORLD_SURFACE".to_string(), heightmaps);
        //
        // let biomes = vec![1_i32; 1024];
        //
        // Self {
        //     x,
        //     z,
        //     full_chunk: true,
        //     primary_bit_mask: types::VarInt(bitmask),
        //     heightmaps: blob,
        //     biomes,
        //     data: sections,
        //     block_entities: LengthVec::new(),
        // }
    }
}

// let end = (y * 16 + x + 1) * 9 / 64;
fn extract_value(table: &[i64; 36], x: u8, z: u8) -> i16 {
    let x = x as usize;
    let z = z as usize;
    let start_index = (z * 16 + x) * 9 / 64;
    let end_index = ((z * 16 + x + 1) * 9 - 1) / 64;
    let start_bit = (z * 16 + x) * 9 % 64;

    if start_index == end_index {
        let buffer = table[start_index] as u64;
        let mask = 0b111_111_111_u64 << (start_bit as u64);

        ((buffer & mask) >> (start_bit as u64)) as i16
    } else {
        let buffer = ((table[start_index + 1] as u64 as u128) << 64) | (table[start_index] as u64 as u128);
        let mask = 0b111_111_111_u128 << (start_bit as u128);

        ((buffer & mask) >> (start_bit as u128)) as i16
    }
}

fn set_value(table: &mut [i64; 36], x: u8, z: u8, value: i16) {
    let x = x as usize;
    let z = z as usize;
    let start_index = (z * 16 + x) * 9 / 64;
    let end_index = ((z * 16 + x + 1) * 9 - 1) / 64;
    let start_bit = (z * 16 + x) * 9 % 64;

    if start_index == end_index {
        let clear = !(0b111_111_111_u64 << (start_bit as u64));
        let mut buffer = table[start_index] as u64 & clear;
        buffer |= (value as u64) << (start_bit as u64);
        table[start_index] = buffer as i64;
    } else {
        let clear = !(0b111_111_111_u128 << (start_bit as u128));
        let mut buffer = (((table[start_index + 1] as u64 as u128) << 64) | (table[start_index] as u64 as u128)) & clear;
        buffer |= (value as u16 as u128) << (start_bit as u128);

        table[start_index + 1] = (buffer >> 64) as i64;
        table[start_index] = buffer as i64;
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
    palette: Option<LengthVec<VarInt>>,
    data_array: LengthVec<i64>,
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
            data_array: LengthVec(data_array),
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

#[cfg(test)]
mod tests {
    use super::*;
    use rand::seq::SliceRandom;

    const ALL_4: [i64; 36] = [
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
    ];

    #[test]
    fn test_extract_value() {
        let table = [0_i64; 36];
        assert_eq!(extract_value(&table, 0, 0), 0);

        let mut table = [0_i64; 36];
        table[0] |= 0b111_111_111;
        assert_eq!(extract_value(&table, 0, 0), 0b111_111_111);

        let mut table = [0_i64; 36];
        table[0] |= 0b111_111_111 << 9;
        assert_eq!(extract_value(&table, 1, 0), 0b111_111_111);

        let mut table = [0_i64; 36];
        table[0] |= 0b111_111_111 << 14;
        assert_eq!(extract_value(&table, 1, 0), 0b111_100_000);
        assert_eq!(extract_value(&table, 2, 0), 0b000_011_111);

        let mut table = [0_i64; 36];
        table[0] |= 0b000_000_001 << 63;
        table[1] |= 0b011_111_111 << 0;
        assert_eq!(extract_value(&table, 7, 0), 0b111_111_111);

        for z in 0..16 {
            for x in 0..16 {
                assert_eq!(extract_value(&ALL_4, x, z), 4);
            }
        }
    }

    #[test]
    fn test_set_value() {
        let mut table = [0_i64; 36];

        set_value(&mut table, 0, 0, 5);
        assert_eq!(extract_value(&table, 0, 0), 5);

        set_value(&mut table, 1, 0, 2);
        assert_eq!(extract_value(&table, 0, 0), 5);
        assert_eq!(extract_value(&table, 1, 0), 2);


        let mut rng = rand::thread_rng();
        let mut z_range = (0..16).collect::<Vec<_>>();
        z_range.shuffle(&mut rng);

        for z in z_range {
            let mut x_range = (0..16).collect::<Vec<_>>();
            x_range.shuffle(&mut rng);

            for x in x_range {
                set_value(&mut table, x, z, 4);
                assert_eq!(extract_value(&table, x, z), 4);
            }
        }

        assert_eq!(table.to_vec(), ALL_4.to_vec());
    }
}
