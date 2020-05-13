use crate::types::{self, BlockPosition, Receive, TAsyncRead, VarInt};
use anyhow::{ensure, Result};

#[derive(Debug)]
pub enum PlayerDigging {
    StartedDigging(BlockPosition, Face),
    CancelledDigging(BlockPosition, Face),
    FinishedDigging(BlockPosition, Face),
    DropStackItem,
    DropItem,
    UsingItem,
    SwapItem,
}

impl PlayerDigging {
    pub const PACKET_ID: VarInt = VarInt(0x1A);
}

#[async_trait::async_trait]
impl types::FromReader for PlayerDigging {
    async fn from_reader<R: TAsyncRead>(reader: &mut R) -> Result<Self> {
        let status: VarInt = reader.receive().await?;
        ensure!((0..=6).contains(&*status), "invalid status");

        let location: BlockPosition = reader.receive().await?;

        let face = reader.receive::<u8>().await?;
        ensure!((0..=5).contains(&face), "invalid face");
        let face = Face::from(face);

        use PlayerDigging::*;
        Ok(match *status {
            0 => StartedDigging(location, face),
            1 => CancelledDigging(location, face),
            2 => FinishedDigging(location, face),
            3 => DropStackItem,
            4 => DropItem,
            5 => UsingItem,
            6 => SwapItem,
            _ => unreachable!(),
        })
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum Face {
    Bottom = 0,
    Top,
    North,
    South,
    West,
    East,
}

impl From<u8> for Face {
    fn from(n: u8) -> Face {
        unsafe { std::mem::transmute(n) }
    }
}
