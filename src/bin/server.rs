use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use one_world_many_players::protocol::{Command, parse_command};

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    writeln!(stream, "Welcome. Commands: look, quit")?;

    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;

        if bytes == 0 {
            break;
        }

        match parse_command(&line) {
            Ok(Command::Look) => writeln!(stream, "You see an empty room.")?,
            Ok(Command::Say(message)) => writeln!(stream, "You say: {message}")?,
            Ok(Command::Who) => writeln!(stream, "You are the only player.")?,
            Ok(Command::Quit) => {
                writeln!(stream, "Goodbye")?;
                break;
            }
            Err(error) => writeln!(stream, "Invalid command: {error:?}")?,
        }
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4000")?;
    println!("Server listening on 127.0.0.1:4000");

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                let address = stream.peer_addr()?;
                println!("Connected: {address}");

                thread::spawn(move || {
                    if let Err(error) = handle_connection(stream) {
                        eprintln!("Connection {address} failed: {error}");
                    }
                });
            }
            Err(error) => eprintln!("Accept failed: {error}"),
        }
    }

    Ok(())
}
