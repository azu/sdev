# Command distribution and guideline access

[日本語](requirements.ja.md)

## About this document

This feature lets users obtain and replace a published standalone executable, and lets AI agents read its bundled guidelines and templates through command output.
It assumes the parent [sdev Product](../../product.md).
[Guideline and template access](../guideline-reference/requirements.md) defines which materials to read, which specifications come first, and what to do when materials cannot be read. This feature does not redefine those rules.

### Scope

This feature covers obtaining a published executable, checking its version, and updating it by downloading a replacement.
It covers distributing the command and materials together, selecting bundled materials, and reading them from command output.
It also covers manual releases with an explicit version and local installation from source.
The initial distribution supports macOS, Linux, and Windows.

### Out of scope

Automatic updates, dedicated version switching, and distribution channels other than GitHub Releases, such as Homebrew, are not initial requirements.
Independent updates of materials, copying the guidelines into a consuming repository, and automatically advancing conversations or implementation are excluded.
Lint and visualization have separate requirements.

## User stories

### Obtain a command for my environment

As a person developing products with people and AI from specifications, I want a publicly available executable for my OS so I can start without installing a runtime or compiler.

**Acceptance criteria:**

Scenario: First use in a supported environment needs no additional runtime

- Given: The user has a supported OS, CPU, and OS version, without Node.js, Bun, Rust, or another additional runtime or compiler.
- When: They follow the instructions to obtain the matching version and platform from GitHub Releases and run it.
- Then: They can read usage without being asked to install a runtime or compiler.

Scenario: A user can identify an unsupported environment

- Given: The user's CPU or OS version is outside the published support range.
- When: They read the artifact selection instructions.
- Then: The supported environments make that exclusion clear, without promising operation on their environment.

Scenario: An unavailable download is not treated as a completed installation

- Given: The user can read the instructions but cannot retrieve the chosen artifact.
- When: They read the download failure guidance.
- Then: They learn that a downloaded executable is required and should check the source, OS, and version; the unavailable command is not treated as usable.

### Let AI read bundled materials

As a person working with AI, I want an agent to select guidelines and templates from command usage so it can follow the shared workflow without placing all guidelines in my repository.

**Acceptance criteria:**

Scenario: An agent can discover the available materials

- Given: The agent can run the command but does not know how to select materials.
- When: It requests usage.
- Then: The output lists guides by task and shared topic, template types, and how to read them, without requiring internal filenames.

Scenario: A selected material has the same version as the command

- Given: The repository does not contain a copy of the guidelines, and the agent can run the command.
- When: It selects a guide or template by the documented task, topic, or type.
- Then: The selected body appears as text without generating or copying guidelines into the repository.

Scenario: An unknown material is reported as missing

- Given: The requested material is not bundled with the running command.
- When: The agent requests it.
- Then: The output explains that it does not exist and how to list choices; another material is not returned as a successful substitute.

Scenario: Template selection explains what to check before using the default

- Given: The agent needs to check repository instructions, conventions, and any designated template.
- When: It reads the shared template selection guide.
- Then: The guide prioritizes templates designated for the target document, uses a built-in default when none applies, and distinguishes templates from arbitrary Markdown and existing specifications.

Scenario: Changes to an existing specification preserve its structure

- Given: A specification already exists and must be distinguished from a template for a new document.
- When: The agent reads the shared guide on existing documents.
- Then: It is told to respect the existing structure rather than mechanically rewrite it into the built-in template.

### Check the running version and update

As a person working with AI, I want to check my version and replace the executable so I can update the command, guidelines, and templates together.

**Acceptance criteria:**

Scenario: Version output identifies the running command

- Given: The user can run the distributed executable.
- When: They request its version.
- Then: It prints the running command's version.

Scenario: Replacing the executable updates materials together

- Given: The user has an older version and a newer version is published.
- When: They download the supported new executable and replace the old one using the update instructions.
- Then: Version output and bundled materials both match the new version.

Scenario: Failed updates explain what to check before replacement

- Given: The old version works, but the replacement cannot be downloaded.
- When: The user reads the update failure instructions.
- Then: They learn to confirm the new executable was obtained before replacing the old one; a failed download is not treated as a completed update.

### Publish verified artifacts as a maintainer

As a maintainer, I want to release an explicit version manually so all verified target artifacts correspond to the intended commit.

**Acceptance criteria:**

Scenario: A manual release fixes its version and source commit

- Given: The release version and project LICENSE are on main, and that version is unpublished.
- When: A maintainer selects main and manually starts a release with the same version.
- Then: All target artifacts and the tag correspond to the starting commit, and publication follows successful verification of every target.

Scenario: Invalid versions or prerequisites prevent publication

- Given: The requested version differs from the source version, or the project LICENSE is missing.
- When: A maintainer starts the release.
- Then: It stops with the missing or mismatched prerequisite and creates neither a tag nor a public Release.

Scenario: Failure on any target prevents a partial release

- Given: Build, tests, or artifact verification fail for at least one target OS and CPU.
- When: The release process evaluates the results.
- Then: It does not publish a Release containing only successful targets.

Scenario: Existing tags and public Releases are not overwritten

- Given: The tag points to another commit, or the version already has a public Release.
- When: The maintainer selects that version.
- Then: The process stops with a conflict and leaves existing tags, Releases, and attachments intact.

Scenario: Attachment failure leaves a resumable draft

- Given: After target verification, attachment or comparison fails and leaves a draft created by this run.
- When: The maintainer reruns the original run while the same commit remains at the head of main.
- Then: The process completes and checks the same draft's attachments before publishing, without creating a separate release.

### Install locally from source

As a developer with a source checkout, I want to install sdev through Cargo so I can use a local build without waiting for a published executable.

**Acceptance criteria:**

Scenario: A local installation includes the command and materials

- Given: The developer has the source, pinned Rust environment, and OS build tools.
- When: They install from the local Cargo package.
- Then: The installed sdev provides version output, guides, templates, and lint usage.

## KPIs

### Supported environments with successful first use

- **Current value:** Not measured; verification across all initial targets is incomplete.
- **Target value:** All five initial OS/CPU combinations meeting the Design's technical conditions can run without an additional runtime and display materials and version information (5/5).
- **Measurement:** Follow first-use, material access, and version scenarios on each target.
- **Period:** Before initial publication and whenever supported targets or material delivery change.

## Functional requirements

### Instructions for obtaining and updating executables

**Overview:** Provide public artifacts and instructions for downloading, starting, and replacing them.

**Input:** The user's OS, desired version, and environment selected from the published support range.

**Output:** A supported standalone executable and its installation, usage, and update instructions.

#### Applicable conditions

##### Distribution scope

GitHub Releases is the initial public source. Downloads must not require individual permission.
Support macOS arm64/x64, Linux arm64/x64, and Windows x64 from the initial release.
Artifact selection must distinguish environments and versions, without promising every CPU or older OS version.
32-bit x86, Windows arm64, and specialized Linux environments are not initial requirements.
The Design states technical conditions, including minimum OS versions.

##### Installation and update effort

Provide a standalone executable without requiring Node.js, Bun, Rust, or another runtime or compiler.
Instructions include platform-specific startup, usage and version output, and replacement with a newly downloaded artifact.
Update command and materials together without requiring a separate material download.

AI agents are the main command users, so fast startup matters, but there is no millisecond pass criterion or binary size limit.
Choose an implementation using measured behavior, distribution feasibility, and future extension needs, rather than prioritizing speed or size alone.
Unmeasured values are not targets or results. If numerical acceptance criteria become necessary, developers and users revisit the requirements using measurements.

#### Exceptional behavior

Selection guidance identifies unsupported environments without guaranteeing operation there.
If a source or selected artifact is unavailable, explain how to check its source, OS, and version.
Confirm the new executable was obtained before replacing the old one; download failure is not update success.

### Output bundled materials and version information

**Overview:** Usage explains material selection, and the command prints the selected body or running version.

**Input:** A usage request, task/shared topic/template type, or version request.

**Output:** Usage, the selected material body, or running version as text.

#### Applicable conditions

##### Materials to output

Bundle guidelines, templates, and selection guidance.
Follow the existing access feature for task/topic/type selection without requiring internal filenames or IDs.
Related materials in guides must be reachable through commands without assuming the consuming repository contains all guidelines.
Guide output may replace material links with commands while preserving meaning and the original Markdown used on GitHub.
Templates are built-in defaults and retain their original Markdown body.
Guides prioritize templates designated by repository instructions, then built-in defaults; existing specifications retain their structure.
The CLI does not discover, infer, generate, or overwrite consuming repositories' templates.
Preserve the requirement to read the consuming project's Product, Requirements, and Design rather than substituting sdev's own specifications.
Keep human access through GitHub documents.
Include required legal notices in distributions, without requiring them at runtime.
Language selection and translated materials follow the [localization requirements](../localization/requirements.md).

##### Version and operation boundaries

Materials and executable share a version, available through the command.
Material access ends at text output: it does not generate or save consuming specifications or enforce agent compliance.
The Design defines subcommands, arguments, and output structure.

#### Exceptional behavior

For an unavailable material or unsupported selection, identify what failed and how to see usage.
Do not invent a missing body or return another material as the requested one.

### Manual releases and local installation

**Overview:** Publish an explicit version only after verification and comparison, and document local installation from source.

**Input:** Release version, source, verification results, and artifacts for every target.

**Output:** A tag and public Release for the same commit, with all executables, checksums, notices, and LICENSE, or a reason publication could not proceed.

#### Applicable conditions

Commit version changes before manual execution; the release process does not create version-bump commits.
Tag pushes alone do not initiate publication.
Verify all targets and consistency of source, version, and attachments before publishing; do not overwrite an already published version.
The maintainer must choose and supply the project license before publication.
Local source installation needs a development environment; users of published executables still need no additional runtime.

#### Exceptional behavior

A pre-publication failure is not completion. Leave enough information to identify remaining state and the run to retry.
Do not automatically reuse or delete drafts from another commit or run.
If the publication response is unknown, inspect the public state before retrying and never automatically replace published attachments.

## Data requirements

### Required data

- **Distribution information:** Version, supported OS/CPU/OS versions, and matching artifacts for environment selection.
- **Bundled materials:** Guide, template, and selection bodies from the access feature, with selection metadata.
- **Release information:** Requested version, source commit, run ID, verified artifacts, tag, and publication state for retries and distinguishing existing releases.
- **Command version:** The version shared by the running command and bundled materials.
- **Usage instructions:** Download, startup, material access, version output, replacement updates, and download failure checks.

### Relationships

Each published version has matching platform artifacts and bundled materials.
The same version exposes the same guidelines and templates on every OS.
The access feature owns material meaning and task selection; this feature owns obtaining and reading them through command output.

### Personal information

This feature does not need users' personal information or repository contents.
GitHub downloads and document browsing are subject to that service's terms.

## External service dependencies

- **GitHub Releases and GitHub documents**
  - **Related user stories:** Obtain a command, check and update its version, and let AI read materials.
  - **Reason:** Public distribution, replacement downloads, and human document access. Output of bundled materials needs no network retrieval.
  - **Usage decision:** GitHub Releases is the chosen public channel. Follow applicable terms without independent cost or capacity guarantees.

## Security considerations

Do not include personal, confidential, or authentication information in artifacts, materials, or instructions.
Material and version requests do not need to send consuming repository documents or code externally.
Only document download URLs verified after actual publication; do not present inferred URLs as published ones.

## Open questions

None. User-facing conditions are decided; the corresponding Design specifies technical conditions and command structure.
Pre-publication operations belong in [Distribution and updates](../../../../engineering/guidelines/releasing.md).
