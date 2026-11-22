# Knowledge memory

MemoryWhale's debugging memory records what a terminal did. Knowledge memory
is a second, opt-in store for durable statements you want several assistants
to share: a preference, a fact about you, a project decision, a standing
instruction. The design proposal is
[ADR 0001](https://github.com/wuisabel-gif/MemWhale/pull/522), still under review.

Nothing is used until you approve it, and a store that never runs a
`mw knowledge` command behaves as it did before.

## Bring in what you have already said

```bash
mw knowledge import
```

With no arguments, this reads every source it finds on this machine:

| Source | Where it looks | What it takes |
| --- | --- | --- |
| Claude Code memory files | `~/.claude/projects/<project>/memory/*.md` | Each file's one-line `description`. |
| Claude Code transcripts | `~/.claude/projects/<project>/*.jsonl` | Sentences you typed that state a preference, a fact about you, a decision, a project fact, or a standing instruction. |
| Codex sessions | `~/.codex/sessions/**/rollout-*.jsonl` | The same, from what you typed. |
| ChatGPT export | A `.zip` in `~/Downloads` that holds `conversations.json`, or its unpacked folder | The same, from your messages. |
| Claude chat export | The same, for a Claude export | The same, from your messages. |

ChatGPT and Claude do not let another program read your conversations, so
those two need one manual step first. In ChatGPT: Settings, Data controls,
Export data. In Claude: Settings, Privacy, Export data. Each emails a download
link; save the file to `~/Downloads` and run the import again.

To import one file or folder, name it:

```bash
mw knowledge import ~/Downloads/my-export.zip
```

Only what you typed is read. Assistant replies, tool output, and text a client
adds to your turn are skipped, and so are questions, code blocks, and long
pasted text. A sentence that still looks like it holds a secret after masking
is not stored. Running the import again proposes nothing it has seen before,
including statements you rejected.

Extraction is a fixed list of English sentence patterns. It favors a short
review queue over finding everything, and it will miss statements phrased
another way. The curated Claude Code memory files are the richest source.

## Review

```bash
mw knowledge pending
```

```bash
mw knowledge approve 3 7
```

| Command | Effect |
| --- | --- |
| `mw knowledge approve <id...\|all>` | Use the statement. Agents and context packs may show it. |
| `mw knowledge keep <id...\|all>` | Use the statement, but keep it private: shown only when you ask with `--private`. |
| `mw knowledge reject <id...\|all>` | Do not use it, and do not propose it again. |
| `mw knowledge rm <id>` | Delete it. A record that something was deleted stays; its text does not. |

There is no setting that approves statements automatically.

## Use

```bash
mw knowledge add "I prefer small pull requests" --kind preference
```

```bash
mw knowledge search pull requests --project alpha
```

```bash
mw knowledge export --project alpha --client claude-code
```

`export` prints a bounded Markdown block between
`<!-- memorywhale:knowledge:start -->` and `<!-- memorywhale:knowledge:end -->`
markers. Paste it into a chat, or into a `CLAUDE.md` or `AGENTS.md` file.
Each statement is quoted as a fact with its source and date, and the block
says it is not an instruction. MemoryWhale never edits your instruction files
itself.

A statement about a project or a decision belongs to the folder it was said
in, and is shown only with `--project <that folder>`. Other statements are
global.

## Limits, stated plainly

- Project and private limits decide what a configured client is shown. They
  are not a security boundary: any program running as you can read the store
  file.
- MemoryWhale does not read or write the memory built into ChatGPT or Claude.
  Neither offers a way to.
- The ChatGPT and Claude export parsers follow the publicly described
  structure of `conversations.json`. They have not yet been checked against a
  real export.
- Archived Codex sessions are compressed and are not read.
- Knowledge never reaches debugging recall. The command stop, failure notes,
  the MCP tools, `mw export`, and `mw memory publish` do not read it.
