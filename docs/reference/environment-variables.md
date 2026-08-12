# Environment variables

## `MEMORYWHALE_DATA_DIR`

Overrides the platform-default data directory for the CLI and helper binaries,
including `mw-mcp`.

```bash
MEMORYWHALE_DATA_DIR=/path/to/data mw list
```

When an MCP client launches `mw-mcp`, set the variable in that client's server
environment rather than only in an unrelated interactive shell.

## `HERMES_HOME`

Used by `mw integrate hermes` to locate Hermes configuration. Without it, the
command writes to `~/.hermes/config.yaml`.

Additional command-specific variables are documented beside their commands in
the [CLI reference](cli.md).
