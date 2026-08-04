# CrowClaw integration

[CrowClaw](https://github.com/subinium/CrowClaw) is a TypeScript agent runtime,
and its agents connect to MCP servers — so it gets MemoryWhale's four tools:
`recent_errors`, `search_memory`, `get_context`, `remember`.

## Connect the MCP server

CrowClaw supports custom (non-preset) MCP servers. Add MemoryWhale as one.

**From the dashboard:** open the CrowClaw dashboard → **MCP** → **Add server**,
and enter:

| Field | Value |
| --- | --- |
| Name | `memorywhale` |
| Command | `mw-mcp` |
| Args | *(none)* |

**Or via the REST API** the dashboard uses — post the block from
[`server.json`](server.json) to `/api/mcp/servers`:

```json
{
  "name": "memorywhale",
  "command": "mw-mcp",
  "args": [],
  "custom": true
}
```

Either way the definition is persisted under `~/.crowclaw/`. `mw-mcp` must be on
`PATH` (the standard MemoryWhale install); otherwise use its absolute path as
`command`. For a non-default database add
`"env": { "MEMORYWHALE_DATA_DIR": "/path/to/dir" }`.

Verify the server answers and its tools were discovered:

```
GET /api/mcp/servers/memorywhale/tools
```

(or use the dashboard's per-server reconnect/tools view). You should see the
four tools listed.

## Tell it when to reach for the memory

The MCP server gives CrowClaw the *tools*; a preset's `systemPromptAppend` (or
your agent's system prompt) teaches it *when* to reach for them:

> When a build/test/deploy fails, before proposing a fix, use `search_memory`
> or `recent_errors` (MemoryWhale MCP) to check whether this failure has a known
> cause or a saved lesson. Once you've figured out why something failed or how a
> fix worked, use `remember` to save that conclusion.

## Without the MCP server

The `mw` CLI still works: `mw context --last-error`, `mw search "…"`,
`mw remember "…"`.
