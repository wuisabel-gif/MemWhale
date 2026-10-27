# pstack + MemoryWhale

[pstack](https://github.com/cursor/plugins/tree/main/pstack) is Lauren Tan's
([@poteto](https://x.com/poteto)) MIT-licensed skill stack for rigorous
AI-assisted engineering, for Cursor, Claude Code, Codex, and other agents.
This folder adds one thing: a MemoryWhale source playbook for pstack's `why`
skill, which pstack's `recall` also uses. The playbook is MemoryWhale's own
text, written in the format of pstack's `why/references/sources/` playbooks.
No pstack files are copied here.

## Status

Written against `cursor/plugins` at commit
`ab1f702ce8e8b6557edb56b09f52cde8bfb83f17` (October 27, 2026). The `why` skill
there last changed in `ce649c03ac3b75b6ae80cbd15b7e2c536caedb5a`. The relevant
upstream files are
[`why/SKILL.md`](https://github.com/cursor/plugins/blob/ab1f702ce8e8b6557edb56b09f52cde8bfb83f17/pstack/skills/why/SKILL.md),
[`why/references/source-playbook.md`](https://github.com/cursor/plugins/blob/ab1f702ce8e8b6557edb56b09f52cde8bfb83f17/pstack/skills/why/references/source-playbook.md),
and
[`recall/SKILL.md`](https://github.com/cursor/plugins/blob/ab1f702ce8e8b6557edb56b09f52cde8bfb83f17/pstack/skills/recall/SKILL.md).

`why` lists the MCP servers an agent has, sorts each into one evidence
category, and sends one investigator per category with that source's
playbook. `recall` reuses those investigators to ask what was tried and did
not hold. MemoryWhale belongs under **error / exception tracking**.

- **Verified:** every call in [`sources/memorywhale.md`](sources/memorywhale.md)
  runs against `mw-mcp` on a synthetic store, and the strings the playbook
  tells an investigator to read are what `mw-mcp` returns.
- **Not verified:** a pstack `why` or `recall` run that picks up the playbook.

## Requirements

- pstack installed in your agent, per its README.
- MemoryWhale's `mw-mcp` connected in that same agent, for example
  [Claude Code](../claude-code/README.md), [Cursor](../cursor/README.md), or
  [Codex](../codex/README.md). `why` finds MCP servers from the agent's own
  configuration; this folder configures nothing.
- A store with captured history. The playbook only reads.

## Setup

Give pstack the playbook in one of two ways.

**In your own copy of pstack (recommended).** pstack's README invites forks.
Copy [`sources/memorywhale.md`](sources/memorywhale.md) to
`pstack/skills/why/references/sources/memorywhale.md`, and add a row to
`pstack/skills/why/references/source-playbook.md`:

```markdown
| Error / exception tracking | [`memorywhale.md`](./sources/memorywhale.md) | MemoryWhale `mw-mcp` (local terminal and agent history) |
```

In a fork of `cursor/plugins`, regenerate the Claude Code copy afterwards with
`node scripts/generate-pstack-harness-trees.mjs`, as pstack's README says.

**Without a copy (unverified).** pstack's Claude Code adapter reads project
rules from `CLAUDE.md`, or `AGENTS.md` when there is no `CLAUDE.md`. Add one
line there, with the absolute path to this file:

```markdown
When pstack's `why` or `recall` sends an investigator to the MemoryWhale MCP, give it `/absolute/path/to/integrations/pstack/sources/memorywhale.md` as its source playbook.
```

Do not edit the installed plugin under `~/.claude/plugins/cache/`. Updates
replace it, and pstack's own `reflect` skill says never to edit there.

## Verify

From a MemoryWhale checkout:

```bash
cargo build -p memorywhale-cli --bins
python3 scripts/test-pstack-integration.py --bin-dir target/debug
```

This runs the playbook's calls against a temporary synthetic store. It does
not run pstack. In your agent, run `why` on code with a known failure and
check that the answer's Sources Consulted section has a MemoryWhale line.

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Through the agent's own MemoryWhale MCP configuration. The playbook uses only `stats`, `similar_failures`, `recent_errors`, `search_memory`, and `get_context`, and never `remember` |
| Automatic execution capture | No; capture comes from the agent's own MemoryWhale hooks, if installed |
| Memory-use guidance | Yes, a source playbook for `why` (also used by `recall`); loading inside pstack is unverified |

## Example prompt

In Claude Code, pstack's skills are prefixed and invoked by hand:

> /pstack:why why does `calc::add` widen to i64?

> /pstack:recall my work on the calc add bug

## Troubleshooting

- **Sources Consulted has no MemoryWhale line, or skips error tracking for
  lack of an MCP.** The agent running pstack has no MemoryWhale MCP. Check its
  MCP list (`claude mcp list` in Claude Code) and the client guide.
- **MemoryWhale is searched but the playbook is ignored.** Use the copy route
  in Setup; the `CLAUDE.md` pointer is unverified.
- **`stats` shows `"memories": 0` or an unexpected `db`.** The agent's MCP
  config selects another store. Check `MEMORYWHALE_DATA_DIR` there.
- **Every search returns the same rows.** `search_memory` has no relevance
  floor. The playbook says how to tell a match from filler.

## Uninstall

Delete `memorywhale.md` and its `source-playbook.md` row from your copy of
pstack, or remove the line from `CLAUDE.md` or `AGENTS.md`. This leaves
MemoryWhale's MCP configuration and its database untouched.
