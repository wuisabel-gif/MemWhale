# MemoryWhale — Handoff Notes

Context summary so any new session (human or AI) can pick up where prior work left off.
Last updated: 2026-06 (terminal-recorder work).

## What MemoryWhale is
A local-first Rust/Tauri desktop "memory" system that turns terminal commands, error
logs, notes, transcripts, and imported text into a searchable knowledge graph. Built for
robotics/AI-assisted debugging across machines (e.g. a Jetson + a laptop for USC AUV work)
where shell history alone loses the full debugging context.

- Rust workspace (`crates/memorywhale-core`, `crates/mw-cli`), Tauri desktop
  shell (`src-tauri`), and SQLite local DB.
- React + TypeScript frontend (`src/`, `index.html`).
- Everything stored locally; nothing is uploaded.

## Repo layout (IMPORTANT)
`main` is a Cargo workspace rooted at the repository top level. CLI binaries
live in `crates/mw-cli/src/bin/`, shared retrieval in
`crates/memorywhale-core/`, the frontend in `src/`, and the desktop shell in
`src-tauri/`.

> Gotcha for tools: run `git` from the **repo root**, or use `git ls-tree --full-tree …`.
> Running `git ls-tree` from a subdirectory only shows that subdir's slice and gives
> misleading "file not found / empty" results.

## Binaries (`crates/mw-cli/src/bin/`)
- `mw-remember` — manual single-command memory. You pass `--exit-code`, `--stdout`,
  `--stderr`, `--notes` and the command after `--`. It does NOT run the command; it just
  records what you tell it. Stores into the `command_runs` + `command_arguments` tables.
- `mw` — automatic **whole-session** recorder. Wraps the system `script` tool to capture an
  entire shell session (every command + real output) truthfully until you `exit`. Stores a
  raw transcript file under `<data_local>/MemoryWhale/sessions/` plus a cleaned, searchable
  transcript + metadata in the `sessions` table. Subcommands: `mw list`, `mw show <id>`,
  and `mw global on|off|status` (opt-in auto-record of every new terminal via a guarded
  shell-rc hook — no manual `.bashrc` editing).
- `mw-run`, `mw-screenshot` — additional helpers (screenshot capture is opt-in, local-only).
- `mw-serve`, `mw-view`, `mw-recover` — local dashboard, single-memory view,
  and interrupted-session recovery.
- `mw-mcp` — the six-tool MCP server.

> Note: `mw.rs`/`mw-run.rs` on `main` were consolidated/refactored during cleanup, so their
> exact structure may differ from intermediate versions discussed in chat. Read the current
> source before assuming behavior.

## Data locations (per machine, local only)
- DB: `<data_local>/MemoryWhale/memorywhale.sqlite3`
  - Linux/Jetson: `~/.local/share/MemoryWhale/…`  • macOS: `~/Library/Application Support/MemoryWhale/…`
- Session transcripts: `<data_local>/MemoryWhale/sessions/`
- Screenshots: `<data_local>/MemoryWhale/screenshots/`

Memory is **per-machine** and not synced — the Jetson and laptop each have their own DB.

## Build / run
```bash
export PATH="$HOME/.cargo/bin:$PATH"      # cargo may not be on the default PATH
cargo build --release -p memorywhale-cli --bins
cargo run -p memorywhale-cli --bin mw -- --notes "what I'm debugging"
# frontend (browser, demo store): npm install && npm run dev
# desktop app: npm run tauri:dev
```

## Conventions / decisions
- Commits in this repo have used author `Isabel Wu <wuisabel@usc.edu>` with **no**
  `Co-Authored-By` trailer (explicit user choice; overrides default).
- Privacy: recorders are **opt-in**, store **locally**, never upload. Transcripts/screenshots
  can contain secrets — keep that in mind before sharing.
- Don't commit directly to `main` for feature work unless explicitly asked; prefer a branch.

## Branch state (as of this note)
- `main` — canonical, complete, flattened. Use this.
- `memorywhale-test`, `feat/mw-terminal-recorder` — older **nested-layout** copies; largely
  superseded by `main`. A few files still differ from `main` (`index.html`, `README.md`,
  and `DEBUG.md` exists only on `memorywhale-test`) — reconcile before deleting them.
- `memorywhale-only` — constitution/docs work.

## Open items / TODO
- Decide whether `main`'s `index.html` + `README.md` or the `memorywhale-test` versions win,
  and whether `DEBUG.md` should be kept; then retire the redundant nested branches.
- Optional dev-environment cleanup: the primary working copy has lived on iCloud under a
  folder name with a trailing space (`…/GitHub/MemWhale /…`), which breaks paths. Consider
  re-cloning to a clean path like `~/code/MemWhale` and opening tools at the repo root.
- Possible features discussed (not built): a friendly local web view to render a
  memory/session for humans/agents; per-command structuring of `mw` sessions via shell hooks.
