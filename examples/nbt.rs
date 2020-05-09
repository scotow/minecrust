use minecrust::stream::ReadExtension;
use std::fs::{read};

fn main() {
    let mut buf = futures::io::Cursor::new(read(std::env::args().skip(1).next().unwrap()).unwrap());
    smol::run(async {
        buf.read_i64().await.unwrap();
        buf.read_u8().await.unwrap();
        buf.read_var_int().await.unwrap();

        let pos = buf.position() as usize;
        let mut buf = buf.into_inner()[pos..].to_vec();
        let mut buf = std::io::Cursor::new(&mut buf);
        let nbt = nbt::Value::from_reader(0x0a, &mut buf);
        dbg!(nbt);
    });
}
