use crate::impl_packet;
use crate::types::Size;

#[derive(Debug, macro_derive::Size, macro_derive::Send)]
pub struct Chunk {
    x: i32,
    z: i32,
    data: Vec<u8>,
}
impl_packet!(Chunk, 0x22);

impl Chunk {
    pub fn new(x: i32, z: i32, path: &str) -> Self {
        let data = std::fs::read(path).unwrap();
        // let x = usize::from_be_bytes(data[0..4]);
        // let z = usize::from_be_bytes(data[4..8]);
        // dbg!(x, z);
        let size = *(x.size() + z.size()) as usize;
        Self {
            x,
            z,
            data: data[size..].to_vec(),
            // data,
        }
    }
}
