# sdevの開発

[English](AGENTS.md)

日本語で応答する。
リポジトリのルートで`cargo build --locked --package sdev --bin sdev`を実行し、`./target/debug/sdev help`から作業に必要なガイドを読んで従う。
仕様は[文書と作業の入口](docs/README.ja.md)から確認する。
このプロジェクトはsdev.tomlで英語を既定、日本語を翻訳言語としている。`.md`と`.ja.md`を同じ変更で更新し、必要なら`--lang ja`と日本語版の仕様を使ってレビューする。
文書変更では`mise run md:lint`、Rust変更では`mise run rs:check`を実行する。
