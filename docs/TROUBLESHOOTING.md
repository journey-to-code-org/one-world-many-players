# Troubleshooting

## The server says the address is already in use

Another process owns port `4000`, often a copy of the server left running in another terminal. Stop that process and run the server again.

## The client says the connection was refused

Start `cargo run --bin server` first and leave it running. The client connects only to a server on the same computer at `127.0.0.1:4000`.

## The client appears to be waiting

The client and server are interactive. Keep the server in one terminal and each client in its own terminal. A read with no available line waits by design.

## Cargo cannot choose a binary

This package contains `server` and `client`. Name the one you want:

```bash
cargo run --bin server
cargo run --bin client
```

## A saved world will not load

Do not replace invalid JSON with an empty world silently. Move `world.json` aside so it can be inspected, then report the error and the checkpoint you used. The file is ignored by Git because it belongs to your local run.

## Windows runs out of linker memory

Limit Cargo to two build jobs:

```bash
cargo test -j 2 --all-targets
```

## My code differs from `main`

That is expected while following an earlier lesson. Switch this repository to the article's checkpoint, such as `git switch article_8`, or compare with its immutable `article-08` tag.
