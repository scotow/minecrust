use minecrust::types::{Receive, VarInt};

fn main() {
    let mut buf = futures::io::Cursor::new(vec![0x8C, 0x10]);
    smol::run(async {
        let int = buf.receive::<VarInt>().await.unwrap();
        println!("{:?}", int);
    });
}
