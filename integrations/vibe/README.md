# Mistral Vibe + MemoryWhale

## Status

Configuration was checked against the Mistral Vibe README at revision
[`7c19608af06f6c61d63f8f7a5c3430da73fba2ab`](https://github.com/mistralai/mistral-vibe/blob/7c19608af06f6c61d63f8f7a5c3430da73fba2ab/README.md)
on September 26, 2026. Vibe documents local stdio MCP servers in `config.toml`
using `[[mcp_servers]]` blocks with `transport = "stdio"`, `command`, `args`,
`env`, per-server `startup_timeout_sec`/`tool_timeout_sec`, and per-tool
`[tools.{server}_{tool}]` permissions. This MemoryWhale integration is
MCP-first; native Vibe execution capture is not implemented or claimed.

The MemoryWhale transport was exercised directly (see
[`VERIFICATION.md`](VERIFICATION.md)). A live Vibe session was not run in this
repository, so native `/mcp` connection and in-session tool calls remain
pending.

## Requirements

- Mistral Vibe from a pinned, reviewed release.
- MemoryWhale `mw-mcp` on an absolute path and an explicitly selected local
  store. Build the helpers from the MemoryWhale repository root with
  `cargo build --release -p memorywhale-cli --bins`.
- macOS or Linux for the documented local stdio path. Windows behavior is not
  verified here.
- A configured Vibe model/provider for normal use. The isolated transport check
  uses synthetic data and does not read provider credentials.

## Setup

Vibe reads `config.toml` first from `./.vibe/config.toml` (project) and then
falls back to `~/.vibe/config.toml` (user). Project configuration launches a
local command when a trusted session starts, so use it only in a trusted
repository. Choose one scope; do not duplicate the server in both files.

Add the following to the selected `config.toml`:

```toml
[[mcp_servers]]
name = "memorywhale"
transport = "stdio"
command = "/absolute/path/to/mw-mcp"
args = []
env = { "MEMORYWHALE_DATA_DIR" = "/absolute/path/to/a-new-memorywhale-store" }
startup_timeout_sec = 30
tool_timeout_sec = 120

[tools.memorywhale_recent_errors]
permission = "always"

[tools.memorywhale_search_memory]
permission = "always"

[tools.memorywhale_get_context]
permission = "always"

[tools.memorywhale_remember]
permission = "ask"

[tools.memorywhale_similar_failures]
permission = "always"

[tools.memorywhale_stats]
permission = "always"
```

The example is also available as
[`mcp_servers.example.toml`](mcp_servers.example.toml). Use an absolute store
path and review every command/env value before trusting a project folder. Vibe
names MCP tools `{server_name}_{tool_name}`, so the six MemoryWhale tools appear
as `memorywhale_recent_errors`, `memorywhale_search_memory`,
`memorywhale_get_context`, `memorywhale_remember`,
`memorywhale_similar_failures`, and `memorywhale_stats`. Keep the reviewed write
(`memorywhale_remember`) on `permission = "ask"` rather than `"always"`.

Restart the Vibe session after changing configuration; an existing session does
not necessarily register a newly added server.

### Optional guidance

Copy the reviewed [`SKILL.md`](skills/memorywhale-debugging/SKILL.md) into an
unused `.vibe/skills/memorywhale-debugging/` directory for a project, or into
`~/.vibe/skills/memorywhale-debugging/` for all projects. Vibe follows the
[Agent Skills specification](https://agentskills.io/specification) and also
discovers `.agents/skills/` and `~/.agents/skills/`. Do not replace `AGENTS.md`,
native preferences, or another skill. The guidance never grants permission,
approves tools, or changes MemoryWhale review policy.

## Verify

First run the direct MemoryWhale transport check with an isolated store:

```bash
D="$(mktemp -d)/mw-store"; mkdir -p "$D"
printf '%s\n%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2026-07-28","clientInfo":{"name":"vibe-verify","version":"1"},"capabilities":{}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | MEMORYWHALE_DATA_DIR="$D" /absolute/path/to/mw-mcp
```

The `initialize` reply reports `serverInfo` name `memorywhale`, and `tools/list`
returns all six tools. This verifies MemoryWhale directly, not Vibe's client
registry.

For native validation, use a fresh Vibe trusted workspace and a new synthetic
store. Run `/mcp` and confirm `memorywhale` is connected with all six tools
present. Ask the client to search for a known approved synthetic marker, then
explicitly ask it to save a separate proposed lesson; confirm the proposed note
remains pending in MemoryWhale's ordinary review flow. A save acknowledgement is
not approval or proof of correctness. Restart Vibe and repeat the read. Remove
only the `memorywhale` entry, start a new session, and confirm Vibe still runs
without removing either store.

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes, configuration and transport verified; native client call pending in this repository PR |
| Automatic execution capture | No; Vibe hooks are not a verified receipt contract here |
| Memory-use guidance | Optional skill, asset-checked; native skill loading pending |

The six MCP tools are `recent_errors`, `search_memory`, `get_context`,
`remember`, `similar_failures`, and `stats` (exposed to Vibe as
`memorywhale_*`). MemoryWhale remains a separate store from Vibe sessions and
any hosted Mistral service.

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

- If Vibe reports no MCP tools, verify the `config.toml` location
  (`./.vibe/config.toml` vs `~/.vibe/config.toml`), the `[[mcp_servers]]` block
  shape, the absolute executable path, executable permissions, that the folder
  is trusted, and that the session was restarted after configuration changes.
- If the server connects but calls fail, run the direct `mw-mcp` handshake
  above, inspect the tool name in `/mcp`, and verify the child `env` points to
  the intended store.
- Keep `memorywhale_remember` on `permission = "ask"`; do not use a broad allow
  rule to hide an unreviewed write.
- If a call response is lost, treat the write outcome as unknown and inspect the
  store before retrying. Do not assume a reconnect makes `remember` idempotent.
- Project-level stdio servers run local commands when a trusted folder opens. Do
  not place this configuration in an untrusted repository.

## Uninstall

Remove only the `memorywhale` `[[mcp_servers]]` block and its
`[tools.memorywhale_*]` entries from the selected `config.toml`, then restart
Vibe. Remove the optional skill only if it is the copy installed for this
integration. Do not delete the MemoryWhale database, Vibe sessions, or unrelated
MCP entries.
