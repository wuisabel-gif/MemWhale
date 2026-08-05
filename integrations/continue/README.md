# Continue integration

[Continue](https://github.com/continuedev/continue) (the open-source AI code
assistant for VS Code and JetBrains) connects to MCP servers, so it gets
MemoryWhale's four tools: `recent_errors`, `search_memory`, `get_context`,
`remember`.

## Connect the MCP server

Continue reads MCP servers from YAML. Add the `mcpServers` block from
[`config.yaml`](config.yaml) to your global config at `~/.continue/config.yaml`:

```yaml
mcpServers:
  - name: memorywhale
    command: mw-mcp
    args: []
```

`mcpServers` entries are **list items** — the leading `-` matters, and mixing
tabs with spaces silently breaks YAML parsing. If you already have an
`mcpServers:` list, add `memorywhale` as another item rather than replacing it.

Alternatively, drop a standalone server file into `.continue/mcpServers/` inside
a project.

`mw-mcp` must be on `PATH` (the standard MemoryWhale install); otherwise use its
absolute path as `command`. For a non-default database add an `env` map:
`env: { MEMORYWHALE_DATA_DIR: "/path/to/dir" }`.

Continue does not always pick up MCP edits live — run **Reload Window** (VS Code)
or reload the extension so it re-reads the config.

## Tell it when to reach for the memory

The MCP server gives Continue the *tools*; a rule (Continue's rules / system
message) teaches it *when*:

> When a build/test/deploy fails, before proposing a fix, use `search_memory`
> or `recent_errors` (MemoryWhale MCP) to check whether this failure has a known
> cause or a saved lesson. Once you've figured out why something failed or how a
> fix worked, use `remember` to save that conclusion.

## Without the MCP server

The `mw` CLI still works: `mw context --last-error`, `mw search "…"`,
`mw remember "…"`.
