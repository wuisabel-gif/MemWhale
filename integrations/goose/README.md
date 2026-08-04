# Goose integration

[Goose](https://github.com/block/goose) is Block's on-machine AI agent (CLI +
desktop). It calls MCP servers "extensions", so it gets MemoryWhale's four
tools: `recent_errors`, `search_memory`, `get_context`, `remember`. Goose is a
terminal/dev-loop agent — the same world MemoryWhale's memory is about.

## Connect the MCP server (recommended: the wizard)

```bash
goose configure
```

Choose **Add Extension → Command-line Extension**, then:

| Prompt | Value |
| --- | --- |
| Name | `memorywhale` |
| Command | `mw-mcp` |
| Timeout | `300` (default) |

The wizard writes the entry for you (no hand-editing). In the desktop app the
same lives under **Settings → Extensions → Add**.

## Or edit the config directly

Add this under `extensions:` in `~/.config/goose/config.yaml`
([`config.yaml`](config.yaml)):

```yaml
extensions:
  memorywhale:
    type: stdio
    name: memorywhale
    cmd: mw-mcp
    args: []
    enabled: true
    timeout: 300
    env_keys: []
    envs: {}
    description: "MemoryWhale persistent local memory"
```

`mw-mcp` must be on `PATH` (the standard MemoryWhale install); otherwise use its
absolute path as `cmd`. For a non-default database, add
`MEMORYWHALE_DATA_DIR` to `envs` (e.g. `envs: { MEMORYWHALE_DATA_DIR: "/path/to/dir" }`).

## Tell it when to reach for the memory

The extension gives Goose the *tools*; a hint in your `.goosehints` (or the
session's system prompt) teaches it *when*:

> When a build/test/deploy fails, before proposing a fix, use `search_memory`
> or `recent_errors` (MemoryWhale MCP) to check whether this failure has a known
> cause or a saved lesson. Once you've figured out why something failed or how a
> fix worked, use `remember` to save that conclusion.

## Without the MCP server

The `mw` CLI still works: `mw context --last-error`, `mw search "…"`,
`mw remember "…"`.
