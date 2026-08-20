# `mw pet` — the MemoryWhale companion

`mw pet` is a memory-record read-only terminal view of the local MemoryWhale
store. It renders an ASCII whale whose mood reflects how recently the store was
used. It does not call a model or contact a network service.

It uses the normal MemoryWhale database initializer. The first run can create
the configured data directory and database schema, migrations can run, and
opening the store can apply due-note expiry housekeeping. It does not
intentionally add commands, sessions, lessons, or links.

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
| `well-fed` | Fewer than two whole elapsed UTC days since the most recently used memory. |
| `content` | At least two and fewer than eight whole elapsed UTC days since the most recently used memory. |
| `sleepy` | At least eight whole elapsed UTC days since the most recently used memory. |

The snapshot also shows:

- the number of loaded memories;
- the number of links in `memory_links`;
- the age of the most recently used memory;
- a small spout animation when recent activity or expired memories warrant it.

The pet reads the same SQLite database as the other CLI commands. It follows
`MEMORYWHALE_DATA_DIR` when set, so it can inspect a selected local store:

```bash
MEMORYWHALE_DATA_DIR=/path/to/MemoryWhale mw pet
```

## What it does not do

The pet is intentionally presentation-only with respect to user-created
memory content. It does not:

- add or delete memories, sessions, lessons, or links;
- record terminal commands or sessions;
- start an MCP server;
- require an API key or an AI provider;
- upload store contents.

Opening the store may still perform the initialization and due-note expiry
housekeeping described above.

For a searchable view, use `mw search`, `mw tui`, or `mw-serve`. To provide
memory to a coding agent, configure the local `mw-mcp` server instead.
