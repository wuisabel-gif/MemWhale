# Cline integration

[Cline](https://github.com/cline/cline) (the autonomous coding agent for VS Code)
supports MCP servers, so it gets MemoryWhale's four tools: `recent_errors`,
`search_memory`, `get_context`, `remember`.

## Connect the MCP server

In VS Code, click the **Cline** icon → open the menu (top-right of the Cline
panel) → **MCP Servers** → **Configure MCP Servers**. That opens Cline's
`cline_mcp_settings.json`. Add the `memorywhale` entry from
[`mcp_settings.json`](mcp_settings.json):

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
other servers rather than replacing it.

The file lives under VS Code's global storage (path may vary by OS/version):

```
~/Library/Application Support/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json   # macOS
~/.config/Code/User/globalStorage/saoudrizwan.claude-dev/settings/cline_mcp_settings.json                       # Linux
%APPDATA%\Code\User\globalStorage\saoudrizwan.claude-dev\settings\cline_mcp_settings.json                        # Windows
```

Prefer editing through the Cline UI (above) — it points at the right file.
`mw-mcp` must be on `PATH` (the standard MemoryWhale install); otherwise use its
absolute path as `command`. For a non-default database add
`"env": { "MEMORYWHALE_DATA_DIR": "/path/to/dir" }`.

## Tell it when to reach for the memory

The MCP server gives Cline the *tools*; a rule teaches it *when*. Add to a
`.clinerules` file (or Cline's custom instructions):

> When a build/test/deploy fails, before proposing a fix, use `search_memory`
> or `recent_errors` (MemoryWhale MCP) to check whether this failure has a known
> cause or a saved lesson. Once you've figured out why something failed or how a
> fix worked, use `remember` to save that conclusion.

## Without the MCP server

The `mw` CLI still works: `mw context --last-error`, `mw search "…"`,
`mw remember "…"`.
