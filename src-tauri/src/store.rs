//! What the desktop app reads from the shared store: the same SQLite file the
//! CLI and the agent hooks write. Every string that came from a capture is
//! redacted again before it reaches the window.

use crate::AppState;
use chrono::Utc;
use memorywhale_cli::{redact, RecallEvent, RecallStats};
use memorywhale_core::engine::MemoryEngine;
use memorywhale_core::provenance;
use rusqlite::{params, Connection};
use serde::Serialize;

type Res<T> = Result<T, String>;

fn db(state: &AppState) -> Res<std::sync::MutexGuard<'_, Connection>> {
    state
        .db
        .lock()
        .map_err(|_| "database lock poisoned".to_string())
}

fn sql_err(e: rusqlite::Error) -> String {
    e.to_string()
}

/// Open the store the CLI uses (creating it on a fresh machine, private to
/// this user) and add the app's embedding cache.
pub fn open() -> Res<Connection> {
    let path = memorywhale_cli::database_path()?;
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir)
            .map_err(|e| format!("failed to create {}: {e}", dir.display()))?;
        memorywhale_cli::restrict_path_permissions(dir, true)?;
    }
    let conn = memorywhale_cli::storage::open_path(&path)?;
    memorywhale_cli::restrict_path_permissions(&path, false)?;
    // Semantic search caches Ollama vectors here; the CLI never reads it.
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS memory_embeddings (
            memory_id INTEGER NOT NULL,
            model TEXT NOT NULL,
            text_hash TEXT NOT NULL,
            vec BLOB NOT NULL,
            PRIMARY KEY (memory_id, model)
        );",
    )
    .map_err(sql_err)?;
    Ok(conn)
}

// ── Overview ─────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct Overview {
    commands: i64,
    failures: i64,
    sessions: i64,
    lessons: i64,
    pending_lessons: i64,
    /// Commands per agent label (`terminal` for a person at a prompt).
    agents: Vec<(String, i64)>,
    recall: RecallStats,
    store: String,
}

fn overview_of(conn: &Connection) -> Res<Overview> {
    let count = |sql: &str| {
        conn.query_row(sql, [], |r| r.get::<_, i64>(0))
            .map_err(sql_err)
    };
    let rows = conn
        .prepare("SELECT agent, COUNT(*) FROM command_runs GROUP BY agent")
        .map_err(sql_err)?
        .query_map([], |r| {
            Ok((r.get::<_, Option<String>>(0)?, r.get::<_, i64>(1)?))
        })
        .map_err(sql_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_err)?;
    let mut agents: Vec<(String, i64)> = Vec::new();
    for (agent, n) in rows {
        let label = provenance::label(agent.as_deref());
        match agents.iter_mut().find(|(l, _)| l == label) {
            Some((_, total)) => *total += n,
            None => agents.push((label.to_string(), n)),
        }
    }
    agents.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    Ok(Overview {
        commands: count("SELECT COUNT(*) FROM command_runs")?,
        failures: count("SELECT COUNT(*) FROM command_runs WHERE exit_code <> 0")?,
        sessions: count("SELECT COUNT(*) FROM sessions")?,
        lessons: count("SELECT COUNT(*) FROM bookmarks")?,
        pending_lessons: count("SELECT COUNT(*) FROM bookmarks WHERE approved = 0")?,
        agents,
        recall: memorywhale_cli::recall_stats(conn),
        store: memorywhale_cli::database_path()
            .map(|p| p.display().to_string())
            .unwrap_or_default(),
    })
}

#[tauri::command]
pub fn overview(state: tauri::State<AppState>) -> Res<Overview> {
    overview_of(&*db(&state)?)
}

/// A note or warning plus the program it was about (`cargo build`), which
/// reads better than an agent's whole one-off command line.
#[derive(Debug, Serialize)]
pub struct Event {
    #[serde(flatten)]
    pub event: RecallEvent,
    pub stem: String,
}

/// The newest `limit` notes and warnings, redacted, each with its program.
pub(crate) fn events(conn: &Connection, limit: usize) -> Res<Vec<Event>> {
    let mut stem_of = conn
        .prepare(
            "SELECT mw_stem(r.argv_json) FROM recall_hints h
             JOIN command_runs r ON r.id = h.run_id WHERE h.id = ?1",
        )
        .map_err(sql_err)?;
    memorywhale_cli::recall_history(conn, limit)?
        .into_iter()
        .map(|mut event| {
            event.command = redact(&event.command);
            event.text = event.text.as_deref().map(redact);
            let stem: String = stem_of
                .query_row([event.id], |r| r.get(0))
                .unwrap_or_default();
            Ok(Event {
                stem: redact(&stem),
                event,
            })
        })
        .collect()
}

/// Every note and warning an agent was shown, newest first, with its outcome.
#[tauri::command]
pub fn recall_history(state: tauri::State<AppState>, limit: Option<usize>) -> Res<Vec<Event>> {
    events(&*db(&state)?, limit.unwrap_or(100))
}

// ── Command runs ─────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct Run {
    id: i64,
    command: String,
    cwd: Option<String>,
    agent: &'static str,
    exit_code: Option<i64>,
    created_at: String,
    notes: String,
}

#[derive(Debug, Serialize)]
pub struct RunDetail {
    run: Run,
    stdout: String,
    stderr: String,
    /// The next runs in the same directory: often the fix.
    next: Vec<Run>,
}

const RUN_COLUMNS: &str = "id, command, argv_json, cwd, agent, exit_code, created_at, notes";

fn to_run(r: &rusqlite::Row<'_>) -> rusqlite::Result<Run> {
    let command: String = r.get(1)?;
    let argv_json: String = r.get(2)?;
    let full = serde_json::from_str::<Vec<String>>(&argv_json)
        .ok()
        .filter(|argv| !argv.is_empty())
        .map(|argv| argv.join(" "))
        .unwrap_or(command);
    Ok(Run {
        id: r.get(0)?,
        command: redact(&full),
        cwd: r.get(3)?,
        agent: provenance::label(r.get::<_, Option<String>>(4)?.as_deref()),
        exit_code: r.get(5)?,
        created_at: r.get(6)?,
        notes: redact(&r.get::<_, String>(7)?),
    })
}

fn runs(conn: &Connection, query: &str, failures_only: bool, limit: usize) -> Res<Vec<Run>> {
    let query = query.trim();
    conn.prepare(&format!(
        "SELECT {RUN_COLUMNS} FROM command_runs
         WHERE (?1 = 0 OR exit_code <> 0)
           AND (?2 = '' OR argv_json LIKE ?3 OR command LIKE ?3 OR notes LIKE ?3
                OR stderr LIKE ?3 OR cwd LIKE ?3)
         ORDER BY created_at DESC, id DESC LIMIT ?4"
    ))
    .map_err(sql_err)?
    .query_map(
        params![failures_only, query, format!("%{query}%"), limit as i64],
        to_run,
    )
    .map_err(sql_err)?
    .collect::<Result<Vec<_>, _>>()
    .map_err(sql_err)
}

/// The last `max` bytes of captured output, cut at a character boundary.
fn tail(text: &str, max: usize) -> &str {
    let mut start = text.len().saturating_sub(max);
    while !text.is_char_boundary(start) {
        start += 1;
    }
    &text[start..]
}

fn detail(conn: &Connection, id: i64) -> Res<RunDetail> {
    let (run, stdout, stderr) = conn
        .query_row(
            &format!("SELECT {RUN_COLUMNS}, stdout, stderr FROM command_runs WHERE id = ?1"),
            [id],
            |r| Ok((to_run(r)?, r.get::<_, String>(8)?, r.get::<_, String>(9)?)),
        )
        .map_err(sql_err)?;
    let next = conn
        .prepare(&format!(
            "SELECT {RUN_COLUMNS} FROM command_runs WHERE id > ?1 AND cwd IS ?2 ORDER BY id LIMIT 5"
        ))
        .map_err(sql_err)?
        .query_map(params![id, run.cwd], to_run)
        .map_err(sql_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_err)?;
    Ok(RunDetail {
        stdout: redact(tail(&stdout, 6000)),
        stderr: redact(tail(&stderr, 6000)),
        run,
        next,
    })
}

#[tauri::command]
pub fn list_runs(
    state: tauri::State<AppState>,
    query: Option<String>,
    failures_only: Option<bool>,
    limit: Option<usize>,
) -> Res<Vec<Run>> {
    runs(
        &*db(&state)?,
        query.as_deref().unwrap_or(""),
        failures_only.unwrap_or(false),
        limit.unwrap_or(200),
    )
}

#[tauri::command]
pub fn run_detail(state: tauri::State<AppState>, id: i64) -> Res<RunDetail> {
    detail(&*db(&state)?, id)
}

// ── Lessons ──────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct Lesson {
    id: i64,
    label: String,
    created_at: String,
    approved: bool,
    provenance: String,
}

/// Every saved lesson, newest first, including agent-written ones waiting for
/// review so they can be approved here.
#[tauri::command]
pub fn list_lessons(state: tauri::State<AppState>) -> Res<Vec<Lesson>> {
    db(&state)?
        .prepare(
            "SELECT id, label, created_at, author_kind, author_name, source_session_id, approved
             FROM bookmarks ORDER BY id DESC LIMIT 500",
        )
        .map_err(sql_err)?
        .query_map([], |r| {
            let created_at: String = r.get(2)?;
            let author_kind: String = r.get(3)?;
            let author_name: Option<String> = r.get(4)?;
            Ok(Lesson {
                id: r.get(0)?,
                label: redact(&r.get::<_, String>(1)?),
                provenance: memorywhale_cli::provenance_label(
                    &author_kind,
                    author_name.as_deref(),
                    &created_at,
                    r.get(5)?,
                ),
                created_at,
                approved: r.get::<_, i64>(6)? != 0,
            })
        })
        .map_err(sql_err)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(sql_err)
}

/// Approve a pending (agent-written, review-mode) lesson so retrieval includes it.
#[tauri::command]
pub fn approve_lesson(state: tauri::State<AppState>, id: i64) -> Res<()> {
    memorywhale_cli::approve_note(&*db(&state)?, id)
}

#[tauri::command]
pub fn delete_lesson(state: tauri::State<AppState>, id: i64) -> Res<()> {
    db(&state)?
        .execute("DELETE FROM bookmarks WHERE id = ?1", [id])
        .map_err(sql_err)?;
    Ok(())
}

/// Save a lesson as the person at the keyboard; agents see it first when the
/// failure it mentions comes back.
#[tauri::command]
pub fn add_lesson(text: String) -> Res<i64> {
    let text = text.trim();
    if text.is_empty() {
        return Err("A lesson needs some text.".into());
    }
    memorywhale_cli::remember(text, None)
}

// ── Explainable search ───────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct Signal {
    name: String,
    score: f32,
    applicable: bool,
    detail: String,
}

/// A retrieved memory, where it came from, and why it ranked.
#[derive(Debug, Serialize)]
pub struct SearchHit {
    id: i64,
    /// `command`, `note`, `session`, `conversation`, or `document`.
    source: &'static str,
    /// The row id in that source, e.g. the command run to open.
    row_id: i64,
    text: String,
    score: u32,
    reasons: Vec<String>,
    signals: Vec<Signal>,
    created_at: String,
    agent: &'static str,
}

fn to_hit(sm: &memorywhale_core::ScoredMemory) -> SearchHit {
    let (source, row_id) = memorywhale_core::sqlite::decode_id(sm.memory.id);
    SearchHit {
        id: sm.memory.id,
        source: source.tag(),
        row_id,
        text: redact(&sm.memory.text),
        score: sm.percent(),
        reasons: sm.reasons(),
        signals: sm
            .signals
            .iter()
            .map(|s| Signal {
                name: s.name.clone(),
                score: s.score,
                applicable: s.applicable,
                detail: s.detail.clone(),
            })
            .collect(),
        created_at: sm.memory.created_at.to_rfc3339(),
        agent: provenance::label(sm.memory.agent.as_deref()),
    }
}

/// The embedding model whose vectors we cache (matches `OllamaEmbedder::default`).
const EMBED_MODEL: &str = "nomic-embed-text";

/// Stable-ish hash of a memory's text, used to invalidate the embedding cache
/// when the underlying text changes.
fn text_hash(s: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    s.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn cache_get(conn: &Connection, id: i64, hash: &str) -> Option<Vec<f32>> {
    conn.query_row(
        "SELECT vec FROM memory_embeddings WHERE memory_id = ?1 AND model = ?2 AND text_hash = ?3",
        params![id, EMBED_MODEL, hash],
        |r| r.get::<_, Vec<u8>>(0),
    )
    .ok()
    .map(|b| memorywhale_core::embed::bytes_to_vec(&b))
    .filter(|v| !v.is_empty())
}

fn cache_put(conn: &Connection, id: i64, hash: &str, vec: &[f32]) {
    let _ = conn.execute(
        "INSERT OR REPLACE INTO memory_embeddings (memory_id, model, text_hash, vec)
         VALUES (?1, ?2, ?3, ?4)",
        params![
            id,
            EMBED_MODEL,
            hash,
            memorywhale_core::embed::vec_to_bytes(vec)
        ],
    );
}

/// Fill each memory's embedding from the cache, computing and caching misses.
/// Fails when Ollama is not running, so the caller falls back to lexical.
fn fill_embeddings(
    conn: &Connection,
    mems: &mut [memorywhale_core::Memory],
    embedder: &dyn memorywhale_core::embed::Embedder,
) -> anyhow::Result<()> {
    for m in mems.iter_mut() {
        let hash = text_hash(&m.text);
        if let Some(v) = cache_get(conn, m.id, &hash) {
            m.embedding = Some(v);
        } else {
            let v = embedder.embed(&m.text)?;
            cache_put(conn, m.id, &hash, &v);
            m.embedding = Some(v);
        }
    }
    Ok(())
}

/// Lexical ranking always; local Ollama embeddings too when the server runs,
/// with each memory's vector cached so only the query is embedded per search.
fn build_engine(
    conn: &Connection,
    mut mems: Vec<memorywhale_core::Memory>,
) -> memorywhale_core::engine::BuiltinEngine {
    use memorywhale_core::engine::BuiltinEngine;
    let embedder = std::sync::Arc::new(memorywhale_core::embed::OllamaEmbedder::default());
    if fill_embeddings(conn, &mut mems, embedder.as_ref()).is_ok() {
        BuiltinEngine::new(mems.clone())
            .with_embedder(embedder)
            .unwrap_or_else(|_| BuiltinEngine::new(mems))
    } else {
        BuiltinEngine::new(mems)
    }
}

#[tauri::command]
pub fn search(
    state: tauri::State<AppState>,
    query: String,
    limit: Option<usize>,
) -> Res<Vec<SearchHit>> {
    let engine = {
        let conn = db(&state)?;
        let mems = memorywhale_core::sqlite::load_memories(&conn).map_err(|e| e.to_string())?;
        build_engine(&conn, mems)
    };
    let q = memorywhale_core::Query::new(query, Utc::now());
    Ok(engine
        .retrieve(&q, limit.unwrap_or(20))
        .iter()
        .map(to_hit)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_filter_and_detail_shows_what_ran_next() {
        let conn = Connection::open_in_memory().unwrap();
        memorywhale_cli::storage::initialize(&conn).unwrap();
        let add = |argv: &[&str], cwd: &str, exit: i64, notes: &str| {
            conn.execute(
                "INSERT INTO command_runs(command, argv_json, cwd, exit_code, stderr, notes, created_at)
                 VALUES(?1, ?2, ?3, ?4, 'error: linker `cc` not found', ?5, datetime('now'))",
                params![
                    argv[0],
                    serde_json::to_string(argv).unwrap(),
                    cwd,
                    exit,
                    notes
                ],
            )
            .unwrap();
        };
        add(
            &["cargo", "build"],
            "/w",
            101,
            "token=ghp_abcdefghijklmnopqrstuvwxyz0123456789",
        );
        add(&["xcode-select", "--install"], "/w", 0, "");
        add(&["cargo", "build"], "/w", 0, "");
        add(&["ls"], "/elsewhere", 0, "");

        assert_eq!(runs(&conn, "", false, 10).unwrap().len(), 4);
        let failures = runs(&conn, "", true, 10).unwrap();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].command, "cargo build");
        assert!(
            !failures[0].notes.contains("ghp_abcdef"),
            "{}",
            failures[0].notes
        );
        assert_eq!(runs(&conn, "xcode", false, 10).unwrap().len(), 1);
        assert_eq!(runs(&conn, "linker", false, 10).unwrap().len(), 4);

        let d = detail(&conn, failures[0].id).unwrap();
        assert_eq!(d.stderr, "error: linker `cc` not found");
        let next: Vec<_> = d.next.iter().map(|r| r.command.as_str()).collect();
        assert_eq!(next, ["xcode-select --install", "cargo build"]);

        conn.execute(
            "INSERT INTO recall_hints(run_id, fix_argv_json, created_at, kind, text)
             VALUES(?1, '[\"xcode-select\",\"--install\"]', datetime('now'), 'note', 'token=ghp_abcdefghijklmnopqrstuvwxyz0123456789')",
            [failures[0].id],
        )
        .unwrap();
        let e = events(&conn, 10).unwrap();
        assert_eq!(e[0].stem, "cargo build");
        assert!(!e[0].event.text.as_deref().unwrap().contains("ghp_abcdef"));

        assert_eq!(tail("ab€cd", 3), "cd");
        let o = overview_of(&conn).unwrap();
        assert_eq!((o.commands, o.failures), (4, 1));
        assert_eq!(o.agents, vec![("terminal".to_string(), 4)]);
    }
}
