# Project bootstrap

[日本語](requirements.ja.md)

## About this document

This feature belongs to the [sdev Product](../../product.md). It covers executable installation and project setup for accessing materials.
The same project-local command should work on a person's machine and in Claude Code on the web.

### Scope

Provide a bootstrap installer for published macOS/Linux executables and `init` instructions in AGENTS.md.
Bootstrap can install only the executable or continue with initial setup.
On macOS/Linux, `init` also generates a launcher that can be committed and shared, at a configurable path, so clones use the same version.
Always compare checksums and optionally verify provenance with GitHub Artifact Attestations when requested.
Language configuration and translated instructions follow the [localization requirements](../localization/requirements.md).

### Out of scope

Do not create specification bodies or empty specification directories, copy all guidelines, or automatically add agent-specific settings or hooks.
Windows bootstrap, OS signing/notarization, custom package management, and self-updates are excluded.
Windows can use existing executable downloads and `init`.

## User stories

### Install into a project without another runtime

As a developer, I want a project-local sdev matching my OS and CPU so I can tell AI where to run it without configuring PATH or Rust.

**Acceptance criteria:**

Scenario: A published version can be installed on a supported environment

- Given: The user has a supported macOS/Linux CPU, download access, and the required download and hashing tools.
- When: They run bootstrap from the project root.
- Then: The selected executable is placed at `./bin/sdev`, with its version and location reported.

Scenario: Download, checksum, or startup failure preserves an existing executable

- Given: An executable is already in use at `./bin/sdev`.
- When: Download, checksum comparison, or startup verification of a replacement fails.
- Then: Bootstrap exits nonzero without changing the existing executable or project instructions.

Scenario: Unsupported environments and invalid arguments prevent downloads

- Given: The OS/CPU is unsupported or an option/version is invalid.
- When: Bootstrap runs.
- Then: It reports the reason and exits nonzero without downloading or executing an artifact.

### Verify provenance before use

As a developer, I want to check that downloaded files came from sdev's release workflow so I can distinguish checksum consistency from provenance verification.

**Acceptance criteria:**

Scenario: Requested attestation verification precedes execution and installation

- Given: GitHub CLI and the required trust information are available.
- When: Bootstrap runs with attestation verification enabled.
- Then: It executes and installs the artifact only after verifying the repository, release workflow, and main source ref.

Scenario: Missing or invalid attestations or tools do not trigger an unverified fallback

- Given: Attestation verification was requested.
- When: Retrieval or verification fails, or a required tool is unavailable.
- Then: Bootstrap exits nonzero without running the download or changing the existing installation.

Scenario: A curl-only installation reports skipped provenance verification

- Given: Attestation verification was not requested.
- When: Download and checksum comparison succeed.
- Then: Output states that provenance was not verified through attestations.

### Add an entry point for AI

As a developer, I want project instructions to explain how to read sdev so AI can choose from the current list as guides grow.

**Acceptance criteria:**

Scenario: First setup adds reference instructions and local executable exclusions

- Given: AGENTS.md and .gitignore may be missing or contain existing text.
- When: `init` runs from the project root.
- Then: It preserves existing text, adds task examples and directions to discover help/templates in AGENTS.md, and adds local executable exclusions to .gitignore.

Scenario: First setup explains request scope and review

- Given: AGENTS.md has no sdev instructions.
- When: `init` runs.
- Then: AGENTS.md explains that wanting to build something alone does not authorize implementation, document-only requests end with presentation, implementation requests include intermediate summaries, and decisions requiring the user are confirmed.

Scenario: First setup explains questions and follow-up

- Given: AGENTS.md has no sdev instructions.
- When: `init` runs.
- Then: Instructions say to ask rather than assume unclear goals or conditions, prefer available question tools, and choose follow-up questions from answers.

Scenario: Repeated setup preserves edited instructions without duplicates

- Given: Setup is complete and the user may have edited sdev instructions.
- When: `init` runs again.
- Then: It preserves those instructions without duplicating instructions or exclusions.

Scenario: Unsafe target files or broken markers prevent setup

- Given: AGENTS.md or .gitignore cannot safely be updated as an ordinary UTF-8 file, or the sdev instruction markers are malformed.
- When: `init` runs.
- Then: It reports the target and reason, leaving every prevalidated target unchanged.

### Prepare executables in cloud environments

As a developer using a cloud environment, I want the executable prepared at session start so I can use tracked instructions without depending on my local installation.

**Acceptance criteria:**

Scenario: Installing only an executable preserves tracked instructions

- Given: AGENTS.md and .gitignore are tracked, but the executable is absent.
- When: Bootstrap runs without initialization.
- Then: It prepares the executable without changing AGENTS.md or .gitignore.

### Use the same entry point after cloning

As a developer, I want to share acquisition instructions and the chosen version with the project so other developers and cloud environments can read materials without my machine's installation.

**Acceptance criteria:**

Scenario: A custom launcher path is also used in reference instructions

- Given: A supported macOS/Linux environment has no conflicting file at the requested path.
- When: `sdev init --path ./scripts/sdev` runs.
- Then: It creates an executable launcher there, references that path in AGENTS.md and completion output, and permits tracking the launcher while excluding only downloaded executables.

Scenario: First invocation after clone verifies and runs the pinned version

- Given: The launcher and instructions are shared, but the executable is not cached.
- When: The launcher receives arguments such as `help`.
- Then: It downloads the matching release executable, checks checksum and version before execution, preserves arguments and caller directory, and leaves instructions unchanged.

Scenario: A cached executable works without network access

- Given: The pinned version for the current OS/CPU is cached locally.
- When: The launcher runs.
- Then: It starts that executable without downloads and returns its exit code.

Scenario: Invalid or conflicting destinations preserve setup

- Given: The requested path is outside the project, a symlink, directory, or another existing file.
- When: `init` runs.
- Then: It fails with a reason and preserves files and instructions. Repeating setup with the identical generated launcher does not duplicate it.

Scenario: First-download failure never runs an unverified executable

- Given: No executable is cached and download, checksum, or version verification fails.
- When: The launcher runs.
- Then: It exits nonzero without installing or running the unverified executable and preserves the committed launcher and instructions.

## KPIs

### Installation without a language runtime

- **Current value:** Not measured for this new feature.
- **Target value:** Bootstrap requires no Rust, Node.js, or Python.
- **Measurement:** Verify download, installation, and startup on supported macOS/Linux environments.
- **Period:** On feature changes and before publication.

## Functional requirements

### Prepare the executable

**Overview:** Install a specified published version or the latest stable Release into the project.

**Input:** Optional version, attestation verification, independently verified source commit/trust information, and whether to continue with initialization.

**Output:** `./bin/sdev`, selected version, verification method, and installation path. Report each download, with a progress indicator on interactive terminals.

#### Applicable conditions

Use public sdev Releases without login or GitHub API lookup for the latest version.
Always compare checksums. Requested attestation verification uses GitHub CLI and a bundle downloaded with the artifacts.
An independently supplied source commit must also match the attested release record.
Attest public executables and installers, and distribute a record identifying the selected version's source.
Replace ordinary existing files only after successful checks; reject symlink/directory executable destinations.
If subsequent `init` fails, return that failure and retain the installed executable.

#### Exceptional behavior

Stop if retrieval or verification fails.
Explain that checksums do not establish provenance and attestations do not guarantee harmless software.
Treat installer trust separately: executing an unverified script is not an attestation-verified installation of that script.

### Initialize the project

**Overview:** Add reference instructions, Git exclusions, and language configuration using the current directory as root.

**Input:** `sdev init`, optional language settings, and macOS/Linux `--path PATH` for the launcher.

**Output:** AGENTS.md, .gitignore, sdev.toml, a macOS/Linux launcher, and a report of changes or preserved files.

#### Applicable conditions

Default to `./bin/sdev` on macOS/Linux and allow project-relative paths.
Instructions reference that launcher. Windows references the manually installed `./bin/sdev.exe`.
Pin the launcher to the generating sdev version and fetch the executable only when needed.
`init` performs no network operations and creates only required launcher parent directories, not empty specification directories.
Preserve identical generated launchers; do not overwrite another user script.
Initial installer setup may replace the currently running sdev executable with the default launcher.
If existing instructions reference another path, report where manual adjustment is needed rather than rewriting them silently.
Show task examples but direct agents to current help/template lists rather than copying all choices into instructions.
Briefly explain request scope, review, question tools, and follow-up, with the shared workflow guide owning details.
Preserve existing text, line endings, permissions, and valid configuration as required by localization.
Follow consuming repository conventions for specifications, using `docs/products/` only when none is specified.
Create no specification files or empty directories.

#### Exceptional behavior

Argument errors show usage and exit 2. File processing errors show the target/reason and exit 1.
A failure during multi-file saving identifies its location; cross-file transactional success is not guaranteed.

## Data requirements

### Required data

- **Artifacts:** Selected executable, checksum, and optional attestation bundle.
- **Existing instructions and exclusions:** Text to preserve while adding material access and local executable exclusions.
- **Launcher:** A tracked entry point containing the pinned version and acquisition procedure.
- **Language configuration:** Shared default and translation languages as defined by localization.

### Relationships

Executable, checksum, and attestation belong to one Release.
AGENTS.md points to the installed command; .gitignore excludes downloaded executables.
On macOS/Linux, track the launcher and remove only a legacy exclusion exactly matching its path.

### Personal information

None. Project text is not sent externally.

## External service dependencies

- **GitHub Releases and Artifact Attestations**
  - **Related user stories:** Install without another runtime; verify provenance before use.
  - **Reason:** Distribute public executables and attestations.
  - **Usage decision:** Reuse GitHub distribution in a public repository initially, without another paid service.

## Security considerations

Requested verification failures never become unverified success.
Compare before execution/replacement and reject updates to nonregular files.
Explain installer trust, artifact consistency, and provenance as separate checks.

## Open questions

No implementation decisions remain.
After source changes and publication, maintainers verify the documented download URL and actual retrieval/verification in Claude Code on the web.
