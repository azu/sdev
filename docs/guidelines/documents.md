# Document responsibilities and updates

[日本語](documents.ja.md)

Product describes the product's purpose and place, Requirements describes feature behavior, and Design describes how to realize it.
People and agents read upstream documents before writing downstream ones, avoiding duplicate requirements in separate documents.

Keep guidelines, templates, and specifications in `docs/` when they will remain in use and be maintained.
Keep task plans, verification results, and completion records in the conversation or PR description.

## Three responsibilities

| Type | Content | When to write | Excludes |
| --- | --- | --- | --- |
| Product (product.md) | What is offered, to whom, and why; background, users, value, success, development principles, service guarantees, and reliability targets | Product creation or changes in purpose or users | Feature inputs, outputs, acceptance criteria, and implementation methods |
| Requirements (requirements.md) | Feature scope, user actions, acceptance criteria, success measures, conditions, data needs, external usage dependencies, and safety conditions | Defining a feature or changing its behavior | Repeated Product explanations and implementation details such as types, storage, or APIs |
| Design (design.md) | Methods and reasons sufficient to implement its Requirements: responsibilities, flows, structures, interfaces, integration, protection, deployment notes, and alternatives | Before implementing the corresponding Requirements; update as described below | Redefining requirements, restating Product reliability targets, and test design or execution procedures |

Requirements should let someone write Design without redefining the meaning of the requirements.
Design and Requirements together should let someone implement the feature.
Establish upstream context through links rather than copied text.

## Placement and document relationships

Use this layout for sdev documents, replacing product and feature placeholders with descriptive names.

```text
docs/
├── README.md
├── guidelines/
├── templates/
└── products/
    └── <product>/
        ├── product.md
        └── features/
            └── <feature>/
                ├── requirements.md
                └── design.md
```

Honor the consuming project's placement instructions and conventions first.
Without another rule, put specifications under `docs/products/` using the default layout.
Read guidelines and templates from their source; creating specifications does not require copying the entire framework.

One Product has multiple Requirements documents, and each Requirements document corresponds to one Design.
Directory nesting establishes the parent Product; placement in the same feature directory pairs Requirements and Design.
Link Requirements to its parent Product, and Design to its Requirements and Product, using relative Markdown links.
A Product may point to a feature index without repeating feature conditions.

Link directly to existing target files and verify their existence after writing.
Put cross-feature design separately under `docs/engineering/design/`.
Keep ongoing development conventions under `docs/engineering/guidelines/` and read them before designing.
Create these directories when needed.

Use this directory layout and Markdown links as the starting point for relationships that lint and future visualization can both read.
Apply the reference-direction rules below.
Decide more detailed requirement-to-design notation or dedicated metadata when real examples demonstrate the need.

## Document languages

Use the default and translation languages configured by the consuming project. Without a specified default, use English.
`sdev init --lang en --translations ja` records these choices in sdev.toml.
Default-language documents use ordinary `.md` names; other versions use `.<language>.md`.
For an English default, use `requirements.md` and `requirements.ja.md`; for a Japanese default, use `requirements.md` and `requirements.en.md`.
The language of a guide you are reading does not override the consuming project's document-language instructions.

Language versions are one document with the same responsibility, conditions, and decisions.
Update both in the same change when meaning changes, and resolve disagreements before dependent implementation.
Choose the editing order for the task; a default language is not permission to ignore a conflicting translation.
Use links and headings in the same language where available, and provide navigation to the other language version.
Keep original legal notices, identifiers, and quoted source text unchanged when translation would misrepresent them.

## Reference directions

Write Markdown references according to each document's responsibility.
The established permissions are:

| Source / target | Product | Requirements | Design | Guidelines | Implementation files |
| --- | --- | --- | --- | --- | --- |
| Product | Yes | Yes | No | Yes | No |
| Requirements | Yes | Yes | No | Yes | No |
| Design | Yes | Yes | No | Yes | No |

Yes allows a reference; No forbids it.
If feature Requirements repeat this table, update both the table and explanation when rules change.
Resolve differences between common rules and Requirements before proceeding to Design or implementation.
Product-to-Requirements links are allowed.
This table does not add direction rules for guideline sources, but their Markdown links are still checked for broken targets.
Code-origin references are outside the initial analysis scope.

Requirements may reference other Requirements, including those of other products.
Direction permissions are separate from whether a dependency must be expressed or a reference is mandatory.
Design cannot reference another Design, including another product's or the common development platform's Design.
A heading within the same Design is checked as an internal heading link, not a reference to another Design.
Navigation to a corresponding language version of that same Design is also permitted; another feature's Design remains forbidden in every language.

Put conditions for using shared infrastructure in Requirements and ongoing conventions in guidelines.
Keep technical comparisons and selection reasons in Design; do not relabel an entire Design as a guideline merely to make a link permissible.
Retain the principle of tracing implementation code back to documented assumptions and reasons.
Code-comment link parsing is outside the initial document linter.

The [Living Documentation article](https://efcl.info/2017/05/12/living-documentation-ddd/) describes references from changing things to more stable things, and from code to documentation rather than the reverse.
The rule forbidding links between different Designs is sdev's own decision, not an explicit rule from that article.

## Choosing templates

Before writing a new document, read the consuming project's instructions and conventions and identify any template designated for that document type.
Use an applicable designated template first; otherwise use sdev's embedded fallback.
An arbitrary Markdown file is not automatically a template. Establish its intended use from existing instructions or the user.

Respect the structure of existing specifications when editing them.
Do not confuse existing specifications with templates for new documents or mechanically replace their structure with sdev's fallback.
Use `sdev template` to list embedded templates.
The command only displays bodies; it does not discover local templates, infer applicability, generate, or overwrite documents.

## Handling changes

Product, Requirements, and Design reflect decided purposes, behavior, and methods whether or not implementation is complete.
Use Git history for past content, rather than dated parallel documents for changes to the same feature.
Establish implementation status from code and verification, rather than inferring it from specification wording.

| Change | What to update |
| --- | --- |
| Purpose, target users, or value | Update the same Product and inspect affected Requirements and Design |
| New feature | Read the parent Product and create Requirements and Design in a new feature directory |
| Existing feature behavior | Update existing Requirements and its Design together, including specification corrections discovered during implementation |
| Design improvement or correction during implementation | Update Design with implementation; if behavior or constraints change, update Requirements first |
| Method-only improvement after implementation | Leave Requirements unchanged; update Design when its approach or reasons change. For behavior changes, use the preceding rule |
| Typo or broken link | Fix the document without changing meaning; if meaning changes, apply the relevant rule above |

Create and change Design together with its Requirements.
After changing Requirements or Design, present the document and key points following [request scope and review](agent.md#request-scope-and-review).
Multiple affected documents do not authorize proceeding through implementation in one step.
After implementation, update the same Design when its approach or reasons change; it need not list every small code edit.
Decide whether a new design is a separate feature from differences in behavior and scope.

## Writing and distinguishing decisions

State decided conditions directly, avoiding vague temporary qualifiers such as For now.
Update content when it changes and use Git for the earlier version.
For pre-implementation specifications, describe the intended change, such as Add or Change, rather than merely saying it does not exist.

Keep proposals separate from decided requirements.
For an open question, state what is unresolved, its interim treatment, when to revisit it, and who decides.
Resolve decisions needed for the feature before implementation.
Retain only questions that do not block the current scope and state exclusions explicitly.

For new documents, retain the applicable template's fixed headings and explain None for inapplicable sections.
For existing documents, follow template selection above and respect their structure.
Replace repeatable headings with concrete topic names and create as many as needed.
Distinguish measured values from targets; do not invent measurements or market facts.
SLA means a guarantee to users, SLI a reliability measure, and SLO its target.
Explain decisions not to offer guarantees or measurements too.

Give each paragraph one main point and state it first.
Keep list items at a consistent level and separate permissions, inputs, processing, and completion conditions into meaningful topics when needed.
Choose headings from content rather than splitting mechanically by item count.
Write in the configured document language and explain uncommon terms.

## Boundary between Requirements and Design

Follow [scenario guidelines](scenarios.md) for acceptance criteria, describing outcomes observable by users.
Functional requirements establish scope, inputs, outputs, conditions, and exceptions.
Scenarios supply concrete examples; functional requirements establish shared conditions without repeating them throughout the document.

Data requirements describe needed data, its roles, main information, and relationships.
Put user-facing collection, retention, and deletion conditions in Requirements; leave types, storage, deletion processes, and protection mechanisms to Design.
Treat information identifying a person through easy combination with other data as personal information too.

List external dependencies requiring contractual, cost, or usage decisions in Requirements, including new reliance on an existing service.
Leave connection, authentication, and implementation choices without such decisions to Design.
When technical choices introduce new cost or usage decisions, ask the user to resolve the Requirements gap.

Design specifies deliverables, responsibilities, primary flows, and failure handling.
Leave detailed internal file lists to placement conventions.
For document features, externally referenced entry points and document links are interfaces and may be described in Design.
Use Mermaid `sequenceDiagram` for interactions and `erDiagram` for data relationships when diagrams help.
Explain why a diagram is unnecessary when omitting it.

Deployment notes cover only unusual cutover, migration, or recovery beyond normal deployment.
When ordinary procedures suffice, write None. Do not restate Product reliability targets.
Test design, execution, document-link checks, and acceptance-verification results belong to implementation and verification work, rather than Design.

## Checking references

Ground documents in the request, existing documents, actual files, and verified primary sources.
Before adding a URL, open it to verify its existence and content, then link it near the supported claim.
Do not present an unpublished file as a published URL.
Use real local Markdown references while publication locations remain undecided.
