# Rust development platform

[日本語](product.ja.md)

## Overview

The Rust development platform gives people and AI working on sdev shared environment setup, verification procedures, and criteria for changes.
It carries the specification-led development defined by the [sdev Product](../sdev/product.md) through Rust implementation and verification.
It is neither a standalone application nor a rule requiring every consuming project to adopt Rust.

The [shared development guide Requirements](features/common-development-guide/requirements.md) define its feature, while the [Rust development guide](../../engineering/guidelines/rust-development.md) covers daily work.
Start environment and CI changes from the [CI guidance](../../engineering/guidelines/rust-development.md#ci-and-other-operating-systems).

## Background and problems

### Background

sdev is developed in Rust.
For people and AI to hand work over, they need to share the environment and verified results alongside specifications.
Rust provides formatting and static-analysis tools with Cargo.
See the official [development tools introduction](https://doc.rust-lang.org/stable/book/appendix-04-useful-development-tools.html?highlight=clippy) for their roles.

### Problems to solve

Choosing environments and verification methods anew for every task makes results inconsistent and harder to compare in review.
Shared procedures and reporting boundaries help distinguish local success from verification on other operating systems.

## Target users

### People developing and reviewing sdev

People judging a change's purpose, impact, and verification results use this platform.
They need to review AI's work against shared criteria regardless of how often they write Rust themselves.
They should be able to understand reasons and verified scope without relying on a particular editor or level of Rust expertise.

### AI agents implementing and verifying Rust

Agents implement from specifications, repair failures, and report results.
They select procedures from configured documents when starting, handing work over, or changing environments.
They need criteria that prevent inventing tools or rules absent from the conversation.

## Value provided

### Verify changes through shared basic procedures

Provide common guidance for people and AI to align environments and verification.
This makes results comparable across contributors and execution environments.

### Trace the reasons for the environment

Explain standard tools and conditions for considering additional mechanisms.
Choose tools by whether they provide the necessary verification, rather than by tool count.

## Definition of success

People and AI can read shared guidance and settings, select necessary verification, and distinguish success, failure, and unverified conditions in review.
When environments or procedures change, they can trace the reasons and affected documents.
No time-saving metric has been measured or claimed as evidence of success.

## Development principles

Use Rust's official tools as the core of the smallest environment that meets sdev's needs.
Prioritize procedures people and AI can both execute and verify over a curriculum for learning Rust.
Keep direct Cargo procedures available without requiring a runner, custom framework, or specific editor.

Define product usage, distribution destinations, artifacts, and supported systems in sdev's product specifications.
This Product covers shared development and verification criteria without redesigning distribution features.
Avoid duplicated procedures: Product defines purpose, Requirements conditions, Design methods and reasons, and shared guides daily operations.

## SLA, SLI, and SLO

### SLA

No service-level guarantee is provided.
This repository's development platform is not a public service offering contractual availability.

### SLI and SLO

No continuously measured reliability indicators or numerical targets are defined.
Check document reachability and consistency with settings when making changes; CI results apply to their specific commit.
If an operational service is introduced, developers will determine indicators with its scope.

## Open questions

None within this scope.
Consider additional tools when a concrete problem cannot be resolved with the standard environment, comparing their benefit and maintenance cost.
