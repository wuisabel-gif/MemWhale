# Issue #2 — `MemPalaceEngine`: real MCP client, feature-gated

## What changed

`crates/mw-memory/src/engine.rs` no longer ships a stub that prints placeholder
text. `MemPalaceEngine` is now a real MCP **client** that spawns a local
`mempalace-mcp` server over stdio, does the handshake, calls its search tool, and
maps the hits into this crate's `Memory` / `Signal` / `ScoredMemory` types.

- **Cargo feature `mempalace`, off by default.** `[features] default = []`.
  No new dependencies — the client is built on `std::process`, `serde_json`
  and `anyhow`, all of which mw-memory already had. `cargo install` of the CLI
  pulls in nothing new.
- **When the feature is off the type does not exist at all** — the struct, its
  `Default`/inherent impls, the `MemoryEngine` impl, the `mcp` module, and the
  mapping tests are all `#[cfg(feature = "mempalace")]`.
- **New private module `crates/mw-memory/src/mcp.rs`** — a ~130-line
  JSON-RPC-2.0-over-stdio client covering exactly `initialize`,
  `notifications/initialized`, `tools/list`, `tools/call`. Modelled on the
  message shapes in `crates/mw-cli/src/bin/mw-mcp.rs`, which is our *server* for
  the same protocol. No MCP client crate was added: none was in `Cargo.lock`, and
  the wire format we need is three JSON objects.
- **Errors, never placeholders.** `MemPalaceEngine::try_retrieve` returns
  `anyhow::Result`, so a missing binary surfaces as
  `failed to start MCP server \`mempalace-mcp\`` and a bad handshake as
  ``MCP handshake with `mempalace-mcp` failed``. The `MemoryEngine` trait method
  can't return an error (it yields a `Vec`), so `retrieve` logs the full error
  chain to stderr and returns empty; callers that need to *handle* failure call
  `try_retrieve`.
- **Scoring.** MemPalace ranks server-side, so we don't re-score. Its relevance
  becomes a single `similarity` `Signal` with `weight 1.0` and the reason string
  `mempalace semantic score 0.87`.
- Docs updated: `crates/mw-memory/src/lib.rs` header + `Memory` doc comment,
  `engine.rs` module header, the crate `description` in `Cargo.toml`, and
  roadmap item 7 in `VISION.md` (marked shipped, feature-gated). README mentions
  are attribution/credits only and were left alone. No DB schema changes.

## Expected search-tool payload

The search tool's text content is parsed as either a bare JSON array or
`{"results": [...]}`. Per hit: `text` and `score` are used; `id`, `tags`,
`created_at`, `last_used`, `mentions`, `importance` are optional (timestamps fall
back to `Query::now`, which keeps tests deterministic).

## Fixture: exact JSON-RPC exchanged

No network. The end-to-end test runs `sh crates/mw-memory/tests/fixtures/fake-mempalace-mcp.sh`
as the "server". Below is the literal, byte-for-byte traffic (newline-delimited;
client lines captured from a real run of `try_retrieve`).

**→ client to server**

```json
{"id":1,"jsonrpc":"2.0","method":"initialize","params":{"capabilities":{},"clientInfo":{"name":"memorywhale","version":"0.1.0"},"protocolVersion":"2024-11-05"}}
```

**← server to client**

```json
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"mempalace","version":"0.0.0-fake"}}}
```

**→ client to server** (notification, no reply)

```json
{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}
```

**→ client to server**

```json
{"id":2,"jsonrpc":"2.0","method":"tools/list","params":{}}
```

**← server to client**

```json
{"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"search","inputSchema":{"type":"object","properties":{"query":{"type":"string"}}}}]}}
```

**→ client to server** (for `Query::new("rust for systems", …)` with `k = 5`)

```json
{"id":3,"jsonrpc":"2.0","method":"tools/call","params":{"arguments":{"limit":5,"query":"rust for systems"},"name":"search"}}
```

**← server to client** (the `text` field is the JSON payload, escaped as a JSON string)

```json
{"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":"[{\"id\":143,\"text\":\"I use Rust for systems software.\",\"score\":0.87,\"tags\":[\"rust\"],\"created_at\":\"2026-06-07T12:00:00Z\",\"last_used\":\"2026-06-27T12:00:00Z\",\"mentions\":27,\"importance\":0.98},{\"id\":22,\"text\":\"Use Tokio for the async runtime.\",\"score\":0.41}]"}]}}
```

That inner payload also lives standalone as
`crates/mw-memory/tests/fixtures/mempalace_search.json`, which the pure mapping
test reads via `include_str!`.

## Tests added (all under `#[cfg(feature = "mempalace")]`)

| test | covers |
| --- | --- |
| `mempalace_maps_hits_to_signals` | captured JSON → `ScoredMemory`: id, tags, `percent() == 87`, signal name/detail, timestamp fallback to `Query::now` |
| `mempalace_rejects_garbage` | non-JSON and shape-less JSON produce errors, not empty success |
| `mempalace_talks_to_a_fake_server` (unix) | full spawn + handshake + `tools/list` + `tools/call` against the shell fixture |
| `missing_server_is_a_clear_error` | absent binary → `failed to start MCP server`, never placeholder text |

The old `mempalace_stub_is_pluggable` test (which asserted the name
`"mempalace (stub)"`) is gone with the stub.

## Verification

| command | result |
| --- | --- |
| `cargo build --workspace` | pass — default build unchanged, no new deps |
| `cargo build -p mw-memory --features mempalace` | pass, no warnings |
| `cargo test --workspace` | pass |
| `cargo test -p mw-memory --features mempalace` | pass (17 tests) |

Both consumers of `mw-memory` (`crates/mw-cli` and `src-tauri`) build unchanged;
neither referenced `MemPalaceEngine`.
