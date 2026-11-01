# One World, Many Players

Companion code for **Rustbound II: One World, Many Players**, a Journey to Code series about building a multiplayer text-game server in Rust.

This repository begins small on purpose. Each `article_N` branch holds the tested project at the end of that lesson. Follow the article using the matching branch instead of comparing your work with the evolving `main` branch.

## Run this checkpoint

Open two terminals. In the first:

```bash
cargo run --bin server
```

In the second:

```bash
cargo run --bin client
```

The first checkpoint accepts one connection, sends one welcome line, and exits.
