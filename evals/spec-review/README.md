# Specification-centered review experiment

[日本語](README.ja.md)

Use one real change to test whether a person can judge a specification from implementation decisions and concrete results before reading the whole code diff.
This is an optional local experiment, not an adopted sdev guideline or a new product command.
Keep Requirements, Design, and implementation on one branch; stacked PRs are unnecessary.
Read the consuming project's instructions, Product, Requirements, Design, and `sdev help workflow` / `sdev help documents` first.

## Prepare a real change

Choose a small change with the user. Before implementation, identify its purpose and observable acceptance criteria from existing specifications and confirmed user intent.
Distinguish an established condition from a proposed clarification. Do not silently turn the proposal into the user's decision.
Save the base commit, exact specification revision or file hashes, and the implementation being checked. Hash uncommitted files as well as naming their base commit.
Keep local observations and executable copies under Git-ignored `target/spec-review/<run>/`; keep durable results in the conversation or PR description, outside `docs/`.

At the time of a meaningful choice, record the choice, a short reason, and the specification or convention supporting it in the conversation or local observations.
Label later explanations as inferences and missing reasons as unknown. For existing code, report the searched scope rather than inventing the original author's intent.
Routine implementation choices need no additional approval. Ask about unresolved purpose, scope, or user-visible behavior, and wait before work depending on that decision.

## Present the specification and results

Start with items requiring a decision. Offer the representative results and supporting detail separately, so the person can choose how much to read.
Group by behavior or condition in the specification, not just by changed file or function.
For each item, show the relevant specification, then before-and-after results for the same input, followed by the implementation decision and one focused question for the person.
Label the original behavior, the implemented candidate, and any unimplemented proposal separately. Do not mix an additional proposal into the candidate's observed output.
For each selected item, present:

| Item | Content |
| --- | --- |
| Specification and purpose | Relevant Requirements/Design section and the Product purpose it serves |
| Concrete result | Input, output, operation, or screen; label executed observations and illustrative examples separately |
| Implementation decision | What was added, its recorded reason, and the supporting source; distinguish inference and unknown rationale |
| Decision needed | Mismatch, purpose not met, or missing rationale, with user impact and uncertainty; do not invent a problem |
| Follow-through | Human decision, appropriate document/code changes, and recheck of the affected conditions |

Link from this review output to relevant specifications and optional code/diffs. Do not add reverse links from Requirements to Design or code.
Show specification and implementation revisions with each observation; rerun affected checks after edits rather than reusing an old result as current evidence.
Report test success separately from whether the specification meets its purpose.

## Check the review method

Use known cases to check the distinction between a mismatch and an inadequate specification. Freeze the expected classification before generating the review.
Keep calibration examples separate from observations of the real change, and disclose whether the reviewer saw the expected answer.
For the real lint-language change, the following are illustrative calibration cases, not extra defects observed in sdev:

| Fixed input | Expected distinction |
| --- | --- |
| Specification requires Japanese reason and usage; the reason is English | Implementation mismatch |
| Specification requires only a Japanese reason; output is just `エラー` and gives no recovery information | It may follow that narrow condition while failing the purpose of helping the user correct an invocation; review Requirements |
| Reason and usage follow the selected language and show valid formats | Explain the concrete result without inventing a finding |
| Old code lacks an explanation of why its reason was English | Unknown historical rationale, not proof of a deliberate policy or a specification defect |

The person compares the decision-only view with the view including representative results and reasons.
Record which decisions were possible without reading code, which needed code and why, useful or burdensome information, false findings, and misses.
This is qualitative feedback from a small experiment, not a claim about review accuracy or time saved.

## Close the loop

Ask the person to respond in the session chat. Apply confirmed behavior to Requirements, realization choices to Design, and update affected code/checks.
Track each finding through a human decision and recheck. On GitHub, use the same review thread and resolve only after checking the response; posting or resolving requires the applicable authorization.
The absence of a finding in a later AI response alone is not resolution.

The experiment is complete only after the real change has been reviewed, decisions and necessary updates have been rechecked, and the person's comparison is recorded.
If feedback is pending, say so; a clean test run is not completion of the experiment.
Record limitations even if no benefit was demonstrated. Consider moving useful practices into existing guides only after reviewing the results.
