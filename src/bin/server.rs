use std::io;
use std::sync::mpsc::{self, Sender};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc as tokio_mpsc;

use one_world_many_players::protocol::{Command, parse_command};
use one_world_many_players::world::World;
use one_world_many_players::world_server::{self, WorldMessage};

async fn handle_connection(stream: TcpStream, world_tx: Sender<WorldMessage>) -> io::Result<()> {
    let (reader, mut writer) = stream.into_split();
    let mut lines = BufReader::new(reader).lines();
    let (outgoing, mut incoming) = tokio_mpsc::unbounded_channel::<String>();
    let direct = outgoing.clone();

    writer.write_all(b"What is your name?\n").await?;
    let Some(name) = lines.next_line().await? else {
        return Ok(());
    };
    let name = name.trim().to_owned();
    let (reply_tx, reply_rx) = mpsc::channel();

    world_tx
        .send(WorldMessage::Join {
            name,
            outgoing,
            reply: reply_tx,
        })
        .map_err(io::Error::other)?;
    let player_id = reply_rx.recv().map_err(io::Error::other)?;
    writer
        .write_all(b"Welcome. Commands: look, go, who, say, quit\n")
        .await?;
    let writer_task = tokio::spawn(async move {
        while let Some(message) = incoming.recv().await {
            writer.write_all(message.as_bytes()).await?;
            writer.write_all(b"\n").await?;
        }
        Ok::<(), io::Error>(())
    });

    while let Some(line) = lines.next_line().await? {
        match parse_command(&line) {
            Ok(command) => {
                let is_quit = command == Command::Quit;
                world_tx
                    .send(WorldMessage::Command { player_id, command })
                    .map_err(io::Error::other)?;
                if is_quit {
                    break;
                }
            }
            Err(error) => {
                direct.send(format!("Invalid command: {error:?}")).ok();
            }
        }
    }

    world_tx
        .send(WorldMessage::Leave { player_id })
        .map_err(io::Error::other)?;
    drop(direct);
    writer_task.await.map_err(io::Error::other)??;
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4000").await?;
    let (world_tx, world_rx) = world_server::channel();

    std::thread::spawn(move || world_server::run(World::default(), world_rx));
    println!("Server listening on 127.0.0.1:4000");

    loop {
        let (stream, address) = listener.accept().await?;
        let world_tx = world_tx.clone();

        tokio::spawn(async move {
            if let Err(error) = handle_connection(stream, world_tx).await {
                eprintln!("Connection {address} failed: {error}");
            }
        });
    }
}
