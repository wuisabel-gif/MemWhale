# Generic stdio MCP integration

## Status

Supported interface contract for any MCP client that can launch a local stdio
server.

## Requirements

- MemoryWhale installed.
- `mw-mcp` available on the `PATH` seen by the client.
- A client that supports local stdio MCP servers.

## Capabilities

| Capability | Available |
| --- | --- |
| MCP memory read/write | Yes |
| Automatic execution capture | No |
| Memory-use guidance | Client-specific |

## Setup

Configure one local server with this minimum contract:

```yaml
transport: stdio
command: mw-mcp
args: []
```

The exact surrounding schema depends on the client. No arguments are required.
To select another database, set `MEMORYWHALE_DATA_DIR` in the server's
environment.

## Verify

Initialize the server directly:

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | mw-mcp
```

Then verify tool discovery in the client. The authoritative tool descriptions
are in the [MCP reference](../../docs/reference/mcp.md).

## How to use

Ask the client:

> Use MemoryWhale to check whether I have encountered a similar failure before.

## Automatic capture

The generic MCP connection does not record normal terminal or agent commands.
Use MemoryWhale's terminal capture or a verified client-specific hook.

## Troubleshooting

- Run `command -v mw-mcp` from the same environment that starts the client.
- Restart the client after changing its MCP configuration.
- If the wrong database opens, set `MEMORYWHALE_DATA_DIR` in the server entry.
- Run `mw doctor` to verify the local installation.

## Remove integration

Delete the `memorywhale` server entry from the client's MCP configuration and
restart the client. This does not delete the local MemoryWhale database.
