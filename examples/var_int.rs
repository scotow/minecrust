use minecrust::stream::ReadExtension;
use std::fs::{read, write};

fn main() {
    let mut buf = futures::io::Cursor::new(vec![0x8C, 0x10]);
    smol::run(async {
        let int = buf.read_var_int().await.unwrap();
        println!("{:?}", int);
    });
}
