# Issue 8 — Always-on lightweight capture via shell hooks

## What shipped

`mw hooks install` / `mw hooks uninstall`: an Atuin-style always-on tier that
records **command, cwd, exit code, duration** — never output — into the same
`command_runs` table, marked `capture_kind = 'hook'`.

## The exact rc-file block

`mw hooks install` writes the generated hook script to
`<data dir>/memorywhale.sh` (or `memorywhale.fish`) and appends exactly this to
`~/.zshrc`, `~/.bashrc`, or `~/.config/fish/config.fish`:

```sh
# >>> memorywhale shell hooks >>>
# Managed by `mw hooks` — edit above/below, not inside.
[ -f "/Users/you/Library/Application Support/MemoryWhale/memorywhale.sh" ] && . "/Users/you/Library/Application Support/MemoryWhale/memorywhale.sh"
# <<< memorywhale shell hooks <<<
```

fish gets the same delimiters with a fish source line:

```fish
# >>> memorywhale shell hooks >>>
# Managed by `mw hooks` — edit above/below, not inside.
test -f "/Users/you/.../memorywhale.fish"; and source "/Users/you/.../memorywhale.fish"
# <<< memorywhale shell hooks <<<
```

Nothing else is sourced into the user's shell. Install strips any existing
block before appending, so it is idempotent; uninstall removes exactly the
lines between (and including) the two markers and deletes the generated script.

## Schema

**Migration 3** — `command_runs.capture_kind TEXT NOT NULL DEFAULT 'full'`.
`LATEST_SCHEMA_VERSION` bumped 2 → 3. Migrations 1 and 2 untouched (migration 2
now pins `user_version = 2` explicitly instead of jumping to the constant).

Because half a dozen binaries still create `command_runs` with their own
`CREATE TABLE IF NOT EXISTS`, a database can reach version 3 *before* the table
exists. `memorywhale_cli::ensure_capture_kind(conn)` (the body of migration 3)
is public and called unconditionally by `mw-remember` before it inserts, so the
column is always there at the write site.

## Write path

The hook shells out to the existing `mw-remember` binary — no new binary, no
new dependency — with a new `--capture-kind` flag. `mw-remember` already
consults `capture_rule_for(cwd)` **before opening the database**, so an `off`
directory produces zero rows and `commands-only` is exactly this tier. The call
is fire-and-forget in a detached subshell, so prompt latency is a `fork`, not a
database round-trip.

## Double-capture guard

The `mw` wrapper already exports `MW_RECORDING=1` into the recorded session
(`bin/mw.rs`, both the `script` spawn and the global-hook `exec` path). The hook
returns early when it sees it. No new env var was needed — `MW_RECORDING` is set
on the child environment of the pty, so every shell inside a full capture
session inherits it, whereas `MW_TRANSCRIPT` is a path that other tooling reads
and is a weaker signal of intent.

## Safety properties

- Every failure path returns 0: missing binary, unreadable rc, locked DB, bad
  path. The write itself is detached and its output discarded.
- `$?` is preserved: both `__mw_precmd` implementations capture `$?` first and
  `return $code`; fish's `__mw_postexec` does the same with `$status`.
- Interactive shells only, guarded against double-loading.
- `MW_HOOK_OFF=1` disables it for one shell.

## Tests

`crates/mw-cli/tests/shell_hooks.rs` — runs a real `bash -i` against a sandbox
`HOME` and `MEMORYWHALE_DATA_DIR`:

1. `hook_records_command_cwd_and_exit_code` — asserts the managed block is
   present and non-duplicated after two installs, then asserts a row with
   `capture_kind='hook'`, the right cwd, and exit code 7.
2. `wrapper_session_suppresses_hook_rows` — `MW_RECORDING=1` set → zero rows.
3. `off_gated_directory_records_nothing` — `.mwignore` with `capture = "off"` →
   zero rows.

`cargo build --workspace` and `cargo test --workspace` both pass.

## Bugs found while verifying (fixed here)

Running the scripted bash test in a loop exposed three real races, all of which
would have silently eaten memories in production:

1. **No `busy_timeout` in `mw-remember`.** Two hooks firing back to back both
   write; the loser hit "database is locked" and dropped its row. Now
   `PRAGMA busy_timeout = 3000`.
2. **Fresh-database creation race.** Concurrent `PRAGMA journal_mode = WAL` and
   `ALTER TABLE` on a brand-new file lose in ways `busy_timeout` doesn't cover.
   `open_ready()` now retries open + schema up to 5 times with a short backoff.
   `add_column_if_missing` also treats "duplicate column name" as success — two
   writers can both read a column as missing and both try to add it.
3. **The bash `DEBUG` trap is live for the rest of the sourced file**, so the
   `case` that wires `PROMPT_COMMAND` latched *itself* as "the user's command"
   and a bogus `case` row was recorded — and, worse, the latch then blocked the
   real command. Fixed with an `__MW_IN_HOOK` re-entrancy guard, a `__MW_*`
   ignore pattern, and clearing the latch at the end of the file.

## Follow-ups

- **PowerShell** is deliberately out of scope for this issue; needs its own
  `prompt`-function hook and a `Microsoft.PowerShell_profile.ps1` managed block.
- Duration is second-granularity (`$SECONDS` / `CMD_DURATION`), recorded in the
  notes as `dur:Ns`. A dedicated `duration_ms` column is the upgrade path if
  sub-second timing turns out to matter.
