# Cross-agent handoff: Claude to Rho

This walkthrough demonstrates the Agent-Native Memory flow shipped for
MemoryWhale 0.10. It is deterministic and offline: the Claude event is a
checked-in fixture, while the Rho step uses the real `mw-mcp` stdio protocol.
No model provider, API key, network connection, or real MemoryWhale database is
needed.

## What it demonstrates

```text
Claude Bash failure
    ↓ verified Claude hook
MemoryWhale: agent=claude
    ↓ shared local SQLite store
Rho search_memory(agent=claude)
    ↓ real mw-mcp protocol
Rho retrieves Claude's prior evidence
```

Normal terminal activity remains separate:

```text
Claude hook capture       agent=claude
ordinary terminal command agent=terminal
```

MCP access alone does not capture commands. The agent-specific capture hook is
the part that records the Claude event.

## Run the offline demo

Build the CLI helpers from the repository root:

```bash
cargo build -p memorywhale-cli --bins
```

Run:

```bash
scripts/agent-handoff-demo.sh
```

The script creates a temporary data directory, runs the Claude hook parser with
`tests/fixtures/agent-handoff/claude-post-tool-use-failure.json`, verifies the
stored/searchable `agent:claude` result, records a separate terminal command,
and sends `server/discover`, `tools/list`, and `search_memory` requests to the
actual `mw-mcp` binary as a Rho client.

Expected output includes:

```text
captured and searchable as agent:claude
terminal-only command remains terminal-attributed
Rho retrieved Claude's prior failure from the shared store
Agent handoff complete: Claude captured the failure; Rho found it without rediscovery.
```

The script defaults to `target/debug` binaries. Override them when testing a
release build:

```bash
MW_BIN=target/release/mw \
MW_REMEMBER_BIN=target/release/mw-remember \
MW_MCP_BIN=target/release/mw-mcp \
scripts/agent-handoff-demo.sh
```

## Use real integrations

For real local clients, install the verified integrations first:

```bash
mw integrate claude
mw integrate rho
```

Then start each client with access to the same local data directory. Use
`mw doctor` to check MCP, automatic capture, and skill status independently.
The exact provider/model can vary; the durable handoff contract is the shared
local store and the structured agent provenance.

Do not commit real hook payloads, transcripts, API keys, personal paths, or
provider output. Use the sanitized fixture and temporary data directory for
repeatable documentation and CI.
