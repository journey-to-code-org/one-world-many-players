use std::io;
use std::sync::mpsc::{self, Sender};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, watch};
use tokio::task::JoinSet;

use crate::protocol::{Command, parse_command};
use crate::world::{PlayerId, WorldEvent};
use crate::world_server::WorldMessage;

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

async fn run_connection(
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

pub async fn serve(
    listener: TcpListener,
    world_tx: Sender<WorldMessage>,
    world_events: broadcast::Sender<WorldEvent>,
    mut shutdown: watch::Receiver<bool>,
) -> io::Result<()> {
    let mut connections = JoinSet::new();

    loop {
        tokio::select! {
            accepted = listener.accept() => {
                let (stream, _address) = accepted?;
                let world_tx = world_tx.clone();
                let world_events = world_events.subscribe();
                let connection_shutdown = shutdown.clone();

                connections.spawn(async move {
                    run_connection(stream, world_tx, world_events, connection_shutdown).await
                });
            }
            changed = shutdown.changed() => {
                changed.map_err(io::Error::other)?;
                if *shutdown.borrow() {
                    break;
                }
            }
        }
    }

    while let Some(result) = connections.join_next().await {
        result.map_err(io::Error::other)??;
    }

    Ok(())
}
