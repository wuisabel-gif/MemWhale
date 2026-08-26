//! The one loader: build scorable [`Memory`] items from a MemoryWhale SQLite DB.
//!
//! Both surfaces call this — the desktop app (`documents` + `command_runs` +
//! `agent_turns`) and the CLI (`sessions` + `command_runs` + `bookmarks`). The
//! two databases have different tables, so every source is queried
//! independently and an absent optional table is treated as zero rows. Once a
//! source table is present, however, schema, query, and row errors are returned
//! to the caller instead of being mistaken for an empty source.
//!
//! Ids are namespaced per source so `explain(id)` stays stable and unique
//! across sources; [`decode_id`] recovers the source + original row id for
//! display (e.g. `mw replay <id>` / `mw show <id>`).

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use chrono::{DateTime, Utc};
use rusqlite::Connection;

use crate::Memory;

const CMD_NS: i64 = 1_000_000_000;
const TURN_NS: i64 = 2_000_000_000;
const NOTE_NS: i64 = 3_000_000_000;
const SESSION_NS: i64 = 4_000_000_000;

/// Which store a memory came from, plus its original per-table row id.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    Document,
    Command,
    Conversation,
    Note,
    Session,
}

impl Source {
    pub fn tag(self) -> &'static str {
        match self {
            Source::Document => "document",
            Source::Command => "command",
            Source::Conversation => "conversation",
            Source::Note => "note",
            Source::Session => "session",
        }
    }
}

/// Recover the source and original row id from a namespaced memory id.
pub fn decode_id(id: i64) -> (Source, i64) {
    if id >= SESSION_NS {
        (Source::Session, id - SESSION_NS)
    } else if id >= NOTE_NS {
        (Source::Note, id - NOTE_NS)
    } else if id >= TURN_NS {
        (Source::Conversation, id - TURN_NS)
    } else if id >= CMD_NS {
        (Source::Command, id - CMD_NS)
    } else {
        (Source::Document, id)
    }
}

fn parse_ts(ts: &str) -> DateTime<Utc> {
    match DateTime::parse_from_rfc3339(ts) {
        Ok(d) => d.with_timezone(&Utc),
        Err(e) => {
            // A malformed timestamp must not masquerade as `now()`: that would
            // give a corrupted row an unearned recency boost (recency halves by
            // age, so "now" ranks highest) and silently hide the corruption.
            // Fall back to the Unix epoch so the row sorts as oldest, and warn.
            eprintln!("memorywhale: unparseable timestamp {ts:?} ({e}); treating as epoch");
            DateTime::from_timestamp(0, 0).unwrap_or_default()
        }
    }
}

/// The stage at which loading a present source failed.
#[derive(Debug)]
pub enum LoadErrorKind {
    /// Reading SQLite schema metadata failed.
    Schema { source: rusqlite::Error },
    /// A present table has a shape outside the supported compatibility set.
    UnsupportedSchema { details: String },
    /// Preparing the source query failed. This includes an incompatible
    /// present-table schema (for example, a missing required column).
    Prepare { source: rusqlite::Error },
    /// Executing or advancing a source query failed.
    Query { source: rusqlite::Error },
    /// A row could not be decoded into the source's canonical memory shape.
    RowDecode { row: usize, source: rusqlite::Error },
}

/// A structured retrieval-loading failure.
///
/// `table` identifies the source whose evidence could not be read. Optional
/// source tables that are genuinely absent do not produce an error; this type
/// is reserved for failures after schema inspection finds a source, or for a
/// failure to inspect that schema.
#[derive(Debug)]
pub struct LoadError {
    pub table: &'static str,
    pub kind: LoadErrorKind,
}

impl LoadError {
    fn schema(table: &'static str, source: rusqlite::Error) -> Self {
        Self {
            table,
            kind: LoadErrorKind::Schema { source },
        }
    }

    fn prepare(table: &'static str, source: rusqlite::Error) -> Self {
        Self {
            table,
            kind: LoadErrorKind::Prepare { source },
        }
    }

    fn unsupported_schema(table: &'static str, details: impl Into<String>) -> Self {
        Self {
            table,
            kind: LoadErrorKind::UnsupportedSchema {
                details: details.into(),
            },
        }
    }

    fn query(table: &'static str, source: rusqlite::Error) -> Self {
        Self {
            table,
            kind: LoadErrorKind::Query { source },
        }
    }

    fn row_decode(table: &'static str, row: usize, source: rusqlite::Error) -> Self {
        Self {
            table,
            kind: LoadErrorKind::RowDecode { row, source },
        }
    }
}

impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            LoadErrorKind::Schema { source } => {
                write!(f, "failed to inspect {} schema: {source}", self.table)
            }
            LoadErrorKind::UnsupportedSchema { details } => {
                write!(f, "unsupported {} schema: {details}", self.table)
            }
            LoadErrorKind::Prepare { source } => {
                write!(
                    f,
                    "failed to prepare {} retrieval query: {source}",
                    self.table
                )
            }
            LoadErrorKind::Query { source } => {
                write!(f, "failed to query {}: {source}", self.table)
            }
            LoadErrorKind::RowDecode { row, source } => write!(
                f,
                "failed to decode row {} from {}: {source}",
                row + 1,
                self.table
            ),
        }
    }
}

impl Error for LoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match &self.kind {
            LoadErrorKind::Schema { source }
            | LoadErrorKind::Prepare { source }
            | LoadErrorKind::Query { source } => Some(source),
            LoadErrorKind::RowDecode { source, .. } => Some(source),
            LoadErrorKind::UnsupportedSchema { .. } => None,
        }
    }
}

/// Load everything MemoryWhale remembers as scorable memories, tolerant of
/// optional source tables being absent (so it serves both desktop and CLI
/// schemas). A present source that cannot be read is an error.
pub fn load_memories(conn: &Connection) -> Result<Vec<Memory>, LoadError> {
    let mut mems = Vec::new();
    mems.extend(documents(conn)?);
    mems.extend(command_runs(conn)?);
    mems.extend(agent_turns(conn)?);
    mems.extend(bookmarks(conn)?);
    mems.extend(sessions(conn)?);
    Ok(mems)
}

/// Check SQLite's schema catalogs rather than using a failed source prepare as
/// the missing-table signal. Include temporary tables because callers/tests may
/// use a temporary source on an otherwise ordinary connection.
fn table_exists(conn: &Connection, table: &'static str) -> Result<bool, LoadError> {
    conn.query_row(
        "SELECT EXISTS(
             SELECT 1 FROM sqlite_master
              WHERE type IN ('table', 'view') AND name COLLATE NOCASE = ?1
             UNION ALL
             SELECT 1 FROM sqlite_temp_master
              WHERE type IN ('table', 'view') AND name COLLATE NOCASE = ?1
         )",
        [table],
        |row| row.get::<_, i64>(0),
    )
    .map(|present| present != 0)
    .map_err(|source| LoadError::schema(table, source))
}

/// Read a present table's column names. The bookmark loader uses this metadata
/// to choose the supported legacy filtering shape before preparing SQL.
fn table_columns(conn: &Connection, table: &'static str) -> Result<Option<Vec<String>>, LoadError> {
    if !table_exists(conn, table)? {
        return Ok(None);
    }

    let pragma = "SELECT name FROM pragma_table_info(?1)";
    let mut stmt = conn
        .prepare(pragma)
        .map_err(|source| LoadError::schema(table, source))?;
    let mut rows = stmt
        .query([table])
        .map_err(|source| LoadError::schema(table, source))?;
    let mut columns = Vec::new();
    while let Some(row) = rows
        .next()
        .map_err(|source| LoadError::schema(table, source))?
    {
        columns.push(
            row.get::<_, String>(0)
                .map_err(|source| LoadError::schema(table, source))?,
        );
    }
    Ok(Some(columns))
}

/// Read a present source table. Keeping query execution and row decoding
/// separate means a database/locking failure while stepping the query is not
/// reported as a bad row.
fn load_rows_existing<T, F>(
    conn: &Connection,
    table: &'static str,
    sql: &str,
    mut decode: F,
) -> Result<Vec<T>, LoadError>
where
    F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
{
    let mut stmt = conn
        .prepare(sql)
        .map_err(|source| LoadError::prepare(table, source))?;
    let mut rows = stmt
        .query([])
        .map_err(|source| LoadError::query(table, source))?;
    let mut decoded = Vec::new();
    let mut row_number = 0;
    while let Some(row) = rows
        .next()
        .map_err(|source| LoadError::query(table, source))?
    {
        decoded
            .push(decode(row).map_err(|source| LoadError::row_decode(table, row_number, source))?);
        row_number += 1;
    }
    Ok(decoded)
}

/// Read an optional source table after its existence has been established from
/// schema metadata.
fn load_rows<T, F>(
    conn: &Connection,
    table: &'static str,
    sql: &str,
    decode: F,
) -> Result<Vec<T>, LoadError>
where
    F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
{
    if !table_exists(conn, table)? {
        return Ok(Vec::new());
    }
    load_rows_existing(conn, table, sql, decode)
}

/// Documents / notes (desktop). `text` = title + content.
fn documents(conn: &Connection) -> Result<Vec<Memory>, LoadError> {
    let rows = load_rows(
        conn,
        "documents",
        "SELECT id, title, content, source_type, created_at FROM documents",
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        },
    )?;
    Ok(rows
        .into_iter()
        .map(|(id, title, content, source_type, created)| {
            let when = parse_ts(&created);
            Memory {
                id,
                text: format!("{title}. {content}"),
                created_at: when,
                last_used: when,
                mentions: 1,
                importance: 0.5,
                tags: vec!["document".into(), source_type],
                embedding: None,
            }
        })
        .collect())
}

/// Command runs (both). Reinforcement = how often the same command recurs;
/// failures are more important than successes.
fn command_runs(conn: &Connection) -> Result<Vec<Memory>, LoadError> {
    let rows = load_rows(
        conn,
        "command_runs",
        "SELECT id, command, argv_json, notes, stderr, exit_code, created_at FROM command_runs",
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, Option<i64>>(5)?,
                r.get::<_, String>(6)?,
            ))
        },
    )?;
    let mut counts: HashMap<String, u32> = HashMap::new();
    for (_, cmd, ..) in &rows {
        *counts.entry(cmd.to_lowercase()).or_insert(0) += 1;
    }
    Ok(rows
        .into_iter()
        .map(
            |(id, command, argv_json, notes, stderr, exit_code, created)| {
                let when = parse_ts(&created);
                let ok = exit_code == Some(0);
                Memory {
                    id: CMD_NS + id,
                    text: format!("{command} {argv_json} {notes} {stderr}"),
                    created_at: when,
                    last_used: when,
                    mentions: *counts.get(&command.to_lowercase()).unwrap_or(&1),
                    importance: if exit_code.unwrap_or(0) != 0 {
                        0.65
                    } else {
                        0.4
                    },
                    tags: vec![
                        "command".into(),
                        if ok { "ok".into() } else { "error".into() },
                    ],
                    embedding: None,
                }
            },
        )
        .collect())
}

/// Agent conversation turns (desktop; written by Delphin / hooks).
fn agent_turns(conn: &Connection) -> Result<Vec<Memory>, LoadError> {
    let rows = load_rows(
        conn,
        "agent_turns",
        "SELECT id, ts, direction, text FROM agent_turns",
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        },
    )?;
    let turns: Vec<_> = rows
        .into_iter()
        .filter(|(_, _, _, t)| !t.trim().is_empty())
        .collect();
    let mut counts: HashMap<String, u32> = HashMap::new();
    for (_, _, _, t) in &turns {
        *counts.entry(t.trim().to_lowercase()).or_insert(0) += 1;
    }
    Ok(turns
        .into_iter()
        .map(|(id, ts, direction, text)| {
            let when = parse_ts(&ts);
            let importance = match direction.as_str() {
                "user" => 0.6_f32,
                "agent" => 0.45,
                _ => 0.3,
            };
            let mentions = *counts.get(&text.trim().to_lowercase()).unwrap_or(&1);
            Memory {
                id: TURN_NS + id,
                text,
                created_at: when,
                last_used: when,
                mentions,
                importance,
                tags: vec!["conversation".into(), direction],
                embedding: None,
            }
        })
        .collect())
}

/// Remembered lessons / bookmarks (CLI `mw mark` / `mw remember`).
fn bookmarks(conn: &Connection) -> Result<Vec<Memory>, LoadError> {
    // Review mode is enforced at write time (agent notes land with approved=0),
    // so every reader filters approved=1 when that column exists. Lifecycle
    // filtering is likewise applied only when the status column exists. Older
    // databases are selected from schema metadata, not by treating an
    // arbitrary prepare error as a legacy-schema signal.
    let Some(columns) = table_columns(conn, "bookmarks")? else {
        return Ok(Vec::new());
    };
    let has_column = |name: &str| {
        columns
            .iter()
            .any(|column| column.eq_ignore_ascii_case(name))
    };
    let has_approved = has_column("approved");
    let has_status = has_column("status");
    if has_status && !has_approved {
        return Err(LoadError::unsupported_schema(
            "bookmarks",
            "status exists without approved; review filtering cannot be enforced",
        ));
    }
    let sql = if has_approved && has_status {
        "SELECT id, label, created_at FROM bookmarks WHERE approved = 1 AND status = 'active'"
            .to_string()
    } else if has_approved {
        "SELECT id, label, created_at FROM bookmarks WHERE approved = 1".to_string()
    } else {
        "SELECT id, label, created_at FROM bookmarks".to_string()
    };
    let rows = load_rows_existing(conn, "bookmarks", &sql, |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })?;
    Ok(rows
        .into_iter()
        .filter(|(_, label, _)| !label.trim().is_empty())
        .map(|(id, label, created)| {
            let when = parse_ts(&created);
            Memory {
                id: NOTE_NS + id,
                text: label,
                created_at: when,
                last_used: when,
                mentions: 1,
                importance: 0.55,
                tags: vec!["note".into()],
                embedding: None,
            }
        })
        .collect())
}

/// Recorded terminal sessions (CLI). `text` = notes + cleaned transcript, so
/// the same content `mw search` used to LIKE-match still drives similarity.
fn sessions(conn: &Connection) -> Result<Vec<Memory>, LoadError> {
    let rows = load_rows(
        conn,
        "sessions",
        "SELECT id, notes, transcript, started_at FROM sessions",
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        },
    )?;
    Ok(rows
        .into_iter()
        .filter_map(|(id, notes, transcript, started)| {
            let text = format!("{notes}\n{transcript}").trim().to_string();
            if text.is_empty() {
                return None;
            }
            let when = parse_ts(&started);
            Some(Memory {
                id: SESSION_NS + id,
                text,
                created_at: when,
                last_used: when,
                mentions: 1,
                importance: 0.5,
                tags: vec!["session".into()],
                embedding: None,
            })
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE command_runs (id INTEGER PRIMARY KEY, command TEXT, argv_json TEXT,
                 notes TEXT, stderr TEXT, exit_code INTEGER, created_at TEXT);
             CREATE TABLE bookmarks (id INTEGER PRIMARY KEY, label TEXT, created_at TEXT);
             INSERT INTO command_runs VALUES
                 (1, 'cargo', '[\"cargo\",\"build\"]', '', 'error: linker failed', 1, '2026-06-20T12:00:00+00:00'),
                 (2, 'cargo', '[\"cargo\",\"build\"]', '', '', 0, '2026-06-26T12:00:00+00:00');
             INSERT INTO bookmarks VALUES
                 (1, 'the linker failure was a missing -lstdc++', '2026-06-25T12:00:00+00:00');",
        )
        .unwrap();
        conn
    }

    #[test]
    fn parse_ts_valid_roundtrips_and_malformed_falls_back_to_epoch() {
        // Valid RFC3339 parses to the same instant.
        assert_eq!(
            parse_ts("2026-06-20T12:00:00+00:00"),
            DateTime::parse_from_rfc3339("2026-06-20T12:00:00+00:00")
                .unwrap()
                .with_timezone(&Utc),
        );
        // Malformed must NOT become `now()` — it becomes the epoch, so a
        // corrupted row sorts as oldest instead of newest.
        let epoch = DateTime::from_timestamp(0, 0).unwrap();
        assert_eq!(parse_ts("not-a-timestamp"), epoch);
        assert_eq!(parse_ts(""), epoch);
    }

    #[test]
    fn tolerates_missing_tables_and_namespaces_ids() {
        let mems = load_memories(&fixture()).unwrap();
        // 2 command_runs + 1 bookmark; documents/agent_turns/sessions absent.
        assert_eq!(mems.len(), 3);
        assert!(mems.iter().any(|m| decode_id(m.id) == (Source::Command, 1)));
        assert!(mems.iter().any(|m| decode_id(m.id) == (Source::Note, 1)));
        // recurring command is reinforced.
        let cmd = mems
            .iter()
            .find(|m| decode_id(m.id) == (Source::Command, 1))
            .unwrap();
        assert_eq!(cmd.mentions, 2);
    }

    #[test]
    fn empty_database_has_no_optional_sources_and_no_error() {
        let conn = Connection::open_in_memory().unwrap();
        assert!(load_memories(&conn).unwrap().is_empty());
    }

    #[test]
    fn bookmarks_preserve_legacy_variants_and_current_filters() {
        // Pre-provenance bookmarks have neither filtering column and remain
        // readable as long as their canonical columns are present.
        let legacy = Connection::open_in_memory().unwrap();
        legacy
            .execute_batch(
                "CREATE TABLE bookmarks (id INTEGER PRIMARY KEY, label TEXT, created_at TEXT);
                 INSERT INTO bookmarks VALUES
                   (1, 'legacy lesson', '2026-01-01T00:00:00Z');",
            )
            .unwrap();
        let legacy_mems = load_memories(&legacy).unwrap();
        assert_eq!(legacy_mems.len(), 1);
        assert_eq!(legacy_mems[0].text, "legacy lesson");

        // The intermediate schema had approval but not lifecycle status.
        let approved_only = Connection::open_in_memory().unwrap();
        approved_only
            .execute_batch(
                "CREATE TABLE bookmarks (
                     id INTEGER PRIMARY KEY, label TEXT, created_at TEXT, approved INTEGER
                 );
                 INSERT INTO bookmarks VALUES
                   (1, 'approved lesson', '2026-01-01T00:00:00Z', 1),
                   (2, 'pending lesson', '2026-01-02T00:00:00Z', 0);",
            )
            .unwrap();
        let approved_mems = load_memories(&approved_only).unwrap();
        assert_eq!(approved_mems.len(), 1);
        assert_eq!(approved_mems[0].text, "approved lesson");

        // A status-only variant is unsupported: it could silently bypass
        // review-mode filtering, because no released migration creates it.
        let status_only = Connection::open_in_memory().unwrap();
        status_only
            .execute_batch(
                "CREATE TABLE bookmarks (
                     id INTEGER PRIMARY KEY, label TEXT, created_at TEXT, status TEXT
                 );
                 INSERT INTO bookmarks VALUES
                   (1, 'active lesson', '2026-01-01T00:00:00Z', 'active'),
                   (2, 'stale lesson', '2026-01-02T00:00:00Z', 'stale');",
            )
            .unwrap();
        let status_error = load_memories(&status_only).unwrap_err();
        assert!(matches!(
            status_error.kind,
            LoadErrorKind::UnsupportedSchema { .. }
        ));

        // A current schema applies both review and lifecycle predicates.
        let current = Connection::open_in_memory().unwrap();
        current
            .execute_batch(
                "CREATE TABLE bookmarks (
                     id INTEGER PRIMARY KEY, label TEXT, created_at TEXT,
                     approved INTEGER, status TEXT
                 );
                 INSERT INTO bookmarks VALUES
                   (1, 'active approved', '2026-01-01T00:00:00Z', 1, 'active'),
                   (2, 'pending active', '2026-01-02T00:00:00Z', 0, 'active'),
                   (3, 'released stale', '2026-01-03T00:00:00Z', 1, 'stale');",
            )
            .unwrap();
        let current_mems = load_memories(&current).unwrap();
        assert_eq!(current_mems.len(), 1);
        assert_eq!(current_mems[0].text, "active approved");
    }

    #[test]
    fn malformed_present_rows_return_a_structured_decode_error() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE command_runs (
                 id INTEGER PRIMARY KEY, command TEXT, argv_json TEXT,
                 notes TEXT, stderr TEXT, exit_code INTEGER, created_at TEXT
             );
             INSERT INTO command_runs VALUES
                 (1, NULL, '[]', '', '', 1, '2026-01-01T00:00:00Z');",
        )
        .unwrap();

        let error = load_memories(&conn).unwrap_err();
        assert_eq!(error.table, "command_runs");
        assert!(matches!(
            error.kind,
            LoadErrorKind::RowDecode { row: 0, .. }
        ));
        assert!(error.to_string().contains("command_runs"));
    }

    #[test]
    fn invalid_present_schema_returns_a_structured_prepare_error() {
        let conn = Connection::open_in_memory().unwrap();
        // `documents` is present, so its incompatible shape must not be
        // mistaken for the desktop source being absent.
        conn.execute_batch("CREATE TABLE documents (id INTEGER PRIMARY KEY, title TEXT);")
            .unwrap();

        let error = load_memories(&conn).unwrap_err();
        assert_eq!(error.table, "documents");
        assert!(matches!(error.kind, LoadErrorKind::Prepare { .. }));
        assert!(error.to_string().contains("documents"));
    }

    #[test]
    fn decode_roundtrips_each_source() {
        assert_eq!(decode_id(5), (Source::Document, 5));
        assert_eq!(decode_id(CMD_NS + 5), (Source::Command, 5));
        assert_eq!(decode_id(TURN_NS + 5), (Source::Conversation, 5));
        assert_eq!(decode_id(NOTE_NS + 5), (Source::Note, 5));
        assert_eq!(decode_id(SESSION_NS + 5), (Source::Session, 5));
    }

    // The CLI (`mw search`, `mw-mcp`) and the desktop Recall panel both build
    // their engine from *this* loader and score with a caller-supplied `now`.
    // So for a fixed fixture DB and a fixed `now` they must produce identical
    // rankings. This asserts that agreement directly, deterministically.
    #[test]
    fn cli_and_desktop_rank_identically_for_fixed_now() {
        use crate::engine::{BuiltinEngine, MemoryEngine};
        use crate::Query;
        use chrono::{TimeZone, Utc};

        let conn = fixture();
        let now = Utc.with_ymd_and_hms(2026, 6, 27, 12, 0, 0).unwrap();

        // Two independently constructed engines standing in for the two
        // surfaces; both go through the shared loader with the same `now`.
        let cli = BuiltinEngine::new(load_memories(&conn).unwrap());
        let desktop = BuiltinEngine::new(load_memories(&conn).unwrap());
        let q = Query::new("linker failure", now);

        let cli_rank: Vec<(i64, u32)> = cli
            .retrieve(&q, 20)
            .iter()
            .map(|s| (s.memory.id, s.percent()))
            .collect();
        let desktop_rank: Vec<(i64, u32)> = desktop
            .retrieve(&q, 20)
            .iter()
            .map(|s| (s.memory.id, s.percent()))
            .collect();

        assert_eq!(
            cli_rank, desktop_rank,
            "CLI and desktop must rank identically"
        );
        // The bookmark that literally names the linker failure should top it.
        assert_eq!(decode_id(cli_rank[0].0).0, Source::Note);
    }
}
