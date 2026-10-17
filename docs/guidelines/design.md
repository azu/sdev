# Writing Design

[日本語](design.ja.md)

Read [Document responsibilities and updates](documents.md), [Agent working principles](agent.md), and the [Design template](../templates/design.md).
Identify and read the parent Product and target Requirements, creating missing upstream documents first.
For an existing Design, determine whether this is an improvement during implementation, a requirements change, or a method-only change after implementation, then choose the update policy.
Ask only about decisions the request and code cannot establish.
Follow [template selection](documents.md#choosing-templates): the linked template is sdev's fallback when no applicable project template is designated.

## Investigate the existing system and design decisions

Follow [questions to make ideas concrete](agent.md#make-ideas-concrete-through-questions), separating technical facts established by research from constraints and decisions needing the user.
If alternatives change user expectations or operations, use question tools to explain differences and a reasoned recommendation, then follow up on the answer.
Make delegated technical decisions from existing conventions and research with reasons; do not ask the user to choose every implementation detail.

Inspect the existing code and documentation, conventions, affected behavior and data, and similar implementations.
Read ongoing development guidelines. Put missing usage conditions in Requirements and shared conventions in guidelines.
Follow [reference directions](documents.md#reference-directions) when linking from Design; do not link to other Designs or implementation files.
This link rule does not prevent reading existing documents and code for research.
If there is no impact on the existing system, establish why.

Define deliverables and responsibilities, processing and failure flows, data types and relationships, and public interfaces.
Detail external connections, authentication, dependency failures, collection, use, storage, deletion, protection, and permission boundaries.
Where performance, scale, or operational conditions apply, choose methods meeting Requirements and Product targets.
Check compatibility, technical limits, and any unusual publication, migration, or recovery needs.

Compare alternatives for decisions that are not obvious, and explain choices using simplicity and existing conventions.
Verify technical assumptions with primary sources as needed.
When a choice needs new contractual, cost, or usage decisions, ask the user to resolve the missing Requirements.
Do not change requirements through Design alone.

## Write

Save a new document using the chosen template and the consuming project's placement rules.
Without another rule, put `design.md` beside its corresponding Requirements.
Link to Requirements and Product at the start without copying their requirements.
Describe deliverables, responsibilities, flows, data structures, interfaces, and data handling precisely enough to implement.
For document features without types or APIs, describe the actual documents and links as their interfaces.

Use the template to decide whether diagrams help, and give reasons when they are unnecessary.
Leave fine-grained internal file lists to placement conventions and separate cross-feature designs into their own documents.
Set deployment notes to None when ordinary deployment is sufficient.
Follow [implementation and verification](agent.md#implementation-and-verification) for test design and execution.

## Review

Check that the design provides what implementation needs without redefining Requirements.
Verify that data requirements lead to storage, deletion, and protection methods, exceptions have clear flows, and alternatives have reasons.
Resolve technical decisions that block implementation.
Verify saved contents and links, then present the document and key points.
When only Design work is requested, stop after requesting review.
For implementation requests and decisions needing user input, follow [request scope and review](agent.md#request-scope-and-review).
When implementation is requested, provide Design together with Requirements as its input.
