# Project bootstrap

[日本語](requirements.ja.md)

## About this document

This feature belongs to the [sdev Product](../../product.md). It covers executable installation and project setup for accessing materials.
The same project-local command should work on a person's machine and in Claude Code on the web.

### Scope

Provide a bootstrap installer for published macOS/Linux executables and `init` instructions in AGENTS.md.
Bootstrap can install only the executable or continue with initial setup.
On macOS/Linux, `init` also generates a launcher that can be committed and shared, at a configurable path, so clones use the same version.
Add explicit project updates through `sdev update`; rerunning the installer at an existing generated launcher performs the same update.
Pin both the selected version and every supported target's SHA-256 in the committed launcher. Always compare checksums and optionally verify provenance with GitHub Artifact Attestations when requested.
Offer mise as an alternative installation method using the same Release executables; mise is not required for the standalone launcher.
Language configuration and translated instructions follow the [localization requirements](../localization/requirements.md).

### Out of scope

Do not create specification bodies or empty specification directories, copy all guidelines, or automatically add agent-specific settings or hooks.
Windows bootstrap, OS signing/notarization, background updates, and replacing package-manager or global installations are excluded.
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

### Manage sdev with mise

As a developer already using mise, I want to install and update sdev alongside my other tools using the same published executables.

**Acceptance criteria:**

Scenario: A mise-managed executable provides the same guides

- Given: The project selects a supported sdev Release in mise configuration.
- When: mise installs it and the user runs `mise exec -- sdev help`.
- Then: The selected version provides the same guides and templates as the standalone distribution, without requiring a project launcher.

Scenario: mise owns its installation and updates

- Given: sdev is managed by mise.
- When: The project shares or updates the selected version and checksums.
- Then: It uses mise's configuration, lockfile, and update operations; sdev does not rewrite that configuration or add a mise-specific initialization mode.

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
- Then: It downloads the matching release executable, checks the checksum committed in the launcher and the version before execution, preserves arguments and caller directory, and leaves instructions unchanged.

Scenario: Replacing both the remote executable and its checksum is rejected

- Given: The launcher was committed with a complete checksum set and the executable is not cached.
- When: The distribution serves a different executable and a matching replacement checksum.
- Then: The launcher rejects the executable against its committed value without running it or changing project files.

Scenario: An incomplete checksum set prevents launcher generation

- Given: A checksum is missing or invalid for any supported target, including one different from the current machine.
- When: Initial setup or update prepares a launcher.
- Then: It fails with a reason instead of writing a partially pinned launcher; existing project instructions and launchers are preserved.

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

### Update the version shared by the project

As a developer, I want to update the committed launcher through sdev or the installer, so I can share new materials without manually moving files or rewriting project instructions.

**Acceptance criteria:**

Scenario: Updating a generated launcher preserves the project setup

- Given: The project has an unmodified generated launcher at its default or custom path.
- When: The user runs `sdev update`, optionally selecting a published version.
- Then: The launcher selects that version, or the latest stable release when omitted. Its path, AGENTS.md, .gitignore, language configuration, and specifications remain unchanged, and the result identifies the version and the file to commit.

Scenario: Rerunning installation updates an older launcher

- Given: The destination is an unmodified generated launcher, including a version without the update command.
- When: The user reruns the installer, with or without `--init`.
- Then: The same entry point is updated without requiring a manual move or replacing it with a standalone binary.

Scenario: Updating to the current version leaves the launcher unchanged

- Given: The generated launcher already selects the requested version.
- When: The update verifies the selected release.
- Then: After verifying and reproducing the existing launcher, it reports that the version is current and preserves the launcher's bytes and permissions, without acquiring the same executable and checksum set again.

Scenario: A changed or unsafe launcher cannot be overwritten

- Given: The target is edited, is another file, traverses a symlink, or changes while the update is prepared.
- When: The user requests an update.
- Then: The command stops with a reason and preserves the existing launcher and project documents.

Scenario: A failed update leaves the previous version usable

- Given: The previous launcher works.
- When: Downloading, requested verification, startup, or preparing the replacement fails.
- Then: The launcher still selects the previous version, and an unverified candidate is not run or installed as its executable.

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

### Update the project launcher

**Overview:** Explicitly change the version shared through a generated macOS/Linux launcher.

**Input:** `sdev update`, optional `--version X.Y.Z` and `--path PATH`, and the same provenance options as installation.

**Output:** A verified executable in the project cache and an updated launcher, with old/new versions and a reminder to review and commit the launcher.

#### Applicable conditions

Run at the project root. A generated launcher supplies its configured relative path to the command; direct invocation defaults to `bin/sdev` and accepts an explicit project-relative path.
Rerunning installation at an existing launcher uses the same update, including for versions predating this command.
Compare the original launcher with its published version's generated content before allowing replacement; a header alone is insufficient.
Check the new executable and all target checksums before switching. If the original launcher has a checksum pin, use that pin before running the old executable to reproduce its launcher. Preserve the previous launcher on failure and preserve all project documents.
Use the existing checksum and optional attestation policy, without downgrading a requested verification after a failure.
Updating to the current version verifies and reproduces the existing launcher once, then preserves it without repeating executable and checksum acquisition. Requested provenance and source-commit checks still apply. Selecting an older supported published version is explicit through `--version`.

#### Exceptional behavior

Reject missing, edited, unsafe, or concurrently changed launcher targets. Do not turn another file into a launcher through update.
Leave unsupported Windows and global installations to their existing installation method.

### Initialize the project

**Overview:** Add reference instructions, Git exclusions, and language configuration using the current directory as root.

**Input:** `sdev init`, optional language settings, and macOS/Linux `--path PATH` for the launcher.

**Output:** AGENTS.md, .gitignore, sdev.toml, a macOS/Linux launcher, and a report of changes or preserved files.

#### Applicable conditions

Default to `./bin/sdev` on macOS/Linux and allow project-relative paths.
Instructions reference that launcher. Windows references the manually installed `./bin/sdev.exe`.
Pin the launcher to the generating sdev version and all supported targets' expected SHA-256 values. Resolve the complete checksum set before writing project files. Fetch only the executable needed by the current environment on first invocation, using its committed checksum rather than a newly retrieved expected value.
`init` retrieves the selected Release's complete checksum set over HTTPS before generating a launcher. It requires curl and network access on macOS/Linux; retrieval or validation failure leaves project files unchanged. Windows initialization does not generate a launcher and needs no checksum retrieval. Create only required launcher parent directories, not empty specification directories.
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
- **Launcher:** A tracked entry point containing the pinned version, all target SHA-256 values, and acquisition procedure.
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

No implementation decisions remain. `init` retrieves the published checksum set; do not add a public offline-input option or a mise-specific initialization mode.
After source changes and publication, maintainers verify the documented download URL and actual retrieval/verification in Claude Code on the web.
