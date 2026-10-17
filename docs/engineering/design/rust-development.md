# Rust environment and shared CI design

[日本語](rust-development.ja.md)

Based on the [Rust development platform Product](../../products/development-platform/product.md), this document defines the development environment and CI shared by features.
Daily operations and update procedures belong to the [Rust development guide](../guidelines/rust-development.md).
This defines adopted choices, rather than recording that settings are applied or CI has run.

## Rust and configuration responsibilities

Pin development and CI to Rust 1.96.0 through the channel in `rust-toolchain.toml`.
Use the minimal profile with rustfmt and clippy.
Cargo and rustc must come from that toolchain, avoiding conflicting environment variables or directory overrides.
Follow [rustup's override documentation](https://rust-lang.github.io/rustup/overrides.html).

Use edition 2024 and rust-version 1.96 in Cargo.toml.
The rust-version field defines source-build compatibility separately from the development pin, following [Cargo's guidance](https://doc.rust-lang.org/cargo/reference/rust-version.html).
Product versions, optimization, and OS-specific distribution settings belong to product features.

Manage dependency requests in Cargo.toml and exact resolutions in Cargo.lock.
Pins and the lockfile prevent unexpected dependency resolution changes during verification; they do not guarantee byte-identical builds.

## Shared CI structure

Run shared checks from `.github/workflows/` with the same pinned environment and lockfile as local work.
Prefer installation from repository settings; update duplicate installation declarations in the same change when unavoidable.
Collect common Rust checks and release preparation in the workspace's Rust development package, xtask.
The Cargo alias starts it, and it selects pinned Cargo and rustc.
Prepend that toolchain's bin directory for child commands so Cargo subcommands such as Clippy and rustfmt cannot resolve to another installation.
The product does not depend on xtask, and ordinary product builds target the product package only.
Rust development tasks remain directly usable without another language runtime or general runner.

Use serde_json for development-task JSON and sha2 for SHA-256.
They support Cargo metadata, verification manifests, and existing checksum formats.
Use the standard library for temporary storage and processes.
Do not implicitly add development-task dependencies to the product; feature designs decide product dependencies.

Formatting, static analysis, and tests form the shared check, with commands collected in the [development guide](../guidelines/rust-development.md#formatting-static-checks-and-tests).
Treat static-analysis warnings as failures.
A failed check fails the job rather than becoming success in a later step.

Run on pull requests, main changes, and manual requests.
Select OS and CPU targets from product support conditions and inspect results for the changed commit.
Distribution builds and artifact verification may accompany OS-specific checks, but their targets and formats belong to the distribution feature.
Separate verification from publication decisions and operations.

Use minimal workflow permissions, including contents: read for ordinary read-only checks.
Pin Actions to commits and keep credentials out of code under test and produced artifacts.
Identify runs for the same change, cancel obsolete runs, and expose failures by environment.
Caches and saved artifacts assist work rather than justify skipped checks.

## Development command and environment entry point

Use [mise](https://mise.jdx.dev/tasks/) as the entry for tools and daily commands.
Pin Node.js, pnpm, and hk in mise.toml, recording download URLs and checksums in mise.lock.
Install with `mise install --locked`, keeping pnpm aligned with package.json's packageManager.
Rust continues to use rustup and rust-toolchain.toml without a duplicate version in mise.toml.
Disable mise's Rust version-file detection and Rust tool management so locked installs cover only the tools recorded in mise.lock.
Keep Rust verification in cargo xtask check and retain direct Cargo usage.

Use `mise run md:lint` for documents, `mise run md:fix` for automatic prose fixes, and `mise run rs:check` for Rust.
The [postinstall hook](https://mise.jdx.dev/hooks.html) calls setup to install locked documentation dependencies and register this repository's Git hook.
CI sets MISE_NO_HOOKS to disable postinstall, then explicitly installs documentation dependencies.

## README prose checks

Check English README.md with the [slopless textlint preset](https://github.com/berelevant-ai/slopless), and Japanese README.ja.md with [textlint](https://textlint.org/docs/getting-started/) and the [technical-writing preset](https://github.com/textlint-ja/textlint-rule-preset-ja-technical-writing).
Keep prose checks scoped to these READMEs; review the structure and accuracy of guides, templates, and specifications through reading and document checks.

Manage development dependencies in package.json and pnpm-lock.yaml and pin pnpm through packageManager.
Pin textlint and both presets exactly. Install slopless under the textlint-rule-preset-slopless alias so textlint can resolve the preset by its standard naming convention.
English settings are in .textlintrc.json; Japanese settings are in .textlintrc.ja.json.
Start from preset defaults. Any exception needs a reason scoped to its document.

Document checks also verify that English/Japanese files are paired with language navigation and run sdev lint for actual files, headings, and reference directions.
Run the same task locally, in hooks, and in a dedicated documentation CI job using mise.toml.
The document-link step uses Cargo and the pinned Rust toolchain; prose steps use Node.js and pnpm.
Development dependencies are not product runtime or installer requirements.
See [README checks](../guidelines/rust-development.md#check-readme-prose) for operations.

## Pre-commit checks

Manage hooks with [hk](https://hk.jdx.dev/), defining file filters and commands in hk.pkl.
Setup runs `hk install --mise` for this repository without changing global configuration.
Git 2.54+ uses configuration-based hooks that compose with existing hooks.
Older Git uses hk's scripts without forcibly overwriting existing hooks.

Markdown and documentation-rule changes select md:lint; Rust source and Cargo settings select rs:check.
hk saves and restores unstaged work so partial commits check the staged content.
Failures block commits; checks do not fix or restage files.
CI still checks documentation if local hooks are missing or bypassed.

## Update policy

Update Rust for a reason, such as required language features, dependencies, or fixes, rather than automatically following every release.
Keep this design and rust-toolchain.toml aligned, reviewing CI declarations, development tasks, and rust-version when needed.
Follow the guide for verification and effects on distribution notices.

Do not mechanically raise rust-version for a development-only toolchain update. Review why source-build compatibility changes and what to verify.
When formatting or Clippy output changes, distinguish toolchain changes from product behavior in review.

## Boundary with distribution features

Feature Designs refer to shared conventions through the [Rust development guide](../guidelines/rust-development.md).
This design owns adoption reasons; the guide owns conventions and procedures followed by features.
Distribution features define build targets, standalone-executable conditions, notices, naming, and public interfaces.
Link to common startup and verification procedures instead of repeating them in every feature.

## Alternatives and reasons

An unpinned stable toolchain reduces update work but can introduce formatting changes and warnings without code changes.
Pin a concrete version so review conditions are comparable and updates can be evaluated independently.

Use Cargo and Rust development tasks for the shared verification criteria.
mise provides installation and command entry points without moving the underlying checks.
Compare extra tools when a concrete problem exceeds the standard setup.
