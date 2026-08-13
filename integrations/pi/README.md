# Pi coding agent + MemoryWhale

## Status

CLI/context workflows are documented against the Pi coding agent documentation
available in August 2026. MemoryWhale does not currently provide a native Pi
MCP adapter. Pi supports TypeScript extensions, but an official built-in MCP
configuration was not verified. Treat third-party MCP adapters as separate
software and review them before installing.

- [Pi settings](https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/settings.md)
- [Pi extensions](https://github.com/badlogic/pi-mono/blob/main/packages/coding-agent/docs/extensions.md)
- [MemoryWhale generic MCP contract](../generic-mcp/README.md)

## Requirements

- MemoryWhale installed and `mw` available on `PATH`.
- Pi coding agent installed separately.
- A shell environment where Pi and MemoryWhale use the intended
  `MEMORYWHALE_DATA_DIR`, if a non-default store is needed.

## Capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Unverified; no native Pi MCP setup is documented here |
| Automatic execution capture | No |
| Memory-use guidance | Yes, through a Pi project instruction or extension |

## Setup: use the CLI today

MemoryWhale can be used with Pi without a Pi-specific plugin. Capture a command
explicitly, then give Pi the retrieved context:

```bash
mw-run -- cargo test
mw context --last-error
mw search "linker error"
```

You can paste the output of `mw context --last-error` into a Pi conversation.
For a different store, set the variable in the same shell environment:

```bash
MEMORYWHALE_DATA_DIR=/path/to/store mw context --last-error
```

## Optional Pi guidance

Pi supports project-local extensions and other project resources. Add a short
instruction to the project guidance you already use, or create a trusted Pi
extension, telling Pi when to consult MemoryWhale:

> When a build, test, or deploy fails, ask the user for the output of
> `mw context --last-error` or `mw search "<distinctive error>"` before proposing
> a fix. After the cause is confirmed, suggest saving the lesson with
> `mw remember "<what fixed it>"`.

Do not place secrets or a private database path in project guidance.

## Verify

Verify the MemoryWhale side independently:

```bash
command -v mw
mw doctor
mw context --last-error
```

An empty store may return an empty context. That is expected. If you later add
an MCP adapter, verify its tool discovery separately and use the canonical
[MCP reference](../../docs/reference/mcp.md) for the six MemoryWhale tools.

## Automatic capture

This integration does not automatically record Pi prompts, responses, or shell
commands. `mw-run` captures an explicitly wrapped command; normal terminal
capture and supported hooks remain separate. MCP access, if added through a
third-party adapter, would provide memory access rather than automatic capture.

## Limitations

- There is no verified native Pi-to-`mw-mcp` configuration in this repository.
- The CLI workflow requires copying or piping context into Pi.
- Pi project extensions execute with broad local permissions; only install
  extensions you trust.
- MemoryWhale does not silently synchronize the local database.

## Troubleshooting

- Run `command -v mw` from the environment used to start Pi.
- Use the absolute path to `mw` if Pi's environment has a different `PATH`.
- Set `MEMORYWHALE_DATA_DIR` explicitly when selecting a non-default store.
- Run `mw doctor` before debugging a missing or empty store.

## Remove integration

Remove any Pi-specific instruction or extension you added. This does not delete
the MemoryWhale database; use the documented `mw rm` or retention commands for
data lifecycle operations.
