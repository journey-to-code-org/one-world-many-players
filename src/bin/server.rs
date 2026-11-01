use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

use one_world_many_players::protocol::{Command, parse_command};
use one_world_many_players::world::World;

type SharedWorld = Arc<Mutex<World>>;

fn handle_connection(mut stream: TcpStream, world: SharedWorld) -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    writeln!(stream, "What is your name?")?;

    let mut name = String::new();
    if reader.read_line(&mut name)? == 0 {
        return Ok(());
    }
    let name = name.trim().to_owned();
    let player_id = world.lock().expect("world mutex poisoned").join(name);

    writeln!(stream, "Welcome. Commands: look, who, say, quit")?;

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        match parse_command(&line) {
            Ok(Command::Look) => writeln!(stream, "You see an empty room.")?,
            Ok(Command::Say(message)) => writeln!(stream, "You say: {message}")?,
            Ok(Command::Who) => {
                let names = {
                    let world = world.lock().expect("world mutex poisoned");
                    world.player_names().join(", ")
                };
                writeln!(stream, "Online: {names}")?;
            }
            Ok(Command::Quit) => {
                writeln!(stream, "Goodbye")?;
                break;
            }
            Err(error) => writeln!(stream, "Invalid command: {error:?}")?,
        }
    }

    world.lock().expect("world mutex poisoned").leave(player_id);
    Ok(())
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4000")?;
    let world = Arc::new(Mutex::new(World::default()));
    println!("Server listening on 127.0.0.1:4000");

    for connection in listener.incoming() {
        let stream = connection?;
        let address = stream.peer_addr()?;
        let world = Arc::clone(&world);

        thread::spawn(move || {
            if let Err(error) = handle_connection(stream, world) {
                eprintln!("Connection {address} failed: {error}");
            }
        });
    }

    Ok(())
}
