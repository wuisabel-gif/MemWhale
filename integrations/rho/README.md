# Rho + MemoryWhale

[Rho](https://github.com/opencode-ai/rho) is a coding-agent harness with native
support for Model Context Protocol servers. `mw-mcp` plugs in as a local stdio
MCP server, giving the Rho agent direct access to MemoryWhale's six
retrieval tools.

Rho also loads durable instructions from `AGENTS.md` files, which complements MCP
by telling the agent *when* to consult and update memory.

## Status

Verified against Rho 1.41.0 in August 2026. MCP servers are configured under
`[mcp.servers]` in `~/.rho/config.toml` (global) or a project-level Rho config.
Confirm with `rho mcp list` and `rho mcp show <id>`.

- `rho mcp list` — list configured MCP servers
- `rho mcp show <id>` — show one server by its `[mcp.servers.<id>]` key

- [MemoryWhale MCP reference](../../docs/reference/mcp.md)
- [MemoryWhale CLI reference](../../docs/reference/cli.md)

## Requirements

- MemoryWhale installed with `mw-mcp` on `PATH`.
- Rho installed and running.
- Optional: a deliberate `MEMORYWHALE_DATA_DIR` if a non-default store is needed.

## Capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes |
| Automatic execution capture | No |
| Memory-use guidance | Yes — through Rho's `AGENTS.md` (global or project) |

## Setup

### 1. Connect the MCP server

Add the server to `~/.rho/config.toml`:

```toml
[mcp.servers.memorywhale]
command = "mw-mcp"
```

For a non-default store, add the environment:

```toml
[mcp.servers.memorywhale]
command = "mw-mcp"
env = { MEMORYWHALE_DATA_DIR = "/path/to/store" }
```

If `mw-mcp` is not on the `PATH` Rho sees, use its absolute path as the
`command`.

### 2. Verify the connection

```bash
rho mcp list
rho mcp show memorywhale
```

`rho mcp list` should show `memorywhale`; `rho mcp show memorywhale` should
display the configured command. If Rho does not discover the server, restart
Rho after saving the config.

Confirm the server works independently:

```bash
command -v mw-mcp
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | mw-mcp
```

### 3. Add memory-use guidance (optional)

Rho loads a global `~/.rho/AGENTS.md` (every session) and a project-local
`AGENTS.md` (project sessions). Add guidance to whichever scope fits:

```markdown
## Use MemoryWhale as durable terminal memory

Before debugging a build/test/deploy error, use the MemoryWhale MCP tools
(`search_memory`, `recent_errors`, `similar_failures`) to check whether it
was solved before. Once you have figured out why something failed or how a
fix worked, use `remember` to save that conclusion.
```

## Verify

Ask Rho:

> Use MemoryWhale to check whether I have encountered a similar build failure
> before. Search for `openssl` and explain which saved evidence is relevant
> before suggesting a fix.

The MemoryWhale MCP server exposes six tools:

- `recent_errors`
- `search_memory`
- `get_context`
- `remember`
- `similar_failures`
- `stats`

An empty store is valid — the tools return empty results, not errors. `stats`
should report zero records on a fresh store.

## The loop

1. **Session start** — recall relevant prior work:

   ```bash
   mw context project:myapp          # compact digest for pasting
   mw search "openssl linker"        # targeted full-text recall
   ```

2. **During work** — capture a command you want recorded:

   ```bash
   mw-run -- cargo test              # captures command + output + exit code
   ```

3. **After solving something non-obvious** — save the conclusion:

   ```bash
   mw remember "the segfault was a use-after-free in camera-driver; fix: drop the buffer before recv"
   ```

4. **Session handoff** — export for a later agent or teammate:

   ```bash
   mw agent > session.md             # full transcript as Markdown
   mw context project:myapp          # compact paste-ready digest
   ```

## Automatic capture

Rho's `bash` tool calls are **not** automatically recorded by MemoryWhale.
MCP access lets the agent read and explicitly write memory, but ordinary
commands are captured only through:

- `mw-run -- <command>` wraps and records one command explicitly.
- `mw-remember` saves a command with output you already have.
- `mw --notes "project:…"` records a whole interactive shell session.

## How Rho sessions and MemoryWhale differ

Rho keeps its own saved session transcripts in `~/.rho/sessions/` and may
compact or summarize them. MemoryWhale stores durable evidence in its own
SQLite database at `<data_local>/MemoryWhale/memorywhale.sqlite3`. The two are
independent: compaction in Rho does not touch MemoryWhale data, and
MemoryWhale data survives Rho restarts and machine transfers. Use `mw agent`
or `mw context` to bridge a Rho session into MemoryWhale when you want the
evidence to outlive the current session.

## Security

`mw-mcp` is a local stdio process. Review the canonical
[local stdio trust model](../../docs/reference/mcp.md#trust-model) before
connecting a sensitive store. Only connect stores you intend the Rho agent to
read and write. Scoping with `MEMORYWHALE_DATA_DIR` is not an access-control
mechanism.

## Troubleshooting

- Run `command -v mw-mcp` from the environment used to launch Rho.
- Use the absolute path to `mw-mcp` if Rho cannot find your shell's `PATH`.
- Run `rho mcp list` to confirm the server is discovered.
- Run `mw doctor` to verify the data directory, database, and `script` status.
- Set `MEMORYWHALE_DATA_DIR` in the server's `env` block, not only in an
  unrelated terminal.
- Restart Rho after changing the MCP configuration.

## Remove integration

Remove the `[mcp.servers.memorywhale]` entry from `~/.rho/config.toml` and
remove any MemoryWhale section from `~/.rho/AGENTS.md` or the project
`AGENTS.md`. Restart Rho. This does not delete the MemoryWhale database; use
`mw rm` or the documented retention commands for data lifecycle operations.