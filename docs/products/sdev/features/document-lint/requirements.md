# Check Markdown links and document reference directions

[日本語](requirements.ja.md)

## About this document

These Requirements define product lint for local Markdown links under the [sdev Product](../../product.md).
The initial implementation uses standard placement to select and classify documents, check files/headings, and enforce agreed reference directions, without lint configuration.
Results must be readable by people and AI and distinguish violations from processing failures for CI.

### Scope

Check internal Markdown links and reference directions when people and AI create or change specifications.
Targets are root README.md/AGENTS.md and their language variants, plus Markdown under `docs/`.
Classify by standard placement, without user-defined target or type configuration initially.
Internal links mean local file and heading references, including headings in the same document.
Aim to match GitHub's Markdown heading links.

Apply this initial direction table:

| Source / Destination | Product | Requirements | Design | Guidelines | Implementation |
| --- | --- | --- | --- | --- | --- |
| Product | Allowed | Allowed | Forbidden | Allowed | Forbidden |
| Requirements | Allowed | Allowed | Forbidden | Allowed | Forbidden |
| Design | Allowed | Allowed | Forbidden | Allowed | Forbidden |

Allow Product-to-Requirements and Requirements-to-Requirements across products.
The Design column forbids references to other Designs, including shared platform and other product Designs.
Headings in the same Design and navigation to a translation of that same Design are not dependencies on another Design; still check file and heading existence.
Apply identical rules to localized filenames as specified by the [localization requirements](../localization/requirements.md).
Guidelines as sources receive link checks only, without additional direction rules.
Code and code comments as sources are excluded initially.

[Document reference directions](../../../../guidelines/documents.md#reference-directions) owns the shared rule. This table repeats the same rule for this feature.
Update both tables and explanations together; resolve differences before proceeding to Design or implementation.

### Out of scope

External HTTP requests, parsing code/comment links, judging whether rationale is semantically missing, visualization, automatic repair, and document generation are excluded initially.
This does not discard the principle of tracing code to requirements/design rationale.
Product lint is separate from development Clippy and formatting checks.
No lint configuration or custom document placement is provided initially. Future customization is a candidate, not an initial requirement or open question.

## User stories

### Find broken links

As a person or AI changing specifications, I want to check internal destinations so I can find references that prevent readers from reaching needed materials.

**Acceptance criteria:**

Scenario: A missing local file is identified

- Given: A target Markdown document links to a nonexistent local file.
- When: Lint runs.
- Then: Results identify source, destination, and the missing file.

Scenario: An existing target in an allowed direction passes

- Given: Requirements link to an existing parent Product.
- When: Lint runs.
- Then: That link has neither a missing-file nor a direction violation.

Scenario: A missing heading is identified

- Given: A target document links to a nonexistent heading in another existing document.
- When: Lint runs.
- Then: Results identify source, destination, and the missing heading.

Scenario: A heading within the same Design is not another Design dependency

- Given: A Design links to an existing heading in itself.
- When: Lint runs.
- Then: Neither a forbidden Design dependency nor a missing heading is reported.

### Find reference direction violations

As a person or AI changing specifications, I want links that conflict with document responsibilities identified so I can remove dependencies that propagate implementation/design changes upward.

**Acceptance criteria:**

Scenario: Requirements cannot reference Design

- Given: Requirements link to an existing Design.
- When: Lint runs.
- Then: Results identify source/destination and the forbidden direction.

Scenario: A shared platform Design is still another Design

- Given: A Design links to another product's shared development platform Design.
- When: Lint runs.
- Then: That link is reported as a forbidden Design-to-Design reference.

Scenario: Another feature's Design in the same product is forbidden

- Given: A Design links to another feature's Design in the same product.
- When: Lint runs.
- Then: That link is reported as a forbidden Design-to-Design reference.

Scenario: Design cannot reference implementation

- Given: A Design links to an existing implementation file.
- When: Lint runs.
- Then: Results identify source/destination and the forbidden direction.

Scenario: Design can reference its allowed document types

- Given: A Design links to existing corresponding Requirements, parent Product, and guidelines.
- When: Lint runs.
- Then: Those links have no direction violations.

Scenario: Requirements can reference another product's Requirements

- Given: Requirements link to existing Requirements of another product.
- When: Lint runs.
- Then: The Requirements-to-Requirements reference is allowed.

Scenario: An allowed Product-to-Requirements direction still checks existence

- Given: A Product links to missing Requirements.
- When: Lint runs.
- Then: Report a missing target, without a direction violation.

Scenario: Product can reference Product, Requirements, and guidelines

- Given: A Product links to existing documents of those types.
- When: Lint runs.
- Then: Those links have no direction violations.

Scenario: Product cannot reference Design or implementation

- Given: A Product links to an existing Design and implementation file.
- When: Lint runs.
- Then: Each forbidden direction is reported with source and destination.

Scenario: Requirements can reference guidelines

- Given: Requirements link to existing guidelines.
- When: Lint runs.
- Then: No direction violation is reported.

Scenario: Requirements cannot reference implementation

- Given: Requirements link to an existing implementation file.
- When: Lint runs.
- Then: Results identify source/destination and the forbidden direction.

Scenario: Code comment links are not initial lint targets

- Given: An implementation comment links to a document.
- When: Markdown lint runs.
- Then: That comment's link is neither checked nor described as checked.

### Distinguish completed checks from processing failures

As a person or AI verifying specifications, I want incomplete checks distinguished from clean results so unread documents do not pass and CI can judge the result.

**Acceptance criteria:**

Scenario: Completed checks without violations succeed

- Given: Target documents are readable and have no internal link or direction violations.
- When: Lint runs.
- Then: People, AI, and CI can identify completed checks with no violations.

Scenario: Unreadable documents produce processing failures

- Given: A target document cannot be read.
- When: Lint runs.
- Then: Results identify the document/reason and distinguish incomplete checks from violations or success.

Scenario: Mixed violations and failures preserve both

- Given: Targets include a broken link and an unreadable document.
- When: Lint runs.
- Then: Both are reported, and the overall result indicates incomplete checks.

## KPIs

### Results expose the agreed link rules

- **Current value:** Not implemented or measured at specification time.
- **Target value:** Acceptance criteria for agreed allowed/forbidden relationships and broken links pass, without fixed file/link counts.
- **Measurement:** Check documents representing those conditions and inspect source, destination, and verdict.
- **Period:** Initial implementation and changes to rules or target handling.

## Functional requirements

### Check internal Markdown links

**Overview:** Show broken links and agreed direction violations for internal links in selected documents.

**Input:** Markdown in standard sdev placement and its internal links.

**Output:** Relevant sources, destinations, and broken-link or forbidden-direction results.

#### Applicable conditions

##### Select targets by standard placement

Use the default patterns in [standard document placement](../../../../guidelines/documents.md#placement-and-document-relationships) without lint configuration.
Do not add user-defined type mappings or exclusions initially.
The Design specifies patterns; fixed document or link counts do not define scope.
Project language configuration does not change these lint rules.

##### Separate existence from reference direction

Whether a target exists and whether it may be referenced are independent conditions.
Existence alone does not permit a forbidden reference.
Apply the complete table in Scope and keep it synchronized with the shared guideline rather than changing it independently.
There are no undecided cells.

##### Heading links

Check headings in the same and other documents; an existing file with a missing heading is a broken link.
Use [GitHub's section link documentation](https://docs.github.com/en/get-started/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#section-links) as the reference for equivalent anchors/hash URLs.
The Design specifies duplicates, punctuation, and matching from official material.
Do not confuse a heading in the same Design with a reference to another Design.

##### Table applicability

The table governs direction only, not mandatory references or dependency notation.
Guidelines as sources receive link checks without new direction rules.
Distinguish ordinary documents recognized by patterns from targets whose required direction classification is unknown.
When insufficient information prevents checking, identify the target and reason.

##### Results and guideline navigation

Show source location, destination, and reason, with a route to the relevant guideline.
The [Living Documentation article](https://efcl.info/2017/05/12/living-documentation-ddd/) informs the practice of routing lint results to the required convention.
Distinguish success, violations, and processing failures for people, AI, and CI.
The Design specifies command names, output formats, navigation, and numerical exit codes.

#### Exceptional behavior

If a document or destination cannot be checked, report its location/reason as a processing failure.
Distinguish confirmed absence from inability to determine existence.
Continue independent readable targets and retain both violations and failures.
Do not invent forbidden directions for sources outside the table.

## Data requirements

### Required data

- **Target documents:** Markdown links and information needed to classify sources.
- **Destinations:** Linked files/headings and information needed to classify direction.
- **Reference rules:** Agreed permissions for each document type pair.
- **Results:** Source/destination and verdict or the fact that checking was impossible.

### Relationships

Links connect sources to destinations, with both types evaluated against rules.
Associate existence and direction results with each relevant link.
Require no dedicated persistent store or history.

### Personal information

Bodies and filenames may contain personal information; collecting identities or behavior is not a goal.
Show locations, destinations, and reasons without printing entire document bodies.

## External service dependencies

None. Initial lint checks local internal Markdown links without external requests.
Reference articles are not runtime connections.

## Security considerations

Never execute code or commands from documents or destinations.
Do not modify documents for checking; repair and generation are excluded initially.
Use standard placement and report only diagnostic locations, destinations, and reasons.

## Open questions

None. Initial targets, direction rules, and result distinctions are settled.
The Design specifies CLI, exit codes, Markdown, headings, and path handling.
