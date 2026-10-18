# Specification-centered review

[日本語](requirements.ja.md)

## About this document

Help users of [sdev](../../product.md) read related Requirements and Design passages and inspect code changes whose specification could not be found. Jev supplies candidates for human investigation. It does not certify correctness or generate explanations and examples.

### Scope

Capture a local Git comparison, display its specifications, find related passages in either direction, and separately list changed code regions with no corresponding specification or an uncertain result. Support inline comments, a local server, and optional prepared JSON or standalone HTML.

### Out of scope

Running agents or tests, editing repository files from the viewer, posting reviews, generating rationale, and automatically declaring specification violations are outside this feature. There is no permanent Implementation column.

## User stories

### Read related requirements and design

As a reviewer, I want to follow the actual Design content related to a requirement, or the Requirements related to a design, while reading. Select a sentence in prose and read its counterpart as a paragraph with context. Select a row in a table and read a related table with its headers and rows intact.

**Acceptance criteria:**

Scenario: When a sentence is selected, read counterparts from the opposite specification layer

- Given: A fixed comparison with Requirements and Design, served with Jev enabled
- When: The reviewer clicks a sentence or moves between sentences with J/K
- Then: The viewer shows related passages from the opposite specification layer and updates them for the selected sentence. Clicking focuses the text and keeps details open. Do not add a related-action button or underline to every passage. Select prose by sentence and preserve its quote in comments, including when multiple sentences share a source line. Show counterparts as paragraphs or list items that retain the method or conditions, rather than isolated sentences.


Scenario: When details are open, hovering or tabbing does not replace them

- Given: Related passages are open for a selected sentence
- When: The pointer or Tab focus moves to a different sentence
- Then: Retain the selected sentence and related passages without opening counterparts for the other sentence.

Scenario: When details are dismissed, retain the document being read

- Given: Related passages are open for a selected sentence
- When: The reviewer presses Escape or clicks outside the details
- Then: Close the details and retain the document and saved comments.

Scenario: When a corresponding sentence is clear, identify it within the related paragraph

- Given: A related paragraph contains several sentences, with one directly corresponding to the selected requirement
- When: The reviewer selects a sentence to find its counterpart
- Then: Read the full related paragraph with the directly corresponding sentence emphasized.


Scenario: When no individual sentence is clear, read the paragraph without emphasis

- Given: A related paragraph was found, but its corresponding sentence is uncertain
- When: The reviewer selects a sentence to find its counterpart
- Then: Show the full paragraph without emphasizing an uncertain sentence.

Scenario: When emphasis lookup fails, retain the paragraph and allow retry

- Given: The related paragraph is available but its sentence-emphasis lookup fails
- When: The reviewer selects a sentence to find its counterpart
- Then: Retain the paragraph and show the emphasis failure and retry action, without reporting no related passage.

Scenario: When specifications contain a table, select a row and read its counterparts

- Given: Requirements or Design contains a Markdown table with column headers and several data rows
- When: The reviewer clicks a row or moves to it with J/K
- Then: Query the selected row with its column labels and support relationships in either direction. Display related tables with headers and their row/column structure, emphasizing a corresponding row when one can be identified. Comments retain the row quotation and source location. Place details and comments after the table without disturbing its cells. Identify partial tables when input limits omit rows.

Scenario: When a requirement has a concrete realization, read that Design

- Given: Candidates include the same topic, restatements, and Design passages describing a concrete mechanism
- When: The reviewer requests related passages from Requirements
- Then: Show designs directly addressing the selected condition, action, or result, and omit uncertain candidates. Keep correspondence distinct from whether the design meets the requirement.

Scenario: When no clear counterpart exists in scope, identify that result without inventing one

- Given: Inspected candidates only share the topic or restate the selected text, without a clear counterpart
- When: The reviewer requests counterparts for a sentence
- Then: Report no clear candidate within the inspected scope without filling the result with uncertain passages or claiming no counterpart exists anywhere.

Scenario: When a design method serves a required behavior, read that Requirements passage

- Given: Design explains how to split sentences and Requirements describes selecting individual sentences
- When: The reviewer queries the sentence explaining that design method
- Then: Show the Requirements paragraph or list item containing the action, result, or constraint the method implements, even without the same technical vocabulary. Surrounding explanation may clarify the selected method's purpose, but a different feature in that context is not a counterpart to the selected sentence. Do not assume every technical detail has a direct requirement; allow no clear counterpart.

Scenario: When query scope is limited, identify omitted candidates and unavailable text

- Given: Some candidates are omitted or their text is unavailable
- When: The reviewer requests related passages
- Then: The viewer distinguishes these states from no related candidate, shows the candidate and inspected counts, and identifies omissions or unavailable content. Do not represent uninspected scope as checked.


Scenario: When the relation API fails, identify the failure and retry

- Given: The related-passage API fails
- When: The reviewer requests counterparts for a sentence
- Then: Show an error and manual retry, distinct from an empty successful result or no related passage.

Scenario: When a selected step is ambiguous alone, interpret it in its scenario

- Given: A scenario about identifying the sentence to notice within a paragraph has Given, When, and Then steps
- When: The reviewer selects “When: show the counterpart”
- Then: Use the surrounding scenario to find the design for emphasizing the corresponding sentence, rather than linking generic navigation merely because it mentions showing something. Keep the selected step as the subject, and identify partial context when limits omit text.

### Compare specifications while retaining both reading positions

As a reviewer, I want Requirements and Design open together so I can compare them while retaining each document's reading position.

**Acceptance criteria:**

Scenario: When two specifications are open together, scroll only the document being read

- Given: Requirements and Design are open in comparison mode
- When: The reviewer scrolls one document
- Then: Only that document moves; retain the other document's position and do not transfer scrolling at the boundary. Direct the contents and J/K navigation to the document being read. Retain both positions when leaving and returning to comparison mode. Retain independent reading positions and controls when the viewport width changes.

### Navigate a specification by its contents

As a reviewer, I want to see the structure of the document I am reading and move directly to a section.

**Acceptance criteria:**

Scenario: When headings repeat, navigate to the exact contents entry selected

- Given: An open Requirements or Design document has repeated heading names
- When: The reviewer selects one heading in the contents
- Then: Navigate to that occurrence in the document without requesting a related judgment or starting a comment.

Scenario: When the document scrolls, identify the current section in its contents

- Given: A specification with headings is open
- When: The reviewer scrolls the document
- Then: Indicate the contents entry for the section being read.

Scenario: When another document is selected, show its contents

- Given: The comparison includes several specification documents
- When: The reviewer selects another document
- Then: Show that document's headings in their source order and hierarchy.

Scenario: When saved headings change, update the contents with the document

- Given: A working-tree specification is open with automatic updates
- When: A document is saved with added, changed, or deleted headings
- Then: Reflect the updated document's heading order and hierarchy in its contents.

Scenario: When headings match a filter, retain their exact destinations

- Given: The open document has headings containing a search string
- When: The reviewer enters that string in the contents search
- Then: Show matching headings and allow navigation to each exact occurrence.

Scenario: When no heading matches a filter, identify the empty search result

- Given: No heading in the open document contains the search string
- When: The reviewer enters that string in the contents search
- Then: Report that no heading matches the search.

Scenario: When a document has no headings, explain why it has no contents

- Given: A captured specification contains no headings
- When: The reviewer opens that document
- Then: Report that the document has no headings.


### Investigate changes without a corresponding specification

As a reviewer, I want to inspect actual code differences that may introduce behavior not covered by Requirements or Design.

**Acceptance criteria:**

Scenario: When code changes are checked, inspect diffs that need a specification-coverage decision

- Given: Captured code changes and a bounded set of specification candidates
- When: The reviewer opens the dedicated implementation-check entry and starts the additional code-change check
- Then: A separate confirmation list offers actual diffs for regions with no corresponding passage or an uncertain result. It reports inspected and omitted regions. Ordinary implementation details are not declared violations merely because no passage was found.

Before execution, show the actual changed files and specification inputs. During execution, show current files and progress. Candidates distinguish whether the change needs specification from whether coverage was found, alongside the actual diff. Stopping or failure remains incomplete and can resume.

### Bring a review back to a conversation

As a reviewer, I want to comment while reading, confirm the target together with my comment, and give the agent enough context to act on it.

**Acceptance criteria:**

Scenario: When commenting on text, retain the selected target and typed character

- Given: A specification block or difference line has focus
- When: The reviewer presses Enter or types a printable character other than J/K
- Then: Start a comment on the focused target and retain the first character. Controls and editors retain normal input behavior, and drafts keep their original targets.

Scenario: When saved comments are copied, include their original targets and comparisons

- Given: Saved comments on a specification passage or a question without a specification reference, and an available clipboard write
- When: The reviewer copies the notes
- Then: Put readable Markdown on the clipboard, including comment text, the original target or question, and the comparison being reviewed. Distinguish before/after text and retain the selected sentence when available. Keep comments available with their targets. Do not modify specifications or send comments to the judgment service. Unfinished drafts are not included.

Scenario: When the clipboard is unavailable, retain comments and offer another handoff

- Given: Saved comments exist and clipboard writes are unavailable or rejected
- When: The reviewer copies the notes
- Then: Identify the copy failure without marking notes copied, retain comments, and offer retry or file export.

Scenario: When notes are sent through the server, identify delivery to the launch output

- Given: Saved comments in server mode and an agent able to read the server's execution output
- When: The reviewer explicitly sends the notes to that agent
- Then: Return comments and their original targets and comparisons through the server, without requiring clipboard pasting. Confirm delivery to the execution output. Separate delivery from whether the agent has read or acted on the feedback; sending does not automatically resume an idle agent.

Markdown and JSON files remain available for saving notes or using a browser that cannot copy to the clipboard. A saved comment refers to the original comparison even after live updates.

Scenario: When server handoff fails, retain comments and allow retry

- Given: Saved comments exist and the request or launch-output write fails
- When: The reviewer sends the notes to the launching agent
- Then: Identify failure without claiming delivery, retain comments, and allow retry or copying.

Scenario: When the agent verifies a fix, read the original comment as resolved

- Given: A comment asking how sentence boundaries are defined has been delivered to the launching agent
- When: The agent verifies the updated Design explains sentence boundaries and reports that comment as resolved
- Then: Retain the original comment and quotation, mark it resolved, and expose the verified comparison and fix summary. Editing or deleting text alone does not resolve a comment; changed targets remain changed and unverified. Reject resolution of an undelivered comment or a stale revision. Copying and exporting preserve the resolution.

### Read saved changes through a local server

As a reviewer, I want saved specification and code changes reflected at the same local URL, while retaining fixed comparisons and a standalone HTML option.

**Acceptance criteria:**

Scenario: When the working tree is saved, update the open review

- Given: A working-tree comparison served without `--input` or `--head`
- When: Requirements, Design, or code is saved, added, or deleted
- Then: Update specification text, diffs, and code-check inputs without restarting the server or reloading the page. Keep the original comparison base, identify the new capture, and retain the reading location where its passage still exists. Related queries use the new inputs. Discard previous scan results and repeat an active or completed code check against the changed inputs; leave an unstarted or stopped check under user control.

Scenario: When reviewed inputs change, retain original comments and discard old judgments

- Given: A saved comment, an unfinished comment, or an in-flight judgment on the displayed text
- When: The working tree changes
- Then: Keep the editor intact while typing and apply the update after it loses focus. Retain each comment's original quotation and comparison. An unchanged, unambiguous sentence may keep its inline comment after moving; changed or unavailable targets remain visible as comments on earlier text. A response about the earlier comparison cannot appear as a judgment on the new text.

Scenario: When recapture fails, retain readable text and identify the failure

- Given: A review is open and obtaining saved changes fails
- When: The server recaptures the review inputs
- Then: Retain the last available document with a visible error, without claiming it is updated, and automatically retry capture.

Scenario: When a fixed comparison is open, keep working-tree changes from replacing it

- Given: Prepared `--input`, an explicit two-commit `--head` comparison, or a standalone HTML
- When: Repository files change
- Then: Continue showing that captured comparison, without automatically replacing it from the working tree.

Scenario: When external queries are disabled, read captured text without external transmission

- Given: Jev was not enabled, or a standalone HTML is opened offline
- When: The reviewer reads specifications
- Then: Captured text, optional agent material, and comments remain available without external queries. Only explicitly enabling Jev on the local server allows specification text and code differences to be sent to the API.

## KPIs

### Decisions made from specifications

- **Current value:** Unmeasured
- **Target value:** Complete one review using related passages or code candidates and return a human decision
- **Measurement method:** Observe which returned passages or changes supported the decision and record misleading or missing candidates
- **Measurement period:** First human review; a functioning UI or API response alone does not establish usefulness

## Functional requirements

### Capture and presentation

**Overview:** The primary workflow reads specifications and queries related places. A separate operation checks changed code for missing correspondence. Agent-written links, rationale, and examples are optional material under the [review guide](../../../../guidelines/review.md).

**Input:** Comparison revisions, additional specifications, display language, output mode, explicit Jev enablement, model, and cache location.

**Output:** Prepared input, standalone HTML, or a local review URL. The normal document view contains specifications; only code candidates requiring investigation expose their actual diffs in a separate list.

#### Applicable conditions

The default comparison is HEAD against staged, unstaged, and non-ignored untracked working-tree content. A target commit compares two resolved commits. Same-directory specifications and explicitly selected documents provide candidate context, not evidence of a relationship.

Working-tree server mode automatically recaptures saved changes, with the original resolved base pinned. Fixed comparisons retain their captured inputs. Queries identify the selected text and their candidate range. Counts and partial states distinguish inspected material from omitted or unavailable material. No related candidate is neither proof of conformance nor a confirmed missing specification.

Repeated judgments may reuse memory and disk caches only for matching text, differences, candidates, model, and judgment conditions. Changed inputs require a new judgment. Model aliases have bounded cache validity; a cached value has the same limitations as its original judgment.

Readers can switch specification text and differences, choose unified or split differences, and restore reading positions through navigation. Missing specifications remain visible as missing context. Prepared input and optional agent items remain compatible, without requiring manual item preparation for live queries.

#### Exceptional behavior

Invalid arguments, revisions, input formats, or references fail before output. Existing files are preserved. Empty comparisons and unavailable text are explained. API failures remain errors until a manual retry succeeds; an incomplete search is not silently reported as complete.

## Data requirements

### Required data

| Data | Purpose |
| --- | --- |
| Comparison | Resolved revisions or working-tree snapshot, capture time, and available contents |
| Related-passage judgments | Selected passage, candidate identities, returned values, inspected range, and partial or error state |
| Code candidates | Actual changed regions, possible specification targets, and investigation status |
| Cache | Input identity, model, judgment conditions, validity information, and reusable results |
| Comments and optional items | Human intent or additional agent material tied to the captured comparison |

### Relationships

A Requirements sentence or table row may correspond to several Design paragraphs or tables. Conversely, a Design sentence or table row can lead to Requirements paragraphs or tables. A changed code region can refer to either layer or remain uncertain. Returned values identify candidate relationships; they do not establish semantic correctness. Comments reference the same comparison and retain their passage, side, line, or question target. The viewer does not require maintained documents to add artificial relationships.

### Personal information

Repository contents and comments may contain confidential information. Prepared JSON can contain implementation source; HTML contains specifications and optional agent material, without credentials or full implementation files. Explicit Jev queries send their specification and code context to the service. Users control exported files and caches.

## External service dependencies

Live judgments use the [official TypeSafe API](https://api.typesafe.ai/openapi.json) and require the user's service credentials. The server owns authentication. Static HTML and local reading without Jev do not require this service. API failure leaves local reading available and offers a manual retry.

## Security considerations

Opening another webpage must not let that page send captured documents to the external service, deliver comments to the agent, or mark comments resolved. Reject operations originating outside the local review origin.

Only explicit `--serve --jev` enables external queries. API credentials stay in the server environment, outside HTML, browser requests, logs, and cache files. Browser requests stay on the same local origin; callers cannot select an external destination.

Bind the server to loopback, expose only review and bounded query operations, and provide no filesystem browsing or repository write API. Escape source text and disable executable Markdown content and remote resources. Preserve existing output and protect cached judgments with restricted file permissions and safe paths.

## Open questions

Evaluate the usefulness of related-passage ranking and code candidates in human reviews. Model values and classification thresholds are investigation aids, not verified confidence or proof of coverage. Persistent comment import and broader syntax highlighting can follow demonstrated need.
