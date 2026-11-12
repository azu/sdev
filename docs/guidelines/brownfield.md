# Start specifications in a project with existing code

[日本語](brownfield.ja.md)

These steps let a project that already has code but no Product, Requirements, or Design start its next feature change or new feature from specifications.
Read [Document responsibilities and updates](documents.md) and [Agent working principles](agent.md).
Do not write specifications for the whole codebase; write only the documents needed to start.

## Check when this applies

When a Product and a feature list exist, and Requirements and Design are written for the features the change touches, do not repeat these steps.
Change the existing documents by following [Document responsibilities and updates](documents.md#handling-changes).

When a Product and a feature list exist and only the specifications of the touched features are missing, continue from [Choose the touched features](#choose-the-touched-features).
When creating a new feature that does not change existing features, do not write the current state of existing features.
Create Requirements for the new feature by following [Writing Requirements](requirements.md), and add a row to the feature list in the Product.

The steps for starting are complete when the following state is reached.
Do not add documents that are not needed for it.

| Work to start | Documents needed |
| --- | --- |
| A change to an existing feature | The parent Product, and Requirements and Design for the features the change touches |
| A new feature | The parent Product and the feature list |

## What to read and what to ask

Confirm the current behavior and structure (what the code does) in the code and tests.
Use descriptions of behavior in the README and existing documents as hints for where to read in the code, and write them only after confirming them in the code and tests.
Do not ask the developer about what can be confirmed in the code and tests.

Reasons (why the code does it) often cannot be read from the code.
When the README, existing documents, or history explain a reason and it does not disagree with the behavior of the code, you may adopt that reason and write it.
Show adopted reasons as taken from existing documents in the breakdown of the basis when presenting the documents, so that the developer can correct them in review.
A reason that is not in the material and that you thought of from the material is a guess, so do not write it as settled content.

Ask the developer about reasons that are not in the material, and about contradictions between materials.
Limit questions about reasons to those needed to start the next feature change or new feature.
Leave reasons that are not needed to start as unknown.

When asking for a reason, you may show candidates you can think of from the material as options.
A candidate is material for the question.
Do not write it in a document as a reason until the developer chooses it or answers in their own words.
Follow [questions to make ideas concrete](agent.md#make-ideas-concrete-through-questions) for how to ask.

When you can talk with the developer, you may ask your questions together before writing the documents.
When you write the documents without waiting for answers, leave the items that wait for an answer as unconfirmed, following [Separate settled and unconfirmed content](#separate-settled-and-unconfirmed-content), and ask about them when presenting.

When an existing document and the behavior of the code disagree, show both to the developer and confirm which is the intended behavior.
Treat behavior that appears only in existing documents and cannot be confirmed in the code and tests the same way as a disagreement.
Do not settle either as the current state before receiving an answer.

## Separate settled and unconfirmed content

Write only settled content in the body of a document.
Settled content is behavior and structure confirmed in the code or tests, reasons taken from existing documents, and reasons the developer gave.

Write settled content as a description of the product or feature itself.
Do not write it as what a material says, such as "The README explains that ..." or "In the code, ...".
Report which material confirmed it in the [breakdown of the basis](#present-the-current-state) when presenting the documents.

Do not write content that is waiting for the developer's answer in the body; write it as unconfirmed in the open questions of that document.
Examples are contradictions between materials, and candidates you thought of for a reason that is not in the material.
For an unconfirmed item, write what needs confirmation, the candidate and the material it is based on, that it stays out of the body until decided, and who decides.
When the developer confirms it, write the confirmed content in the body and remove the item from the open questions.

A reason that the developer does not know either can be neither settled nor waiting for confirmation, so state in the body that it is unknown.
Follow [Write the current state](#write-the-current-state) for how to write it.

## Check the material

Check the README and existing documents, the tests, the structure of packages and directories, and the next change the developer explains.
Read history and issues only where a reason needs to be confirmed.
Do not copy non-public information, credentials, or the names and contact details of the people who wrote the material into documents.

## Write the Product

Create the Product by following [Writing a Product](product.md) and the [Product template](../templates/product.md).
Create the Product only the first time.

The purpose and usage scenes express why the product was made and who uses it in which situations.
When the README or other documents explain the motivation or policy, show the developer what you read and confirm that it is still correct before writing it.
Do not settle an existing explanation as the current purpose without confirmation.
Ask the developer about users or value that the material does not show, and do not fill them with assumptions before receiving an answer.
Write items waiting for confirmation in the open questions of the Product, following [Separate settled and unconfirmed content](#separate-settled-and-unconfirmed-content).

The key concepts and the feature list express what exists now.
Read them from the code and existing documents, then show the developer what you read and confirm it.

For the key concepts, write the names and meanings of concepts that are visible to users.
Use names already used in the code or existing documents; do not create new names for writing the documents.
Do not include names of types and functions used only internally.
When different names are used for the same concept, show both to the developer and confirm which to use as the reference.

For the feature list, write each feature's name, a one-line description, and its specification.
A feature is a unit of behavior that users can call by name; do not turn package or directory names directly into features.
In the specification column, write a link to the Requirements for a feature that has them, and "Not written" for a feature that does not.
Do not create per-feature Requirements or Design at this stage.

```markdown
| Feature | Description | Specification |
| --- | --- | --- |
| Configuration loading | Finds and validates the configuration file and decides the check conditions | [Requirements](features/config-loading/requirements.md) |
| Result output | Shows check results in the chosen format | Not written |
```

Do not write details of behavior, acceptance criteria, code locations, or dependencies between features in the list.
When a feature missing from the list is found later, add it to the list.
When the list is large and hard to read, the project chooses how to split it.
The Product may be split, or the list may be moved to a separate document linked from the Product.
In either case, keep the feature list reachable from the Product.

## Choose the touched features

Decide what to write from the feature list, based on the next change the developer explains.
Show the developer the features the change touches, the features it does not touch, and the features you cannot judge, as separate groups, so that the developer can correct the range.

Show a feature that cannot be judged from the descriptions in the list as one you cannot judge.
Do not treat it as untouched until it is settled by checking the code or by the developer's answer.
Untouched features only appear in the feature list.

## Write the current state

Create Requirements and Design only for the chosen features, following [Writing Requirements](requirements.md) and [Writing Design](design.md).
There is no dedicated template or document type for documents that describe the current state.
Write them as ordinary Requirements and Design, and replace the specification column in the feature list with a link.

Write only the current behavior and structure in the current-state documents.
Do not include the change to be made this time.
When only part of a feature is written, state in the out-of-scope section of the Requirements that some behavior is not yet written, and its range.

Write the structure and processing flow in the Design after confirming them in the code and tests.
Do not copy a design explanation in the README or existing documents into the Design without confirming it in the code.

Where the reason was still unknown after asking the developer, state in prose, at the position in the Design where the rationale is written, that the reason is unknown.
For alternatives considered, when there is no record, state that there is no record.
Do not write it in a way that reads as if no alternatives were considered, such as "None, because this describes the current state".
Do not write a guessed reason.

When an unknown reason is needed to decide the next change, ask the developer.
When the developer does not know either, leave it unknown and write the reason for the change anew as this decision.

## Present the current state

When presenting the documents to the developer, report the breakdown of the basis in these five groups.

- Content confirmed in code or tests
- Reasons taken from existing documents (and which material)
- Content based on the developer's answers
- Content left as unconfirmed in the open questions
- Content left unknown

Keep the breakdown in the conversation or PR description; do not save it in a document.
Only the unconfirmed items in the open questions and the unknowns in the body remain in the documents.

Show the reasons taken from existing documents so that the developer can check in review whether any no longer applies.
When unconfirmed items remain, ask the developer about them when presenting the documents.

## Write the change

After the developer confirms the current-state documents, write the next change as an update to those documents.
When unconfirmed items needed to decide the next change remain, confirm them with the developer first.
Do not settle the current state and the change mixed in the same update before receiving confirmation.
Keeping them separate lets the difference in behavior before and after the change be read from the Git diff.

From here, proceed as in a project that has specifications.
Follow [Document responsibilities and updates](documents.md#handling-changes) and [Request scope and review](agent.md#request-scope-and-review).
For later changes, choose the touched features from the feature list, and write the current state only for features without a specification.
