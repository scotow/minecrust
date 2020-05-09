use anyhow::Result;
use futures::prelude::*;
use minecrust::stream::ReadExtension;
use minecrust::types::{Send, Size};
use piper::Arc;
use smol::{Async, Task};

use std::net::{TcpListener, TcpStream};
use std::fmt::Display;
use serde::export::Formatter;

use minecrust::packets::play::chat_message::OutChatMessage;
use minecrust::packets::play::join_game::JoinGame;
use minecrust::packets::play::player_info::PlayerInfo;
use minecrust::packets::play::player_position::{OutPlayerPositionLook, InPlayerPosition, InPlayerPositionRotation, InPlayerRotation, OutViewPosition};
use minecrust::packets::play::spawn_player::SpawnPlayer;
use minecrust::packets::play::keep_alive::KeepAlive;
use minecrust::packets::play::chunk::Chunk;
use minecrust::packets::play::entity_position::{OutPosition, OutPositionRotation, OutRotation, OutEntityHeadLook};
use minecrust::packets::play::destroy_entity::DestroyEntity;

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
    let mut first = true;
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

        use minecrust::packets::Packet;
        use minecrust::packets::*;

        let server_to_client = vec![
            0x00, *Handshake::PACKET_ID, *StatusRequest::PACKET_ID,
            0x01, *Ping::PACKET_ID,
            0x02, *LoginRequest::SUCCESS_PACKET_ID,
            0x05, *SpawnPlayer::PACKET_ID,
            0x0F, *OutChatMessage::PACKET_ID,
            0x21, *KeepAlive::PACKET_ID,
            0x22, *Chunk::PACKET_ID,
            0x26, *JoinGame::PACKET_ID,
            0x29, *OutPosition::PACKET_ID,
            0x2A, *OutPositionRotation::PACKET_ID,
            0x2B, *OutRotation::PACKET_ID,
            0x34, *PlayerInfo::PACKET_ID,
            0x36, *OutPlayerPositionLook::PACKET_ID,
            0x38, *DestroyEntity::PACKET_ID,
            0x3C, *OutEntityHeadLook::PACKET_ID,
            // 0x1E, /* Unload chunk */
            // 0x3E, /* World border */
            // 0x25, /* Update light */
            // 0x4F, /* Time update */
            0x41, *OutViewPosition,
        ];
        let client_to_server = vec![
            0x00, *StatusRequest::PACKET_ID,
            0x01, *Ping::PACKET_ID,
            0x0F, /*Keep Alive*/
            0x11, *InPlayerPosition::PACKET_ID,
            0x12, *InPlayerPositionRotation::PACKET_ID,
            0x13, *InPlayerRotation::PACKET_ID
        ];

        if (direction == Direction::ServerToClient && server_to_client.contains(&*packet_id)) ||
            (direction == Direction::ClientToServer && client_to_server.contains(&*packet_id)) {
            // println!("{}: {:02X?} ..", direction, *packet_id);
            writer.write_all(&packet).await?;
        } else if *packet_id == 0x41 {
            println!("{}: {:02X?} ..", direction, *packet_id);
        }

        // if direction == Direction::ClientToServer ||
        //     direction == Direction::ServerToClient && *packet_id != 0x25 {
        //     writer.write_all(&packet).await?;
        // } else {
        //     dbg!(*packet_id);
        // }

        // if direction == Direction::ServerToClient && !server_to_client.contains(&*packet_id) {
        //     println!("{}: {:02X?} ..", direction, *packet_id);
        //     // writer.write_all(&packet).await?;
        // }

        // if *packet_id == 0x25 && first {
        //     first = false;
        //     std::fs::write("mojang_skylight.bin", &packet);
        //
        //     let mut reader = futures::io::Cursor::new(&packet);
        //     &reader.read_var_int().await.unwrap();
        //     &reader.read_var_int().await.unwrap();
        //
        //     dbg!(&reader.read_var_int().await.unwrap());
        //     dbg!(&reader.read_var_int().await.unwrap());
        //     println!("{:#018b}", **&reader.read_var_int().await.unwrap());
        //     println!("{:#018b}", **&reader.read_var_int().await.unwrap());
        //     println!("{:#018b}", **&reader.read_var_int().await.unwrap());
        //     println!("{:#018b}", **&reader.read_var_int().await.unwrap());
        //     dbg!(&reader.read_var_int().await.unwrap());
        // }
    }
}
