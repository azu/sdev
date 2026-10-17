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
Link to Requirements and Product at the start, then establish the overall approach and main flow.

Develop that overview through the responsibilities, processing, data, and interfaces of its parts.
Explain a mechanism's role and purpose before naming APIs, types, or numerical parameters.
Separate changes of subject or detail, such as user actions, internal processing, and presentation, into appropriate subheadings and paragraphs.
Develop each paragraph from its point using the [shared writing criteria](documents.md#organize-the-explanation).

Choose the main location for each explanation; elsewhere, give only the necessary context and reference.
Leave detailed internal file lists to placement conventions and separate cross-feature designs into their own documents.

Adapt the template's sections to the subject:

- Diagrams: use them when structure or interactions are hard to follow in prose; explain why a diagram is unnecessary when omitting it.
- Interfaces: for document features without types or APIs, describe the actual documents and links.
- Deployment: set notes to None when ordinary deployment is sufficient.

### Example: connect the explanation

Consider a fictional feature that saves a list as CSV and tells the user the result.
This version leaves the connection between processing and presentation unclear:

```text
export() creates CSV. Show a notification. Return the saved file's path.
```

Establish the roles, then connect the result to its use:

```text
The export action calls export() so the user can save the list as a file.
export() converts the list to CSV, saves it, and returns the saved file's path.

After saving succeeds, show that path in the completion notification.
```

The improvement explains how the notification uses the path returned by saving, rather than just classifying statements under headings.

## Review

First compare against Requirements: is there enough information to implement without redefining behavior?
Check that data needs lead to storage, deletion, and protection methods, and that exception flows and selection reasons are clear.
Resolve technical decisions that block implementation.

Then read from the start: can readers move from the overview to details and identify each section's subject?
Even with clear headings, revise paragraphs that merely list disconnected facts.
Check what each sentence takes from its predecessor and whether API names or numbers appear before their role is explained.
Reconsider repetition, placement, and mixed levels of detail following the [change policy](documents.md#handling-changes).

Verify saved contents and links, then present the document and key points.
Report automated checks separately from reading judgments following the [review procedure](agent.md#check-after-writing).
When only Design work is requested, stop after requesting review.
For implementation requests or decisions needing user input, follow [request scope and review](agent.md#request-scope-and-review), providing Design and Requirements together as implementation inputs.
