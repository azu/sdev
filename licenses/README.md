# Distributed notices

[日本語](README.ja.md)

`rust-1.96.0-COPYRIGHT-library.html` is an unchanged copy of `share/doc/rust/COPYRIGHT-library.html` from Rust 1.96.0 installed by rustup.
Release preparation places it alongside the executable as `THIRD-PARTY-NOTICES.html`.
When updating Rust, replace this notice with the matching distribution's notice.

Release preparation gathers notices for external product crates in `THIRD-PARTY-CRATES.html`.
Cargo supplies the target's resolved normal dependencies from Cargo.lock. The document includes their license declarations and the contents of LICENSE, COPYING, NOTICE, and UNLICENSE files found in each package.
Dependencies used only by development tasks or build scripts are excluded.
Missing required notices fail release preparation rather than being omitted.
There is no CLI operation for displaying notices or reading notice files at runtime.

The review viewer embeds JavaScript dependencies as product assets.
`node scripts/build-review.mjs` gathers full notices from the packages included in its bundle and commits them with the generated assets.
The executable includes those notices and places them in every generated review HTML; viewing and generation do not need separate notice files.
Run `node scripts/build-review.mjs --check` after installing locked development dependencies to verify the committed bundle and notices.

sdev itself uses the MIT License, whose text is in the root LICENSE file.
Release preparation copies LICENSE alongside the other notices.
