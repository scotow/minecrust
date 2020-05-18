use futures::prelude::*;
use minecrust::game::map::generator::FlatChunkGenerator;
use minecrust::game::ServerBuilder;
use smol::{Async, Task};
use std::net::TcpListener;
use std::time::Duration;

fn main() -> ! {
    let world = ServerBuilder::new()
        .with_players((1, 0))
        .with_description("Rusty Minecraft Server".into())
        .with_icon_from_path("./examples/assets/server-icon.png")
        .unwrap()
        .build_leak(FlatChunkGenerator::new());

    let world = smol::block_on(world);
    eprintln!("World map generated.");

    let listener = Async::<TcpListener>::bind("127.0.0.1:25565").unwrap();
    let mut incoming = listener.incoming();
    smol::run(async move {
        Task::spawn(world.run(Duration::from_secs(1))).detach();

        while let Some(stream) = incoming.next().await {
            Task::spawn(async move {
                // ignore what happens if a connection fail
                let _ = world.handle_connection_stream(stream.unwrap()).await;
            })
            .detach();
        }
    });
    panic!("This should never happens");
}
