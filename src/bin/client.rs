use std::io::{self, BufRead, BufReader, Write};
use std::net::TcpStream;

fn main() -> io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:4000")?;
    let mut reader = BufReader::new(stream.try_clone()?);
    let stdin = io::stdin();

    loop {
        let mut reply = String::new();
        if reader.read_line(&mut reply)? == 0 {
            break;
        }
        print!("{reply}");

        let mut command = String::new();
        stdin.read_line(&mut command)?;
        stream.write_all(command.as_bytes())?;

        if command.trim() == "quit" {
            let mut goodbye = String::new();
            reader.read_line(&mut goodbye)?;
            print!("{goodbye}");
            break;
        }
    }

    Ok(())
}
