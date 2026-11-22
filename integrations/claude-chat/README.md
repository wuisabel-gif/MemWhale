# Claude + MemoryWhale

Claude is a chat product, not a coding agent with local hooks. MemoryWhale
reaches it through files: your conversations come in through the data export
Claude gives you, and approved statements go back as a context block you
paste. See [knowledge memory](../../docs/reference/knowledge.md).

## Status

Import is implemented and unit-tested against the publicly described
structure of the export's `conversations.json`. It has **not** been checked
against a real Claude export. The export steps follow
[Claude's help page](https://support.claude.com/en/articles/9450526-export-your-claude-data), read on
2026-11-22.

## Requirements

- MemoryWhale 0.30.0 or later, with `mw` on your `PATH`.
- `unzip` on your `PATH`, to read the export without unpacking it (present
  on macOS and most Linux systems).
- A Claude plan that offers data export.

## Setup

1. In Claude, open Settings, Privacy, Export data, and request the export.
2. Claude emails a download link. Save the `.zip` to `~/Downloads`.
3. Import it. With no arguments MemoryWhale finds it there, along with your
   Claude Code and Codex conversations:

```bash
mw knowledge import
```

To import only this file, name it:

```bash
mw knowledge import ~/Downloads/<the-export>.zip
```

Only messages you wrote are read. Nothing is used until you approve it, and
nothing is sent anywhere.

## Verify

```bash
mw knowledge pending
```

Statements from this export are marked `from claude_chat`. If the import reports
`0 new statement(s)` for a first import, the export's structure may differ
from what the parser expects; see Troubleshooting.

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | No. Claude reaches a custom connector from its own cloud, which would mean exposing MemoryWhale on a public address. |
| Automatic execution capture | No |
| Memory-use guidance | Not applicable |
| Import your messages from a data export | Yes, unverified against a real export |
| Give approved statements back | Yes, by pasting the output of `mw knowledge export` |
| Read or write Claude's built-in memory | No. Anthropic offers no way for another program to write it; the export may list memory entries, which this import does not read yet. |

## Example prompt

Paste the output of `mw knowledge export` at the start of a chat, then:

> The block above is what I have told my other assistants. Keep it in mind
> for this conversation.

## Troubleshooting

- **No export found.** The import looks in `~/Downloads` for a `.zip` that
  lists `conversations.json`, or a folder that holds it. Name the file if it
  is somewhere else.
- **Zero statements from a large export.** Extraction keeps only sentences
  that state a preference, a fact about you, a decision, or a standing
  instruction, in English. If you expected more, open an issue and say how
  the export is laid out, without sharing its contents.
- **`unzip` is missing.** Unpack the archive yourself and import the folder.

## Uninstall

Nothing is installed in Claude. To remove what was imported, reject it in
`mw knowledge pending`, or delete approved statements with
`mw knowledge rm <id>`. The export file in `~/Downloads` is yours to delete.
