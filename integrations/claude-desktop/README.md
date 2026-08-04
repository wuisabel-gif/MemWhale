# Claude Desktop integration

[Claude Desktop](https://claude.ai/download) is Anthropic's desktop app and the
reference MCP host. Point it at MemoryWhale's MCP server and it gets the four
tools: `recent_errors`, `search_memory`, `get_context`, `remember`.

(This is the desktop app. For the Claude Code CLI, see
[`../claude-code/`](../claude-code/).)

## Connect the MCP server

Edit Claude Desktop's config file:

- **macOS:** `~/Library/Application Support/Claude/claude_desktop_config.json`
- **Windows:** `%APPDATA%\Claude\claude_desktop_config.json`

Add the `memorywhale` entry from
[`claude_desktop_config.json`](claude_desktop_config.json):

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
other servers rather than replacing it. Then **fully restart Claude Desktop**
(quit and reopen) — it reads MCP config only at launch.

`mw-mcp` must be on `PATH` (the standard MemoryWhale install); otherwise use its
absolute path as `command`. For a non-default database add
`"env": { "MEMORYWHALE_DATA_DIR": "/path/to/dir" }`. After restart, the tools
appear under the 🔌/tools menu in the composer.

## Without the MCP server

The `mw` CLI still works: `mw context --last-error`, `mw search "…"`,
`mw remember "…"`.
