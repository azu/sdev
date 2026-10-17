# Acceptance scenarios

[日本語](scenarios.ja.md)

Write acceptance criteria as verifiable scenarios describing what happens when an action is taken under given conditions.
Attach them to every Requirements user story, following [Document responsibilities and updates](documents.md).

## Format

```markdown
Scenario: When a required document cannot be read, report the unavailable reference

- Given: A guideline needed for the work cannot be read at its reference location.
- When: The agent attempts to retrieve the guideline.
- Then: It reports the unavailable reference and failure to the user, and does not write specifications as if it had read the document.
```

This illustrates the writing format; it does not require the same failure handling in every feature.

## Headings, preconditions, actions, and results

Include both a condition and its expected result in each heading, such as When X, do Y or When X, Y is unavailable.
A heading giving only a precondition, such as Document not found, is incomplete.

Given describes the state before the action; When describes one action by a user or system.
A sequence serving one purpose can be grouped, but separate actions with different purposes.
Do not put an outcome such as Retrieval succeeds in When.

Then describes observable results: document contents, returned values, created or changed files, or displayed errors.
Statements such as Handle appropriately, Reject, or Complete do not establish what happens.
For example, specify that no document is created and an error names the duplicate, rather than merely rejecting registration.

## Separate cases

Use separate scenarios when results differ.
Do not pack initial use, existing documents, and unavailable references into one Then.
Consider failure and boundary cases alongside the successful case.

Name the targets of actions and results.
Reading a template, creating a document from it, and saving that document are different actions.
Distinguish being able to read a file from producing a document that follows its guidelines.

## Boundary with implementation methods

Scenarios define behavior visible to users.
Leave screen structure, APIs, and storage methods to Design, avoiding implementation-specific acceptance criteria.
After implementation, assess the result against these scenarios.
Choose verification methods fitting the condition, including document review and type checks where useful, rather than tests alone.
