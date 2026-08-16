# Rho + MemoryWhale

[Rho](https://github.com/opencode-ai/rho) is a coding-agent harness. Rho does
not currently expose a native MCP-server configuration, so MemoryWhale connects
through its CLI: Rho invokes `mw` commands through its shell tool, and durable
instructions in an `AGENTS.md` file tell the agent when to recall and record.

This keeps the two systems cleanly separated: Rho owns session transcripts and
compaction; MemoryWhale owns the cross-session evidence that survives across
Rho restarts, different machines, and different agents.

## Status

Verified against Rho 1.41.0 in August 2026. Rho's `~/.rho/config.toml` and
built-in configuration guidance expose no MCP-server configuration, so this
guide documents a CLI + instructions workflow. If Rho adds native stdio MCP
support, the [Generic MCP guide](../generic-mcp/README.md) will be the right
starting point and this guide should be updated.

- [Rho configuration](https://github.com/opencode-ai/rho) — `~/.rho/config.toml`, `~/.rho/AGENTS.md`
- [MemoryWhale CLI reference](../../docs/reference/cli.md)
- [MemoryWhale MCP reference](../../docs/reference/mcp.md)

## Requirements

- MemoryWhale installed with `mw` on `PATH` (the path Rho's shell tool sees).
- Rho installed and running.
- A project tag so related work groups across sessions. Example: `project:myapp`.

## Capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | No — Rho has no verified native MCP-server config |
| Automatic execution capture | No — Rho shell-tool calls are not auto-captured |
| Memory-use guidance | Yes — through Rho's `AGENTS.md` (global or project) |
| Explicit CLI read/write | Yes — `mw search`, `mw context`, `mw remember`, `mw-run` |

## Setup

### 1. Confirm the CLI is visible

```bash
command -v mw
mw doctor
```

If `mw` is not on the `PATH` Rho's shell tool uses, reference its absolute path
in the instructions instead.

### 2. Add durable instructions

Rho loads a global `~/.rho/AGENTS.md` (every session) and a project-local
`AGENTS.md` (project sessions, loaded after the global file). Add the recall
and record guidance to whichever scope fits:

**Per-project** — create or edit `AGENTS.md` in the repository root:

```markdown
## Use MemoryWhale as durable terminal memory

Before debugging a build/test/deploy error, check whether it was solved
before:

    mw search "linker error"
    mw context --last-error

Once you have figured out *why* something failed or *how* a fix worked, save
the conclusion so a future session does not re-derive it:

    mw remember "the E0308 was a string field parsed as i32; fix: parse i32"

Tag related work: `mw-remember --notes "project:myapp …" -- <command>`.
```

**Global** — edit `~/.rho/AGENTS.md` with the same content to apply it to every
Rho session regardless of project. Project files load later and take
precedence on conflict.

### 3. Select a non-default store (optional)

If a project should use its own MemoryWhale database, set the variable in the
shell environment before launching Rho or in the project's setup:

```bash
MEMORYWHALE_DATA_DIR=/path/to/store mw doctor
```

Scoping a store is not an access-control mechanism; review the
[local stdio trust model](../../docs/reference/mcp.md#trust-model) before
sharing a sensitive store.

## Verify

From a Rho shell-tool call, confirm each read and write path:

```bash
# write a marker
mw-remember --cwd "$(pwd)" --exit-code 0 \
  --notes "project:rho-integration-test" -- echo "rho test marker qzx"

# recall it
mw search "qzx"
mw context project:rho-integration-test
```

`mw search` should list the recorded command; `mw context` should show an
empty-but-valid digest (no failures yet). An empty store is valid — the tools
return empty results, not errors.

## The loop

1. **Session start** — recall relevant prior work before acting:

   ```bash
   mw context project:myapp          # recent failures + lessons, compact
   mw search "openssl linker"        # targeted full-text recall
   ```

2. **During work** — capture a command you want recorded:

   ```bash
   mw-run -- cargo test              # captures the command + output + exit code
   ```

3. **After solving something non-obvious** — save the conclusion, not the
   process:

   ```bash
   mw remember "the segfault was a use-after-free in camera-driver; fix: drop the buffer before recv"
   ```

4. **Session handoff** — export the session for a later agent or a teammate:

   ```bash
   mw agent > session.md   # full transcript as Markdown
   mw context project:myapp  # compact paste-ready digest
   ```

## Automatic capture

Rho's `bash` tool calls are **not** automatically recorded by MemoryWhale.
MemoryWhale captures commands in three ways, none of which is Rho-specific:

- `mw-run -- <command>` wraps and records one command explicitly.
- `mw-remember` saves a command with output you already have.
- `mw --notes "project:…"` records a whole interactive shell session (separate
  from the Rho session).

MCP access — if Rho adds it in the future — would let the agent read and
explicitly write memory; it would still not auto-capture Rho's internal tool
calls.

## How Rho sessions and MemoryWhale differ

Rho keeps its own saved session transcripts in `~/.rho/sessions/` and may
compact or summarize them. MemoryWhale stores durable evidence in its own
SQLite database at `<data_local>/MemoryWhale/memorywhale.sqlite3`. The two are
independent: compaction in Rho does not touch MemoryWhale data, and
MemoryWhale data survives Rho restarts and machine transfers. Use `mw agent`
or `mw context` to bridge a Rho session into MemoryWhale when you want the
evidence to outlive the current session.

## Troubleshooting

- Run `command -v mw` inside a Rho shell-tool call to confirm `PATH` visibility.
- Use the absolute path to `mw` if Rho's environment differs from your shell.
- Run `mw doctor` to verify the data directory, database, and `script` status.
- Set `MEMORYWHALE_DATA_DIR` in the environment that launches Rho, not only in
  an unrelated terminal.
- If `mw search` returns nothing, the store may genuinely be empty — run
  `mw doctor` to confirm it can see the database.

## Remove integration

Remove the MemoryWhale section from `~/.rho/AGENTS.md` or the project
`AGENTS.md`. This does not delete the MemoryWhale database; use `mw rm` or the
documented retention commands for data lifecycle operations.