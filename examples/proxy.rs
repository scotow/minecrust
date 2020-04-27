use anyhow::Result;
use futures::prelude::*;
use minecrust::stream::ReadExtension;
use minecrust::types::{Send, Size};
use piper::Arc;
use smol::{Async, Task};

use std::net::{TcpListener, TcpStream};

fn main() {
    let listener = Async::<TcpListener>::bind("127.0.0.1:25566").unwrap();
    let mut incoming = listener.incoming();
    smol::run(async {
        while let Some(stream) = incoming.next().await {
            let stream = stream.unwrap();
            Task::spawn(handle_connexion(stream)).unwrap().detach();
        }
    });
}

async fn handle_connexion(
    client_stream: Async<TcpStream>,
) -> Result<()> {
    let mut client_reader = Arc::new(client_stream);
    let mut client_writer = client_reader.clone();

    let server_stream = Async::<TcpStream>::connect("127.0.0.1:25565").await?;
    let mut server_reader = Arc::new(server_stream);
    let mut server_writer = server_reader.clone();

    futures::join!(
        filter_packet(&mut server_reader, &mut client_writer, "S -> C"),
        filter_packet(&mut client_reader, &mut server_writer, "C -> S")
    );
    Ok(())
}

async fn filter_packet<R, W>(reader: &mut R, writer: &mut W, direction: &str) -> Result<()>
    where
        R: AsyncRead + Unpin + Sized + std::marker::Send,
        W: AsyncWrite + Unpin + std::marker::Send,
{
    loop {
        let size = reader.read_var_int().await?;

        let mut packet = Vec::with_capacity(*size as usize);
        size.send(&mut packet).await?;

        let packet_id = reader.read_var_int().await?;
        packet_id.send(&mut packet).await?;

        reader.take((*size - *packet_id.size()) as u64).read_to_end(&mut packet).await?;

        // futures::io::copy(server.take((*size - *packet_id.size()) as u64), client).await?;

        if [0x00, 0x01, 0x02, 0x26, 0x36].contains(&*packet_id) {
            println!("{}: {:02X?} ..", direction, *packet_id);
            writer.write_all(&packet).await?;
        } else if [0x48, 0x15, 0x4E, 0x4F, 0x4E, 0x3E, 0x19, 0x22, 0x32, 0x40, 0x5B, 0x5C, 0x41, 0x1C, 0x12, 0x37, 0x34, 0x25, 0x17, 0x3F, 0x49, 0x30, 0x0E, 0x4A].contains(&*packet_id) && direction == "S -> C" {
            // println!("{}: {:02X?}", direction, &packet[*size.size() as usize..]);
        } else {
            // println!("{}: {:02X?} ..", direction, *packet_id);
            // writer.write_all(&packet).await?;
        }

        // if *packet_id == 0x25 {
        //     println!("Size: {}", *size);
        // }
    }

    Ok(())
}
