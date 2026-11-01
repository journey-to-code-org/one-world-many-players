use std::io;
use std::net::SocketAddr;
use std::time::Duration;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, watch};

use one_world_many_players::server::serve;
use one_world_many_players::world::{World, WorldEvent};
use one_world_many_players::world_server;

struct TestClient {
    lines: Lines<BufReader<OwnedReadHalf>>,
    writer: OwnedWriteHalf,
}

impl TestClient {
    async fn connect(address: SocketAddr, name: &str) -> io::Result<Self> {
        let stream = TcpStream::connect(address).await?;
        let (reader, writer) = stream.into_split();
        let mut client = Self {
            lines: BufReader::new(reader).lines(),
            writer,
        };
        client.expect_line("What is your name?").await?;
        client.send(name).await?;
        client
            .expect_line("Welcome. Commands: look, go, who, say, quit")
            .await?;
        Ok(client)
    }

    async fn send(&mut self, line: &str) -> io::Result<()> {
        self.writer.write_all(line.as_bytes()).await?;
        self.writer.write_all(b"\n").await
    }

    async fn expect_line(&mut self, expected: &str) -> io::Result<()> {
        tokio::time::timeout(Duration::from_secs(2), async {
            while let Some(line) = self.lines.next_line().await? {
                if line == expected {
                    return Ok(());
                }
            }
            Err(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "connection closed before the expected line",
            ))
        })
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "server reply timed out"))?
    }
}

#[tokio::test]
async fn two_players_share_chat() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let (shutdown_tx, shutdown_rx) = watch::channel(false);
    let (world_events, _) = broadcast::channel::<WorldEvent>(32);
    let (world_tx, world_rx) = world_server::channel();
    let world_events_for_owner = world_events.clone();
    let world_thread = std::thread::spawn(move || {
        world_server::run(World::default(), world_rx, world_events_for_owner);
    });

    let server = tokio::spawn(serve(listener, world_tx, world_events, shutdown_rx));
    let mut bryce = TestClient::connect(address, "Bryce").await?;
    let mut ryan = TestClient::connect(address, "Ryan").await?;

    bryce.send("say hello").await?;
    ryan.expect_line("Bryce says: hello").await?;

    shutdown_tx.send(true)?;
    server.await??;
    world_thread
        .join()
        .map_err(|_| io::Error::other("world owner thread panicked"))?;

    Ok(())
}
