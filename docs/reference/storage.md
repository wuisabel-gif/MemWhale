# Storage reference

MemoryWhale stores its durable state in `memorywhale.sqlite3` below the platform
data directory:

- Linux: typically `~/.local/share/MemoryWhale/`
- macOS: `~/Library/Application Support/MemoryWhale/`
- Override: `MEMORYWHALE_DATA_DIR`

The database contains captured evidence and remembered lessons. Transcripts and
related artifacts may live beside it. Treat the entire directory as sensitive
development history and preserve its permissions and SQLite companion files
when backing it up.

Use MemoryWhale's export, import, retention, and deletion commands rather than
editing tables directly. See the [CLI reference](cli.md) for the current
commands.
