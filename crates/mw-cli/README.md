# memorywhale-cli

**MemoryWhale 0.10.0 — Agent-Native Memory · September 6, 2026.**

Local-first terminal memory. Records commands, arguments, output, errors, and
whole sessions into local SQLite, so what already failed stays searchable —
across crashes, SSH drops, and explicit transfers between machines. No
background upload or synchronization.

Installs these binaries: `mw`, `mw-run`, `mw-remember`, `mw-serve`, `mw-view`,
`mw-recover`, `mw-screenshot`, and `mw-mcp` (a Model Context Protocol server so
an AI agent can query your memory directly).

```bash
cargo install memorywhale-cli
mw --version
mw doctor                # inspect install, MCP, and Claude/Rho hook + skill status
mw                       # record a shell session; it explains itself on first run
mw-run -- cargo build    # capture one command's output + exit code
mw search "linker error" # full-text search your history
mw context --last-error  # a paste-ready digest for an AI agent
mw-serve                 # local web dashboard (works headless, e.g. on a Jetson)
```

## Agent-native interfaces

```bash
mw integrate claude       # install Claude Code hook, skill, and MCP registration
mw integrate rho          # Rho equivalents; stdio is the default
mw search "linker error" agent:claude
mw-serve --api             # dashboard + POST /mcp + opt-in read-only /api/v1
mw github context 123     # explicit read-only PR context through your gh login
```

Use `mw integrate claude --revert` or `mw integrate rho --revert` to remove
an integration without deleting memory. `mw integrate rho --http [url]`
selects HTTP MCP; non-loopback access requires a token. GitHub context prints
bounded, redacted metadata, check-runs, classic commit statuses, and reviews;
it does not check out code, submit reviews, or automatically save the fetched
context.

Capture and retrieval remain independent. Rho hook events without command text
preserve failure metadata using a sentinel; successful events without commands
are skipped. Skills guide tool use, but automatic task-start recall, failure
lookup, and pre-compaction saving are not implemented lifecycle automation.

## Storage boundary

`memorywhale_cli::storage` is the only production owner of the SQLite
connection policy and base schema. Every CLI binary opens the database through
that module, which applies WAL mode, foreign keys, a shared busy timeout, the
canonical tables and indexes, and numbered migrations. Table definitions and
connection pragmas stay out of individual binaries so every entry point
converges fresh and partially upgraded databases to the same shape.

Schema 10 preserves canonical repository/worktree identity and nullable
`command_runs.agent` provenance (`claude`, `rho`, or `NULL`). `terminal` is the
display/filter label for `NULL`, including manual and legacy records; it is
not a source type or evidence of human authorship. Existing rows are not
re-attributed from notes.

This product release uses `memorywhale-core` 0.5.0. Rust callers constructing
`Memory` must provide the new `agent: Option<String>` field (use `agent: None`
for unknown provenance). Its serde default keeps older JSON readable.

Build all eight helpers from the repository root with
`cargo build --release -p memorywhale-cli --bins`. The CLI does not require
the desktop shell's GTK/WebKit dependencies.

Full documentation, the web dashboard, team sharing, and AI-agent integration:
<https://github.com/wuisabel-gif/MemWhale>.

Captured output is scrubbed for common secret shapes before storage; set
`MEMORYWHALE_NO_REDACT=1` to store raw. License: MIT.
