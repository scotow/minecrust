use crate::types::{FromReader, Receive, Send, Size, TAsyncRead, TAsyncWrite};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::{Add, Deref};

#[derive(Debug, Copy, Clone, Default, Ord, PartialOrd, PartialEq, Eq, Hash)]
pub struct VarInt(pub i32);

impl VarInt {
    const MAX_SIZE: i32 = 5;

    pub fn new(n: i32) -> Self {
        Self(n)
    }

    /// TODO: delete this in favor of the ReadExtension
    pub async fn parse(reader: &mut impl Receive) -> Result<Self> {
        let mut read_int: i32 = 0;
        let mut bytes_read: i32 = 0;
        loop {
            let incoming_byte = reader.receive::<u8>().await?;
            read_int |= ((incoming_byte & 0b0111_1111) as i32) << 7 * bytes_read;
            bytes_read += 1;
            if incoming_byte >> 7 == 0 {
                return Ok(Self::new(read_int));
            } else if bytes_read == Self::MAX_SIZE {
                return Err(anyhow!("VarInt bigger than 5 bytes sent"));
            }
        }
    }
}

#[async_trait::async_trait]
impl FromReader for VarInt {
    async fn from_reader<R: TAsyncRead>(reader: &mut R) -> Result<Self> {
        VarInt::parse(reader).await
    }
}

impl Size for VarInt {
    fn size(&self) -> VarInt {
        Self::new(match self.0 {
            std::i32::MIN..=-1 => Self::MAX_SIZE,
            0 => 1,
            1..=std::i32::MAX => ((self.0 as f64).log2() as i32) / 7 + 1,
        })
    }
}

#[async_trait]
impl Send for VarInt {
    async fn send<W: TAsyncWrite>(&self, writer: &mut W) -> Result<()> {
        let mut n = self.0 as u32;
        loop {
            let tmp = n as u8 & 0b0111_1111;
            n >>= 7;
            if n == 0 {
                tmp.send(writer).await?;
                break;
            } else {
                (tmp | 0b1000_0000).send(writer).await?;
            }
        }
        Ok(())
    }
}

impl Add for VarInt {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        VarInt::new(self.0 + rhs.0)
    }
}

impl Deref for VarInt {
    type Target = i32;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for VarInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
        // write!(f, "{}", *self) // thread 'main' has overflowed its stack
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_u8() {
        assert_eq!(*VarInt::new(-1).size(), VarInt::MAX_SIZE);
        assert_eq!(*VarInt::new(0).size(), 1);
        for i in 0..=((1 << 7) - 1) {
            assert_eq!(*VarInt::new(i).size(), 1);
        }
        assert_eq!(
            *VarInt::new(0b0000_0000__0010_0000__0000_0000__0000_0000).size(),
            4
        );
        assert_eq!(
            *VarInt::new(0b0000_0000__0001_0000__0000_0000__0000_0000).size(),
            3
        );
        assert_eq!(*VarInt::new(std::i32::MAX).size(), VarInt::MAX_SIZE);
    }
}
