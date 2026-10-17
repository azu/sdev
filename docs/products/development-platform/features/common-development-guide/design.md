# Shared development guide design

[日本語](design.ja.md)

## About this document

Implement the [shared development guide Requirements](requirements.md) with repository Markdown and links.
Refer to the [Rust development platform Product](../../product.md) for purpose and users.

### Audience

People and AI maintaining development guidance.

### Prerequisites

[Document responsibilities and placement](../../../../guidelines/documents.md) and the [Rust development guide](../../../../engineering/guidelines/rust-development.md).

### Outside this document

Rust CLI implementation, distribution methods, CI construction, and verification design and execution procedures.
Daily development belongs to the shared guide; verification of document changes follows [implementation and verification](../../../../guidelines/agent.md#implementation-and-verification).

## Implementation approach

Place the development platform as a Product alongside sdev.
The shared guide is a convention referenced by multiple features, so place it separately from product specifications under `docs/engineering/guidelines/`.
Manage cross-feature tool versions, CI structure, and their reasons in the shared environment design.
Keep conventions and daily procedures in the [Rust development guide](../../../../engineering/guidelines/rust-development.md), separately from adoption reasons.
Express membership through Product-to-feature links and guide-to-Design and Product links.

Add navigation to the platform specifications and guide in the [documentation entry point](../../../../README.md).
Separate this from ordinary task guidance and identify its scope as developing sdev itself.
Do not expand the consuming-project guides or add CLI reference topics for this development guide.

Define adopted versions and common CI in the shared design and reflect them in repository settings.
Keep common execution procedures in the guide.
Explain prerequisites, missing settings, and mismatches without linking to another working directory.
Keep implementation status and verification results in work reports rather than copying them into this design.

## Architecture and sequence diagrams

The entry point directs people and AI to specifications, guidance, and settings.
No engine or service communication is added, so a sequence diagram is unnecessary.

## Data model and ER diagrams

Use UTF-8 Markdown and ordinary relative links.
Place Product, Requirements, and Design according to conventions, connecting them through shared guides.
No copied settings or dedicated database are needed, so there is no ER diagram.

## APIs and interfaces

The interfaces are the [documentation entry point](../../../../README.md) and [Rust development guide](../../../../engineering/guidelines/rust-development.md).
Readers choose task-relevant sections and report missing settings or references.
No HTTP API or guide-specific CLI command is added.

## External service integration

None. Reading Markdown requires no authentication or network.
Rust's official links support explanations; they are not required connections for reading the guide body.

## Data handling

Read and edit the same guide files, preserving history in Git.
Guides describe setting roles; shared design collects adopted choices.
Do not link to unavailable files.
No verification-log collection or retention management is added.
Update incoming links when moving or deleting documents.

## Authentication and authorization notes

None. Use existing repository read/write permissions without new accounts or credentials.

## Security considerations

Open official sources and verify contents before adding URLs.
Link installation procedures to official guidance, without examples containing credentials or personal paths.
Publicly shareable structural ideas may inform the guide; exclude private text and internal URLs.

## Deployment notes

None. Ordinary review can publish Markdown changes without product releases or migration.

## Alternatives considered

### Add development procedures to the sdev Product

This reduces reference locations but mixes user value with sdev's development environment.
Use a separate Product and shared guide so changes retain distinct reasons.

### Add a dedicated runner or tool suite for the guide

A runner can combine operations but needs implementation and maintenance merely to provide procedural reference.
Documents explaining the Cargo-based environment meet this feature's requirements.
When shared design adopts Rust development tasks or other tooling, explain their roles in the guide rather than implementing them as this reference feature.

## Open questions

None.
