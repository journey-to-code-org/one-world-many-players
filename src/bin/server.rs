use std::io::{self, BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};

fn handle_connection(mut stream: TcpStream) -> io::Result<()> {
    let mut reader = BufReader::new(stream.try_clone()?);
    writeln!(stream, "Welcome. Commands: look, quit")?;

    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;

        if bytes == 0 {
            break;
        }

        let command = line.trim();
        if command.is_empty() {
            continue;
        }

        if command == "quit" {
            writeln!(stream, "Goodbye")?;
            break;
        }

        writeln!(stream, "You entered: {command}")?;
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4000")?;
    println!("Server listening on 127.0.0.1:4000");

    let (stream, address) = listener.accept()?;
    println!("Connected: {address}");
    handle_connection(stream)
}
