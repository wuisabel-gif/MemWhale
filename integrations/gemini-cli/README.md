# Gemini CLI integration

Google's [Gemini CLI](https://github.com/google-gemini/gemini-cli) is a terminal
AI agent that connects to MCP servers — so it gets MemoryWhale's four tools:
`recent_errors`, `search_memory`, `get_context`, `remember`. It's a
terminal/dev-loop agent, the same world MemoryWhale's memory is about.

## Connect the MCP server

Add the `memorywhale` entry from [`settings.json`](settings.json) to your Gemini
CLI settings:

- **Every project:** `~/.gemini/settings.json`
- **This project only:** `.gemini/settings.json` in the project root

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

Verify inside Gemini CLI with the `/mcp` command — `memorywhale` should list its
four tools.

## Tell it when to reach for the memory

The MCP server gives Gemini CLI the *tools*; a `GEMINI.md` instruction (in your
project or `~/.gemini/`) teaches it *when*:

> When a build/test/deploy fails, before proposing a fix, use `search_memory`
> or `recent_errors` (MemoryWhale MCP) to check whether this failure has a known
> cause or a saved lesson. Once you've figured out why something failed or how a
> fix worked, use `remember` to save that conclusion.

## Without the MCP server

The `mw` CLI still works: `mw context --last-error`, `mw search "…"`,
`mw remember "…"`.
