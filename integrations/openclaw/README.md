# OpenClaw integration

[OpenClaw](https://github.com/openclaw/openclaw) agents connect to MCP servers,
so they get MemoryWhale's four tools: `recent_errors`, `search_memory`,
`get_context`, `remember`.

## Connect the MCP server

`mw-mcp` is a Model Context Protocol server over stdio. Register it with the CLI:

```bash
openclaw mcp add memorywhale --command mw-mcp
openclaw mcp doctor memorywhale --probe
```

The probe matters — saving a definition proves nothing about reachability.

Or write it straight into config. Merge the block from
[`openclaw.mcp.json5`](openclaw.mcp.json5) into `~/.openclaw/openclaw.json`
(OpenClaw accepts JSON5):

```json5
{
  mcp: {
    servers: {
      memorywhale: { command: "mw-mcp", enabled: true },
    },
  },
}
```

`mw-mcp` must be on `PATH` (the standard MemoryWhale install); otherwise use its
absolute path as `command`. For a non-default database add
`env: { MEMORYWHALE_DATA_DIR: "/path/to/dir" }`. A running Gateway may need a
restart or runtime reload before it picks up a config-file change.

## Tell it when to reach for the memory

The MCP server gives OpenClaw the *tools*; a workspace instruction teaches it
*when* to reach for them. OpenClaw agents read an `AGENTS.md` from their
workspace (default `~/.openclaw/workspace/AGENTS.md`). Add:

> When a build/test/deploy fails, before proposing a fix, use `search_memory`
> or `recent_errors` (MemoryWhale MCP) to check whether this failure has a known
> cause or a saved lesson. Once you've figured out why something failed or how a
> fix worked, use `remember` to save that conclusion.

## Without the MCP server

The `mw` CLI still works: `mw context --last-error`, `mw search "…"`,
`mw remember "…"`. And with no integration at all, `mw ask` packages the last
failure onto your clipboard for any chat.
