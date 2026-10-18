# Documentation and task entry point

[日本語](README.ja.md)

Read the consuming project's instructions and conventions first, then the relevant guidelines, prerequisite specifications, and applicable template.
The templates below are sdev's defaults when the consuming project has not designated one for that document type.
Respect an existing document's structure when making changes.
This entry point and its linked documents support work from writing specifications through implementation and revision.

## References by task

| Task | Guidelines | Prerequisite specifications | Template |
| --- | --- | --- | --- |
| Define product purpose, users, and value | [Writing a Product](guidelines/product.md) | Existing target Product, if any | [product.md](templates/product.md) |
| Specify feature behavior and acceptance criteria | [Writing Requirements](guidelines/requirements.md), [Scenarios](guidelines/scenarios.md) | Target Product; existing Requirements and Design for changes | [requirements.md](templates/requirements.md) |
| Choose how to meet requirements | [Writing Design](guidelines/design.md) | Target Product, Requirements, and existing Design | [design.md](templates/design.md) |
| Implement, verify, or revise existing documents | [Document responsibilities and updates](guidelines/documents.md), [Implementation and verification](guidelines/agent.md#implementation-and-verification) | Target Product, Requirements, and Design | Template applicable to the document |
| Prepare material to judge Requirements and Design | [Prepare a specification-centered review](guidelines/review.md) | Target Product, changed Requirements and Design, and captured Git comparison | Agent review items described in the guide |

Common decision criteria are in [Document responsibilities and updates](guidelines/documents.md); dialogue and research practices are in [Agent working principles](guidelines/agent.md).
Guidelines define judgment and working practices; templates define document structure.

## sdev's specifications

| Document | Reference |
| --- | --- |
| Product | [sdev](products/sdev/product.md) |
| Requirements | [Referencing guidelines and templates](products/sdev/features/guideline-reference/requirements.md) |
| Design | [Guideline reference design](products/sdev/features/guideline-reference/design.md) |
| Requirements | [Command distribution and material reference](products/sdev/features/command-distribution/requirements.md) |
| Design | [Command distribution and material reference design](products/sdev/features/command-distribution/design.md) |
| Requirements | [Markdown links and reference directions](products/sdev/features/document-lint/requirements.md) |
| Design | [Document-link checking design](products/sdev/features/document-lint/design.md) |
| Requirements | [Project bootstrap](products/sdev/features/project-bootstrap/requirements.md) |
| Design | [Project bootstrap design](products/sdev/features/project-bootstrap/design.md) |
| Requirements | [Default language and multilingual documents](products/sdev/features/localization/requirements.md) |
| Design | [Language and document localization design](products/sdev/features/localization/design.md) |
| Requirements | [Specification-centered review](products/sdev/features/specification-review/requirements.md) |
| Design | [Specification-centered review design](products/sdev/features/specification-review/design.md) |

Follow [Document responsibilities and updates](guidelines/documents.md) for placement and relationships.
For selecting distributions, reading materials through commands, and preparing updates and publication, read [Releasing and updating](engineering/guidelines/releasing.md).

## sdev's development platform

For Rust implementation and verification, read the [Rust development guide](engineering/guidelines/rust-development.md).
Purpose and scope are defined in the separate [Rust development platform Product](products/development-platform/product.md). Conditions and structure for the shared guide are in its [Requirements](products/development-platform/features/common-development-guide/requirements.md) and [Design](products/development-platform/features/common-development-guide/design.md).
Start environment and CI changes from the [cross-platform CI guidance](engineering/guidelines/rust-development.md#ci-and-other-operating-systems).
Adopted environment and CI decisions are in [Rust environment and shared CI design](engineering/design/rust-development.md).
This environment applies to sdev itself; it does not require every consuming project to adopt Rust or these tools.
Use the [local agent evaluations](../evals/README.md) to compare intake response quality through independent subagent judgments when changing help descriptions.
