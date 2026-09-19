# Kimi Code CLI + MemoryWhale

## Status

Configuration and boundaries were checked against Kimi Code source revision
[`bd06178913d2cc4ebd229ecee6714081105e5ae1`](https://github.com/MoonshotAI/kimi-code/tree/bd06178913d2cc4ebd229ecee6714081105e5ae1)
on September 16, 2026. Kimi Code documents local stdio MCP servers in
`mcp.json`, project and user scopes, `env`, tool allow/deny lists, and normal
approval rules. This MemoryWhale integration is MCP-first; native Kimi Code
execution capture is not implemented or claimed.

The older `MoonshotAI/kimi-cli` project is winding down into Kimi Code. This
guide targets Kimi Code only. The source contract was inspected, but a live
provider-backed Kimi Code session is not part of the repository checks.

## Requirements

- Kimi Code CLI from a pinned, reviewed release or source revision.
- MemoryWhale `mw-mcp` on an absolute path and an explicitly selected local
  store. Build matching helpers from the MemoryWhale repository root.
- macOS or Linux for the documented local stdio path. Windows behavior is not
  verified here.
- A configured Kimi Code model for normal use. The isolated transport checks use
  synthetic data and do not read provider credentials.

## Setup

Kimi Code reads MCP configuration from the user file
`$KIMI_CODE_HOME/mcp.json` (default `~/.kimi-code/mcp.json`) and the project file
`.kimi-code/mcp.json`. Project configuration is appropriate only in a trusted
repository because it launches a local command when the session starts. Choose
one scope; do not duplicate the server in both files.

Add this object to the existing `mcpServers` object:

```json
{
  "mcpServers": {
    "memorywhale": {
      "command": "/absolute/path/to/mw-mcp",
      "args": [],
      "env": {
        "MEMORYWHALE_DATA_DIR": "/absolute/path/to/a-new-memorywhale-store"
      },
      "enabledTools": [
        "recent_errors",
        "search_memory",
        "get_context",
        "remember",
        "similar_failures",
        "stats"
      ],
      "startupTimeoutMs": 30000,
      "toolTimeoutMs": 120000
    }
  }
}
```

The example is also available as [`mcp.example.json`](mcp.example.json). Use
an absolute store path and review every command/env value before trusting a
project folder. Keep the six-tool allowlist rather than granting unrelated
servers or wildcard permissions. Kimi Code names tools with its MCP namespace;
expect names such as `mcp__memorywhale__search_memory` in approvals and model
requests.

Kimi Code also supports `/mcp-config` and `/mcp`. These are useful to inspect
connection status, but a successful listing is not proof that a tool call or
MemoryWhale write succeeded. Restart the session after changing project/user
configuration; an existing session does not necessarily register a newly added
server.

### Optional guidance

Copy the reviewed [`SKILL.md`](skills/memorywhale-debugging/SKILL.md) into an
unused `.kimi-code/skills/memorywhale-debugging/` directory for a project, or
into the Kimi Code user skills directory documented by the selected release.
Do not replace `AGENTS.md`, native preferences, or another skill. The guidance
never grants permission, approves tools, or changes MemoryWhale review policy.

## Verify

First run the repository's transport and configuration checks:

```bash
python3 scripts/test-kimi-code-integration.py
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}' \
  | mw-mcp
```

For native validation, use a fresh Kimi Code profile/workspace and a new
synthetic store. In `/mcp`, confirm `memorywhale` is connected and all six
allowlisted tools are present. Ask the client to search for a known approved
synthetic marker, then explicitly ask it to save a separate proposed lesson.
Confirm that the proposed agent note remains pending in MemoryWhale's ordinary
review flow; a save acknowledgement is not approval or proof of correctness.
Restart Kimi Code and repeat the read. Remove only the `memorywhale` entry,
start a new session, and confirm Kimi Code still runs without removing either
store. Record native, headless, or ACP results separately; this guide does not
turn one surface into certification of the others.

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes, configuration and transport verified; native client call pending in this repository PR |
| Automatic execution capture | No; Kimi hooks are not a verified receipt contract here |
| Memory-use guidance | Optional skill, asset-checked; native skill loading pending |

The six MCP tools are `recent_errors`, `search_memory`, `get_context`,
`remember`, `similar_failures`, and `stats`. MemoryWhale remains a separate
store from Kimi Code sessions and any hosted Kimi service.

## Example prompt

> Search MemoryWhale for this exact compiler error. Report only recorded evidence,
> separate observations from hypotheses, and do not save a lesson unless I ask.

For an explicit write:

> Save this proposed debugging lesson to MemoryWhale, including the failed
> command, the proposed fix, the verification command, and unresolved assumptions.
> Do not treat it as verified until I approve it in MemoryWhale.

Retrieved excerpts can enter the configured model context. Local stdio and a
local database do not make model inference offline.

## Troubleshooting

- If Kimi Code reports no MCP tools, verify the selected `KIMI_CODE_HOME`, JSON
  object shape, absolute executable path, executable permissions, and that the
  session was restarted after configuration changes.
- If the server connects but calls fail, run the direct `mw-mcp` discovery check,
  inspect the tool name in `/mcp`, and verify the child `env` points to the
  intended store.
- Keep normal approvals for `remember`; do not use a broad `mcp__*` allow rule
  to hide an unreviewed write.
- If a call response is lost, treat the write outcome as unknown and inspect the
  store before retrying. Do not assume a reconnect makes `remember` idempotent.
- Project-level stdio servers run local commands when a trusted folder opens.
  Do not place this configuration in an untrusted repository.

## Uninstall

Remove only the `memorywhale` object from the selected `mcp.json` and restart
Kimi Code. Remove the optional skill only if it is the copy installed for this
integration. Do not delete the MemoryWhale database, Kimi Code sessions, or
unrelated MCP entries.
