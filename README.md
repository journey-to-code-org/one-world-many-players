# One World, Many Players

Companion code for **[Rustbound II: One World, Many Players](https://journeytocode.io/series/one-world-many-players/)**, a Journey to Code series about building a multiplayer text-game server in Rust.

This is a sequel in skills, not source code. Rustbound I built a browser idle game. This series starts a new Cargo project so sockets, shared ownership, threads, channels, Tokio, and persistence can appear without a WebAssembly application hiding them.

## Follow one lesson

Each `article_N` branch is the tested project at the end of that lesson. Matching immutable tags use `article-01` through `article-15`.

```bash
git clone https://github.com/journey-to-code-org/one-world-many-players.git
cd one-world-many-players
git switch article_1
cargo test -j 2 --all-targets
```

| Checkpoint | Lesson | Main idea |
| --- | --- | --- |
| `article_1` | A Server With Nobody to Talk To | `TcpListener` and `TcpStream` |
| `article_2` | One Message Is Not a Conversation | connection loops |
| `article_3` | A String Is Not a Command | `Option`, `Result`, and command parsing |
| `article_4` | The Second Player Is Still Waiting | worker threads and `move` |
| `article_5` | Two Players Do Not Make a World | `Arc<Mutex<World>>` |
| `article_6` | Say It Where Everyone Can Hear It | channels and broadcasts |
| `article_7` | The World Should Own Itself | one state owner and typed messages |
| `article_8` | Walking Needs Somewhere to Go | newtypes and room graphs |
| `article_9` | Different Things Can Share a Behavior | traits and generic bounds |
| `article_10` | One Room, Several Kinds of Things | `Box<dyn Trait>` |
| `article_11` | A View Should Not Copy the World | borrowed views and lifetimes |
| `article_12` | One Thread Per Player Is Still One Thread Per Player | Tokio tasks and async I/O |
| `article_13` | The Server Is Waiting on Three Things | `tokio::select!` |
| `article_14` | Closing the Server Should Not Erase the World | snapshots and graceful shutdown |
| `article_15` | Two Players, No Humans | async integration tests |

## Run the current project

Start the server:

```bash
cargo run --bin server
```

Open two more terminals and run a client in each:

```bash
cargo run --bin client
```

Press Ctrl+C in the server terminal to shut down cleanly and write `world.json`.

## Before you begin

The recommended preparation is Rustbound I through Article 10. You should be familiar with Cargo, functions, structs, ownership and borrowing, enums, `match`, modules, tests, and `Result`. The WebAssembly and browser lessons are optional.

If a lesson does not behave as described, check [Troubleshooting](docs/TROUBLESHOOTING.md), then start a [Q&A Discussion](https://github.com/journey-to-code-org/one-world-many-players/discussions/categories/q-a). Questions are useful project feedback.

## Check your work

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test -j 2 --all-targets
```

The tutorial checkpoints remain frozen. `main` contains the finished teaching server plus reader-support documentation.
