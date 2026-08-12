# MCP reference

`mw-mcp` is a [Model Context Protocol](https://modelcontextprotocol.io) server
that exposes your MemoryWhale store to a coding agent (Claude Code, Codex,
Cursor, …) directly — so instead of you pasting past errors and fixes into the
chat, the agent queries them itself. It speaks newline-delimited JSON-RPC 2.0
over stdio and is read-mostly: the only tool that writes is `remember`, and it
saves a single note.

If you're working from a source checkout, the binary is
`cargo run -p memorywhale-cli --bin mw-mcp --`; installed, it's just `mw-mcp` on
your PATH (see the README's Install section). It reads the same local database
as the `mw` CLI — nothing leaves your machine.

## Wire it into an agent

Claude Code, one line:

```bash
claude mcp add memorywhale -- mw-mcp
```

Or add a stanza to an MCP config (`claude_desktop_config.json`, `.mcp.json`, or
your agent's equivalent) pointing at the installed binary:

```json
{
  "mcpServers": {
    "memorywhale": {
      "command": "mw-mcp"
    }
  }
}
```

No arguments, no API key. If `mw-mcp` isn't on your PATH, use its absolute path
as `"command"`.

## Tools

All six take a JSON object and return text (JSON-RPC `result.content[].text`).
Required args are noted; everything else is optional.
Run `mw-mcp --list-tools` to print the tool names directly from the runtime
registry. `scripts/check-doc-references.sh` verifies this table and the README
against that registry.

| Tool | Purpose | Args | Returns |
| --- | --- | --- | --- |
| `recent_errors` | Recent failed commands (non-zero exit) with their error output — start here when debugging a recurring failure. | `limit` (int, default 8) | A list of failed runs: command, exit code, cwd, the salient stderr line, and any note. |
| `search_memory` | Search remembered commands, sessions, and notes for a term, ranked by the explainable engine. | `query` (string, **required**); `project`, `machine` (string, optional scope) | Ranked hits with a score, a snippet, and the reasons each ranked where it did. |
| `get_context` | The most relevant remembered memory, engine-ranked, optionally scoped. | `project`, `machine` (string, optional) | Up to 8 ranked hits, each with score, snippet, and reasons. |
| `remember` | Save a freeform lesson or conclusion so future sessions don't re-derive it. | `text` (string, **required**) | Confirmation with the new memory id; findable later via `search_memory`/`get_context`. |
| `similar_failures` | Check whether an error you just hit has occurred before, and whether a later run resolved it. | `error_text` (string, **required**); `command` (string, optional — enables an exact fingerprint match) | Evidence-only history: occurrence count, how often a later run of the same command succeeded, and a pointer to a concrete past occurrence. |
| `stats` | Health/liveness check: confirm the store is reachable and populated before relying on the other tools. | none | JSON: total memory count, how many are recorded failures, the most-recent memory timestamp (or `"none"`), and the database file path. |

A fresh, empty store is handled gracefully — reads return empty results (and
`stats` returns zero counts) rather than erroring.

## The loop

The point isn't any single tool — it's the loop between them:

1. The agent runs a command and hits an error.
2. It calls `similar_failures` (or `search_memory`) to check whether this
   failure has been seen before, and whether a later run resolved it.
3. Once it figures out *why* it failed or *how* the fix worked, it calls
   `remember` to record that conclusion.
4. Next time the same error shows up — in a later session, or on a teammate's
   machine that imported the store — `similar_failures`/`search_memory` surface
   the fix instead of the agent re-deriving it from scratch.

Whether having the memory in hand actually changes whether an agent solves a
failure is measured in
[benchmarks/agent_eval/AGENT_EVAL.md](../../benchmarks/agent_eval/AGENT_EVAL.md);
whether the right memory is retrievable in the first place is measured by the
retrieval benchmarks in [benchmarks/BENCHMARKS.md](../../benchmarks/BENCHMARKS.md).
