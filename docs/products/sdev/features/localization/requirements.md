# Default language and multilingual documents

[日本語](requirements.ja.md)

## About this document

This feature belongs to the [sdev Product](../../product.md). It defines how users select a language and read the same specification in multiple languages.

### Scope

Use English when no language is configured, and also support Japanese.
Project initialization can save a default language and the translation languages maintained alongside it.
Provide both English and Japanese versions of all existing public Markdown documents.

### Out of scope

Runtime machine translation, translation services, and automatic verification of equivalent meaning are outside this feature.
Initialization does not generate specification or translation bodies.
The initial CLI supports English and Japanese.

## User stories

### Choose a project's languages and read its materials

As a developer, I want to share the languages used in a project so that I can read guides and templates without selecting a language every time.

**Acceptance criteria:**

Scenario: Without language settings, materials are available in English

- Given: The project has no language settings.
- When: The CLI runs without an explicit language.
- Then: Instructions and materials are in English, regardless of the terminal's locale.

Scenario: Initializing multiple languages creates shared settings

- Given: The project has no language settings.
- When: It is initialized with English as the default and Japanese as a translation language.
- Then: The project settings and agent instructions identify both languages and explain how to create and update default-language documents and translations.

Scenario: A Japanese project default selects Japanese when the language is omitted

- Given: Japanese is configured as the project's default language.
- When: A guide or template is requested without an explicit language.
- Then: The Japanese material is displayed.

Scenario: Selecting another language for one command preserves shared settings

- Given: A default language is configured.
- When: A command explicitly selects another supported language.
- Then: It displays that language without changing project settings or documents.

Scenario: Unsupported languages or invalid settings prevent writes

- Given: A language is unknown, translation languages are duplicated, or the settings have invalid types or cannot be read.
- When: The CLI or initialization runs.
- Then: It fails with a reason and does not change settings, agent instructions, or launchers.

Scenario: Reinitialization preserves existing settings and edited instructions

- Given: Settings or user-edited agent instructions already exist.
- When: Initialization runs again with the same choices.
- Then: Existing contents are preserved without duplication. A conflicting language choice fails without changes and explains that the settings need editing.

### Read the same specification in both languages

As a specification reviewer, I want to read English publications in Japanese too so that I can judge the same specification in the language I understand best.

**Acceptance criteria:**

Scenario: Switching language versions leads to the corresponding document

- Given: English and Japanese versions of a document exist.
- When: Its other-language link is followed.
- Then: The corresponding document is available, with links and headings leading to related materials in that language.

Scenario: Reference-direction checks apply in every language

- Given: Product, Requirements, and Design documents have multiple language versions.
- When: Documents are checked.
- Then: Every version is classified by the same responsibility, and actual files and headings are checked. Navigation to another language version of the same Design is not treated as a reference to another Design.

## KPIs

No ongoing adoption target is defined. Validate changes through the acceptance criteria and by reviewing both language versions.

## Functional requirements

### Language selection and initialization

**Overview:** Select the display language from the explicit command option, project settings, then English, in that order.

**Input:** A default language, translation languages, and an optional language for one command.

**Output:** Guides, templates, and instructions in the selected language, plus project settings that can be shared in Git.

If a supported language lacks a material, display the English version and explain the fallback.
Read settings from the current project root; do not implicitly inherit settings from a home directory or parent project.
Distinguish the language of CLI materials from the default language of documents authored in the consuming project, and honor that project's instructions and conventions.

### Multilingual documents

**Overview:** Default-language documents use ordinary names; documents in other languages have names identifying their language.

**Input:** The default language and additional languages for the same content.

**Output:** Language versions that can be read and maintained as the same document with the same responsibility.

English is the default in this repository, and every existing Markdown document has a Japanese version.
Translations do not replace original legal notices or program identifiers.
Language versions express one document, rather than independent requirements or designs.
Update both versions in the same change when meaning changes. Resolve differences before implementing anything that depends on them.

## Data requirements

### Required data

- **Language settings:** The project's default language and translation languages maintained with it.
- **Localized materials:** Bodies and references for each language of the same task or document.

### Relationships

Language versions share one document's responsibility and decisions.

### Personal information

None. Language settings do not need personally identifying data.

## External service dependencies

None. Reading settings and displaying embedded materials work locally.

## Security considerations

Language settings must not execute commands. Initialization must not start writing when settings cannot be read safely.

## Open questions

None within this scope. Add further languages through changes that provide their materials.
