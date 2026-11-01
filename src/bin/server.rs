use std::io;

use tokio::net::TcpListener;
use tokio::sync::{broadcast, watch};

use one_world_many_players::server::serve;
use one_world_many_players::world::{World, WorldEvent};
use one_world_many_players::world_server::{self, WorldMessage};

async fn load_world() -> io::Result<World> {
    match tokio::fs::read("world.json").await {
        Ok(bytes) => {
            let saved = serde_json::from_slice(&bytes).map_err(io::Error::other)?;
            World::from_snapshot(saved)
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(World::default()),
        Err(error) => Err(error),
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4000").await?;
    let world = load_world().await?;
    let (world_tx, world_rx) = world_server::channel();
    let (world_events, _) = broadcast::channel::<WorldEvent>(32);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let events_for_owner = world_events.clone();

    let world_thread =
        std::thread::spawn(move || world_server::run(world, world_rx, events_for_owner));
    println!("Server listening on 127.0.0.1:4000");

    let server = tokio::spawn(serve(
        listener,
        world_tx.clone(),
        world_events.clone(),
        shutdown_rx,
    ));

    tokio::signal::ctrl_c().await?;
    shutdown_tx.send(true).map_err(io::Error::other)?;
    server.await.map_err(io::Error::other)??;

    let (reply, saved) = tokio::sync::oneshot::channel();
    world_tx
        .send(WorldMessage::Snapshot { reply })
        .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "world owner stopped"))?;
    let snapshot = saved.await.map_err(io::Error::other)?;
    let json = serde_json::to_vec_pretty(&snapshot).map_err(io::Error::other)?;
    tokio::fs::write("world.json", json).await?;

    drop(world_tx);
    drop(world_events);
    world_thread
        .join()
        .map_err(|_| io::Error::other("world owner thread panicked"))?;
    Ok(())
}
