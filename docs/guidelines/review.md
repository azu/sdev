# Review specifications and find related passages

[日本語](review.ja.md)

Read Requirements and Design in place and ask Jev to find related passages in the other document. Inspect code changes separately for places whose requirement or design could not be found. These results help a person investigate; they do not establish that a specification is correct or that a change violates it.
The agent works on Requirements, Design, and implementation as usual, following [document responsibilities](documents.md) and [implementation and verification](agent.md#implementation-and-verification). Handwritten links, explanations, and examples in `review.json` are optional additions.

## Start the review

Read the target Product, Requirements, Design, and actual changes. Use the agreed comparison and run at the project's Git root:

```sh
sdev review --serve --jev
```

Set `JEV_API_KEY` in the server's environment before starting. The server uses [TypeSafe's official API](https://api.typesafe.ai/openapi.json); the key stays on the server and is not embedded in HTML or passed to browser code. Only explicit `--jev` enables sending specification text and code differences to the external service.
The command prints a `127.0.0.1` URL. Open that URL and stop the server with Ctrl-C. Without a comparison option, review HEAD against the working tree, including staged, unstaged, and non-ignored untracked changes.

Use `--base REF` to choose another base, `--spec PATH` to add specification context, and `--cache-dir PATH` to choose a disk cache location:

```sh
sdev review --serve --jev --base main --spec docs/feature/requirements.md --cache-dir "$HOME/.cache/sdev/review"
```

The specification path is an example; use an existing Requirements or Design document. The agent need not prepare review items before starting.

## Read a passage and follow its related specification

Use the left contents to see the current section and move to a heading. Switch files in the document header.

Click a Requirements sentence to ask how Design realizes its behavior. From Design, find the Requirements action, result, or constraint that the selected method serves. Keep reading selection at sentence level and show counterparts as paragraphs or list items below a bold destination heading. Emphasize the most direct counterpart sentence when one can be identified, retaining the full paragraph without emphasis otherwise. Failed emphasis lookup leaves the paragraph readable and allows retry. Select tables by row and query values with their column headers. Related tables retain their grid and emphasize a corresponding row when identified; row comments appear after the table. Technical names need not match; headings and the containing paragraph may clarify the selected sentence. The result shows actual captured passages, rather than generated reasons or examples. Use J/K to move to the next or previous sentence and update the related passages as you read.

Hovering or tabbing through text does not open or change details. Clicking focuses the passage and keeps its details open. Escape or a click outside closes the details. Press Enter or type a character other than J/K to begin an inline comment directly below the focused passage; within an editor, keys retain their normal editing behavior.

Judge the returned passages against the text you are reading. A candidate is a place to inspect, not proof that a design meets a requirement. Look for the method or choice that addresses the selected requirement, not merely shared vocabulary or a restatement. Uncertain related candidates are omitted. No clear candidate in the inspected scope does not prove that no corresponding design exists. Keep search limits, unavailable text, and failures visible; do not fill gaps with invented references.

A related passage can still lack a usable design decision. If it merely repeats the requirement, focus that Design sentence and leave an inline comment naming the missing decision and what cannot be judged without it. For example: "The sentence boundary is unspecified, so I cannot judge how Japanese text or multiple sentences on one source line are selected." Keep the proposed implementation separate from the observed gap; do not invent a mechanism to make the relationship look complete.

## Check code changes that may lack a specification

Use the additional code-change check when you want to investigate specification coverage. It judges whether each changed code region has a corresponding Requirements or Design passage. Open the actual diff for candidates with no corresponding passage or an uncertain result.

Before starting, confirm the changed files and the specification files shown as inputs. During the scan, follow the active files and inspected count. Read each candidate together with its separate judgments about needing specification and existing coverage, then inspect the actual diff. A stopped or failed scan remains incomplete and can resume.

Read the condition and resulting behavior before deciding what to change. An unrecorded user-visible rule may belong in Requirements; an important choice needed to understand its realization may belong in Design. Ordinary helper functions or other implementation details do not become violations merely because no specification passage was found.
State what was inspected and the limits of the candidate search. Do not report untested code, unavailable content, or a limited search as a confirmed missing specification. Return questions that need human judgment, and update the relevant document or implementation within the authorized scope.

## Reuse judgments only for the same inputs

Repeated judgments use memory and disk caches. The cache key includes the actual input text, diff, candidates, model, and judgment conditions. Reuse a result only when those inputs match; changes invalidate the earlier judgment. A cached result has the same evidence limits as its original judgment.

Working-tree servers update text, diffs, and judgment inputs after files are saved. Updates wait until an active comment editor loses focus; comments on changed text retain their earlier targets. Check the update status and capture time: a disconnected or failed update still shows earlier text. Prepared `--input`, explicit `--head`, and standalone HTML stay fixed and require a new capture to review later changes. An earlier result is not confirmation of the new text or diff.

## Save comments or use a fixed comparison

Comments remain in page memory after copying, saving, or server handoff. Use "Copy notes" to copy the passage or question, comparison, and comment as Markdown, then paste it into the agent conversation.

In server mode, "Send to agent" returns the same information to the launch output. The launching agent reads `review.feedback` JSON lines from its running command and checks the original quote and comparison before acting. A delivery receipt does not mean the agent has read or resolved the feedback.

After making and verifying the requested fix, wait for the server to capture the updated inputs and confirm that revision before resolving the comment. Use `notes[].id` from `review.feedback`, POST `{revision:""}` to `/api/refresh` to obtain the current revision, and POST `{id, revision, summary}` to `/api/feedback/resolve` with the verified fix summary. Use the exact startup URL as the Origin header and `Content-Type: application/json`. For example, set `SDEV_REVIEW_URL` to the startup URL, `SDEV_COMMENT_ID` to the comment ID, and `SDEV_RESOLUTION` to the verified fix summary:

```python
import json
import os
import urllib.request

origin = os.environ["SDEV_REVIEW_URL"].rstrip("/")

def post(path, value):
    request = urllib.request.Request(
        origin + path, data=json.dumps(value).encode(),
        headers={"Origin": origin, "Content-Type": "application/json"},
    )
    with urllib.request.urlopen(request) as response:
        return json.load(response)

current = post("/api/refresh", {"revision": ""})
print(post("/api/feedback/resolve", {
    "id": os.environ["SDEV_COMMENT_ID"],
    "revision": current["revision"],
    "summary": os.environ["SDEV_RESOLUTION"],
}))
```

A stale revision or an undelivered comment ID is rejected; inspect the updated revision before retrying. Editing text alone or leaving the fix unverified does not justify resolution. The browser receives the status automatically and retains the original quote and verified comparison. Copy or export records needed after server shutdown. Use Markdown or JSON export for a file; add unfinished drafts as comments before handing them over. Report separately what was inspected, executed, and left unverified; passing DOM tests does not establish browser usability or semantic correctness.

A standalone HTML or prepared input remains available when a fixed comparison or additional agent material is useful:

```sh
sdev review --prepare --output /tmp/review.json
sdev review --input /tmp/review.json --output /tmp/review.html
```

These steps are optional. When adding material, edit only `title` and `items`, preserve captured text and versions, and use supported references. Distinguish recorded reasons from interpretations or unknown reasons, and observed results from illustrative or unverified examples.
An offline HTML can display its captured material and comments, but cannot perform live Jev queries. Use the server with `--jev` for those queries.
