# [Feature name] design

[日本語](design.ja.md)

## About this document

[Link to the corresponding Requirements and parent Product with relative Markdown links. State the design's purpose without repeating requirements.]

### Audience

[People and agents implementing or changing this feature.]

### Prerequisites

[Knowledge and documents needed to understand the design.]

### Outside this document

[What belongs to other documents or features.]

## Implementation approach

[Establish deliverables and the main flow, then develop responsibilities, processing, and failures. See the [Design guide](../guidelines/design.md#write) for organizing the explanation. Leave detailed internal file lists to placement conventions.]

## Architecture and sequence diagrams

[Structure, responsibilities, and main interactions. Use Mermaid sequenceDiagram where helpful; explain when no diagram is needed.]

## Data model and ER diagrams

[Types, structures, and relationships meeting Requirements data needs. Use Mermaid erDiagram when a relationship diagram helps.]

## APIs and interfaces

[Explain each interface's role, then its inputs, outputs, and failures. Separate user actions, internal exchanges, and presentation into appropriate subheadings and paragraphs. Without an API, describe actual interfaces such as documents or commands.]

## External service integration

[Connections, authentication, and dependency failures. Name implementation dependencies and their reasons. Return new user-facing usage decisions to Requirements.]

## Data handling

[Sources, collection, use, storage, retention, deletion, protection, and correspondence to the data model. Refer to Requirements for purposes.]

## Authentication and authorization notes

[Who can access what, permission boundaries, and credentials. If not applicable, state None with a reason.]

## Security considerations

[Concrete protection mechanisms or other necessary design conditions. If not applicable, state None with a reason.]

## Deployment notes

[Unusual cutover, migration, or recovery beyond ordinary deployment. Write None when there are no special notes. Do not restate Product reliability targets or test procedures.]

## Alternatives considered

### [Alternative]

[Advantages, disadvantages, why it was not chosen, and reasons for the selected design.]

## Open questions

[Unresolved decisions, interim treatment, revisit conditions, and who decides. Resolve anything needed for implementation. Write None if there are none.]
