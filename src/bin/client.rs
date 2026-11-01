use std::io::{self, BufRead, BufReader};
use std::net::TcpStream;

fn main() -> io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:4000")?;
    let mut reader = BufReader::new(stream);
    let mut message = String::new();

    reader.read_line(&mut message)?;
    print!("{message}");

    Ok(())
}
