# Rust development guide

[日本語](rust-development.ja.md)

Develop and verify sdev's Rust implementation with the pinned environment and Cargo's standard tools.
People and AI use the same procedures and hand verified results to review.
This guide belongs to the [Rust development platform Product](../../products/development-platform/product.md); its placement and responsibility follow the [shared guide Design](../../products/development-platform/features/common-development-guide/design.md).
It does not require consuming projects to adopt Rust or this environment.

## Start work

Read target Product, Requirements, and Design from the [documentation entry point](../../README.md) to establish reasons and scope.
For document creation, follow [template selection](../../guidelines/documents.md#choosing-templates), preferring a template designated by the consuming project before sdev's fallback.
Respect existing specifications' structure.
Distinguish document-only changes from implementation, dependency, or environment changes, selecting the appropriate checks.
Using the reference CLI does not replace the consuming project's specifications with sdev's own.

The following procedures require Cargo.toml, Cargo.lock, rust-toolchain.toml, and target code at the root.
If a documents-only branch lacks these, report the missing prerequisites.
Do not copy unfinished code from other work or generate temporary settings merely to claim the procedure worked; report Rust behavior as unverified.
Inspect actual files for implementation state. This guide's existence is not evidence of a working implementation.

## Install the environment

Install [mise](https://mise.jdx.dev/getting-started.html) and run at the root:

```sh
mise trust
mise install --locked
```

Node.js, pnpm, and hk use the URLs and checksums in mise.lock.
Rust uses rustup and rust-toolchain.toml as described below.
The postinstall hook installs dependencies pinned in pnpm-lock.yaml and registers hk for this repository.
Git 2.54+ uses configuration-based hooks alongside existing ones. On older Git, investigate conflicts rather than force overwrites.
If postinstall was disabled, run `mise run setup` manually.
Then use `mise run` without requiring shell activation.

For tool updates, edit mise.toml and run `mise lock`, updating mise.lock in the same change.
For pnpm changes, also align packageManager and check pnpm-lock.yaml compatibility.

Use rustup, recommended by the [official installation instructions](https://rust-lang.org/tools/install/), to manage Rust.
Keep rustc and Cargo in the same toolchain.
No particular editor is required. Completion or save-time formatting does not replace command-based verification.

OS linker tools are required too.
macOS needs Xcode Command Line Tools; Linux needs a C compiler and linker, as described in the [Rust installation chapter](https://doc.rust-lang.org/book/ch01-01-installation.html).
For Windows MSVC, install C++ tools and the Windows SDK following [rustup's prerequisites](https://rust-lang.github.io/rustup/installation/windows-msvc.html).
Follow [rustup's OS guidance](https://rust-lang.github.io/rustup/installation/index.html) for systems you actually use, rather than installing every platform indiscriminately.

## Pin and verify tool versions

Adopted versions and CI structure are defined in the [shared design](../design/rust-development.md).
Pin a concrete stable Rust version in rust-toolchain.toml with the minimal profile plus rustfmt and clippy.
Do not switch to a moving stable toolchain or an unnecessary nightly.
Follow [rustup's selection rules](https://rust-lang.github.io/rustup/overrides.html).

Check the selected environment at the root:

```sh
rustup show active-toolchain
rustc --version
cargo --version
```

Investigate environment variables, directory overrides, RUSTC, or other settings selecting a different toolchain.
The cargo xtask tasks select the pinned Cargo and rustc themselves.
Understand existing configuration and execution paths before adding a workaround.

Cargo.toml's rust-version is a [source-build compatibility condition](https://doc.rust-lang.org/cargo/reference/rust-version.html), separate from the development pin.
A smaller declared number does not prove compatibility; decide whether verification on that version is required.
Treat edition separately too, changing it only for a reason.

## Run during development

Use `cargo check --locked` for compilation checks and `cargo run --locked -- --help` to run the CLI.
Select arguments from the target feature's specifications.
Exercise writing commands in temporary projects.
See Cargo's [`run`](https://doc.rust-lang.org/cargo/commands/cargo-run.html) and [`check`](https://doc.rust-lang.org/cargo/commands/cargo-check.html) references for their responsibilities.
Their success does not replace tests and distribution checks after changes.

## Formatting, static checks, and tests

Run `mise run rs:check` after Rust changes, or invoke the underlying `cargo xtask check` directly.
It selects the pinned environment and checks formatting, static analysis, and tests for both the product and development tasks.
For individual investigation, use standard commands without creating separate pass criteria:

```sh
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace --all-targets
```

Use `cargo fmt --all` when formatting is needed and review the diff.
Start from rustfmt defaults and add custom settings only for a concrete reason.
See [rustfmt](https://github.com/rust-lang/rustfmt) for formatting and read-only checks.

Clippy detects code issues and warnings fail verification.
This is sdev's policy based on [Clippy's CI guidance](https://doc.rust-lang.org/stable/clippy/continuous_integration/index.html).
Start with default rules rather than enabling every strict rule at once.
Scope any necessary suppression with a reason; do not weaken all rules to obtain a passing result.

Tests verify observable Requirements behavior and the change's effects.
Use unit tests for internal behavior and integration tests under tests/ for CLI input, output, exit codes, and filesystem effects.
Do not mirror implementation or optimize for test counts.
See [cargo test](https://doc.rust-lang.org/cargo/commands/cargo-test.html) for targets, and verify executable documentation examples where the relevant library exists.

The all-targets flag covers Cargo targets such as tests and examples, not every operating system or CPU.
For feature flags, identify valid combinations and select meaningful checks rather than enabling mutually incompatible features together.

Repair failures and rerun affected verification.
Distinguish missing environments or download failures from code defects. Do not report unexecuted checks as success.

## Check README prose

After setup, run:

```sh
mise run md:lint
```

This checks English README.md with slopless, Japanese README.ja.md with the technical-writing preset, language-pair navigation, and document links and reference directions.
CI uses the same Node.js, pnpm, and command, with pinned Rust for the document-link step.
These development dependencies are not needed to use the product.
Use `mise run md:fix` for supported automatic fixes, then review the diff and rerun checks.
People and AI still review structure, accuracy, and equivalent meaning between languages.
The shared design owns rule and CI choices.

Pre-commit selects md:lint for Markdown and document-rule changes, and rs:check for Rust source and Cargo settings.
hk temporarily saves unstaged work and restores it after checks, without automatic fixing or restaging.
Use `mise run pre-commit` manually, and `git hook list pre-commit` to inspect registration.
Remove local registration with:

```sh
mise exec -- hk uninstall
```

## Change dependencies and lockfiles

Before adding a dependency, check the standard library and existing dependencies.
If adding one, investigate purpose, supported Rust and OS versions, license, features, and build-time execution, recording reasons in Design.
Keep test-only dependencies in development dependencies and build-only ones in build dependencies.

Use [`cargo add`](https://doc.rust-lang.org/cargo/commands/cargo-add.html) and [`cargo update`](https://doc.rust-lang.org/cargo/commands/cargo-update.html), targeting relevant packages without unrelated updates.
Let Cargo generate Cargo.lock, keep it in Git, and review it with Cargo.toml.
Do not edit the lockfile manually; see [Cargo's explanation](https://doc.rust-lang.org/cargo/guide/cargo-toml-vs-cargo-lock.html).

Use locked verification to detect unintended resolution changes.
For dependency changes, update the lockfile through Cargo first, then return to locked checks.
This does not guarantee offline execution or byte-identical builds.
Dependencies may execute build code or external commands; account for that when fetching and running them.

## CI and other operating systems

CI uses the same pinned environment, resolved dependencies, and basic checks as local work.
Inspect `.github/workflows/` and the cargo xtask implementation for actual targets and invocation.
Update duplicate CI toolchain declarations alongside Rust and resolve drift.
Caches reduce work rather than establish success.

Select systems from sdev's support conditions and the change's effects.
Check affected systems for path, line-ending, executable-name, environment-variable, and process changes.
Use temporary fixtures without relying on personal working directories or accidental settings.
Distinguish cross-compilation success from execution and tests on that platform.

For distribution builds, standalone checks, artifact formats, and publication, see the distribution [Requirements](../../products/sdev/features/command-distribution/requirements.md), [Design](../../products/sdev/features/command-distribution/design.md), and [release guide](releasing.md).
Do not duplicate target lists or publication commands here, or treat successful CI as published artifacts.

## Separate documents, generated files, and experiments

Keep specifications under docs/products/, common conventions under docs/engineering/guidelines/, and cross-feature designs under docs/engineering/design/.
Follow [document responsibilities](../../guidelines/documents.md) for detailed placement and update decisions.
Document Rust public-item usage near code without copying whole requirements or design reasons.
Use [`cargo doc`](https://doc.rust-lang.org/cargo/commands/cargo-doc.html) to inspect generated code documentation.

Keep Cargo's target/ and release-generated artifacts separate from source and ordinarily outside Git.
Put temporary experiments in dedicated locations with their purposes and execution conditions, rather than implicitly adding them to normal code or checks.
Experiment tools are not mandatory product dependencies; when adopting a result, move the necessary parts into maintained specifications and code.
Distinguish retained experiment material from disposable generated output.

Update Product for purpose or scope changes, Requirements and Design for behavior, and Design for changed methods or reasons.
Update this guide when basic procedures change.
For Rust updates, inspect CI, dependency compatibility, formatting changes, new warnings, and required distribution notices.

## Report results

Provide reasons, executed checks, environments, and results for review.
Identify commands or tasks, Rust version, OS, and targets, with reasons for failures or unverified conditions.
When using CI evidence, identify the commit.
Report successful checks separately from verification of Requirements acceptance criteria.
