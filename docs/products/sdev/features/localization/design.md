# Default language and multilingual document design

[日本語](design.ja.md)

## About this document

Based on the [Requirements](requirements.md) and [sdev Product](../../product.md), this design defines language selection, document placement, and initialization.

### Audience

People and AI creating or changing the CLI and its materials.

### Prerequisites

Read [Document responsibilities and updates](../../../../guidelines/documents.md) and the existing initialization and document-checking requirements.

### Outside this document

Machine translation, translation services, and automatic language downloads.

## Implementation approach

Store `language = "en"` and `translations = ["ja"]` in sdev.toml at the project root.
Keep shared configuration outside the .sdev/ executable cache.
Missing settings default to English; omitted translations default to an empty list.
Support en and ja. Reject a translation matching the default language, duplicates, unknown keys, and invalid types.

Accept an explicit `--lang` option for every command, taking precedence over project settings.
Do not infer a language from LANG or the operating system. Do not add environment variables or personal configuration files.
Look for settings only in the current directory. `--version` must work even when project configuration is invalid.

Use `init --lang en --translations ja` to select the default and translation languages.
Validate existing settings and preserve their bytes when options match or are omitted.
When explicit settings conflict, explain how to edit sdev.toml and leave all initialization targets unchanged.
Integrate validation of configuration, AGENTS.md, .gitignore, and the launcher with the existing staged-write preparation.
Do not overwrite existing sdev instructions solely to migrate their language.

Alongside guide-selection instructions, AGENTS.md tells agents to create and update documents using the default and translation languages in sdev.toml.
Default-language documents use ordinary .md names; other languages use .<language>.md.
Initialization communicates these rules without generating Product, Requirements, Design bodies, or empty directories.

Parse settings with the [TOML library](https://docs.rs/toml/latest/toml/).
The existing serde_json dependency cannot parse TOML, so add toml with only the features needed for parsing.
Do not write a custom line parser. Pin its version and resolved dependencies in Cargo.toml and Cargo.lock.

## Architecture and sequence diagrams

Parse arguments, read settings, select the display language, and dispatch the existing command.
No diagram is needed for this sequential flow within one process.

## Data model and ER diagrams

Language represents en and ja; ProjectConfig holds a default language and translation languages.
Identify embedded materials by operation, topic, and language.
No external database or persistent relationship table is needed.

## APIs and interfaces

Select a guide with `sdev help design --lang ja`, or a template with `sdev template requirements --lang ja`.
Language options may precede or follow the command. Missing values, duplicate options, and unsupported languages return exit code 2.
Initialization uses `sdev init --lang en --translations ja`; translations accepts a comma-separated list.
Invalid shared settings also return exit code 2 without changing files.

English distribution materials keep their existing paths; Japanese versions use .ja.md in the same directory.
Japanese links target Japanese files and headings; English links target English ones.
Each document has a relative link to its other language version.
Register both languages when embedding materials, and rewrite guide links to CLI commands retaining that language.
References written as CLI commands also retain the Japanese selection so the next material uses the same language.
When a translation is unavailable, fall back to English and report that on standard error, leaving the Markdown body clean.

Lint also discovers localized README and AGENTS files, and determines specification responsibilities after removing the language suffix.
Treat only a corresponding language version of the same Design as the same document; references to another feature's Design remain forbidden.
Check actual target files and headings without merging language versions for existence checks.
JSON keys, diagnostic codes, and exit codes stay the same across languages. Localize human-readable text only.

## External service integration

None. Both languages are embedded at build time.

## Data handling

Read sdev.toml as a regular UTF-8 file, rejecting symlinks and directories.
Parse and preserve existing files; serialize the required settings only when creating a new file.
Update language versions together without adding independent requirements or decisions to a translation.

## Authentication and authorization notes

None. This feature handles local documents and settings.

## Security considerations

Validate configured values as supported language identifiers and never interpolate them into shell commands.
Report configuration read errors instead of silently reverting to English.
Apply root-boundary and symlink checks to all language versions too.

## Deployment notes

Existing commands without a language selection change their display to English.
Existing Japanese Markdown paths remain as English-version paths; Japanese versions gain .ja.md.
Existing initialized projects can gain the configuration file while preserving user-edited instructions.
Older executables do not understand the new language options; update the executable before using them.

## Alternatives considered

### Infer the language from the terminal

This reduces configuration but makes output differ between machines and CI. Use explicit shared settings instead.

### Duplicate the documentation tree by language

This groups each language, but makes corresponding specifications harder to compare. Keep the existing layout and place language versions together.

## Open questions

None within this scope.
