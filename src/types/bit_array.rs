use std::ops::{Index, IndexMut, Deref};
use crate::types::{LengthVec, Size, VarInt, Send};
use futures::AsyncWrite;
use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct BitArray<T> {
    data: T,
    bits_per_value: usize,
}

impl BitArray<Vec<u64>> {
    pub fn new(size: usize, bits_per_value: usize) -> Self {
        Self {
            data: vec![0_u64; size],
            bits_per_value,
        }
    }

    pub fn from_slice(data: &[u64], bits_per_value: usize) -> Self {
        Self {
            data: data.to_vec(),
            bits_per_value,
        }
    }
}

impl BitArray<LengthVec<u64>> {
    pub fn new(size: usize, bits_per_value: usize) -> Self {
        Self {
            data: LengthVec::from(vec![0_u64; size]),
            bits_per_value,
        }
    }

    pub fn from_slice(data: &[u64], bits_per_value: usize) -> Self {
        Self {
            data: LengthVec::from(data.to_vec()),
            bits_per_value,
        }
    }
}

impl<'a, T: Deref<Target=[u64]>> BitArray<T> {
    pub fn as_slice(&'a self) -> &'a [u64] {
        &self.data
    }
}

impl<T: Size> Size for BitArray<T> {
    fn size(&self) -> VarInt {
        self.data.size()
    }
}

#[async_trait::async_trait]
impl<T: Send + std::marker::Send + Unpin + Sync> Send for BitArray<T> {
    async fn send<W: AsyncWrite + std::marker::Send + Unpin>(&self, writer: &mut W) -> Result<()> {
        self.data.send(writer).await
    }
}

impl<T: Index<usize, Output=u64> + IndexMut<usize, Output=u64>> BitArray<T> {
    pub fn get(&self, position: usize) -> u16 {
        let start_index = position * self.bits_per_value / 64;
        let end_index = ((position + 1) * self.bits_per_value - 1) / 64;
        let start_bit = (position * self.bits_per_value % 64) as u64;

        if start_index == end_index {
            let buffer = self.data[start_index];
            let mask = !(!0_u64 << self.bits_per_value as u64) << start_bit;

            ((buffer & mask) >> start_bit) as u16
        } else {
            let start_bit = start_bit as u128;

            let buffer = ((self.data[start_index + 1] as u128) << 64) | (self.data[start_index] as u128);
            let mask = !(!0_u128 << self.bits_per_value as u128) << start_bit;

            ((buffer & mask) >> start_bit) as u16
        }.into()
    }

    pub fn set(&mut self, position: usize, value: u16) {
        let start_index = position * self.bits_per_value / 64;
        let end_index = ((position + 1) * self.bits_per_value - 1) / 64;
        let start_bit = (position * self.bits_per_value % 64) as u64;

        if start_index == end_index {
            let clear = !(!(!0_u64 << self.bits_per_value as u64) << start_bit);
            let mut buffer = self.data[start_index] & clear;
            buffer |= ((value as u64) << start_bit) & !clear;

            self.data[start_index] = buffer;
        } else {
            let start_bit = start_bit as u128;

            let clear = !(!(!0_u128 << self.bits_per_value as u128) << start_bit);
            let mut buffer = (((self.data[start_index + 1] as u128) << 64) | (self.data[start_index] as u128)) & clear;
            buffer |= ((value as u128) << start_bit & !clear);

            self.data[start_index + 1] = (buffer >> 64) as u64;
            self.data[start_index] = buffer as u64;
        }
    }
}
