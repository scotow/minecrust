use minecrust::stream::ReadExtension;
use std::net::TcpListener;
use minecrust::packets::{Handshake, StatusRequest, ServerDescription, PingRequest};
use std::io::Write;

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

            let status_request = StatusRequest::parse(&mut stream).unwrap();
            status_request.answer(&mut stream, server_description).unwrap();
            stream.flush().unwrap();

            println!("Status sent.");

            let ping_request = PingRequest::parse(&mut stream).unwrap();
            ping_request.answer(&mut stream).unwrap();
            stream.flush().unwrap();
        });
    }
}