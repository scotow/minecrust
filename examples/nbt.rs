use minecrust::types::{Receive, VarInt};
use std::fs::read;

fn main() {
    let mut buf = futures::io::Cursor::new(read(std::env::args().skip(1).next().unwrap()).unwrap());
    smol::run(async {
        buf.receive::<i64>().await.unwrap();
        buf.receive::<u8>().await.unwrap();
        buf.receive::<VarInt>().await.unwrap();

        let pos = buf.position() as usize;
        let mut buf = buf.into_inner()[pos..].to_vec();
        let mut buf = std::io::Cursor::new(&mut buf);
        let nbt = nbt::Value::from_reader(0x0a, &mut buf);
        let _ = dbg!(nbt);
    });
}
