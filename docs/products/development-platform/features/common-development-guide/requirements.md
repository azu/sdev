# Shared development guide

[日本語](requirements.ja.md)

## About this document

This feature of the [Rust development platform](../../product.md) lets people and AI reference shared development and verification procedures.

### Scope

Explain environment setup, routine verification, dependency changes, checking other environments, and document-update decisions for sdev's own Rust development.
This feature provides procedures; it does not guarantee that they have been executed.

### Out of scope

Rust teaching materials, product distribution specifications, CI implementation, automatic repair of settings, and imposing the environment on external projects.

## User stories

### Find common procedures

As people and AI developing sdev, we want to reach the development guide from the task entry point so that we need not reconstruct setup and verification procedures from conversations.

**Acceptance criteria:**

Scenario: Starting a Rust change, developers can reach the required procedures

- Given: sdev's documents are available.
- When: A developer follows Rust development guidance from the documentation entry point.
- Then: They can read setup, formatting, static checks, tests, relationships with CI, and criteria for dependency and document changes.

### Distinguish verification scope

As implementers, we want to distinguish executed verification from unavailable verification so that local success is not reported as success everywhere.

**Acceptance criteria:**

Scenario: When an environment is unavailable, unverified results and reasons can be reported

- Given: Some operating systems affected by a change are unavailable in the working environment.
- When: An implementer reads the guide's reporting criteria.
- Then: They can distinguish the execution environment, checks, and results from unverified environments and their reasons.

Scenario: Without the required settings, developers can avoid claiming working behavior

- Given: Documents exist but target Rust settings or code are absent.
- When: An implementer reads environment setup guidance.
- Then: They can identify prerequisites and the requirement to report missing items rather than claim success.

### Check the impact of environment changes

As contributors taking over development, we want to find checks for environment and dependency changes so that changing settings does not leave procedures and specifications outdated.

**Acceptance criteria:**

Scenario: Updating Rust or dependencies, developers can select verification and document changes

- Given: The environment or dependencies need changing.
- When: An implementer reads the change guidance.
- Then: They can review reasons, verification, locked dependencies, CI, and affected specifications and guides.

## KPIs

### Acceptance-criterion review coverage

- **Current value:** Unmeasured.
- **Target value:** All acceptance criteria in this document can be verified.
- **Measurement method:** Read documents and verify entry-point reachability and required explanations, separately from procedure execution rates.
- **Measurement period:** Whenever the guide is created or changed.

## Functional requirements

### Reference development and verification criteria

**Overview:** Reach shared development and verification criteria and their prerequisites from the entry point.

**Input:** Work performed by developers or AI and its specifications and settings.

**Output:** Required procedures, decision reasons, and criteria for reporting verification scope.

#### Applicable conditions

People and AI share the guide; no specific editor operation is mandatory.
Distinguish official tool explanations from sdev's adopted development policies.
Explain setting responsibilities and update decisions rather than copying every configured value.

The guide must distinguish command success from satisfaction of product requirements.
Separate generated artifacts and temporary experiments from maintained source and documents. Do not make relaxed rules a routine way to skip necessary checks.

#### Exceptional behavior

Report unavailable references or settings without inventing results.
Investigate differences between settings and guidance, and report actual conditions and unresolved differences.
The document itself does not automatically change settings.

## Data requirements

### Required data

- **Development guide:** Procedures, prerequisites, decision reasons, and primary-source references.
- **Setting references:** Roles of settings determining the environment and verification, without copies of settings or verification logs.

### Relationships

The guide corresponds to this feature and references the development platform purpose and existing document rules.
Keep execution results in work reports, separately from guide rules.

### Personal information

None. Reading development procedures needs no user identification or behavior collection.

## External service dependencies

None. The reference feature works through repository documents.
Connections for environment installation or CI execution are separate from dependencies for reading the guide.
No new paid services or usage contracts are introduced.

## Security considerations

Exclude credentials, machine-specific paths, private source contents, and internal URLs from public guidance.
Explain necessary care when fetching dependencies and executing development code. Do not automatically send environment information or verification logs.

## Open questions

None.
