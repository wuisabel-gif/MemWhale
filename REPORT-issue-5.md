# Issue 5 — Parent-death cleanup for recorder children

**Branch:** `issue/5-parent-death-cleanup`

## Problem

`mw --live` inserts a `status='recording'` session row and then blocks inside the
interactive `script` child. If the process that launched `mw` dies, `mw` is
orphaned and the row is stranded as `recording`. Today it only gets finalized on
the dashboard's *next startup* (`mw-serve` -> `recover_orphans`). Goal: the child
detects parent death and finalizes **immediately**.

## Mechanism (per platform)

A single reusable guard — `memorywhale_cli::guard_parent_death(finalize)` in
`crates/mw-cli/src/lib.rs` — spawns a background watcher and, on parent death,
runs `finalize` then `exit(0)`:

- **Linux:** `prctl(PR_SET_PDEATHSIG, SIGTERM)`. SIGTERM is blocked process-wide
  first (before any other thread spawns) and a dedicated thread `sigwait`s for
  it, so `finalize` runs on a normal thread — SQLite is not async-signal-safe.
  The race where the parent dies *before* `prctl` is handled by re-checking
  `getppid()` immediately after and finalizing on the spot if it changed.
- **macOS / other Unix (no pdeathsig):** if a cooperating parent handed us the
  read end of a pipe via the `MW_PDEATH_FD` env var, a thread blocks reading it
  and treats **EOF** (parent closed its write end) as parent death. If no pipe
  was provided — the real case for an arbitrary terminal/shell parent — it falls
  back to polling `getppid()` for reparenting. (The getppid poll is a small
  addition beyond the task's pipe spec; without it `mw --live` would gain nothing
  on macOS under a normal shell parent, which never holds an MW pipe.)
- **Windows:** untouched — the guard is `#[cfg(unix)]` and the mw wiring is
  `#[cfg(unix)]`, so Windows behavior is unchanged.

On death the guard reuses the **existing** finalization path
(`update_session_from_transcript(id, path, ended_at, "interrupted")` in
`mw.rs`) — it flushes the current transcript to the row and sets the status.
Nothing was duplicated.

## Idempotency / existing signal handling

- `interrupted` is an existing status value (already produced by `mw-recover`
  and rendered by `mw-serve`). Finalization is idempotent: the guard flips an
  existing row, and startup `recover_orphans` skips any transcript that already
  has a session row — so the signal path and startup recovery can both run
  without producing duplicates.
- There was no prior explicit SIGINT/SIGTERM handler in the code (`script`
  handles interactive signals inside the recorded shell; normal exit flows
  through unchanged). The Linux guard only claims SIGTERM, and only within
  `mw --live`; SIGINT and all other binaries are unaffected.

## Scope

Only `mw --live` owns a long-lived in-progress row, so only it wires the guard.
`mw-run` finalizes synchronously (no stranded row) and `mw-serve` owns no session
row, so neither needed wiring.

## Migration

**No migration.** No schema change — the `sessions.status` column and the
`interrupted` value already exist.

## Changed files

- `crates/mw-cli/Cargo.toml` — added `libc` (cfg(unix) dep) and cfg(unix)
  dev-deps (`libc`, `rusqlite`) for the integration test.
- `crates/mw-cli/src/lib.rs` — new `guard_parent_death` + `PDEATH_FD_ENV`.
- `crates/mw-cli/src/bin/mw.rs` — install the guard for `mw --live`.
- `crates/mw-cli/tests/parent_death.rs` — new integration test.

## Verification

- `cargo build --workspace` — passes.
- `cargo test --workspace` — passes. New integration test
  `interrupted_on_parent_death_via_pipe_eof` builds a 3-generation process tree
  (test -> middle -> recorder), SIGKILLs the middle, and asserts the recorder
  finalizes its row to `interrupted` within a 5s timeout. It exercises the
  **pipe/EOF fallback** (the path that runs on this darwin box) and passed.
  A companion `interrupted_on_parent_death_via_prctl` test is gated to
  `cfg(target_os = "linux")` for the prctl path (not run on darwin).
