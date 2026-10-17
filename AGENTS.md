# Developing sdev

[日本語](AGENTS.ja.md)

Respond in Japanese.
At the repository root, run `cargo build --locked --package sdev --bin sdev`, then read the relevant guides through `./target/debug/sdev help`.
Start specifications from the [documentation entry point](docs/README.md).
The project default is English with Japanese translations, configured in sdev.toml. Maintain each `.md` and `.ja.md` pair together; use `--lang ja` and the Japanese specification versions for review when helpful.
Run `mise run md:lint` for document changes and `mise run rs:check` for Rust changes.
