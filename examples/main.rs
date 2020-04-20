use std::io::{Read, Write};
use std::net::TcpListener;

use minecrust::error::Result;
use minecrust::packets::{Handshake, LoginRequest, PingRequest, ServerDescription, StatusRequest};
use minecrust::stream::ReadExtension;

fn main() {
    let mut buffer = std::io::Cursor::new(vec![0xDE, 0xAD]);
    assert_eq!(buffer.read_u8().unwrap(), 0xDE);

    let mut server_description: ServerDescription = Default::default();
    server_description.players = (3, 42);
    server_description.description = "Rusty Minecraft Server".to_string();
    server_description.icon = std::fs::read("./examples/assets/server-icon.png").ok();

    let server_description: &'static ServerDescription = Box::leak(Box::new(server_description));

    let listener = TcpListener::bind("127.0.0.1:25565").unwrap();
    for stream in listener.incoming() {
        std::thread::spawn(move || {
            let mut stream = stream.unwrap();

            let handshake = Handshake::parse(&mut stream).unwrap();
            println!("{:?}", handshake);

            match *handshake.next_state {
                1 => handle_status(&mut stream, server_description),
                2 => {
                    handle_login(&mut stream).unwrap();
                    handle_play(&mut stream)
                }
                _ => panic!("impossible"),
            }
            .unwrap();
        });
    }
}

fn handle_status(
    stream: &mut (impl Read + Write),
    server_description: &ServerDescription,
) -> Result<()> {
    let status_request = StatusRequest::parse(stream)?;
    status_request.answer(stream, server_description)?;
    stream.flush()?;
    println!("Status sent.");

    let ping_request = PingRequest::parse(stream)?;
    ping_request.answer(stream)?;
    stream.flush()?;
    println!("Pong sent.");
    Ok(())
}

fn handle_login(stream: &mut (impl Read + Write)) -> Result<()> {
    let login_start = LoginRequest::parse(stream)?;
    login_start.answer(stream)?;
    stream.flush()?;
    println!("{:?}", login_start);
    Ok(())
}

fn handle_play(stream: &mut (impl Read + Write)) -> Result<()> {
    stream
        .bytes()
        .for_each(|b| print!("{:02x} ", b.unwrap_or(0x00)));
    Ok(())
}
