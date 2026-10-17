# sdev

[日本語](README.ja.md)

sdev is a framework for developing software from specifications with people and AI.
Record purposes, requirements, and design decisions so implementation and review can follow their reasons.
The CLI provides writing guidelines and templates.

Specifications have three responsibilities.
Follow the consuming project's existing documents and conventions when available.

| Document | What it defines |
| --- | --- |
| Product | Whose problem to solve and what to provide |
| Requirements | Feature behavior and acceptance criteria |
| Design | How to meet requirements and why that approach was chosen |

## Install

### macOS and Linux

Run these commands at the root of the project where you want to use sdev.
The installer selects the latest stable release for your OS and CPU and initializes the project.
No Rust, Node.js, or PATH setup is required.

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/azu/sdev/main/install.sh | sh -s -- --init
./bin/sdev help
```

Initialization prepares these files, preserving existing AGENTS.md instructions.

| File | Purpose |
| --- | --- |
| `bin/sdev` | A launcher pinned to a specific version |
| `AGENTS.md` | Instructions for agents to read sdev's guides |
| `sdev.toml` | The project's default and translation languages |
| `.gitignore` | Excludes downloaded binaries in `.sdev/` from Git |

Commit these four files so a clone can use `./bin/sdev help` too.
The launcher downloads the binary when absent and runs cached copies without network access.
Initialization creates no specifications or empty specification directories.

Downloads are checked against their checksum and version.
Add `--verify` to the installer to verify GitHub's evidence of the build's origin too.
See [bootstrap and attestation verification](docs/engineering/guidelines/releasing.md#bootstrap-and-attestation-verification) for details.

### Choose a launcher location

Use initialization's `--path` option to choose a location other than `bin/sdev`.
Download the initial binary elsewhere first; generated AGENTS.md instructions will use the chosen path.

```sh
curl --proto '=https' --tlsv1.2 -fsSL https://raw.githubusercontent.com/azu/sdev/main/install.sh | sh -s -- --install-dir ./.sdev/setup
./.sdev/setup/sdev init --path ./scripts/sdev
./scripts/sdev help
```

Existing unrelated files and edited launchers are preserved.
For changing an initialized location, see [Releasing and updating](docs/engineering/guidelines/releasing.md).

### Windows

Download the Windows x64 executable from [GitHub Releases](https://github.com/azu/sdev/releases) and place it at `bin/sdev.exe`.
Initialize and read a guide from PowerShell:

```powershell
.\bin\sdev.exe init
.\bin\sdev.exe help
```

Windows uses the installed executable directly.
See [distribution targets and selection](docs/engineering/guidelines/releasing.md#platforms-and-artifact-selection) for OS requirements and manual installation.

## Update

Run from the project root to update the version shared by its launcher:

```sh
./bin/sdev update
```

Use `--version X.Y.Z` to select a release, or `--verify` to check its provenance.
The command keeps the launcher path and project documents. Review and commit the updated launcher.
It stops if the launcher has been edited or verification fails.

For versions without `update`, rerun the installation command above. An existing generated launcher is updated in place, including when `--init` is passed.
A custom launcher can be selected with `sh install.sh --update --path ./scripts/sdev` using a reviewed installer.
Updates run when requested; ordinary guide commands continue to use the committed version.

## Choose project languages

English is the default when no language is configured.
For a new project, `sdev init --lang en --translations ja` records English as the default with Japanese translations in `sdev.toml`.
For an initialized project, edit that file to change the choices:

```toml
language = "en"
translations = ["ja"]
```

Default-language documents use `.md`; other language versions use names such as `requirements.ja.md`.
Maintain both versions as the same specification in the same change.
Select Japanese for a single command with `./bin/sdev help design --lang ja`.
The project's default also selects the language of guides and templates when `--lang` is omitted.

## Read a guide

Pass a task name to `help` to read its guide.
List the available choices first:

```sh
./bin/sdev help
```

| Command | Guide |
| --- | --- |
| `./bin/sdev help product` | Product purpose, users, and value |
| `./bin/sdev help requirements` | Feature behavior and acceptance criteria |
| `./bin/sdev help design` | Methods for meeting requirements |
| `./bin/sdev help documents` | Document responsibilities, placement, and updates |
| `./bin/sdev help workflow` | Dialogue, research, implementation, and verification |
| `./bin/sdev help scenarios` | Acceptance scenarios |

Agents use these guides to clarify purpose and scope with the user.
An idea alone does not authorize implementation; a request limited to writing a document ends with presenting it for review.
See [Agent working principles](docs/guidelines/agent.md) for the workflow.

### Use a template

Use a template designated by the consuming project when available.
Otherwise, choose an embedded fallback:

```sh
./bin/sdev template
./bin/sdev template requirements
```

Commands display template bodies.
Respect an existing specification's structure when editing it.
Guides and templates are embedded in the executable, so no separate download is needed.

## Check document links

Run `lint` at the project root to check broken links and reference directions.
It checks README.md, AGENTS.md, their language versions, and Markdown under docs.

```sh
./bin/sdev lint
./bin/sdev lint --format json
```

Text output is the default; JSON is available for other tools.
See [document-lint Requirements](docs/products/sdev/features/document-lint/requirements.md) for scope and exit codes.

## Develop sdev

Prepare Rust, then use [mise](https://mise.jdx.dev/getting-started.html) for development tools and commands.
At the repository root, trust the configuration and install locked tools:

```sh
mise trust
mise install --locked
```

The postinstall hook installs dependencies and registers [hk](https://hk.jdx.dev/) to check document and Rust changes before commits.
English README prose uses [slopless](https://github.com/berelevant-ai/slopless); Japanese README prose uses textlint's technical-writing preset.

```sh
mise run md:lint
mise run md:fix     # Review automatic changes
mise run rs:check   # Rust formatting, static checks, and tests
```

Read the [Rust development guide](docs/engineering/guidelines/rust-development.md) for setup and verification, and [Releasing and updating](docs/engineering/guidelines/releasing.md) for distributions and publication.
Start from the [sdev Product](docs/products/sdev/product.md) and [documentation entry point](docs/README.md) to explore its purpose and specifications.

sdev is released under the [MIT License](LICENSE).
