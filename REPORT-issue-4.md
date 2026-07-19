# Issue 4 — first-class scopes (`project`, `machine`)

Project grouping used to ride on a `project:<name>` string inside session notes,
parsed only by the dashboard. Search could not filter by project or machine.
This makes both first-class columns on `sessions` and wires them through the CLI,
the MCP server, and the retrieval engine.

## Migration 2

`migrate(conn)` in `crates/mw-cli/src/lib.rs` gains a `version < 2` block.
Migration 1 was not touched. The exact SQL executed:

```sql
-- guard: only when the sessions table exists (a bookmarks-only DB has none yet)
SELECT 1 FROM sqlite_master WHERE type='table' AND name = 'sessions';

-- per column, only when absent
SELECT 1 FROM pragma_table_info('sessions') WHERE name = 'project';
ALTER TABLE sessions ADD COLUMN project TEXT;

SELECT 1 FROM pragma_table_info('sessions') WHERE name = 'machine';
ALTER TABLE sessions ADD COLUMN machine TEXT;

-- backfill: lift the legacy note convention into the column, notes untouched
SELECT id, notes FROM sessions WHERE project IS NULL AND notes LIKE '%project:%';
UPDATE sessions SET project = ?1 WHERE id = ?2;   -- ?1 from /project:([\w.\-]+)/

PRAGMA user_version = 2;
```

Safe on a populated DB: `ADD COLUMN` with no default never rewrites rows, the
backfill only writes the new column, and `notes` is read-only throughout (a test
asserts the original notes string survives byte-for-byte).

## What changed per surface

**`crates/mw-cli/src/lib.rs`**
- `LATEST_SCHEMA_VERSION = 2`; migration 2 as above.
- `add_column_if_missing` now takes a table name (was bookmarks-only).
- `project_of(notes)` — the `project:<name>` regex, same shape the dashboard
  has always used, so the column and the note convention can't disagree.
- `machine_name()` — `MW_MACHINE`, else `machine = "..."` in
  `<data dir>/config.toml`, else the `hostname` command's first label, else
  `HOSTNAME`, else `"unknown"`.
- `parse_since("7d" | "24h" | "2w" | "30m")` -> `chrono::Duration`.
- `scope_memories(conn, mems, project, machine, since)` — narrows the loaded
  memory set *before* the engine sees it, so ranking happens within the scope
  instead of being trimmed afterwards. **With no filters it returns the input
  untouched**, which is what keeps unscoped `mw search` byte-identical.
  Scope matches memories that actually carry one: sessions (the new columns)
  and, for `project` alone, command runs still tagged the legacy way in notes.
  Notes/documents/turns have no scope and drop out of a scoped search.

**`crates/mw-cli/src/bin/mw.rs`**
- `open_session_db()` now runs `migrate`, so every command sees both the
  provenance and the scope columns.
- Both session inserts record `project` (parsed from the notes being written)
  and `machine` (`machine_name()`) at capture time.
- New `Scope` struct parses `--project` / `--machine` / `--since` off any
  command line and fails fast on a bad window. `--project project:demo` and
  `--project demo` are equivalent.
- `mw search` accepts all three, and feeds project + machine into the engine
  `Query` as task tags so task-relevance scoring benefits.
- `mw list` accepts all three, filtered in SQL.
- Help text updated for both.

**`crates/mw-cli/src/bin/mw-mcp.rs`** (additive schema changes only)
- `search_memory` gains optional `project` and `machine`.
- `get_context` gains optional `machine`; its existing `project` now really
  scopes the result set (it accepts `demo` or the historical `project:demo`).
- Both feed the scope into the `Query` task tags. Provenance rendering and the
  shared loader's approved-filter are untouched.

## Verification

`cargo build --workspace` and `cargo test --workspace` both pass.

New tests in `crates/mw-cli/src/lib.rs`:
- `migration_2_lifts_project_notes_into_a_column` — full round trip on a
  populated pre-scope DB: legacy notes in, column backfilled, notes preserved,
  scoped retrieval through the shared loader returns only the scoped session,
  unscoped returns everything, migration idempotent.
- `since_parses_relative_windows` — `7d`/`24h`/`2w`/`30m` plus the rejects
  (`7`, `d`, empty, `7y`, `-1d`, `1.5d`, `seven days`).
- `project_tag_is_parsed_out_of_notes`.

Manual run against a scratch DB seeded as a legacy pre-migration database
(`MEMORYWHALE_DATA_DIR` pointing at a temp dir, two sessions whose only project
marker is the note string):

```
--- mw search x --project foo ---
# matches for "x"  (ranked)

 28%  [session] project:foo os:macos  — `mw show 1`

--- mw search x (unscoped) ---
# matches for "x"  (ranked)

 33%  [session] project:foo os:macos  — `mw show 1`
 32%  [session] project:bar os:macos  — `mw show 2`

--- mw list --project bar ---
#2	2026-07-17T10:00:00+00:00	0 bytes	project:bar os:macos

--- mw list --since 7d ---
#1	2026-07-18T10:00:00+00:00	0 bytes	project:foo os:macos
#2	2026-07-17T10:00:00+00:00	0 bytes	project:bar os:macos

--- notes + new columns preserved (id|notes|project|machine|user_version) ---
1|project:foo os:macos|foo||2
2|project:bar os:macos|bar||2

--- bad --since ---
mw: invalid --since "7y"; use e.g. 7d, 24h, 2w
```

MCP, same DB:

```
{"name":"search_memory","arguments":{"query":"x","project":"bar"}}
-> - [session #2] 27% — project:bar os:macos
     reasons: last used 2 days ago; importance 0.50; mentioned 1x
```

Backfilled historical rows have a NULL `machine` — the machine they were
recorded on isn't recoverable. New captures carry it.

## Skipped

- The dashboard (`mw-serve`) still parses `project:` out of notes rather than
  reading the column. It keeps working unchanged; switch it over when the column
  is guaranteed populated everywhere.
- `command_runs` gets no `machine` column — a machine filter therefore excludes
  command runs. Add it when someone wants per-machine command history.
- No scope on `bookmarks`/notes. Add when a remembered lesson needs to be
  project-local rather than global.
