# Agent working principles

[日本語](agent.ja.md)

Start with the purpose and scope the user has already explained. Inspect files and primary sources to establish facts yourself.
These guidelines do not depend on a particular agent's skill format, question tool, or search tool.
Choose a guide for the task from the [entry point](../README.md).

## Check before writing

Read the consuming project's instructions and conventions first, then follow [template selection](documents.md#choosing-templates) to choose an applicable template.
Read the relevant guidelines, chosen template, upstream specifications, and existing documents being changed.
If specifications or code already exist, inspect related behavior, terms, conventions, and similar implementations.
Report missing materials as missing, separately from behavior that has not been verified.

If the initial input is insufficient, ask what the user wants to create.
Do not invent a purpose or usage situation to start writing. Follow [questions to make ideas concrete](#make-ideas-concrete-through-questions).

## Make ideas concrete through questions

Develop the user's ideas through dialogue when creating Product, Requirements, and Design.
Starting from their explanation, ask who struggles in which situation, what should become possible, and what is excluded. Narrow these into concrete conditions from their answers.
Even when the initial input lists every field, ask about unclear meanings and reasons.
Investigate what the conversation, existing specifications, and code already establish; do not ask the user to repeat it.

Focus on one main issue at a time and limit related questions to one to three.
Update your understanding after each answer, then choose the next question from what became clear or remains uncertain.
For a wish to improve usability, ask about a difficult operation or an improved situation, then explore examples, success conditions, and exceptions.
Do not merely exhaust an initial questionnaire. Briefly reflect what you have learned before changing topics.

Prefer an available question tool with selectable answers.
Examples include AskUserQuestion or request_user_input equivalents.
If no such tool is available, ask the same question in the conversation.
Differences in tool names or execution modes are not reasons to skip questions.

Build choices from the user's words and verified materials, explaining what changes with each choice.
Give reasons for recommendations, and do not treat a proposed or preselected choice as the user's decision.
Allow free text for ideas outside the choices. If the user cannot decide yet, offer examples or comparisons and work through them together.

When a question requires the user's answer, wait for it before finalizing dependent documents or implementation.
With asynchronous tools, continue independent research while waiting, but never interpret silence as agreement or selection of the recommendation.
After resolving the main issues, summarize confirmed answers, your proposals, and remaining decisions separately so misunderstandings can be corrected.

The number of questions is not a goal.
Do not repeat questions whose conditions are already concrete. Decide delegated implementation methods and easily changed names or placement from research, with reasons.
Match questions to the document's responsibility. Follow [request scope and review](#request-scope-and-review) for starting the next task.

## Research and decisions

Inspect existing materials first. Use primary sources on the web when external specifications or comparisons affect a decision.
When adding a library or service, investigate its purpose, maintenance, and compatibility with the current system.
Open URLs to verify their contents and cite them near the claims they support.
Do not fill in requirements or designs as though you read an unavailable source.
Read available local materials; if required materials are missing, explain the gap and wait on dependent decisions.

Once the main facts are established, briefly summarize purpose, scope, and decisions.
Ask when a new purpose or scope decision is needed; start writing when the request is sufficiently specific.
Do not turn unresolved questions into decisions by filling in prose.
Follow [Document responsibilities and updates](documents.md) for handling open questions and changes.

## Check after writing

Save files and reread their actual contents.
Fix empty files or failed writes.
Check responsibilities, headings, reasons, exclusions, and open questions against the guidelines and template.

Verify link targets and remove copied upstream text or contradictions.
Briefly report deliverables, verification, and remaining decisions, distinguishing verified and unverified results.
Keep verification results in completion reports or PR descriptions.

## Request scope and review

Treat a statement such as I want to build this as an expression of intent, not authorization to design and implement everything.
First clarify purpose, users, and scope through dialogue and research, then propose the next task, such as drafting Requirements.
Reuse scope established by the conversation instead of asking again.

When only the creation or revision of Requirements or Design is requested, save and check that document, present its key points, and request review before ending that task.
Do not reinterpret a document request as an instruction to write the next document or start implementation.
When revisions are requested, update the document and present it again.

When implementation is requested too, present Requirements and Design decisions during the work and continue within the delegated scope.
Do not impose an approval wait after every document change.
Link saved documents and briefly identify decisions, changes, reasons, and remaining questions so the user can review during the work.

If purpose, scope, or user-visible behavior requires a decision that the request and specifications do not establish, explain the choices and why the user needs to decide.
Wait for the answer before dependent design or implementation.
Your own review or a passing linter does not replace the user's judgment.

If the user explicitly asks to review Design before implementation, wait as instructed.
Silence is not approval, and previously granted permission does not need to be requested again.

## Implementation and verification

Read the target Product, Requirements, and Design before implementation, creating missing specifications first.
Following [request scope and review](#request-scope-and-review), verify that implementation is requested and no unresolved user decision blocks it.
When making changes, update specifications and implementation according to the document update policy.
Even without a requirements change, update Design when its approach or reasons change.
Present the main points when changing requirements or design during implementation too.
If a new user decision is needed, wait on work that depends on it.

For each Requirements scenario, verify observable Then results for success, failure, and boundary conditions.
Run appropriate tests when code needs tests; verify document features through actual reading and link checks.
Report verified results, failures, and unverified conditions separately.

For documents, check existing targets, the one-to-one Requirements–Design relationship, parent Product references, and fixed template headings.
Check that materials are reachable from the entry point without depending on a particular machine's paths or unavailable external documents.
