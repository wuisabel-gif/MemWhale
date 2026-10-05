# Gemini CLI + MemoryWhale

Google's [Gemini CLI](https://github.com/google-gemini/gemini-cli) connects to
MCP servers, so it can use MemoryWhale's six local memory tools.

## Status

Verified against Gemini CLI's
[MCP setup tutorial](https://geminicli.com/docs/cli/tutorials/mcp-setup/)
and the project's
[MCP server docs](https://github.com/google-gemini/gemini-cli/blob/main/docs/tools/mcp-server.md)
on 2026-08-29. Servers go in `mcpServers` inside `~/.gemini/settings.json` or
`.gemini/settings.json`. Stdio servers use `command`. `/mcp list` shows
connection status.

## Requirements

- MemoryWhale installed with `mw-mcp` on `PATH`.
- Gemini CLI installed.

## Setup

```bash
mw integrate gemini            # add MemoryWhale MCP to ~/.gemini/settings.json
mw integrate gemini --check    # confirm it is installed
```

`--dry-run` shows the change without writing, `--revert` removes only what
MemoryWhale added, and `--config PATH` edits another settings file (such as a
project's `.gemini/settings.json`). Your other settings and MCP servers are
kept.

To set it up by hand instead, add the `memorywhale` entry from
[`settings.json`](settings.json) to your Gemini CLI settings:

- Every project: `~/.gemini/settings.json`
- This project only: `.gemini/settings.json` in the project root

```json
{
  "mcpServers": {
    "memorywhale": {
      "command": "mw-mcp"
    }
  }
}
```

If the file already has an `mcpServers` object, add `memorywhale` alongside your
other servers rather than replacing it. `mw-mcp` must be on `PATH` (the standard
MemoryWhale install); otherwise use its absolute path as `command`. For a
non-default database add `"env": { "MEMORYWHALE_DATA_DIR": "/path/to/dir" }`.

### Memory-use guidance

A `GEMINI.md` instruction (in your project or `~/.gemini/`) teaches the CLI
when to use the tools:

> When a build/test/deploy fails, before proposing a fix, use `search_memory`
> or `recent_errors` (MemoryWhale MCP) to check whether this failure has a known
> cause or a saved lesson. Once you've figured out why something failed or how a
> fix worked, use `remember` to save that conclusion.

### Command capture and proactive recall (optional)

```bash
mw integrate gemini --capture            # add the hook to ~/.gemini/settings.json
mw integrate gemini --capture --check    # confirm it is installed
mw integrate gemini --capture --revert   # remove only what MemoryWhale added
```

This adds one `AfterTool` hook for Gemini's `run_shell_command` tool
([Gemini CLI hooks](https://geminicli.com/docs/hooks/)). It runs
`mw-remember --from-hook gemini`, which records each shell command Gemini runs
with its working directory, output, and exit code, secret-scrubbed before it is
written and searchable with `agent:gemini`.

Gemini reports the exit code in the tool result (an `Exit Code:` line only when
it is not zero), so Gemini rows count as failures in `recent_errors` and
`mw context --last-error` like terminal capture. Spawn errors, signals,
cancellations, and timeouts are stored with an unknown exit.

When a command fails with an error that a later command fixed before, Gemini
gets one line in its context naming that fix, and when a command passes right
after failing it is asked to save the lesson, the same notes Claude Code gets.
Set `MEMORYWHALE_HOOK_FEEDBACK=0` to record without them. Gemini may warn
before running a new or changed hook; approve it once.

## Verify

```bash
command -v mw-mcp
mw integrate gemini --check
mw integrate gemini --capture --check
```

Restart Gemini CLI and run `/mcp list`. `memorywhale` should show as connected
with the six tools: `recent_errors`, `search_memory`, `get_context`,
`remember`, `similar_failures`, and `stats`.

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes |
| Automatic execution capture | Yes, opt-in (`--capture`), with exit codes |
| Proactive recall | Yes, with capture |
| Memory-use guidance | Yes, via `GEMINI.md` |

MCP access is not automatic execution capture. Without the MCP server, the
`mw` CLI still works: `mw context --last-error`, `mw search "…"`,
`mw remember "…"`.

## Example prompt

> Use MemoryWhale to check whether I encountered a similar failure before.

## Troubleshooting

- Run `command -v mw-mcp` from the environment that launches Gemini CLI.
- Confirm `mcpServers` is valid JSON and restart the CLI.
- If `/mcp list` shows Disconnected, run `gemini mcp list` and read the
  reported error. Check `PATH`, the `command` and `args` in settings, and
  whether `mw-mcp` starts on its own. A missing binary is only one cause.
- If the wrong database opens, set `MEMORYWHALE_DATA_DIR` in `env`.
- Run `mw doctor` to check the MemoryWhale install.

## Uninstall

Run `mw integrate gemini --capture --revert` and `mw integrate gemini --revert`,
or delete the `memorywhale` entry from `mcpServers` in
`~/.gemini/settings.json` and/or `.gemini/settings.json`. Remove any
`GEMINI.md` instruction you added. Restart Gemini CLI. This does not delete
the MemoryWhale database.
