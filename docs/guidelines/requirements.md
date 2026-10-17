# Writing Requirements

[日本語](requirements.ja.md)

Read [Document responsibilities and updates](documents.md), [Scenarios](scenarios.md), [Agent working principles](agent.md), and the [Requirements template](../templates/requirements.md).
Identify and read the parent Product. If absent, [write the Product](product.md) first.
If the target is unclear, inspect existing Products and feature lists, then present candidates.
Follow [template selection](documents.md#choosing-templates): the linked template is sdev's fallback when no applicable project template is designated.

## Understand the input

Use [questions to make ideas concrete](agent.md#make-ideas-concrete-through-questions) to explore user actions and expected outcomes.
Start with one concrete use case, then expand into success, failure, and boundary conditions based on the answer.
For a request to automate work, ask which decisions remain human and what happens when automation cannot complete. Do not invent acceptance criteria from assumptions.

Clarify the actions and value this feature provides, its scope, and exclusions.
For a change, read existing Requirements, Design, and relevant code to establish the difference in behavior.
Align users, terminology, and success measures with the parent Product.

Identify stories for each user and acceptance conditions covering success, failure, and boundaries.
Clarify current and target metrics, measurement methods and periods, and each function's inputs, outputs, conditions, and exceptions.
Research quantitative claims when needed; do not invent measurements.

Identify the roles, main information, and relationships of required data, collection and retention constraints, and personal information.
Consider identification through combination with other information too.
Investigate external dependencies involving contracts or costs, and security and privacy conditions.
Leave types, storage locations, and connection methods to Design.

## Write

Save a new document using the chosen template and the consuming project's placement rules.
Without another rule, put `requirements.md` in the feature directory of sdev's default layout.
In sdev's fallback template, describe the feature and scope under About this document and link to the parent Product.
Use the consuming template's corresponding structure when one is designated.
Every user story needs scenarios with independently verifiable outcomes.
Name stories, metrics, and functions descriptively, without numbering their headings.
List data and external services with their names in bold.

Group applicable conditions by topic, leaving Product explanations and implementation methods to their respective documents.
Resolve requirements that would prevent Design decisions.
Only retain open questions that do not affect design, with their interim treatment and revisit conditions.

## Review

Use the guidelines and scenario rules to check that headings include conditions and results, and Then statements are observable.
Check scope, inputs, outputs, exceptions, and data for missing or conflicting conditions.
The next person writing Design should not need to redefine user-visible behavior.
Verify saved contents and links, then present the document and key points.
When only Requirements work is requested, stop after requesting review.
For implementation requests and decisions needing user input, follow [request scope and review](agent.md#request-scope-and-review).
Update the same feature's Design when requirements change.
When moving to Design, follow the [Design guidelines](design.md).
