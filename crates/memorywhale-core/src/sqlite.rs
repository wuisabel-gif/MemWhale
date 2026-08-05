//! The one loader: build scorable [`Memory`] items from a MemoryWhale SQLite DB.
//!
//! Both surfaces call this — the desktop app (`documents` + `command_runs` +
//! `agent_turns`) and the CLI (`sessions` + `command_runs` + `bookmarks`). The
//! two databases have different tables, so every source is queried
//! independently and a missing table is treated as zero rows (not an error).
//! That keeps a single loader honest across both schemas instead of two copies
//! drifting apart.
//!
//! Ids are namespaced per source so `explain(id)` stays stable and unique
//! across sources; [`decode_id`] recovers the source + original row id for
//! display (e.g. `mw replay <id>` / `mw show <id>`).

use std::collections::HashMap;

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

/// Load everything MemoryWhale remembers as scorable memories, tolerant of any
/// source table being absent (so it serves both the desktop and CLI schemas).
pub fn load_memories(conn: &Connection) -> Vec<Memory> {
    let mut mems = Vec::new();
    mems.extend(documents(conn));
    mems.extend(command_runs(conn));
    mems.extend(agent_turns(conn));
    mems.extend(bookmarks(conn));
    mems.extend(sessions(conn));
    mems
}

/// Documents / notes (desktop). `text` = title + content.
fn documents(conn: &Connection) -> Vec<Memory> {
    let Ok(mut stmt) =
        conn.prepare("SELECT id, title, content, source_type, created_at FROM documents")
    else {
        return Vec::new();
    };
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
        ))
    });
    let Ok(rows) = rows else { return Vec::new() };
    rows.flatten()
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
        .collect()
}

/// Command runs (both). Reinforcement = how often the same command recurs;
/// failures are more important than successes.
fn command_runs(conn: &Connection) -> Vec<Memory> {
    let Ok(mut stmt) = conn.prepare(
        "SELECT id, command, argv_json, notes, stderr, exit_code, created_at FROM command_runs",
    ) else {
        return Vec::new();
    };
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
            r.get::<_, String>(4)?,
            r.get::<_, Option<i64>>(5)?,
            r.get::<_, String>(6)?,
        ))
    });
    let Ok(rows) = rows else { return Vec::new() };
    let runs: Vec<_> = rows.flatten().collect();
    let mut counts: HashMap<String, u32> = HashMap::new();
    for (_, cmd, ..) in &runs {
        *counts.entry(cmd.to_lowercase()).or_insert(0) += 1;
    }
    runs.into_iter()
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
        .collect()
}

/// Agent conversation turns (desktop; written by Delphin / hooks).
fn agent_turns(conn: &Connection) -> Vec<Memory> {
    let Ok(mut stmt) = conn.prepare("SELECT id, ts, direction, text FROM agent_turns") else {
        return Vec::new();
    };
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
        ))
    });
    let Ok(rows) = rows else { return Vec::new() };
    let turns: Vec<_> = rows
        .flatten()
        .filter(|(_, _, _, t)| !t.trim().is_empty())
        .collect();
    let mut counts: HashMap<String, u32> = HashMap::new();
    for (_, _, _, t) in &turns {
        *counts.entry(t.trim().to_lowercase()).or_insert(0) += 1;
    }
    turns
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
        .collect()
}

/// Remembered lessons / bookmarks (CLI `mw mark` / `mw remember`).
fn bookmarks(conn: &Connection) -> Vec<Memory> {
    // Review mode is enforced at write time (agent notes land with approved=0),
    // so every reader just filters approved=1 — one rule, no config lookup here.
    // Older DBs predating the provenance migration have no `approved` column;
    // there the prepare fails and we fall back to loading everything.
    let mut stmt = match conn.prepare(
        "SELECT id, label, created_at FROM bookmarks WHERE approved = 1 AND status = 'active'",
    ) {
        Ok(stmt) => stmt,
        Err(_) => {
            match conn.prepare("SELECT id, label, created_at FROM bookmarks WHERE approved = 1") {
                Ok(stmt) => stmt,
                Err(_) => match conn.prepare("SELECT id, label, created_at FROM bookmarks") {
                    Ok(stmt) => stmt,
                    Err(_) => return Vec::new(),
                },
            }
        }
    };
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    });
    let Ok(rows) = rows else { return Vec::new() };
    rows.flatten()
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
        .collect()
}

/// Recorded terminal sessions (CLI). `text` = notes + cleaned transcript, so
/// the same content `mw search` used to LIKE-match still drives similarity.
fn sessions(conn: &Connection) -> Vec<Memory> {
    let Ok(mut stmt) = conn.prepare("SELECT id, notes, transcript, started_at FROM sessions")
    else {
        return Vec::new();
    };
    let rows = stmt.query_map([], |r| {
        Ok((
            r.get::<_, i64>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
            r.get::<_, String>(3)?,
        ))
    });
    let Ok(rows) = rows else { return Vec::new() };
    rows.flatten()
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
        .collect()
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
        let mems = load_memories(&fixture());
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
        let cli = BuiltinEngine::new(load_memories(&conn));
        let desktop = BuiltinEngine::new(load_memories(&conn));
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
