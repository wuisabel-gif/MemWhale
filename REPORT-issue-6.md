# Issue 6 — Capture gates

Config-driven capture modes per path prefix, enforced **before** anything reaches
SQLite. Previously the only privacy layer was the pattern-based secret scrubber
(`redact`), which runs on content that is already on its way into the database.

## Capture modes

| Mode | Stored |
| --- | --- |
| `full` | Everything: command, argv, cwd, exit code, stdout/stderr, session transcript. The default — unchanged when no gate is configured. |
| `commands-only` | Command, argv, cwd, exit code, timestamps (start/end, so duration survives). No stdout, stderr, or transcript. |
| `off` | Nothing. No row, no transcript file. |

Accepted spellings: `full`, `commands-only` (or `commands_only`), `off` (or
`none`). Case-insensitive, surrounding quotes optional.

## Config format

### `.mwignore` (per directory tree)

A file named `.mwignore` at the root of a tree. Same hand-rolled `key = "value"`
line format as `config.toml`; `#` starts a comment. Only `capture` is read:

```toml
# ~/finances/.mwignore
capture = "off"
```

It applies to that directory and everything below it.

### `[capture.paths]` in `<data dir>/config.toml`

```toml
machine = "laptop"
review_agent_memories = true

[capture.paths]
"~/finances" = "off"
"~/work/client-repo" = "commands-only"
"/srv/logs" = "full"
```

Keys are path prefixes (leading `~/` expanded). Values are capture modes.

### Resolution order

1. The nearest `.mwignore` walking **up** from the working directory (first one
   found with a valid `capture` line wins).
2. The **longest** matching prefix in `[capture.paths]`.
3. Default `full`.

Both sides of every prefix comparison are canonicalized (`fs::canonicalize`), so
a symlink pointing into a gated tree cannot dodge the gate. A path that does not
exist falls back to a literal comparison.

## What is gated

All ingestion paths consult `memorywhale_cli::capture_rule(cwd)` before opening
the database:

| Path | Behaviour |
| --- | --- |
| `mw` / `mw --live` (`bin/mw.rs`, `record_session`) | `off` → runs a plain interactive shell (`mw` is `exec`d by the global hook, so it must still leave a usable shell) and records nothing. `commands-only` → `script` writes to a scratch file in the temp dir, deleted on exit; the session row is inserted with an empty transcript and an empty `transcript_path`. The live-sync thread honours the same flag. |
| `mw-run` (`bin/mw-run.rs`) | Output is always shown live. `off` → no insert at all. `commands-only` → insert with empty `stdout`/`stderr`, exit code kept. |
| `mw-remember` (`bin/mw-remember.rs`, the shell hook in `linux/shell/memorywhale.sh`) | `off` → returns before the DB is opened. `commands-only` → clears `stdout`/`stderr`. |
| `remember_as` in `lib.rs` | The single choke point for every note write (`mw mark`, `mw remember`, MCP `remember`, desktop). `off` → errors out before opening the DB. Gating here rather than at each caller means new note-writing surfaces are gated by default. |

`commands-only` never suppresses exit codes or timing: `command_runs` keeps
`exit_code` and `created_at`; `sessions` keeps `started_at`/`ended_at`.

## `mw status`

`mw status` (alias for the existing `mw global status`) now also prints:

```
capture mode here (/Users/me/finances): off
  rule: /Users/me/finances/.mwignore
```

The `rule` line names the exact `.mwignore` path or the exact
`config.toml [capture.paths]` key responsible.

## `mw prune --older-than`

Extends the existing `mw prune`. Deletes sessions (plus their transcript files)
and command runs older than a relative window, reusing `parse_since` from issue
4 — so `7d`, `24h`, `2w`, `30m` all work:

```bash
mw prune --older-than 90d --dry-run   # lists what would go
mw prune --older-than 90d
```

## Schema

No migration added. `LATEST_SCHEMA_VERSION` stays at `2` — gating is a write-time
decision and needs no new columns.

## Verification

`cargo build --workspace` and `cargo test --workspace` both pass (20 mw-cli unit
tests, no pre-existing failures). New tests in `crates/mw-cli/src/lib.rs`:

- `capture_modes_parse` — the three modes and their storage predicates.
- `global_config_matches_longest_path_prefix` — longest-prefix wins; a top-level
  `capture = ...` key does not leak into the `[capture.paths]` section.
- `mwignore_beats_global_config_beats_default` — full precedence chain, each
  answer naming its rule.
- `off_directory_writes_zero_rows` — an `off` directory produces zero rows, while
  an ungated directory in the same run still records (no regression).
- `symlinked_cwd_still_hits_the_gate` — a symlink into a gated tree resolves to
  the gate, on both the query and the write path.

Also smoke-tested end to end: `mw-run` in an `off` directory writes nothing, in a
`commands-only` directory writes `echo|0|''`, and `mw status` reports the rule.

## Deliberate simplifications

- Under `commands-only`, a full `mw` session stores metadata with an **empty**
  transcript rather than trying to extract just the command lines from a `script`
  transcript — that needs prompt-boundary heuristics and would be unreliable.
  Per-command capture in that mode is what the shell hook (`mw-remember`) is for.
- No new TOML dependency; parsing follows the existing hand-rolled line parsing
  next to `config_value`/`machine_name`.
