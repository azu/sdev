# Start specifications in a project with existing code

[日本語](requirements.ja.md)

## About this document

This feature belongs to the [sdev Product](../../product.md). It covers how to start specification-first development partway through, in a project that already has code but no Product, Requirements, or Design.
Without earlier specifications, a change cannot be written as a difference from existing documents.
This feature provides, as a guideline, the steps and writing rules for preparing the minimum documents needed to start the next feature change or new feature with sdev.

### Scope

Provide the steps for starting specifications in a project with existing code as a guide readable from the command.
The steps proceed in this order: check the material, write the Product, create the feature list, write the current state of the features the next change touches, then write the change.
Define how to distinguish what was confirmed from what is unknown when writing the current state.
Define the content of the feature list, which represents features that have no specification yet.

Obtaining the executable and adding instructions to AGENTS.md follow the [project bootstrap Requirements](../project-bootstrap/requirements.md).
Asking questions and the scope of a request follow the [agent working principles](../../../../guidelines/agent.md).
Links from the feature list to Requirements follow the [document lint Requirements](../document-lint/requirements.md), like any other document.

### Out of scope

Do not provide a command that generates or overwrites documents from code.
Do not create specifications for the whole codebase at once.
Do not write a guessed reason for processing whose reason was not recorded.
Do not add a dedicated check for the feature list.
Recording relationships that trace from code to specifications is outside this feature.
Adding or changing existing tests to pin current behavior belongs to implementation and verification in the target project.
A document that explains the technical structure of the whole product is not one of the documents needed to start. A project that needs one writes it following the placement in [document responsibilities and updates](../../../../guidelines/documents.md#placement-and-document-relationships).

## User stories

### Read the steps for starting in a project with existing code

As a developer of an existing tool, I want AI to read what to write first in a project without earlier specifications.
This lets me start with the same steps without explaining the approach again for each project.

**Acceptance criteria:**

Scenario: When a change is requested in a project without specifications, the agent can choose the guide for starting

- Given: A project has sdev installed and has no Product, Requirements, or Design.
- When: The agent reads the list of guides.
- Then: From the descriptions in the list, the agent can choose the guide to read when starting specifications in a project with existing code, and can read from that guide the order of the steps and the documents each step creates.

Scenario: When the project already has specifications, the steps for starting are not repeated

- Given: A Product and a feature list exist, and Requirements and Design are written for the features the change touches.
- When: The agent reads the guide for starting.
- Then: The agent can read that it should proceed with the usual way of changing existing documents, and does not recreate the Product or the feature list.

### Write the Product through an interview

As a developer of an existing tool, I want to put the purpose and users I know into a Product, using existing explanations as material.
This creates the parent context for writing feature Requirements.

**Acceptance criteria:**

Scenario: When the README or other documents explain the purpose, the agent confirms that material and writes the Product

- Given: The project's README or existing documents explain the motivation or policy behind the project.
- When: The agent writes the Product.
- Then: The agent shows the developer what it read from the existing explanations, confirms whether it is still correct, and creates the Product from the confirmed content. It does not settle an existing explanation as the current purpose without confirmation.

Scenario: When the purpose or users cannot be found in existing material, the agent asks the developer

- Given: The target users or the value provided cannot be confirmed from existing material.
- When: The agent writes the Product.
- Then: The agent asks the developer about the items it cannot determine, and does not fill those items with assumptions before receiving an answer.

Scenario: When terms recur in the code or existing documents, they are written in the Product as key concepts

- Given: Terms visible to users are used repeatedly in the code or existing documents.
- When: The agent writes the Product.
- Then: The names and meanings of the key concepts can be read from the Product, and the names match those used in the code or existing documents. The agent shows the developer the meanings it read, and the developer can correct them.

### Choose what to write from the feature list

As a developer of an existing tool, I want to see in one place which features the tool has and which have no specification yet.
This lets me choose which features the next change touches and estimate how much to write.

**Acceptance criteria:**

Scenario: When the project has no specifications, one feature list is created

- Given: A Product exists and there are no feature Requirements.
- When: The agent creates the feature list.
- Then: The Product contains one list from which each feature's name, one-line description, and whether its specification is written can be read, and no per-feature Requirements or Design are created.

Scenario: When the change is decided, the agent shows the features it touches from the list

- Given: A feature list exists and the developer has explained the next change.
- When: The agent decides what to write.
- Then: The agent shows the developer the features the change touches, the features it does not touch, and the features it cannot judge, as separate groups, and the developer can correct the range.

Scenario: When the list alone cannot tell whether a feature is touched, the feature is shown as undetermined

- Given: The descriptions in the list alone cannot tell whether a feature is touched by the change.
- When: The agent decides what to write.
- Then: The agent shows that feature as one it cannot judge, and does not treat it as untouched until it is settled by checking the code or by the developer's answer.

### Write the current state of the features the change touches

As a developer of an existing tool, I want to document the current behavior and structure of only the features the next change touches.
This creates a base for writing the change as a difference, without spending effort on untouched features.

**Acceptance criteria:**

Scenario: When the features to write are chosen, Requirements and Design are created only for those features

- Given: A feature list exists and the developer has chosen the features to write.
- When: The agent writes the current state.
- Then: Requirements and Design are created only for the chosen features, their content represents the current behavior and structure, and they do not include the change to be made this time. In the feature list, those features become features whose specification is written.

Scenario: When the reason for processing is in neither the material nor an answer, it is written as unknown

- Given: The reason for the current structure or processing cannot be confirmed from the code, existing documents, history, or the developer's answers.
- When: The agent writes the rationale in the Design.
- Then: The Design states that the reason is unknown, and no guessed reason is written.

Scenario: When current behavior can be confirmed from the code, it is written without asking the developer

- Given: The current behavior of a chosen feature can be confirmed from the code, tests, and existing documents.
- When: The agent writes the current state.
- Then: The agent writes the confirmed behavior in the Requirements, and does not ask the developer what that behavior is.

Scenario: When asking for a reason, candidates may be shown but are not written without confirmation

- Given: A reason needed to start the next change is not in the material, and the agent can think of candidates from the material.
- When: The agent shows the candidates as options and asks the developer.
- Then: Only what the developer chose or answered in their own words is written in the Design, and a candidate without an answer is not written as a reason.

Scenario: When the developer knows the reason, the reason heard is written

- Given: The reason for the current structure is not in the material, and the developer remembers it.
- When: The agent asks for the reason and the developer answers.
- Then: The Design contains that reason, and when presenting the documents the agent reports that the passage is based on the developer's answer.

Scenario: When presenting the current state, the agent reports the basis of confirmation by group

- Given: The agent has finished writing the current state of a feature in Requirements and Design.
- When: The agent presents the documents to the developer.
- Then: The agent reports, as separate groups, content confirmed from code or tests, content taken from existing documents, content based on the developer's answers, and content left unknown.

Scenario: When only part of a feature is written, the unwritten range is shown

- Given: Behavior of a chosen feature that is unrelated to the change has not been written.
- When: The agent saves the Requirements.
- Then: The out-of-scope section of the Requirements shows that some behavior is not yet written, and its range.

### Write the change as a difference on top of the current state

As a developer of an existing tool, I want to write the next specification by changing the documents that describe the current state.
This lets later changes be handled the same way as in a project that has specifications.

**Acceptance criteria:**

Scenario: When the current-state documents are confirmed, the change is written as an update to the existing documents

- Given: Requirements and Design describing the current state of the features the change touches exist, and the developer has confirmed their content.
- When: The agent writes the requirements for the next change.
- Then: The change is written as an update to the existing Requirements and Design, and the difference in behavior before and after the change can be read from the Git diff.

Scenario: When creating a new feature, work can start without writing the current state of existing features

- Given: A Product and a feature list exist, and the developer has explained a new feature that does not change existing features.
- When: The agent writes the requirements for the new feature.
- Then: Requirements for the new feature are created under the parent Product, and the new feature is added to the feature list. Requirements and Design for existing features are not created for this work.

Scenario: When the developer has not confirmed the current-state documents, the change is not settled

- Given: The agent has written the current state, but the developer has not yet confirmed the content.
- When: The agent is about to write the requirements for the next change.
- Then: The agent asks the developer to confirm the current-state documents, and does not settle the current state and the change mixed in the same update before receiving confirmation.

## KPIs

### Effort to start

- **Current value:** None; this is new
- **Target value:** No numeric target is set. The effort needed varies with the feature and the size of the change, so success is judged by whether the "Range of questions" in the functional requirements is followed.
- **Measurement method:** In the target project, count from the conversation record the questions to the developer that asked about content confirmable from the code or existing documents.
- **Measurement period:** The first time this is applied to a project with existing code.

## Functional requirements

### Guide for starting

**Overview:** Show the steps, in a project with existing code, up to the point where the next change can be written as a difference.

**Input:** Existing code, tests, README, existing documents, and the next change explained by the developer.

**Output:** A Product that contains the key concepts and the feature list, and Requirements and Design for the features the next change touches.

#### Applicable conditions

##### Order of steps

Proceed in this order: check the material, Product, feature list, choose the touched features, write the current state, write the change.
Create the Product and the feature list only the first time.
For later changes, choose the touched features from the feature list, and write the current state only for features without a specification.

##### What the Product contains

The Product contains the purpose and usage scenes, the key concepts, and the feature list.
The purpose and usage scenes express why the product was made and who uses it in which situations, and are written in the existing items of the Product template. Ask the developer about the parts that cannot be confirmed from the material.
The key concepts and the feature list express what exists now, and the agent reads them from the code and existing documents.

##### Range to write

Write the current state only for the features the next change touches.
Untouched features only appear in the feature list.

##### Completion condition

The steps for starting are complete when the next feature change or new feature can be started in the same way as in a project that has specifications.
For a change to an existing feature, this means the parent Product and the Requirements and Design of the features the change touches exist.
For a new feature, this means the parent Product and the feature list exist.
Documents not needed for this state are not added for the sake of completion.

##### Range of questions

The agent reads the current behavior and structure from the code, tests, and existing documents, and does not ask the developer about them.
The agent asks the developer about reasons that cannot be confirmed from the material and about contradictions between materials.
Questions about reasons are limited to those needed to start the next feature change or new feature.
Reasons not needed to start are left unknown.

When asking for a reason, the agent may show candidates it can think of from the material as options.
A candidate is material for the question, and is not written in a document as a reason until the developer chooses it or answers in their own words.

##### Separating the current state from the change

The current-state documents contain only the current behavior and structure.
The content of the next change is written as an update to those documents after the developer confirms them.

#### Exceptional behavior

##### Existing documents and code disagree

When the explanation in an existing document and the behavior of the code disagree, the agent shows both to the developer and confirms which is the intended behavior.
Neither is settled as the current state before an answer is received.

### Key concepts

**Overview:** Place the names and meanings of the key concepts used in the product in the Product.

**Input:** Code, existing documents, and the developer's explanation.

**Output:** The names and meanings of the key concepts in the Product.

#### Applicable conditions

The key concepts are the reference for terms used in feature Requirements and Design.
Names are those already used in the code or existing documents; no new names are created for writing the documents.
Only concepts visible to users are included. Names of types and functions used only internally are not included.
The agent reads the names and meanings of the concepts from the material, and the developer confirms what is shown.

#### Exceptional behavior

##### One concept has several names

When the code and existing documents use different names for the same concept, the agent shows both to the developer and confirms which to use as the reference.

### Feature list

**Overview:** Show the project's features and whether their specifications are written, in one place.

**Input:** The structure of packages and directories, README, existing documents, and the developer's explanation.

**Output:** Each feature's name, one-line description, and whether its specification is written, in the Product.

#### Applicable conditions

The feature list is an index for choosing what to write, and is placed in the Product.
With the list, the whole set of features the product provides can be read from the Product.
Details of behavior and acceptance criteria are not written in the list; they are written in that feature's Requirements.
Code locations and dependencies between features are not written in the list; the agent checks them in the code at the time of a change.
A feature whose specification is written can be followed from the list to its Requirements.

#### Exceptional behavior

##### The list grows large

When there are many features and the list becomes hard to read in the Product, the project chooses how to split it.
The Product may be split, or the list may be moved to a separate document referenced from the Product.
In either case, the feature list remains reachable from the Product.

##### A feature missing from the list is found

When a feature missing from the list is found while writing the current state or making a change, it is added to the list.

### Distinguishing the basis of confirmation

**Overview:** Distinguish what the content written in the current-state documents is based on.

**Input:** Code, tests, existing documents, history, and the developer's answers.

**Output:** Documents that state where a reason is unknown, and a report of the breakdown of the basis.

#### Applicable conditions

The basis is divided into four groups: content confirmed from code or tests, content taken from existing documents, content based on the developer's answers, and unknown.
The body of a document states only that something is unknown.
The breakdown of the basis is written in the report given when the documents are presented.

#### Exceptional behavior

##### An unknown blocks a decision

When an unknown reason is needed to decide the next change, the agent asks the developer.
When the developer does not know either, it is left unknown, and the reason for the change is written anew as this decision.

## Data requirements

### Required data

- **Key concepts:** The reference for terms used in Requirements and Design. Placed in the Product, with the name and meaning of each concept.
- **Feature list:** The index for choosing what to write. Placed in the Product, with each feature's name, one-line description, and whether its specification is written.
- **Breakdown of the basis:** The report given when the current-state documents are presented. Not saved in a document; kept in the completion report or PR description.

### Relationships

One row of the feature list corresponds to one feature.
The row of a feature whose specification is written leads to that feature's Requirements.

### Personal information

None is handled.
Even when history or issues are used as material, the names and contact details of the people who wrote them are not copied into documents.

## External service dependencies

None.
The guideline is bundled with the command, and the material is in the target project's repository.

## Security considerations

Non-public information and credentials read as material are not copied into the documents written.

## Open questions

None.
