# Environment variables

## `MEMORYWHALE_HOOK_DIAGNOSTICS`

Set to exactly `1` to enable fixed, nonfatal stderr diagnostics for Cursor hook
processing (`mw-remember --from-hook cursor`). By default, expected invalid
payload and storage failures stay silent. Diagnostics never write to stdout
or include captured payload contents. This development-branch option does not
enable capture, change client permissions, or alter other hook clients.

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

## `CLAUDE_CONFIG_DIR`

Used by `mw integrate claude` and `mw doctor` to locate Claude Code
configuration. Without it, those commands use `~/.claude/`.

## `RHO_HOME`

Used by `mw integrate rho` and `mw doctor` to locate Rho configuration. Without
it, those commands use `~/.rho/`.

Additional command-specific variables are documented beside their commands in
the [CLI reference](cli.md).
