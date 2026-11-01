use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{self, Sender};
use std::thread;

use one_world_many_players::protocol::{Command, parse_command};
use one_world_many_players::world::World;
use one_world_many_players::world_server::{self, WorldMessage};

fn handle_connection(stream: TcpStream, world_tx: Sender<WorldMessage>) -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    let (outgoing, incoming) = mpsc::channel::<String>();
    let mut writer = stream.try_clone()?;

    thread::spawn(move || {
        while let Ok(message) = incoming.recv() {
            if writeln!(writer, "{message}").is_err() {
                break;
            }
        }
    });

    outgoing.send("What is your name?".into()).ok();
    let mut name = String::new();
    if reader.read_line(&mut name)? == 0 {
        return Ok(());
    }
    let name = name.trim().to_owned();
    let (reply_tx, reply_rx) = mpsc::channel();

    world_tx
        .send(WorldMessage::Join {
            name,
            outgoing: outgoing.clone(),
            reply: reply_tx,
        })
        .map_err(io::Error::other)?;
    let player_id = reply_rx.recv().map_err(io::Error::other)?;
    outgoing
        .send("Welcome. Commands: look, go, who, say, quit".into())
        .ok();

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }

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
                outgoing.send(format!("Invalid command: {error:?}")).ok();
            }
        }
    }

    world_tx
        .send(WorldMessage::Leave { player_id })
        .map_err(io::Error::other)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4000")?;
    let (world_tx, world_rx) = world_server::channel();

    thread::spawn(move || world_server::run(World::default(), world_rx));
    println!("Server listening on 127.0.0.1:4000");

    for connection in listener.incoming() {
        let stream = connection?;
        let address = stream.peer_addr()?;
        let world_tx = world_tx.clone();

        thread::spawn(move || {
            if let Err(error) = handle_connection(stream, world_tx) {
                eprintln!("Connection {address} failed: {error}");
            }
        });
    }

    Ok(())
}
