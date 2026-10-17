# Referencing guidelines and templates

[日本語](requirements.ja.md)

## About this document

Define how an agent finds and reads the guidelines and templates needed for document work from a task entry point.
The parent [sdev Product](../../product.md) provides the context.

### Scope

- Reference guidance for creating and changing documents using repository Markdown.
- Selection of document types and prerequisite specifications for a task.
- Guidance when required materials cannot be read.
- Document presentation, review, and user decisions appropriate to the request's scope.
- Question tools and follow-up dialogue based on answers.

### Out of scope

- Web publication, public URLs, or version-specific delivery.
- Automatically conducting dialogue, writing, or implementation, or mechanically enforcing agent compliance.
- Implementation of the CLI, document lint, relationship visualization, or document migration.

## User stories

### Read materials needed for the task

As someone developing specifications with an AI agent, I want a short reference instruction to lead the agent to the right materials so I spend less effort finding and supplying them for every task.

**Acceptance criteria:**

Scenario: Writing a Product, the agent can reach materials for clarifying purpose

- Given: The agent can read the entry point and is asked to organize a product's purpose.
- When: It reads the Product-writing entry.
- Then: It can read document responsibilities, Product-writing guidelines, and the Product template.

Scenario: Writing Requirements, the agent can read scenarios and upstream guidance

- Given: The entry point is available and the feature has a parent Product.
- When: The agent reads the Requirements-writing entry.
- Then: It can read responsibility, scenario, and Requirements guidelines, the template, and instructions to read the parent Product first.

Scenario: Writing Design, the agent can reach materials grounded in Requirements

- Given: The entry point is available and the feature has Requirements.
- When: The agent reads the Design-writing entry.
- Then: It can read responsibility and Design guidelines, the template, and instructions to read the parent Product and corresponding Requirements first.

### Check update policy before changing specifications

As someone developing specifications with AI, I want guidance appropriate to the kind of change so I can select what to update without duplicating past documents.

**Acceptance criteria:**

Scenario: Changing an existing feature, the update policy distinguishes the type of change

- Given: The entry point is available and an existing feature needs changing.
- When: The agent reads implementation and change guidance.
- Then: It can distinguish requirements changes, design improvements during implementation, and method-only improvements after implementation, and learn how to read earlier versions.

### Stay within the request and review requirements and design

As someone developing with AI, I want to review changed Requirements and Design while work stays within my request so I can correct misunderstandings and participate in necessary decisions.

**Acceptance criteria:**

Scenario: An expressed wish to build something does not authorize implementation

- Given: The user expresses an idea, without prior authorization to design and implement it.
- When: The agent reads shared working guidance.
- Then: It can learn to clarify purpose and scope through dialogue and research, propose the next task, and avoid automatically proceeding to design and implementation.

Scenario: A Requirements-only request ends with presenting the document

- Given: The user requests only Requirements creation or revision.
- When: The agent reads Requirements and shared working guidance.
- Then: It can learn to save, check, and present the document and key points, request review, and end the task without expanding into Design or implementation.

Scenario: A Design-only request ends with presenting the document

- Given: The user requests only Design creation or revision.
- When: The agent reads Design and shared working guidance.
- Then: It can learn to save, check, and present the document and key points, request review, and end the task without expanding into implementation.

Scenario: An implementation request permits progress while presenting key decisions

- Given: Implementation is requested, without a specified review stop or unresolved decision needing the user.
- When: The agent reads shared working guidance.
- Then: It can learn to present Requirements and Design decisions during the work and continue within scope without a mandatory approval stop for every document.

Scenario: A required user decision pauses dependent work

- Given: Purpose, scope, or observable behavior requires a decision not established by the request or specifications.
- When: The agent reads shared working guidance.
- Then: It can learn to present choices and reasons to the user and wait before dependent design or implementation.

Scenario: A user-specified review stop is respected

- Given: Implementation is requested, but only after review of Design.
- When: The agent reads shared working guidance.
- Then: It can learn to present Design and wait for instructions without treating silence as approval.

### Make ideas concrete through questions

As someone organizing specifications with AI, I want dialogue to clarify purpose and conditions so unclear ideas are explored without being replaced by the agent's assumptions.

**Acceptance criteria:**

Scenario: Vague initial input leads to questions

- Given: The user expresses an idea whose purpose, usage, or expected results are not yet concrete.
- When: The agent reads the shared and document-specific guidance.
- Then: It can learn to ask questions matching the document's responsibility instead of writing from invented assumptions.

Scenario: Abstract answers guide the next question

- Given: An answer allows multiple interpretations of situations or conditions.
- When: The agent reads shared working guidance.
- Then: It can learn to explore examples, success conditions, and exceptions from that answer, reflecting its understanding as dialogue continues.

Scenario: An available question tool is preferred

- Given: A question is needed and a tool offers selectable answers.
- When: The agent reads shared working guidance.
- Then: It can learn to prefer the tool and explain differences, recommendation reasons, and free-text input.

Scenario: Without a question tool, questions are asked in conversation

- Given: A question is needed but the current environment or mode offers no question tool.
- When: The agent reads shared working guidance.
- Then: It can learn to ask in the conversation rather than skip the question.

Scenario: Confirmed conditions are used without repeated questions

- Given: Concrete conditions are established by conversation or materials and need no further user decision.
- When: The agent reads shared working guidance.
- Then: It can learn to use those conditions without repeating questions merely to complete a checklist or question count.

Scenario: An unanswered asynchronous question does not finalize dependent content

- Given: A question requiring the user's answer is pending.
- When: The agent reads shared working guidance.
- Then: It can learn to wait on dependent documents or implementation without treating silence or preselection as the user's decision.

### Understand failures to read materials

As someone developing specifications with AI, I want guidance when materials are unavailable so work does not proceed as if missing sources had been read.

**Acceptance criteria:**

Scenario: Unavailable materials have readable failure guidance

- Given: The entry point and common guidelines are readable, but another required material cannot be retrieved.
- When: The agent reads retrieval-failure guidance.
- Then: It can learn to report the missing source, avoid pretending it was read, and wait on dependent decisions when no suitable alternative is available.

This verifies the availability of guidance, rather than guaranteeing that an agent always complies with it.

## KPIs

### Reachable materials by task

- **Current value:** No comparable results for this new feature.
- **Target value:** All three writing tasks reach their guidelines and templates from the entry point, 3/3.
- **Measurement method:** Follow actual links according to each acceptance scenario and inspect the contents.
- **Measurement period:** At feature completion and whenever reference placement changes.

## Functional requirements

### Direct readers to reference materials

**Overview:** Select required materials and prerequisite specifications from the kind of work.

**Input:** A task: Product, Requirements, Design, implementation, or revision.

**Output:** Relevant materials and their reading order.

#### Applicable conditions

##### Guidance coverage

The entry point also reaches shared working principles.
Maintain each language version in one place, with entry points and project instructions referencing it.
Distinguish an idea, a document-only request, and an implementation request.
Explain presentation and progress according to request scope, unresolved decisions, and specified review stops.
Honor instructions already given for the task without repeating confirmations.
Explain questions, answer-driven follow-up, preference for question tools, and conversational alternatives.
Distinguish confirmed answers from agent proposals and leave unanswered decisions unresolved.
Repository Markdown remains directly readable without installing a CLI or building materials.

##### Continued use

Exclude organization-private materials, local absolute paths, and dedicated-tool dependencies from distributed guidance.
Provide a short instruction example for another project.
Its reference must be readable; do not advertise an unpublished URL as available.

#### Exceptional behavior

##### Reference failures

Provide guidance meeting the retrieval-failure scenario above.
If the entry point itself is unreadable, the user must check the instruction's location; distributing documents alone does not repair it.

##### Missing prerequisites

Guide readers to create a missing parent Product or Requirements first.
If several targets are possible, investigate existing candidates before asking the user to select one.

## Data requirements

### Required data

- **Guidelines:** Roles, update criteria, acceptance writing, and the practices of preparation, research, writing, and review.
- **Templates:** Structures for Product, Requirements, and Design.
- **Reference guidance:** Task-to-material correspondence, prerequisite specifications, and project instruction examples.

### Relationships

Task guidance references guidelines and templates.
Guidelines direct readers to upstream specifications without copying their contents.

### Personal information

None is needed or retained.
Specifications created by users in other projects are not collected by this feature.

## External service dependencies

None. Reading repository documents requires no service contract, cost decision, or service selection.

## Security considerations

Exclude personal information, confidential data, and credentials from distributed materials and examples.
This feature does not send the consuming project's documents outside the project.

## Open questions

None needed for this feature.
Web publication remains an [open Product question](../../product.md); this feature supplies local reference guidance.
