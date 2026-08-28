# MCP reference

`mw-mcp` is a [Model Context Protocol](https://modelcontextprotocol.io) server
that exposes your MemoryWhale store to a coding agent (Claude Code, Codex,
Cursor, …) directly — so instead of you pasting past errors and fixes into the
chat, the agent queries them itself. It speaks newline-delimited JSON-RPC 2.0
over stdio and is read-mostly: the only tool that writes is `remember`, and it
saves a single note.

If you're working from a source checkout, the binary is
`cargo run -p memorywhale-cli --bin mw-mcp --`; installed, it's just `mw-mcp` on
your PATH (see the README's Install section). It reads the same local database
as the `mw` CLI — nothing leaves your machine.

## Trust model

`mw-mcp` is a local stdio process: the MCP client starts it and communicates
over its standard input and output. It does not listen on a network port and
has no separate protocol authentication. The spawning process and the
permissions of the operating-system user establish the local trust context.

`mw-serve` also exposes the same six tools at `POST /mcp` on the dashboard
port (default `http://127.0.0.1:7071/mcp`). HTTP MCP is one JSON-RPC object
per POST, not an SSE session. Clients may send `2026-07-28` `_meta` on each
request, or the usual `initialize` handshake then `tools/list` without
`_meta` (Rho `streamable_http` does this). Loopback is open by default. If
`--token` is set, MCP clients must send `Authorization: Bearer <token>`. A
non-loopback bind always requires a token.
`--lan` mints `serve.token` in the MemoryWhale data directory when no token
is supplied. `mw-serve --lan --print-token` prints that LAN token.
`--print-token` alone can still mint `serve.token`; tokenless loopback serving
ignores that file. Loopback authentication is enabled when `--token` or
`MEMORYWHALE_TOKEN` is set. A Rho client on another machine stores
`Bearer <token>` in `mcp-authorization` under that same data directory and sets
`headers_from_env = { Authorization = "MEMORYWHALE_AUTHORIZATION" }`. The
data directory is `$MEMORYWHALE_DATA_DIR` when set, otherwise the platform
default (`~/.local/share/MemoryWhale/` on Linux, `~/Library/Application
Support/MemoryWhale/` on macOS). `mw integrate rho --http --token` prints
the export line with the resolved path. Export that file as
`MEMORYWHALE_AUTHORIZATION` in the Rho process; the MemoryWhale capture hook
does not load it. Rho 2.2.0+ needs `allow_insecure_http = true` for a
cleartext LAN URL. `mw integrate rho` still defaults to stdio; `mw integrate
rho --http [url]` writes Rho's `streamable_http` transport key pointing at
this POST endpoint. `--revert` removes the client `mcp-authorization` copy
and leaves `serve.token` in place.

A process running as the same OS user may already be able to read the
MemoryWhale database and environment. MCP access nevertheless matters because
it gives an agent a supported interface for retrieving terminal evidence and
for writing agent-authored memories through `remember`.

| Actor | Capability | Exposure | Mitigation | Residual limitation |
| --- | --- | --- | --- | --- |
| Configured MCP client or agent | Call all six `mw-mcp` tools | Reads can reveal commands, paths, output, errors, and saved lessons | Grant MCP access only to clients you trust with the selected store; prevent sensitive capture where possible | Tool access cannot make already captured sensitive evidence non-sensitive |
| Agent using `remember` | Add a lesson that can influence later retrieval | Incorrect or adversarial memories can bias future context | Review agent-authored memories and remove entries that are not supported by evidence | Review reduces poisoning risk but does not prove a memory is correct |
| Other process running as the same OS user | Read files available to that user and potentially start `mw-mcp` | The local database is not isolated from peer same-user processes | Run untrusted agents under a separate OS identity or stronger sandbox | OS-user separation is only one boundary and may not satisfy every threat model |
| Client configured with `MEMORYWHALE_DATA_DIR` | Select a different MemoryWhale store | Accidental mixing of client data can be reduced | Use a dedicated directory, ideally with a separate OS identity and restrictive filesystem permissions | The variable scopes storage; it is not authentication or an access-control mechanism |

Treat the client process as trusted for the store it can reach. For stronger
isolation, use a separate OS identity or sandbox with a separately protected
data directory. Do not treat an application token passed to a same-user stdio
process as an independent boundary: that process can commonly inspect the
environment or read the underlying files directly.

For broader guidance on protecting captured terminal data, see the
[local data threat model](../SECURITY.md).

## Protocol compatibility

`mw-mcp` supports the current date-based MCP revision, `2026-07-28`, and the
legacy initialization-based revisions `2025-11-25` and `2024-11-05`. MCP does
not use an "MCP 2.0" version string.

Current clients send their protocol version, identity, and capabilities in
each request's `_meta` object. The server implements `server/discover`, reports
the versions it supports, and returns JSON-RPC error `-32022` with `supported`
and `requested` version data when no match is possible. Current responses use
the revision's `resultType`, capability, cache, and server metadata fields.

Legacy clients may still open with an `initialize` request for either
supported legacy revision, followed by `notifications/initialized`. That
handshake works on stdio and on `POST /mcp`.
For another initialization revision, the server negotiates `2025-11-25` as its
latest supported legacy revision. Protocol selection does not change the six
MemoryWhale tools or their semantics.

`remember` attribution follows the transport. Stdio keeps the initialize
`clientInfo.name` for the process. HTTP is one POST at a time, so a later
`tools/call` without `_meta` is stored as a generic agent write. Current
clients that send `clientInfo` in `_meta` keep their name on that request.

A direct current-protocol discovery check is:

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"memorywhale-check","version":"1"},"io.modelcontextprotocol/clientCapabilities":{}}}}' | mw-mcp
```

`mw doctor` performs this negotiation without calling a tool and reports the
selected revision. It falls back to the legacy handshake when an older server
does not implement current-protocol discovery.

## Wire it into an agent

Claude Code, one line:

```bash
claude mcp add memorywhale -- mw-mcp
```

Or add a stanza to an MCP config (`claude_desktop_config.json`, `.mcp.json`, or
your agent's equivalent) pointing at the installed binary:

```json
{
  "mcpServers": {
    "memorywhale": {
      "command": "mw-mcp"
    }
  }
}
```

No arguments, no API key. If `mw-mcp` isn't on your PATH, use its absolute path
as `"command"`.

## Tools

All six take a JSON object and return text (JSON-RPC `result.content[].text`).
Required args are noted; everything else is optional.
Run `mw-mcp --list-tools` to print the tool names directly from the runtime
registry. `scripts/check-doc-references.sh` verifies this table and the README
against that registry.

| Tool | Purpose | Args | Returns |
| --- | --- | --- | --- |
| `recent_errors` | Recent failed commands (non-zero exit) with their error output — start here when debugging a recurring failure. | `limit` (int, default 8, max 64) | A list of failed runs: command, exit code, cwd, the salient stderr line, and any note. |
| `search_memory` | Search remembered commands, sessions, and notes for a term, ranked by the explainable engine. | `query` (string, **required**); `project`, `machine` (string, optional scope) | Ranked hits with a score, a snippet, and the reasons each ranked where it did. |
| `get_context` | The most relevant remembered memory, engine-ranked, optionally scoped. | `project`, `machine` (string, optional) | Up to 8 ranked hits, each with score, snippet, and reasons. |
| `remember` | Save a freeform lesson or conclusion so future sessions don't re-derive it. | `text` (string, **required**) | Confirmation with the new memory id; findable later via `search_memory`/`get_context`. |
| `similar_failures` | Check whether an error you just hit has occurred before, and whether a later run resolved it. | `error_text` (string, **required**); `command` (string, optional — enables an exact fingerprint match) | Evidence-only history: occurrence count, how often a later run of the same command succeeded, and a pointer to a concrete past occurrence. |
| `stats` | Health/liveness check: confirm the store is reachable and populated before relying on the other tools. | none | JSON: total memory count, how many are recorded failures, the most-recent memory timestamp (or `"none"`), and the database file path. |

A fresh, empty store is handled gracefully — reads return empty results (and
`stats` returns zero counts) rather than erroring.

## The loop

The point isn't any single tool — it's the loop between them:

1. The agent runs a command and hits an error.
2. It calls `similar_failures` (or `search_memory`) to check whether this
   failure has been seen before, and whether a later run resolved it.
3. Once it figures out *why* it failed or *how* the fix worked, it calls
   `remember` to record that conclusion.
4. Next time the same error shows up — in a later session, or on a teammate's
   machine that imported the store — `similar_failures`/`search_memory` surface
   the fix instead of the agent re-deriving it from scratch.

Whether having the memory in hand actually changes whether an agent solves a
failure is measured in
[benchmarks/agent_eval/AGENT_EVAL.md](../../benchmarks/agent_eval/AGENT_EVAL.md);
whether the right memory is retrievable in the first place is measured by the
retrieval benchmarks in [benchmarks/BENCHMARKS.md](../../benchmarks/BENCHMARKS.md).
