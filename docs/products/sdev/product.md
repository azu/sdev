# sdev

[日本語](product.ja.md)

## Overview

sdev is a framework for developing products from specifications with people and AI.
Its purpose is to develop code from documented intent, then trace code back to Requirements and Design for review of assumptions and reasons.
Reviewers should be able to understand why a function exists and why behavior or responsibilities were divided in a particular way.

## Background and problems

### Background

sdev is built to practice specification-led development when creating new open-source products.
Using the same workflow across projects creates opportunities to test it and feed experience back into common guidelines and tools.

Writing requirements and designs is itself part of development.
Organizing an audience, purpose, context, outline, and the role of each section resembles the progression from purpose through specifications to code.
sdev uses that connection to carry purpose clarification into the creation of deliverables.

### Problems to solve

Code alone may not establish why behavior is needed or the assumptions behind a choice.
For example, a slash-splitting path function does not tell reviewers whether Windows support is required or which assumptions justify the approach.
The problem here is missing assumptions and reasons, rather than a claim that the function is defective.

Decisions left only in conversation are hard to recover when changing or reviewing code.
Unclear boundaries between purpose, requirements, and design also duplicate conditions and obscure where updates belong.
sdev separates document responsibilities and supports both progressing from purpose to code and tracing code back to its reasons.

## Target users

### People developing products from specifications with AI

Users decide why a product or feature should exist, whose problem it addresses, and what should change, then work with AI to develop requirements, design, and implementation.
They refer to recorded assumptions and reasons during new-product development, feature changes, and code review.

They need to practice the same workflow across projects and return lessons to shared guidance and tools.
Unrestricted customization into a substantially different methodology for every project is not the main purpose.

AI agents draw information from dialogue, follow guidelines, and produce documents and code.
No specific agent product or editor is required.

## Value provided

### Review code through its assumptions and reasons

Give people working with AI a way to trace code back to requirements and design decisions.
Review a function's existence, behavior, and responsibility boundaries against recorded purposes and choices.
Treat missing reasons as unknown rather than inventing a plausible explanation after the fact.

### Develop implementation methods from human purposes

Help people clarify why to build something, whose problem to solve, why address it now, and the outcome they want.
Draw out those purposes and retain them as input for AI-generated requirements, design, and code.
Why now means the reason for undertaking the work, rather than deadlines or priority management.

### Reuse practice and improve it

Provide shared guidelines and tools that are easy to introduce, reference, and update across products.
Reduce the need to reinvent working practices and preserve lessons for the next project.

## Definition of success

People and AI can develop from Product through feature Requirements and Design to code.
During review or changes, they can trace the reverse path and examine why code exists and why its methods were chosen.

Success also includes using the same workflow across products and improving common guidelines and tools from that experience.
sdev applies its own workflow to development.
Judge individual features using the acceptance criteria reached from the [documentation entry point](../../README.md), keeping verification results in completion reports or PR descriptions.

## Development principles

Use dialogue to explore why to build, who has which problem, why address it now, and desired outcomes.
Delegate the development of implementation methods to AI by default, with the human's degree of involvement depending on the situation and the agent's capabilities.
An expressed wish to build something begins a conversation about purpose and scope; it does not by itself authorize design and implementation.
For document-only requests, present the document and key points, then stop. For implementation requests, present decisions during the work so the user can review them.
Ask humans to resolve decisions about purpose, scope, or user-visible behavior when needed.
Do not require human approval at every fixed stage.
Deadline, priority, and task management are outside scope.

First prioritize traceable assumptions and reasons.
Automatic semantic detection of every missing explanation is not the primary goal.
Consider document-rule checking with sdev lint and relationship visualization as tools supporting this purpose.
Visualization means helping people understand relationships; it does not commit to diagrams or a dedicated interface.

Prioritize ordinary, readable Markdown and clear document responsibilities.
Start relationships from directories and Markdown links, choosing more detailed records and code relationships from practice.
Keep adoption and continued use lightweight and make guidance easy to reference and update.

Distribute a standalone executable available to anyone who wants it, without an additional runtime.
Embed guidelines and templates and update them with the executable as one version.
Let agents read command output while people can also browse repository Markdown on GitHub.
Establish distribution and usage conditions before detailing lint and visualization.

Apply the shared progression from purpose to deliverable to both writing and development.
Use it within writing requirements and design too, rather than imposing a mechanical one-to-one mapping between every writing and development stage.

## SLA, SLI, and SLO

### SLA

No service-level guarantee is provided.
sdev supports a development practice rather than operating a public service with contractual guarantees.

### SLI and SLO

No operational reliability indicators or targets are defined.
Check link existence and document creation during work, separately from continuous availability measurements.
If a public service becomes necessary, developers will define indicators and targets with its scope.

## Open questions

A dedicated documentation website and entry URL have not been selected.
Use embedded guides and templates while retaining direct Markdown reading.
If a dedicated website is needed, developers and users will decide its destination.

How to record and reference relationships between code, Requirements, and Design, and the specific visualization features, remain open.
Code comments containing reasons are one option, but no recording location or code-origin reference checker has been selected.
Use existing document placement and links; the initial linter follows the [document-link Requirements](features/document-lint/requirements.md).
When examples show a need for code relationships or additional checks, developers and users will decide each feature.

Whether writing and product development should be unified through modes or operations remains undecided.
Share ideas through guidelines and revisit operational integration when real examples require it.
Expansion into a general writing tool or a dedicated mode has not been decided.
