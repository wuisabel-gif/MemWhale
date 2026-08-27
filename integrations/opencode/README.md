# OpenCode + MemoryWhale

[OpenCode](https://opencode.ai) is an open-source coding agent that runs in the
terminal, IDE, and web. It is an MCP host: local stdio MCP servers configured in
its JSON config become tools the agent can call directly. `mw-mcp` plugs in as
one such server, giving OpenCode access to MemoryWhale's six retrieval tools.

## Status

Verified against OpenCode's official MCP documentation
(<https://opencode.ai/docs/mcp-servers/>) and config documentation
(<https://opencode.ai/docs/config/>). Syntax may change between OpenCode
versions; check those pages if a field is rejected.

## Requirements

- MemoryWhale installed with `mw-mcp` on `PATH`.
- OpenCode installed (<https://opencode.ai/docs/>).
- A model that supports tool calling (MCP tools are ordinary tools from the
  model's perspective).

## Capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes |
| Automatic execution capture | No |
| Memory-use guidance | Example prompt (OpenCode rules can carry standing instructions) |

## Setup

OpenCode reads a merged JSON config from `~/.config/opencode/opencode.json`
(global) and `opencode.json` in the project root (project). Both use the same
schema; JSONC comments are allowed. MCP servers are defined under `"mcp"`.

Add MemoryWhale as a local stdio server:

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "memorywhale": {
      "type": "local",
      "command": ["mw-mcp"],
      "enabled": true
    }
  }
}
```

Notes, from OpenCode's documented local-server options:

- `command` is an **array** — the command and its arguments as one list. If
  `mw-mcp` is not on the `PATH` OpenCode sees, use its absolute path:
  `"command": ["/usr/local/bin/mw-mcp"]`.
- To use a non-default MemoryWhale store, set the environment under
  `"environment"`:

  ```json
  {
    "mcp": {
      "memorywhale": {
        "type": "local",
        "command": ["mw-mcp"],
        "environment": {
          "MEMORYWHALE_DATA_DIR": "/path/to/store"
        }
      }
    }
  }
  ```

- `"enabled": false` disables a server without removing it.
- If other MCP servers are already configured, add the `"memorywhale"` key
  alongside them — do not replace the existing entries.

## Verify

Confirm the server binary first, then check OpenCode discovered it:

```bash
command -v mw-mcp
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}' | mw-mcp
opencode mcp list
```

`opencode mcp list` should show the `memorywhale` server. Start a new OpenCode
session (config is read at startup) and ask:

> Use MemoryWhale to check whether I have encountered a similar build failure
> before. Search for `openssl` and explain which saved evidence is relevant
> before suggesting a fix.

OpenCode registers MCP tools with the server name as prefix, so the six
MemoryWhale tools appear as:

- `recent_errors`
- `search_memory`
- `get_context`
- `remember`
- `similar_failures`
- `stats`

An empty store is valid — the tools return empty results, not errors. `stats`
on a fresh store reports zero records.

## OpenCode Go

OpenCode Go is a low-cost model subscription inside OpenCode, not a separate
client. It changes which models are available; it does not change MCP
configuration. The setup above works the same whether the selected model comes
from OpenCode Go (GLM, Kimi, DeepSeek, …), any other provider, or a local
server — as long as the model supports tool calling.

## Automatic capture

MCP access is not automatic execution capture. Commands OpenCode runs are
recorded by MemoryWhale only through the normal capture paths — `mw-run --`,
`mw-remember`, `mw --notes "project:…"` session recording, or an installed
shell hook. MCP lets the agent read memory and save lessons via `remember`
when asked.

## Limitations

- MCP tools consume model context; OpenCode's docs warn that many servers can
  exhaust the context window. MemoryWhale's six tools are small, but avoid
  stacking it onto large tool sets thoughtlessly.
- The model must actually support tool calling; not every local model does.
- Secret redaction on capture reduces accidental retention but is not a
  security boundary.

## Security

`mw-mcp` is a local stdio process with full read/write access to the connected
MemoryWhale database. Review the canonical
[local stdio trust model](../../docs/reference/mcp.md#trust-model) before
connecting a store that contains sensitive output. `MEMORYWHALE_DATA_DIR`
selects a store; it is not an access-control mechanism.

## Troubleshooting

- Run `command -v mw-mcp` in the same environment OpenCode launches from; use
  an absolute path in `command` if they differ.
- Run `opencode mcp list` to confirm the server is configured and enabled.
- Restart OpenCode after editing the config; MCP servers connect at startup.
- Check the config file parses: OpenCode also accepts JSONC, but a stray comma
  or comment in strict tooling can break parsing.
- Run `mw doctor` to verify the MemoryWhale data directory and database.

## Remove integration

Delete the `"memorywhale"` entry from `"mcp"` in `opencode.json` (global or
project) and restart OpenCode. This does not delete the MemoryWhale database;
use `mw rm` or the documented retention commands for data lifecycle.
