use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use one_world_many_players::protocol::{Command, parse_command};
use one_world_many_players::world::World;

type SharedWorld = Arc<Mutex<World>>;

fn handle_connection(stream: TcpStream, world: SharedWorld) -> io::Result<()> {
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

    let player_id = {
        let mut world = world.lock().expect("world mutex poisoned");
        let id = world.join(name.clone(), outgoing.clone());
        world.broadcast(&format!("{name} joined the world."));
        id
    };
    outgoing
        .send("Welcome. Commands: look, who, say, quit".into())
        .ok();

    loop {
        let mut line = String::new();
        if reader.read_line(&mut line)? == 0 {
            break;
        }

        match parse_command(&line) {
            Ok(Command::Look) => outgoing.send("You see an empty room.".into()).ok(),
            Ok(Command::Say(message)) => {
                world
                    .lock()
                    .expect("world mutex poisoned")
                    .broadcast(&format!("{name} says: {message}"));
                Some(())
            }
            Ok(Command::Who) => {
                let names = {
                    let world = world.lock().expect("world mutex poisoned");
                    world.player_names().join(", ")
                };
                outgoing.send(format!("Online: {names}")).ok()
            }
            Ok(Command::Quit) => {
                outgoing.send("Goodbye".into()).ok();
                break;
            }
            Err(error) => outgoing.send(format!("Invalid command: {error:?}")).ok(),
        };
    }

    let mut world = world.lock().expect("world mutex poisoned");
    world.broadcast(&format!("{name} left the world."));
    world.leave(player_id);
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
