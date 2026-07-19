# Issue 7 — Memory provenance + optional review

Branch: `issue/7-memory-provenance`

## Migration

No migration framework existed before; schema was scattered `CREATE TABLE IF
NOT EXISTS`. Introduced a `PRAGMA user_version`-based runner in
`crates/mw-cli/src/lib.rs::migrate()`.

Migration number chosen: 1 (`PRAGMA user_version = 1`). Applied when
`user_version < 1` against the `bookmarks` table (the remembered-lessons store
used by `mw mark`, `mw remember`, and the MCP `remember` tool). Exact SQL:

```sql
-- ensure base bookmarks table exists first, then:
ALTER TABLE bookmarks ADD COLUMN author_kind TEXT NOT NULL DEFAULT 'human';
ALTER TABLE bookmarks ADD COLUMN author_name TEXT;
ALTER TABLE bookmarks ADD COLUMN source_session_id INTEGER;
ALTER TABLE bookmarks ADD COLUMN approved INTEGER NOT NULL DEFAULT 1;
ALTER TABLE bookmarks ADD COLUMN created_at TEXT;  -- only if missing; base already has it
PRAGMA user_version = 1;
```

Each ADD COLUMN is guarded by a `pragma_table_info` existence check, so it is
safe to re-run and safe whether the table was just created or pre-existing.
Existing rows backfill automatically via the constant DEFAULTs
(author_kind='human', approved=1) — ADD COLUMN with a constant default does not
rewrite rows, safe on a populated DB. RENUMBER NOTE: if merging collides on
user_version=1, bump both the `< 1` check and the `PRAGMA user_version = N`
write in migrate() together.

## Changed files

- crates/mw-cli/src/lib.rs — migrate(), add_column_if_missing(),
  review_agent_memories() (config flag: env MEMORYWHALE_REVIEW_AGENT_MEMORIES=1
  or `review_agent_memories = true` in <data dir>/config.toml; default OFF; no
  new dep), provenance_label(), and remember_as(); remember() now delegates to
  remember_as with human defaults.
- crates/mw-cli/src/bin/mw-mcp.rs — capture clientInfo.name from initialize,
  thread through tools/call; remember tool → remember_as agent + client name;
  search_memory/get_context render provenance and filter approved=1; open()
  runs migrate. Tool schemas unchanged (additive requirement met).
- crates/mw-cli/src/bin/mw.rs — mw search Notes shows provenance, filters approved=1.
- src-tauri/Cargo.toml — added memorywhale-cli path dep (reuse migrate/provenance).
- src-tauri/src/lib.rs — Lesson struct + tauri commands list_lessons(agent_only),
  delete_lesson(id), approve_lesson(id), registered in generate_handler!.
- src/App.tsx — "Remembered lessons" panel: agent-only filter, provenance line,
  Delete button, Approve for pending-review lessons.

## Decisions

- Single choke point: CLI and MCP writes both go through remember_as; provenance
  set in one place.
- Review-mode exclusion enforced at read time via WHERE approved=1 on retrieval
  surfaces (mw search, MCP search_memory/get_context). Diagnostic surfaces
  (mw doctor, mw export) left unfiltered (not retrieval).
- Human memories store author_name=NULL, displayed as "you".

## Verification

- cargo build --workspace: OK.
- cargo test --workspace: OK. mw-cli suite 12 passed / 0 failed. New tests:
  cli_remember_is_human, mcp_remember_is_agent_attributed,
  review_mode_hides_unapproved_agent_memories,
  migrate_backfills_existing_rows_as_human, provenance_label_formats.
- Frontend TS not typechecked (no node_modules / tsc); not part of cargo verify.
- No pre-existing unrelated failures observed.
