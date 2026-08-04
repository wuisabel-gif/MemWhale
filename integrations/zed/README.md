# Zed integration

Zed supports custom MCP servers, so its Agent Panel can use MemoryWhale's four
tools: `recent_errors`, `search_memory`, `get_context`, and `remember`.

## Connect the MCP server

Open **Settings → AI → MCP Servers** and choose **Add Server → Add Local
Server**, or run `agent: open settings` and open the JSON settings file. Merge
the [`settings.json`](settings.json) example into your existing settings:

```json
{
  "context_servers": {
    "memorywhale": {
      "command": "mw-mcp",
      "args": [],
      "env": {}
    }
  }
}
```

`mw-mcp` must be on `PATH` (standard MemoryWhale install); otherwise use its
absolute path. For a non-default database, add
`"env": { "MEMORYWHALE_DATA_DIR": "/path/to/dir" }`.

After saving, open the Agent Panel settings and confirm that `memorywhale` has
a green running indicator. Zed's canonical custom-server documentation is
[Model Context Protocol](https://zed.dev/docs/ai/mcp).

## Tell the agent when to reach for memory

Add this to your project or user rules:

> When a build, test, or deploy fails, use `search_memory` or `recent_errors`
> before proposing a fix. After finding the cause or a working fix, use
> `remember` to save the conclusion.

Without the MCP server, the CLI remains available: `mw context --last-error`,
`mw search "…"`, and `mw remember "…"`.
