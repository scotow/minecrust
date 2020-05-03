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