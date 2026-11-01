use std::io;
use std::sync::mpsc::{self, Sender};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, watch};
use tokio::task::JoinSet;

use one_world_many_players::protocol::{Command, parse_command};
use one_world_many_players::world::{PlayerId, World, WorldEvent};
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

fn send_command(
    world_tx: &Sender<WorldMessage>,
    player_id: PlayerId,
    line: String,
) -> io::Result<bool> {
    let command = parse_command(&line)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, format!("{error:?}")))?;
    let is_quit = command == Command::Quit;
    world_tx
        .send(WorldMessage::Command { player_id, command })
        .map_err(|_| io::Error::new(io::ErrorKind::BrokenPipe, "world owner stopped"))?;
    Ok(is_quit)
}

async fn handle_connection(
    stream: TcpStream,
    world_tx: Sender<WorldMessage>,
    mut world_events: broadcast::Receiver<WorldEvent>,
    mut shutdown: watch::Receiver<bool>,
) -> io::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();

    writer.write_all(b"What is your name?\n").await?;
    let Some(name) = lines.next_line().await? else {
        return Ok(());
    };
    let (reply_tx, reply_rx) = mpsc::channel();
    world_tx
        .send(WorldMessage::Join {
            name: name.trim().to_owned(),
            reply: reply_tx,
        })
        .map_err(io::Error::other)?;
    let player_id = reply_rx.recv().map_err(io::Error::other)?;
    writer
        .write_all(b"Welcome. Commands: look, go, who, say, quit\n")
        .await?;

    loop {
        tokio::select! {
            line = lines.next_line() => {
                match line? {
                    Some(line) => match send_command(&world_tx, player_id, line) {
                        Ok(true) => break,
                        Ok(false) => {}
                        Err(error) if error.kind() == io::ErrorKind::InvalidInput => {
                            writer.write_all(format!("Invalid command: {error}\n").as_bytes()).await?;
                        }
                        Err(error) => return Err(error),
                    },
                    None => break,
                }
            }
            event = world_events.recv() => {
                match event {
                    Ok(event) if event.is_visible_to(player_id) => {
                        writer.write_all(event.message.as_bytes()).await?;
                        writer.write_all(b"\n").await?;
                    }
                    Ok(_) => {}
                    Err(broadcast::error::RecvError::Lagged(skipped)) => {
                        writer.write_all(format!("You missed {skipped} world events.\n").as_bytes()).await?;
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                }
            }
            changed = shutdown.changed() => {
                changed.map_err(io::Error::other)?;
                if *shutdown.borrow() {
                    writer.write_all(b"Server shutting down.\n").await?;
                    break;
                }
            }
        }
    }

    world_tx
        .send(WorldMessage::Disconnect { player_id })
        .map_err(io::Error::other)?;
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4000").await?;
    let world = load_world().await?;
    let (world_tx, world_rx) = world_server::channel();
    let (world_events, _) = broadcast::channel(32);
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let events_for_owner = world_events.clone();

    let world_thread =
        std::thread::spawn(move || world_server::run(world, world_rx, events_for_owner));
    let mut connections = JoinSet::new();
    println!("Server listening on 127.0.0.1:4000");

    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, address) = accepted?;
                let world_tx = world_tx.clone();
                let world_events = world_events.subscribe();
                let shutdown = shutdown_rx.clone();
                connections.spawn(async move {
                    if let Err(error) = handle_connection(stream, world_tx, world_events, shutdown).await {
                        eprintln!("Connection {address} failed: {error}");
                    }
                    Ok::<(), io::Error>(())
                });
            }
            result = tokio::signal::ctrl_c() => {
                result?;
                let _ = shutdown_tx.send(true);
                break;
            }
        }
    }

    while let Some(result) = connections.join_next().await {
        result.map_err(io::Error::other)??;
    }

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
