# [Feature name]

[日本語](requirements.ja.md)

## About this document

[Summarize the feature's behavior and link to its parent Product with a relative Markdown link.]

### Scope

[What this feature covers. Avoid repeating detailed conditions.]

### Out of scope

[What it excludes and boundaries that may belong to another feature.]

## User stories

### [Story name]

As a [user], I want [action] so that [value received].

**Acceptance criteria:**

Scenario: When [condition], [expected result]

- Given: [State before the action]
- When: [An action serving one purpose]
- Then: [Observable result, including absence of changes where relevant]

[Separate different outcomes into different scenarios. Consider success, failure, and boundaries.]

## KPIs

### [Metric name]

- **Current value:** [Measured value, unmeasured, or none for a new feature]
- **Target value:** [Specific result to achieve]
- **Measurement method:** [Which results establish the value]
- **Measurement period:** [When to measure]

## Functional requirements

### [Function name]

**Overview:** [Behavior]

**Input:** [User actions and required information]

**Output:** [What the user receives]

#### Applicable conditions

##### [Topic]

[Group relevant permissions, inputs, and completion conditions by topic. Omit subheadings when only one topic exists.]

#### Exceptional behavior

##### [Exception]

[What happens and what remains unchanged when this exception occurs.]

## Data requirements

### Required data

- **[Data name]:** [Role and main information. Include user-facing collection, retention, or deletion conditions. Leave types and storage locations to Design.]

### Relationships

[Relationships needed for the feature.]

### Personal information

[None, or identifiable data and its purpose, including identification through combination with other data.]

## External service dependencies

- **[Service name]**
  - **Related user stories:** [Stories requiring this service]
  - **Reason for dependency:** [What cannot be achieved without it]
  - **Contractual and usage decisions:** [Costs and usage terms. Leave technical connections and implementation choices to Design.]

[If none, state None and explain why.]

## Security considerations

[Information and access conditions the feature must protect. Leave protection mechanisms to Design.]

## Open questions

[Unresolved decisions, interim treatment, revisit conditions, and who decides. Resolve anything needed for this feature to work. Write None if there are none.]
