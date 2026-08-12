# Hermes Agent integration

[Hermes Agent](https://github.com/NousResearch/hermes-agent) can run local
stdio MCP servers. Hermes remains the agent runtime; `mw-mcp` contributes
MemoryWhale's persistent developer-memory tools.

## Prerequisites

Install both `hermes` and MemoryWhale, then confirm the MCP binary is on
`PATH`:

```bash
command -v hermes
command -v mw-mcp
```

If `mw-mcp` is not on `PATH`, use its absolute path in the command below or in
the configuration file.

## Connect MemoryWhale

The MemoryWhale CLI can update Hermes' configuration safely while preserving
existing settings and MCP servers:

```bash
mw integrate hermes
```

The command honours `HERMES_HOME` and otherwise writes to
`~/.hermes/config.yaml`. It is idempotent, validates existing YAML before
changing it, and refuses malformed configuration without overwriting it.

Alternatively, Hermes' MCP management command can register the local server
directly:

```bash
hermes mcp add memorywhale --command mw-mcp
```

The equivalent entry in `~/.hermes/config.yaml` is:

```yaml
mcp_servers:
  memorywhale:
    command: "mw-mcp"
```

If `mcp_servers` already exists, add `memorywhale` beneath it instead of
replacing the other servers. To use a non-default MemoryWhale database, pass
the data directory through the server environment:

```yaml
mcp_servers:
  memorywhale:
    command: "mw-mcp"
    env:
      MEMORYWHALE_DATA_DIR: "/path/to/memorywhale-data"
```

Start Hermes after saving the configuration:

```bash
hermes chat
```

Hermes prefixes MCP tool names with the server name, but normally selects them
without being told the internal name. The six MemoryWhale tools are:

- `recent_errors`
- `search_memory`
- `get_context`
- `remember`
- `similar_failures`
- `stats`

## Verify the integration

Ask Hermes:

> Use MemoryWhale to check whether I have encountered a similar build failure
> before.

For a deterministic transport check outside Hermes, initialize the server
directly:

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | mw-mcp
```

## Using Kimi K3 through Hermes

Kimi K3 is a model, not an MCP client. If the Hermes model provider you use
offers Kimi K3, select it through Hermes' model configuration and keep the
MemoryWhale MCP configuration above unchanged. The responsibilities remain
separate:

```text
             ┌────────────────────┐
             │    Hermes Agent    │
             │                    │
Kimi K3 ───► │  reasoning/model   │
             │                    │
             │     MCP client     │
             └─────────┬──────────┘
                       │
                       ▼
                    mw-mcp
                       │
                       ▼
                   MemoryWhale
                       │
                       ▼
              local SQLite database
```

The exact Kimi model identifier and credentials depend on the provider. Do not
configure the model process itself to launch `mw-mcp`; Hermes owns that
connection.

## Troubleshooting

- If no tools appear, restart Hermes after adding the server and confirm
  `mw-mcp` is executable from the same environment that launches Hermes.
- If Hermes runs in a container or remote machine, install MemoryWhale there
  too, or point `command` at an executable available in that environment.
- If the wrong database opens, set `MEMORYWHALE_DATA_DIR` in the MCP server's
  `env` block.
- MemoryWhale remains local-first: Hermes receives tool results, but
  MemoryWhale does not upload its SQLite database.

Hermes MCP configuration details are maintained in the
[official Hermes documentation](https://github.com/NousResearch/hermes-agent/blob/main/website/docs/user-guide/features/mcp.md).
