# Distribution and updates

[日本語](releasing.ja.md)

sdev is distributed as a standalone Rust executable.
Guidelines and default templates share the command's version; material output needs no Rust, Node.js, Bun, or network at runtime.
This guide covers usage and manual release operations. Workflow configuration or a successful build does not mean a Release has been published.

## Platforms and artifact selection

Open Releases in the sdev GitHub repository and choose the version and environment.
Do not advertise download URLs until publication has been confirmed.

| Environment | Executable name | Conditions |
| --- | --- | --- |
| macOS arm64 | `sdev-v<version>-macos-arm64` | macOS 11+ |
| macOS x64 | `sdev-v<version>-macos-x64` | macOS 10.12+ |
| Linux arm64 | `sdev-v<version>-linux-arm64` | glibc 2.35+, kernel 4.1+ |
| Linux x64 | `sdev-v<version>-linux-x64` | glibc 2.35+, kernel 3.2+ |
| Windows x64 | `sdev-v<version>-windows-x64.exe` | Windows 10+ |

Other CPUs, 32-bit x86, Windows arm64, and musl are outside initial support.
These are build conditions from the [distribution Design](../../products/sdev/features/command-distribution/design.md), not claims of testing every minimum OS on hardware.
Describe the environments actually checked when publishing.

## Download and use

Download the executable matching your environment and version to a location of your choice.
Compare its SHA-256 value against the separately published checksum when available.
If no matching artifact exists, check the published target list rather than assuming another environment's binary will work.
If retrieval fails, check the Release version, OS, and CPU before proceeding.

On macOS/Linux, rename the file to `sdev`, make it executable, and run it:

```sh
chmod +x ./sdev
./sdev --version
./sdev help
```

On Windows, rename it to `sdev.exe` and use PowerShell:

```powershell
.\sdev.exe --version
.\sdev.exe help
```

Before publication, check OS startup restrictions and signing status.
A working local build does not prove that a browser-downloaded file starts successfully.

## Install with mise

The [mise GitHub backend](https://mise.jdx.dev/dev-tools/backends/github.html) installs the same Release binaries. Configure sdev in the consuming project's mise.toml; this example selects the published v0.4.0:

```toml
[tools."github:azu/sdev"]
version = "0.4.0"
format = "raw"
bin = "sdev"

[tools."github:azu/sdev".platforms]
macos-arm64 = { asset_pattern = "sdev-v*-macos-arm64" }
macos-x64 = { asset_pattern = "sdev-v*-macos-x64" }
linux-arm64 = { asset_pattern = "sdev-v*-linux-arm64" }
linux-x64 = { asset_pattern = "sdev-v*-linux-x64" }
windows-x64 = { asset_pattern = "sdev-v*-windows-x64.exe" }
```

The raw format and `bin` name make a standalone asset available as `sdev`. Each platform pattern selects only its executable, excluding checksum and metadata files.
Review and trust the configuration, then record all five platforms and install the locked version:

```sh
mise trust
mise lock --platform macos-arm64,macos-x64,linux-arm64,linux-x64,windows-x64
mise install --locked
mise exec -- sdev help
```

Commit mise.toml and mise.lock. In AGENTS.md, instruct agents to run `mise exec -- sdev help` and select the relevant guide before working on specifications or implementation.
Use the existing sdev.toml language settings when needed. `sdev init` prepares a project launcher; it is unnecessary for this mise route.
To update, change the selected version in mise.toml, regenerate mise.lock, inspect its URLs/checksums, and run `mise install --locked` again. Use mise for this installation rather than `sdev update`.

## Bootstrap and attestation verification

On macOS/Linux, a reviewed `install.sh` from a checked source can install into project-local `./bin/sdev`.
Use a distribution URL verified after publication, or a verified source checkout.
At the sdev repository root, run `sh install.sh --version 0.1.0 --init`; in another project, run the script by its actual path from that project's root.
Omit the version for the latest stable Release; omit `--init` to prepare only the executable.
`--install-dir DIR` changes the executable destination directory.
Launcher-capable `init` replaces `bin/sdev` with a committed script.
Initialization retrieves every published target checksum and stores them with the version in the launcher. It requires curl and network access; a missing or invalid target checksum prevents generation.
Its next invocation downloads the matching executable into `.sdev/bin/<version>/<os>-<arch>/sdev`, compares the committed SHA-256 without fetching a new expected value, and reuses it afterward.
For a different launcher path, run the installed executable with `sdev init --path ./scripts/sdev`.
AGENTS.md references that path. Manually align existing instructions for a different path before retrying.
Use `sh install.sh --help` to list options.
Project languages can be initialized through `sdev init --lang en --translations ja`; existing configuration is preserved.

Checksums are always compared to detect corruption or a wrong artifact; they do not prove provenance.
Ordinary installation reports that attestation verification was skipped.

`--verify` also checks GitHub Artifact Attestations.
It requires GitHub CLI 2.93.0 or later with the [verification security fix](https://github.com/cli/cli/security/advisories/GHSA-8xvp-7hj6-mcj9).
Attestation retrieval/verification failure does not fall back to checksum-only installation.
Retrieve `attestation.sigstore.json` and `sdev-release.txt` from the same Release and verify the local bundle, matching the attested record's commit to the executable's source and signer workflow commits.
Parse the record as version/commit data; never source it as shell code.

```sh
sh /path/to/sdev/install.sh --version 0.1.0 --verify --init
```

To pin source independently, supply a commit checked separately from the Release tag with `--source-commit "$SDEV_SOURCE_COMMIT"`.
Without it, use the attested record's commit without claiming independent tag verification.

This option verifies the executable that will run, not the already executed installer itself.
To verify the script beforehand, save published `install.sh` and `attestation.sigstore.json`, then use an independently installed GitHub CLI.
Run this example where both files from the same Release have been downloaded:

```sh
gh attestation verify ./install.sh --hostname github.com --repo azu/sdev \
  --bundle ./attestation.sigstore.json \
  --signer-workflow azu/sdev/.github/workflows/build.yml \
  --source-ref refs/heads/main --deny-self-hosted-runners \
  --source-digest "$SDEV_SOURCE_COMMIT" --signer-digest "$SDEV_SOURCE_COMMIT"
```

Set `SDEV_SOURCE_COMMIT` to an independently checked commit SHA for the chosen Release.
Attestations identify provenance and build process; they do not guarantee harmless or bug-free code.
See the [GitHub CLI specification](https://cli.github.com/manual/gh_attestation_verify) for verification semantics and output.

### Setup in Claude Code on the web

Commit the launcher, AGENTS.md, sdev.toml, and .gitignore; exclude downloaded `.sdev/` executables.
In a web environment, `./bin/sdev help` acquires the pinned version when needed.
A Setup script can invoke the same launcher to download it in advance.
It runs noninteractively without changing PATH or shell configuration.
Use `SDEV_VERIFY=1 ./bin/sdev help` to require attestations on first acquisition.
Cached executables are not reverified; delete the relevant cache before reacquiring with another verification method.

The [official environment documentation](https://code.claude.com/docs/en/cloud-environments) describes Ubuntu 24.04 x86_64 and pre-start Setup scripts.
The default Trusted network includes domains used for public files and Release distribution, but other repositories' GitHub APIs and source archives have separate restrictions.
Bootstrap avoids GitHub API for latest-version lookup and attestation retrieval.

If `--verify` cannot retrieve trust information, independently prepare a file with `gh attestation trusted-root` in a trusted environment and supply `--trusted-root /path/to/trusted-root.jsonl`.
Do not trust roots simply because they came with unverified artifacts.
Follow [GitHub's offline verification instructions](https://docs.github.com/en/actions/how-tos/secure-your-work/use-artifact-attestations/verify-attestations-offline) for preparation and updates.

Check actual Release retrieval, the web proxy path, startup, and attestation verification after publication.
Local download tests or Linux CI success alone do not establish verification in the web environment.

## Read guides and templates from an agent

Project instructions must name a command/path that actually works.
Agents first read repository instructions, conventions, and existing documents, then select guides for their task:

```sh
sdev help
sdev help product
sdev help requirements
sdev help design
sdev help documents
sdev help workflow
sdev help scenarios
```

Internal filenames are unnecessary. Guide references point to subsequent commands.
Read the consuming project's Product, Requirements, and Design; do not substitute sdev's own specifications.
Language follows sdev.toml, defaulting to English without configuration; `--lang ja` or `--lang en` overrides one command.

Prefer a template designated for the target document. Otherwise read the built-in default:

```sh
sdev template
sdev template product
sdev template requirements
sdev template design
```

These commands display bodies without discovering, generating, or overwriting project files.
Distinguish arbitrary Markdown and existing specifications from templates, and preserve existing specification structure.
GitHub readers should select documents at the tag matching the command version.

## Update

For a generated macOS/Linux launcher, run from the project root:

```sh
./bin/sdev update
```

Without a version, select the latest stable Release. Use `--version X.Y.Z` for a specific published version and `--path ./scripts/sdev` when invoking a standalone binary for a custom launcher.
When called through a generated launcher, update uses that launcher's path.
Checksums and startup checks are mandatory. `--verify` or `SDEV_VERIFY=1` additionally checks attestations; `--source-commit` and `--trusted-root` have the same meaning as during installation.

The update checks the old generated contents and prepares the new executable and launcher before switching.
It preserves edited launchers, AGENTS.md, .gitignore, sdev.toml, and specifications.
If acquisition, verification, or preparation fails, the old launcher remains usable.
A release already selected is reported without rewriting the script.
If `.sdev/update.lock` remains after a hard termination, check that no update is running before removing that lock directory.

Older versions without `update` can rerun the reviewed installer, including the original `--init` command.
An existing generated launcher enters the same update flow. Use `sh install.sh --update --path ./scripts/sdev` for a custom location.
Review and commit the script's diff so other clones use the selected version. Downloaded cache files remain ignored.
Ordinary invocations do not update in the background.

For a standalone executable on Windows or another installation method, download the matching new binary separately, check its version and checksum, and then replace the old file using that method.
Keep the old executable until the new one has been obtained and checked. Do not replace package-manager installations through project update.
Changing the launcher location remains a separate `init --path` operation: review and move the original launcher and align existing instructions before initializing the new path.

## Prepare distribution artifacts as a developer

Follow the [Rust development guide](rust-development.md) for setup/checks and the [shared Design](../design/rust-development.md) for Rust selection and CI.
Cargo runs Rust development tasks, reading the product version from Cargo.toml.
For macOS arm64, for example:

```sh
cargo xtask build-release --target aarch64-apple-darwin
cargo xtask check --target aarch64-apple-darwin
cargo xtask verify-release dist/sdev-v0.1.0-macos-arm64
```

Use the version in Cargo.toml for filenames and a Design target name for other environments.
`dist/` holds executables, SHA-256 files, and local verification metadata.
Copy Rust notices into THIRD-PARTY-NOTICES.html and generate normal dependency notices as THIRD-PARTY-CRATES.html.
Missing notice sources fail preparation. Include sdev's LICENSE when present.
Only the executable is needed at runtime; notices are not read then.
Artifact generation is not publication.

GitHub Actions' Build and check sdev builds all five combinations natively, checking behavior, document lint, exact template bodies, guide references, version, execution with an empty environment, and system dependencies before saving review artifacts.
Check Linux glibc requirements and additional Windows C runtime DLL dependencies.
Ordinary builds only save artifacts; publication starts through the manual workflow below.

## Check before publication

Choose the project license and place it in root LICENSE.
Rust standard library notices are shipped separately for Rust 1.96.0; update them when updating Rust.

Finalize version/materials and use a commit whose CI passes all five targets.
Check macOS/Windows signing/notarization, downloaded-file restrictions, and supported environments.
Signing and notarization are not yet automated here.

## Release manually

First update the product version in Cargo.toml, synchronize Cargo.lock through Cargo, and merge through normal review to main.
Choose and commit the project LICENSE too. Missing/empty LICENSE or a requested version differing from Cargo prevents release.
Accept stable `X.Y.Z`, such as `0.1.0`, without `v`, prerelease, or build metadata.

In GitHub Actions, open Release sdev, select Run workflow, choose main, and enter the Cargo.toml version.
GitHub CLI invokes the same manual workflow. Replace this example's version with the one being published:

```sh
gh workflow run release.yml --ref main -f version=0.1.0
```

Pin the starting commit and build, check, and verify all targets in the same run.
After success, a dedicated reusable-build job compares targets, commit, version, checksums, notices, and LICENSE, then attests executables, installer, and record.
Only that job receives OIDC and attestation creation permissions, separate from build jobs and the Release-writing publisher.
The publisher verifies every file against the local bundle before creating a tag and draft.
Attach executables/checksums, THIRD-PARTY-NOTICES.html, THIRD-PARTY-CRATES.html, LICENSE, install.sh/checksum, sdev-release.txt, and attestation.sigstore.json.
Verify all attachment names, states, sizes, and hashes on GitHub before publishing.
If main advances, stop publication. New manual runs do not cancel an in-progress release.
Tag pushes alone do not publish, and workflows do not create version-bump commits.

### Failure and retry

Build/verification failure prevents the publication job, tag, and Release creation.
While the same commit remains at main's head, choose Re-run failed jobs or Re-run all jobs on the original run.
Identify artifacts by run ID and target; confirm matching source when combining successful and reverified targets.
Rerun all jobs if artifacts have expired.

Leave a draft after attachment failure. Only retries of its original run may replace matching attachments and publish after full comparison.
A new run for that version stops on the other run's draft.
Do not automatically move/delete another commit's tag, annotated tags, other runs' drafts, or unexpected attachments.
If main advances or another draft exists, maintainers inspect and clean up as needed before starting a new run from current main.

Retries never overwrite public Releases. If the publication response was unknown, inspect public state and attachments first.
If already public, keep that state and publish further fixes as a new version.
Add download links to README only after checking actual Release attachments and sources.

## Install locally from source

With a source checkout, install from the repository root through Cargo.
This needs pinned Rust and OS build tools for developers, not for users of published executables:

```sh
cargo install --path . --locked
sdev --version
sdev --help
```

Add the destination bin directory to PATH.
For an existing installation of the same name, follow Cargo's guidance after checking what would be overwritten.
To try an isolated destination, use the following; add `.exe` to the executable name on Windows:

```sh
cargo install --path . --locked --root ./target/local-install
./target/local-install/bin/sdev --version
```

See [Cargo install](https://doc.rust-lang.org/cargo/commands/cargo-install.html) for destination and update options.
This operation creates no GitHub Release, tag, or package registry publication.
