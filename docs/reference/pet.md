# `mw pet` — the MemoryWhale companion

`mw pet` is a small, read-only terminal view of the local MemoryWhale store.
It renders an ASCII whale whose mood reflects how recently the store was used.
It does not call a model, contact a network service, or write memory.

## Usage

```bash
mw pet              # print one snapshot
mw pet --watch      # refresh and animate until Ctrl-C
NO_COLOR=1 mw pet   # disable ANSI colors
```

`--watch` refreshes the store approximately every 2.5 seconds and redraws the
whale four times per second. Press `Ctrl-C` to stop it.

## Mood rules

| Mood | Condition |
| --- | --- |
| `hungry` | The store has no memories. |
| `well-fed` | The most recently used memory was used today or yesterday. |
| `content` | The most recently used memory was used within the last seven days. |
| `sleepy` | The most recently used memory is older than seven days. |

The snapshot also shows:

- the number of loaded memories;
- the number of links in `memory_links`;
- the age of the most recently used memory;
- the number of expired memories, which can trigger a small spout animation.

The pet reads the same SQLite database as the other CLI commands. It follows
`MEMORYWHALE_DATA_DIR` when set, so it can inspect a selected local store:

```bash
MEMORYWHALE_DATA_DIR=/path/to/MemoryWhale mw pet
```

## What it does not do

The pet is intentionally presentation-only. It does not:

- create, update, expire, or delete memories;
- record terminal commands or sessions;
- start an MCP server;
- require an API key or an AI provider;
- upload store contents.

For a searchable view, use `mw search`, `mw tui`, or `mw-serve`. To provide
memory to a coding agent, configure the local `mw-mcp` server instead.
