# Codex CLI + MemoryWhale

OpenAI's Codex CLI supports MCP servers via TOML config, so it can use
MemoryWhale's six local memory tools.

## Status

Verified against OpenAI's [Codex MCP](https://developers.openai.com/codex/mcp)
documentation on 2026-08-29. User config is `~/.codex/config.toml`. Trusted
projects may also use `.codex/config.toml`. Each server is a
`[mcp_servers.<name>]` table with `command` for stdio. The TUI lists servers
with `/mcp`.

## Requirements

- MemoryWhale installed with `mw-mcp` on `PATH`.
- Codex CLI (or the Codex IDE extension, which opens the same `config.toml`).

## Setup

Add the block from [`config.toml`](config.toml) to `~/.codex/config.toml`:

```toml
[mcp_servers.memorywhale]
command = "mw-mcp"
```

`mw-mcp` must be on `PATH` (standard MemoryWhale install); otherwise use its
absolute path. For a non-default database add
`env = { MEMORYWHALE_DATA_DIR = "/path/to/dir" }`. The top-level key is
`mcp_servers`, not `mcpServers`.

### Memory-use guidance

Codex reads an `AGENTS.md` at your repo root. Add:

> When a build/test/deploy fails, before proposing a fix, use `search_memory`
> or `recent_errors` (MemoryWhale MCP) to check whether this failure has a known
> cause or a saved lesson. Once you've figured out why something failed or how a
> fix worked, use `remember` to save that conclusion.

### Command capture and proactive recall (optional)

```bash
mw integrate codex --capture            # add the hook to ~/.codex/hooks.json
mw integrate codex --capture --check    # confirm it is installed
mw integrate codex --capture --revert   # remove only what MemoryWhale added
```

This adds one `PostToolUse` hook for Codex's `Bash` tool
([Codex hooks](https://developers.openai.com/codex/hooks)). It runs
`mw-remember --from-hook codex`, which records each shell command Codex runs
with its working directory and output. Your other hooks are kept; `--dry-run`
shows the change without writing, and `--hooks-file PATH` edits another file.

When the output contains an error that a later command fixed before, Codex
gets one line in its context naming that fix, the same note Claude Code and
Cursor get. Set `MEMORYWHALE_HOOK_FEEDBACK=0` to record without it.

Limits, checked against Codex's source on 2026-10-04:

- Codex hands the hook the command's combined output as one string with no
  exit code (its docs describe separate `exit_code`, `stdout`, and `stderr`;
  both shapes are accepted). Codex rows are stored with an unknown exit, so
  they do not appear in `recent_errors` or `mw context --last-error`, and the
  "passed after failing, save the lesson" note does not fire for Codex.
- Commands still running in a background session when the tool call returns
  do not reach the hook.
- Output is truncated by Codex before the hook sees it, then secret-scrubbed
  by MemoryWhale before it is written.

## Verify

```bash
command -v mw-mcp
mw integrate codex --capture --check
```

In the Codex TUI, run `/mcp` and confirm `memorywhale` is active. You should
see the six tools: `recent_errors`, `search_memory`, `get_context`, `remember`,
`similar_failures`, and `stats`.

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes |
| Automatic execution capture | Yes, opt-in (`--capture`); exit codes unknown |
| Proactive recall | Yes, with capture |
| Memory-use guidance | Yes, via `AGENTS.md` |

MCP access alone is not execution capture; that needs `--capture`. Without the MCP server, the
`mw` CLI still works: `mw context --last-error`, `mw search "…"`,
`mw remember "…"`.

## Example prompt

> Use MemoryWhale to check whether I encountered a similar failure before.

## Troubleshooting

- Run `command -v mw-mcp` from the environment that launches Codex.
- Confirm you edited `[mcp_servers.memorywhale]`, not an `mcpServers` JSON
  block from another client.
- Restart Codex after editing `config.toml`.
- If the wrong database opens, set `MEMORYWHALE_DATA_DIR` in `env`.
- Run `mw doctor` to check the MemoryWhale install.

## Uninstall

Run `mw integrate codex --capture --revert` if you installed capture. Delete
the `[mcp_servers.memorywhale]` table from `~/.codex/config.toml` (and
any project `.codex/config.toml`). Remove the `AGENTS.md` instruction if you
added one. This does not delete the MemoryWhale database.
