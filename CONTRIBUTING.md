# Contributing

Questions, corrections, platform notes, tests, and clearer explanations all help readers.

- Use [Q&A Discussions](https://github.com/journey-to-code-org/one-world-many-players/discussions/categories/q-a) when a lesson leaves you stuck.
- Open an issue for a reproducible mismatch between an article and its checkpoint.
- Discuss a large code or architecture change before implementing it.

Tutorial branches and tags are frozen learning checkpoints. Fixes land on `main`; if a correction changes a lesson checkpoint, maintainers will update the article and checkpoint together.

Before opening a pull request, run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test -j 2 --all-targets
```

Keep changes focused and explain which lesson they affect.
