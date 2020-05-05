use anyhow::Result;
use futures::prelude::*;
use minecrust::stream::ReadExtension;
use minecrust::types::{Send, Size};
use piper::Arc;
use smol::{Async, Task};

use std::net::{TcpListener, TcpStream};
use std::fmt::Display;
use serde::export::Formatter;

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

async fn handle_connexion(client_stream: Async<TcpStream>) -> Result<()> {
    let mut client_reader = Arc::new(client_stream);
    let mut client_writer = client_reader.clone();

    let server_stream = Async::<TcpStream>::connect("127.0.0.1:25565").await?;
    let mut server_reader = Arc::new(server_stream);
    let mut server_writer = server_reader.clone();

    // ignore the result
    let _ = futures::join!(
        filter_packet(&mut server_reader, &mut client_writer, Direction::ServerToClient),
        filter_packet(&mut client_reader, &mut server_writer, Direction::ClientToServer)
    );
    Ok(())
}

#[derive(Copy, Clone, PartialEq)]
enum Direction {
    ServerToClient,
    ClientToServer,
}

impl Display for Direction {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            Direction::ServerToClient => write!(f, "S - > C")?,
            Direction::ClientToServer => write!(f, "C - > S")?,
        }
        Ok(())
    }
}

async fn filter_packet<R, W>(reader: &mut R, writer: &mut W, direction: Direction) -> Result<()>
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

        reader
            .take((*size - *packet_id.size()) as u64)
            .read_to_end(&mut packet)
            .await?;

        // futures::io::copy(server.take((*size - *packet_id.size()) as u64), client).await?;

        let server_to_client = vec![0x00, 0x01, 0x02, 0x0F, 0x21, 0x22, 0x26, 0x34, 0x36];
        let client_to_server = vec![0x00, 0x01, 0x02, 0x0F, 0x11, 0x12, 0x13, 0x14];

        if (direction == Direction::ServerToClient && server_to_client.contains(&*packet_id)) ||
            (direction == Direction::ClientToServer && client_to_server.contains(&*packet_id)) {
            println!("{}: {:02X?} ..", direction, *packet_id);
            writer.write_all(&packet).await?;
        }

        if direction == Direction::ServerToClient && *packet_id == 0x34 {
            println!("{}: {:02X?}", direction, &packet);
        }

        // let mut first = true;
        // if *packet_id == 0x22 && first {
        //     first = false;
        //     std::fs::write("minecrust_chunk8.bin", &packet);
        //     // std::fs::write("mojang_chunk.bin", &packet);
        // }
    }
}
