use std::io::{self, Write};
use std::net::TcpListener;

fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4000")?;
    println!("Server listening on 127.0.0.1:4000");

    let (mut stream, address) = listener.accept()?;
    println!("Connected: {address}");
    writeln!(stream, "Welcome to One World")?;

    Ok(())
}
