# AGENTS.md

Guidance for AI coding agents (Codex, Claude, etc.) working in this repo.

## Use MemoryWhale as durable terminal memory

This project *is* a terminal-memory system — use it on yourself. It stores
commands, exit codes, stdout/stderr, notes, and whole sessions in a local SQLite
database so a future agent session, a different machine, or a different agent can
continue from what already happened. Everything is local; nothing is uploaded.

Database: `<data_local>/MemoryWhale/memorywhale.sqlite3`
(`~/.local/share/...` on Linux, `~/Library/Application Support/...` on macOS).

### Recall before you act

Before debugging a build/environment error, check whether it was solved before:

```bash
mw search "linker cc not found"      # full-text recall across commands, output, notes
mw context --last-error              # the most recent failure, with its error tail
```

It shows prior matching runs and what was run right after a past failure (often
the fix). Prefer a known-good fix over re-deriving it.

### Record as you go

Log notable commands — especially a failure and the fix that worked:

```bash
mw-remember --cwd "$(pwd)" --exit-code "$?" \
  --stderr "<error output>" \
  --notes "project:<name> what / why" -- <command> [args]
```

Record an exploratory stretch as a whole session:

```bash
mw --notes "project:<name>"    # ...work...  then: exit  (wait for "recorded session #N")
```

Tag related work across terminals with the same `project:<name>` to group it.

### Inspect

- `mw-view <id>` — one memory as a local web page (with suggested next steps).
- `mw-serve` — dashboard at `http://127.0.0.1:7071/`; includes `/graph` and
  project views.
- `mw-recover` — import an interrupted session transcript that didn't save.

## Repo conventions

- Before changing behavior, read `docs/architecture.md` and place the change in
  Capture, Memory, Retrieval, or Interfaces. Keep integrations thin and do not
  introduce agent-specific behavior into core.
- Before debugging, run a focused test and query MemoryWhale for a recurring
  failure. Record non-obvious fixes after they are verified.
- The Cargo workspace lives at the repository root. The reusable retrieval
  crate is `crates/memorywhale-core`, CLI sources are in `crates/mw-cli`, the
  React frontend is in `src`, and `src-tauri` contains only the desktop shell.
- Build every helper from the repository root with
  `cargo build --release -p memorywhale-cli --bins`. The binaries are `mw`,
  `mw-remember`, `mw-run`, `mw-screenshot`, `mw-serve`, `mw-view`,
  `mw-recover`, and `mw-mcp`.
- macOS only: after copying a built binary, re-sign it (`codesign --force --sign -
  <path>`) or it gets `Killed: 9`. See `DEBUG.md`.
- Full usage: `docs/reference/cli.md`. Setup/troubleshooting: `DEBUG.md`.

The agent form of this guidance lives in `crates/mw-cli/integrate/SKILL.md`.
