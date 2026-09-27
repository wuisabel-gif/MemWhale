# Qwen Code + MemoryWhale

## Status

Configuration and boundaries were checked against Qwen Code's
[MCP documentation](https://github.com/QwenLM/qwen-code/blob/main/docs/users/features/mcp.md)
at commit
[`4ddbf227e898ceb322116af5e2004c37a51184f6`](https://github.com/QwenLM/qwen-code/blob/4ddbf227e898ceb322116af5e2004c37a51184f6/docs/users/features/mcp.md)
(the current revision of that file), inspected on September 26, 2026. The
contract pinned in issue #295 (revision `b8def02aadfc384ecb860909155d196267c4fa0c`,
September 16, 2026) was re-verified against this revision.

Qwen Code documents local stdio MCP servers in an `mcpServers` object inside
`settings.json`, at user and project scope, with `command`/`args`/`cwd`/`env`,
per-tool allow/deny lists, timeouts, and a `trust` flag. Servers are also
managed with the `qwen mcp` command. This MemoryWhale integration is MCP-first;
native Qwen Code execution capture is not implemented or claimed. The source
contract was inspected, but a live provider-backed Qwen Code session is not part
of the repository checks.

## Requirements

- Qwen Code CLI from a pinned, reviewed release.
- MemoryWhale `mw-mcp` on an absolute path and an explicitly selected local
  store. Build the helpers from the MemoryWhale repository root with
  `cargo build --release -p memorywhale-cli --bins`.
- macOS or Linux for the documented local stdio path. Windows behavior is not
  verified here. On macOS, re-sign a copied binary (`codesign --force --sign -
  <path>`) or it is killed on launch.
- A configured Qwen Code model for normal use. The isolated transport checks use
  synthetic data and do not read provider credentials.

## Setup

Qwen Code reads MCP configuration from the user file `~/.qwen/settings.json`
(applies to all projects) and the project file `.qwen/settings.json` (project
root, takes precedence). Project configuration is appropriate only in a trusted
repository because it launches a local command when discovery runs. Choose one
scope; do not duplicate the server in both files.

Add this object to the existing `mcpServers` object rather than replacing it:

```json
{
  "mcpServers": {
    "memorywhale": {
      "command": "/absolute/path/to/mw-mcp",
      "args": [],
      "cwd": "/absolute/path/to/your/project",
      "env": {
        "MEMORYWHALE_DATA_DIR": "/absolute/path/to/a-new-memorywhale-store"
      },
      "includeTools": [
        "recent_errors",
        "search_memory",
        "get_context",
        "remember",
        "similar_failures",
        "stats"
      ],
      "timeout": 120000,
      "trust": false
    }
  }
}
```

The example is also available as
[`settings.example.json`](settings.example.json). Use an absolute `command`
path and an absolute store path, and review every command/env value before
trusting a project folder. Keep the six-tool `includeTools` allowlist rather
than granting unrelated servers; do not set `trust: true` to skip approvals for
`remember`. Qwen Code namespaces MCP tools by server, so expect names such as
`memorywhale__search_memory` in tool listings and model requests.

Instead of editing JSON by hand you can register the server with
`qwen mcp add`, for example:

```bash
qwen mcp add --scope project \
  --env MEMORYWHALE_DATA_DIR=/absolute/path/to/a-new-memorywhale-store \
  --include-tools recent_errors,search_memory,get_context,remember,similar_failures,stats \
  memorywhale /absolute/path/to/mw-mcp
```

Qwen Code discovers servers when a session starts. In interactive mode the
status pill shows `N/M MCP servers ready`; in non-interactive mode (`--prompt`,
stream-json, ACP) it waits for discovery to finish before the first turn.
Restart the session or re-run discovery after changing configuration; an
existing session does not necessarily register a newly added server.

### Memory-use guidance

A `QWEN.md` instruction (in the project or `~/.qwen/`) tells the CLI when to use
the tools:

> When a build/test/deploy fails, before proposing a fix, use `search_memory`
> or `recent_errors` (MemoryWhale MCP) to check whether this failure has a known
> cause or a saved lesson. Once you have figured out why something failed or how
> a fix worked, use `remember` to save that conclusion. Do not save a lesson
> unless asked.

## Verify

First run the repository's direct transport check against the built `mw-mcp`
with an isolated store, so the check never touches your real memory:

```bash
STORE="$(mktemp -d)"
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}' \
  | MEMORYWHALE_DATA_DIR="$STORE" /absolute/path/to/mw-mcp
```

The response reports `serverInfo` `memorywhale` and a `tools` capability. See
[VERIFICATION.md](VERIFICATION.md) for the recorded discovery and `tools/list`
output.

For native validation, use a fresh Qwen Code profile/workspace and a new
synthetic store. Run `qwen mcp list` and confirm `memorywhale` is connected with
all six allowlisted tools. Ask the client to search for a known synthetic marker,
then explicitly ask it to save a separate proposed lesson. Confirm the proposed
note remains pending in MemoryWhale's ordinary review flow; a save
acknowledgement is not approval or proof of correctness. Restart Qwen Code and
repeat the read. Then run `qwen mcp remove memorywhale`, start a new session, and
confirm Qwen Code still runs without removing either store.

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes, configuration and transport verified; native client call pending in this repository PR |
| Automatic execution capture | No; Qwen Code has no verified receipt contract here |
| Memory-use guidance | Optional `QWEN.md` instruction; native loading pending |

The six MCP tools are `recent_errors`, `search_memory`, `get_context`,
`remember`, `similar_failures`, and `stats`. MemoryWhale remains a separate store
from Qwen Code sessions and any hosted Qwen service.

## Example prompt

> Search MemoryWhale for this exact compiler error. Report only recorded
> evidence, separate observations from hypotheses, and do not save a lesson
> unless I ask.

For an explicit write:

> Save this proposed debugging lesson to MemoryWhale, including the failed
> command, the proposed fix, the verification command, and unresolved
> assumptions. Do not treat it as verified until I approve it in MemoryWhale.

Retrieved excerpts can enter the configured model context. Local stdio and a
local database do not make model inference offline.

## Troubleshooting

- If Qwen Code reports no MCP tools, verify the selected scope
  (`~/.qwen/settings.json` vs `.qwen/settings.json`), the JSON object shape, the
  absolute executable path, executable permissions, and that the session was
  restarted or discovery re-run after configuration changes.
- Run `qwen mcp list` and read the reported status. A listing shows connection
  state, not that a tool call or a MemoryWhale write succeeded.
- If the server connects but calls fail, run the direct `mw-mcp` discovery check
  above, inspect the namespaced tool name, and verify the child `env` points to
  the intended store.
- Keep normal approvals for `remember`; do not set `trust: true` or a broad
  allowlist to hide an unreviewed write.
- If discovery times out, raise `timeout` (default 600000ms) or
  `discoveryTimeoutMs`, and confirm `mw-mcp` starts on its own.
- Project-level stdio servers run local commands when a trusted folder opens. Do
  not place this configuration in an untrusted repository.

## Uninstall

Remove only the `memorywhale` object from the selected `settings.json` (or run
`qwen mcp remove memorywhale`) and restart Qwen Code. Remove any `QWEN.md`
instruction you added. Do not delete the MemoryWhale database, Qwen Code
sessions, or unrelated MCP entries.
