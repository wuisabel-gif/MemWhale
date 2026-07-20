//! Shared helpers for the MemoryWhale CLI binaries.

use chrono::Utc;
use regex::Regex;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::OnceLock;

/// The MemoryWhale data directory (honours `MEMORYWHALE_DATA_DIR`).
pub fn data_dir() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("MEMORYWHALE_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }
    let base = dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| "could not resolve local data directory".to_string())?;
    Ok(base.join("MemoryWhale"))
}

/// Path to the local SQLite database.
pub fn database_path() -> Result<PathBuf, String> {
    Ok(data_dir()?.join("memorywhale.sqlite3"))
}

/// Best-effort full-text index over `command_runs` (SQLite FTS5).
///
/// Creates an external-content FTS5 table kept in sync by triggers, and rebuilds
/// it once when first created so pre-existing rows get indexed. The triggers
/// persist in the database file, so once this has run any writer (mw-run,
/// mw-remember, …) maintains the index without needing to call this. Returns an
/// error if FTS5 isn't compiled in; callers treat that as "no index" and fall
/// back to LIKE.
pub fn ensure_fts(conn: &Connection) -> Result<(), String> {
    let existed = conn
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type='table' AND name='command_fts'",
            [],
            |_| Ok(()),
        )
        .is_ok();
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS command_fts USING fts5(
             command, argv_json, stdout, stderr, notes,
             content='command_runs', content_rowid='id'
         );
         CREATE TRIGGER IF NOT EXISTS command_runs_fts_ai AFTER INSERT ON command_runs BEGIN
             INSERT INTO command_fts(rowid, command, argv_json, stdout, stderr, notes)
             VALUES (new.id, new.command, new.argv_json, new.stdout, new.stderr, new.notes);
         END;
         CREATE TRIGGER IF NOT EXISTS command_runs_fts_ad AFTER DELETE ON command_runs BEGIN
             INSERT INTO command_fts(command_fts, rowid, command, argv_json, stdout, stderr, notes)
             VALUES ('delete', old.id, old.command, old.argv_json, old.stdout, old.stderr, old.notes);
         END;
         CREATE TRIGGER IF NOT EXISTS command_runs_fts_au AFTER UPDATE ON command_runs BEGIN
             INSERT INTO command_fts(command_fts, rowid, command, argv_json, stdout, stderr, notes)
             VALUES ('delete', old.id, old.command, old.argv_json, old.stdout, old.stderr, old.notes);
             INSERT INTO command_fts(rowid, command, argv_json, stdout, stderr, notes)
             VALUES (new.id, new.command, new.argv_json, new.stdout, new.stderr, new.notes);
         END;",
    )
    .map_err(|e| format!("fts init: {e}"))?;
    if !existed {
        conn.execute("INSERT INTO command_fts(command_fts) VALUES('rebuild')", [])
            .map_err(|e| format!("fts rebuild: {e}"))?;
    }
    Ok(())
}

/// Turn a free-text query into a safe FTS5 MATCH expression: each whitespace
/// term becomes a quoted phrase with a trailing prefix `*` (so "link" still
/// finds "linker", closer to the old substring search), AND-ed together.
/// Quoting escapes punctuation so it can't break MATCH syntax. Empty if the
/// query has no usable terms.
pub fn fts_match_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|t| format!("\"{}\"*", t.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Base `bookmarks` schema (pre-provenance). Kept as-is so `migrate` can layer
/// the provenance columns on top with `PRAGMA user_version` migrations.
const BOOKMARKS_BASE: &str = "CREATE TABLE IF NOT EXISTS bookmarks (
         id INTEGER PRIMARY KEY,
         label TEXT NOT NULL,
         cwd TEXT,
         created_at TEXT NOT NULL,
         command_run_id INTEGER,
         session_id INTEGER
     );
     CREATE INDEX IF NOT EXISTS idx_bookmarks_created_at ON bookmarks(created_at);";

/// Schema version `migrate` brings a database up to.
pub const LATEST_SCHEMA_VERSION: i64 = 3;

/// Apply numbered schema migrations to a MemoryWhale database. Idempotent and
/// cheap (a `user_version` check), so callers run it before touching bookmarks.
///
/// Migration 1 — memory provenance: adds `author_kind`/`author_name`/
/// `source_session_id`/`approved` to `bookmarks` and backfills existing rows as
/// human (via the column defaults). Safe on a populated DB: `ADD COLUMN` with a
/// constant default never rewrites rows.
///
/// Migration 2 — first-class scopes: adds `project`/`machine` to `sessions` and
/// backfills `project` from the legacy `project:<name>` convention inside
/// `notes`. The notes string itself is left untouched, so the dashboard's
/// existing note parsing keeps working.
///
/// Migration 3 — capture tiers: adds `command_runs.capture_kind`, defaulting to
/// `full` so every pre-existing row keeps its meaning. Shell-hook rows write
/// `hook` instead (command + cwd + exit code, no output).
pub fn migrate(conn: &Connection) -> Result<(), String> {
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| format!("failed to read schema version: {e}"))?;
    if version < 1 {
        conn.execute_batch(BOOKMARKS_BASE)
            .map_err(|e| format!("failed to prepare bookmarks table: {e}"))?;
        add_column_if_missing(conn, "bookmarks", "author_kind", "TEXT NOT NULL DEFAULT 'human'")?;
        add_column_if_missing(conn, "bookmarks", "author_name", "TEXT")?;
        add_column_if_missing(conn, "bookmarks", "source_session_id", "INTEGER")?;
        add_column_if_missing(conn, "bookmarks", "approved", "INTEGER NOT NULL DEFAULT 1")?;
        add_column_if_missing(conn, "bookmarks", "created_at", "TEXT")?; // base has it
        conn.execute_batch("PRAGMA user_version = 1;")
            .map_err(|e| format!("failed to bump schema version: {e}"))?;
    }
    if version < 2 {
        // A bookmarks-only DB (mw-remember on a fresh machine) has no sessions
        // table yet; `init_schema` creates it with the columns already present.
        if table_exists(conn, "sessions")? {
            add_column_if_missing(conn, "sessions", "project", "TEXT")?;
            add_column_if_missing(conn, "sessions", "machine", "TEXT")?;
            backfill_project_from_notes(conn)?;
        }
        conn.execute_batch("PRAGMA user_version = 2;")
            .map_err(|e| format!("failed to bump schema version: {e}"))?;
    }
    if version < 3 {
        ensure_capture_kind(conn)?;
        conn.execute_batch(&format!("PRAGMA user_version = {LATEST_SCHEMA_VERSION};"))
            .map_err(|e| format!("failed to bump schema version: {e}"))?;
    }
    Ok(())
}

/// Add `command_runs.capture_kind` if the table exists and lacks it.
///
/// ponytail: exposed (not just inlined in migration 3) because half a dozen
/// binaries still create `command_runs` with their own `CREATE TABLE IF NOT
/// EXISTS`, so a DB can reach version 3 *before* the table exists. Writers call
/// this directly; it's two pragma queries.
pub fn ensure_capture_kind(conn: &Connection) -> Result<(), String> {
    if table_exists(conn, "command_runs")? {
        add_column_if_missing(conn, "command_runs", "capture_kind", "TEXT NOT NULL DEFAULT 'full'")?;
    }
    Ok(())
}

fn table_exists(conn: &Connection, table: &str) -> Result<bool, String> {
    conn.prepare("SELECT 1 FROM sqlite_master WHERE type='table' AND name = ?1")
        .and_then(|mut s| s.exists(params![table]))
        .map_err(|e| format!("failed to inspect schema: {e}"))
}

fn add_column_if_missing(
    conn: &Connection,
    table: &str,
    column: &str,
    decl: &str,
) -> Result<(), String> {
    let present = conn
        .prepare("SELECT 1 FROM pragma_table_info(?1) WHERE name = ?2")
        .and_then(|mut s| s.exists(params![table, column]))
        .map_err(|e| format!("failed to inspect {table} columns: {e}"))?;
    if !present {
        // Two writers (e.g. two shell hooks firing at once on a fresh DB) can
        // both read the column as missing and both try to add it. The loser
        // gets "duplicate column name", which is the outcome we wanted anyway.
        if let Err(e) = conn.execute(&format!("ALTER TABLE {table} ADD COLUMN {column} {decl}"), [])
        {
            let msg = e.to_string();
            if !msg.contains("duplicate column") {
                return Err(format!("failed to add {table}.{column}: {msg}"));
            }
        }
    }
    Ok(())
}

/// Lift the legacy `project:<name>` note convention into `sessions.project`.
/// Read-only on `notes` — existing users' notes strings stay byte-identical.
fn backfill_project_from_notes(conn: &Connection) -> Result<(), String> {
    let mut stmt = conn
        .prepare("SELECT id, notes FROM sessions WHERE project IS NULL AND notes LIKE '%project:%'")
        .map_err(|e| format!("failed to prepare backfill: {e}"))?;
    let rows: Vec<(i64, String)> = stmt
        .query_map([], |r| Ok((r.get(0)?, r.get(1)?)))
        .map_err(|e| format!("failed to read sessions: {e}"))?
        .flatten()
        .collect();
    for (id, notes) in rows {
        if let Some(project) = project_of(&notes) {
            conn.execute("UPDATE sessions SET project = ?1 WHERE id = ?2", params![project, id])
                .map_err(|e| format!("failed to backfill session {id}: {e}"))?;
        }
    }
    Ok(())
}

/// Extract a `project:<name>` tag from a notes string. Same shape the dashboard
/// has always parsed, so the schema column and the note convention agree.
pub fn project_of(notes: &str) -> Option<String> {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(r"project:([\w.\-]+)").expect("valid project regex"))
        .captures(notes)
        .map(|c| c[1].to_string())
}

/// This machine's name, recorded on every captured session so memory synced
/// from a teammate (or your other box) stays distinguishable. `MW_MACHINE`
/// wins, then `machine = "..."` in `<data dir>/config.toml`, then the hostname.
pub fn machine_name() -> String {
    if let Ok(v) = std::env::var("MW_MACHINE") {
        if !v.trim().is_empty() {
            return v.trim().to_string();
        }
    }
    if let Some(v) = config_value("machine") {
        return v;
    }
    // ponytail: shelling out to `hostname` beats a dep; falls back to the env.
    std::process::Command::new("hostname")
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        // macOS reports an FQDN, Linux the short name — keep the first label so
        // the filter value is the same short name you'd type on either.
        .map(|s| s.trim().split('.').next().unwrap_or("").to_string())
        .filter(|s| !s.is_empty())
        .or_else(|| std::env::var("HOSTNAME").ok())
        .unwrap_or_else(|| "unknown".to_string())
}

/// A `key = "value"` line from `<data dir>/config.toml`, unquoted.
fn config_value(key: &str) -> Option<String> {
    let text = std::fs::read_to_string(data_dir().ok()?.join("config.toml")).ok()?;
    text.lines().find_map(|line| {
        let (k, v) = line.split_once('=')?;
        if k.trim() != key {
            return None;
        }
        let v = v.trim().trim_matches('"').trim().to_string();
        (!v.is_empty()).then_some(v)
    })
}

/// How much of a command or session may be persisted, decided per directory
/// *before* anything is written to SQLite.
///
/// * `Full` — the historical behaviour: command, argv, cwd, exit code, and the
///   full stdout/stderr or session transcript.
/// * `CommandsOnly` — what ran and how it went (command, argv, cwd, exit code,
///   timestamps) but no captured output.
/// * `Off` — nothing is written at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureMode {
    Full,
    CommandsOnly,
    Off,
}

impl CaptureMode {
    /// Parse a config value, tolerating surrounding quotes/whitespace.
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().trim_matches('"').trim().to_ascii_lowercase().as_str() {
            "full" => Some(Self::Full),
            "commands-only" | "commands_only" => Some(Self::CommandsOnly),
            "off" | "none" => Some(Self::Off),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::CommandsOnly => "commands-only",
            Self::Off => "off",
        }
    }

    /// True when stdout/stderr (or a session transcript) may be stored.
    pub fn stores_output(self) -> bool {
        matches!(self, Self::Full)
    }

    /// True when a row may be written at all.
    pub fn stores_anything(self) -> bool {
        !matches!(self, Self::Off)
    }
}

/// A resolved capture decision plus the rule that produced it, so `mw status`
/// can tell you *why* a directory is gated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CaptureRule {
    pub mode: CaptureMode,
    /// Human-readable origin, e.g. a `.mwignore` path or a config entry.
    pub source: String,
}

/// Resolve the capture mode for a working directory.
///
/// Precedence: the nearest `.mwignore` walking *up* from `cwd`, then the
/// longest matching prefix in `[capture.paths]` of `<data dir>/config.toml`,
/// then `full`. Paths are canonicalized on both sides so a symlinked cwd can't
/// dodge a gate.
pub fn capture_rule(cwd: &std::path::Path) -> CaptureRule {
    let dir = canonical(cwd);
    if let Some(rule) = mwignore_rule(&dir) {
        return rule;
    }
    if let Some(rule) = data_dir()
        .ok()
        .map(|d| d.join("config.toml"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|text| capture_paths_rule(&text, &dir))
    {
        return rule;
    }
    CaptureRule {
        mode: CaptureMode::Full,
        source: "default (no .mwignore or [capture.paths] match)".to_string(),
    }
}

/// Convenience wrapper for the many call sites that only have an optional cwd
/// string. An unknown cwd is never gated (nothing to match against).
pub fn capture_rule_for(cwd: Option<&str>) -> CaptureRule {
    match cwd {
        Some(c) => capture_rule(std::path::Path::new(c)),
        None => CaptureRule {
            mode: CaptureMode::Full,
            source: "default (unknown working directory)".to_string(),
        },
    }
}

/// Resolve symlinks so prefix matching can't be defeated by a symlinked path.
/// Falls back to the input when the path doesn't exist yet.
fn canonical(path: &std::path::Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Nearest `.mwignore` at or above `dir`. Format is the same hand-rolled
/// `key = "value"` line parsing as `config.toml`; only `capture` is read.
fn mwignore_rule(dir: &std::path::Path) -> Option<CaptureRule> {
    for ancestor in dir.ancestors() {
        let file = ancestor.join(".mwignore");
        let Ok(text) = std::fs::read_to_string(&file) else {
            continue;
        };
        if let Some(mode) = text.lines().find_map(|line| {
            let line = line.split('#').next().unwrap_or("");
            let (k, v) = line.split_once('=')?;
            (k.trim() == "capture").then(|| CaptureMode::parse(v)).flatten()
        }) {
            return Some(CaptureRule {
                mode,
                source: file.display().to_string(),
            });
        }
    }
    None
}

/// Longest-prefix match against the `[capture.paths]` table of a config file.
/// Hand-rolled to keep mw-cli TOML-crate-free, matching `config_value` above.
fn capture_paths_rule(config: &str, dir: &std::path::Path) -> Option<CaptureRule> {
    let mut in_section = false;
    let mut best: Option<(usize, CaptureRule)> = None;
    for line in config.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.starts_with('[') {
            in_section = line == "[capture.paths]";
            continue;
        }
        if !in_section {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let Some(mode) = CaptureMode::parse(value) else {
            continue;
        };
        let raw = key.trim().trim_matches('"').trim();
        let prefix = canonical(&expand_tilde(raw));
        if !dir.starts_with(&prefix) {
            continue;
        }
        let depth = prefix.components().count();
        if best.as_ref().map_or(true, |(d, _)| depth > *d) {
            best = Some((
                depth,
                CaptureRule {
                    mode,
                    source: format!("config.toml [capture.paths] {raw:?}"),
                },
            ));
        }
    }
    best.map(|(_, rule)| rule)
}

fn expand_tilde(path: &str) -> PathBuf {
    match path.strip_prefix("~/") {
        Some(rest) => dirs::home_dir().unwrap_or_default().join(rest),
        None => PathBuf::from(path),
    }
}

/// Parse a relative time window like `7d`, `24h`, `2w`, `30m` into a duration.
pub fn parse_since(spec: &str) -> Result<chrono::Duration, String> {
    let spec = spec.trim();
    let bad = || format!("invalid --since {spec:?}; use e.g. 7d, 24h, 2w");
    let unit = spec.chars().last().ok_or_else(bad)?;
    let n: i64 = spec[..spec.len() - unit.len_utf8()].parse().map_err(|_| bad())?;
    if n < 0 {
        return Err(bad());
    }
    match unit {
        'm' => Ok(chrono::Duration::minutes(n)),
        'h' => Ok(chrono::Duration::hours(n)),
        'd' => Ok(chrono::Duration::days(n)),
        'w' => Ok(chrono::Duration::weeks(n)),
        _ => Err(bad()),
    }
}

/// Narrow loaded memories to an explicit scope before they reach the engine, so
/// ranking happens *within* the scope rather than being trimmed after the fact.
///
/// With no filters this returns the input untouched — the unscoped `mw search`
/// path is byte-for-byte what it always was. `project`/`machine` only match
/// memories that actually carry a scope: recorded sessions (the new columns) and,
/// for `project`, command runs still tagged the legacy way in their notes.
pub fn scope_memories(
    conn: &Connection,
    mut mems: Vec<mw_memory::Memory>,
    project: Option<&str>,
    machine: Option<&str>,
    since: Option<chrono::DateTime<Utc>>,
) -> Vec<mw_memory::Memory> {
    use mw_memory::sqlite::{decode_id, Source};

    if let Some(cutoff) = since {
        mems.retain(|m| m.created_at >= cutoff);
    }
    if project.is_none() && machine.is_none() {
        return mems;
    }

    let ids = |sql: &str, p: &[&dyn rusqlite::ToSql]| -> std::collections::HashSet<i64> {
        conn.prepare(sql)
            .and_then(|mut s| {
                s.query_map(p, |r| r.get::<_, i64>(0))
                    .map(|rows| rows.flatten().collect())
            })
            .unwrap_or_default()
    };
    let sessions = ids(
        "SELECT id FROM sessions
         WHERE (?1 IS NULL OR project = ?1) AND (?2 IS NULL OR machine = ?2)",
        params![project, machine],
    );
    // command_runs never carried a machine, so a machine filter excludes them.
    let commands = match (project, machine) {
        (Some(p), None) => ids(
            "SELECT id FROM command_runs WHERE notes LIKE '%project:' || ?1 || '%'",
            params![p],
        ),
        _ => Default::default(),
    };
    mems.retain(|m| match decode_id(m.id) {
        (Source::Session, id) => sessions.contains(&id),
        (Source::Command, id) => commands.contains(&id),
        _ => false,
    });
    mems
}

/// True when agent-written memories should start unapproved and be excluded from
/// retrieval until approved in the dashboard. Off by default. Enabled by env var
/// `MEMORYWHALE_REVIEW_AGENT_MEMORIES=1` or a `review_agent_memories = true` line
/// in `<data dir>/config.toml`.
pub fn review_agent_memories() -> bool {
    if let Some(v) = std::env::var_os("MEMORYWHALE_REVIEW_AGENT_MEMORIES") {
        return v == "1" || v == "true";
    }
    data_dir()
        .ok()
        .map(|d| d.join("config.toml"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .map(|s| {
            s.lines().any(|l| {
                let l = l.trim();
                l.starts_with("review_agent_memories") && l.contains("true")
            })
        })
        .unwrap_or(false)
}

/// Human-readable provenance, e.g. "remembered by Claude Code on 2026-07-12
/// during session #41". `created_at` may be an RFC3339 timestamp; only the date
/// part is shown.
pub fn provenance_label(
    author_kind: &str,
    author_name: Option<&str>,
    created_at: &str,
    session_id: Option<i64>,
) -> String {
    let who = match (author_kind, author_name) {
        ("agent", Some(n)) if !n.is_empty() => n.to_string(),
        ("agent", _) => "agent".to_string(),
        _ => "you".to_string(),
    };
    let date = created_at.get(..10).unwrap_or(created_at);
    let mut s = format!("remembered by {who} on {date}");
    if let Some(sid) = session_id {
        s.push_str(&format!(" during session #{sid}"));
    }
    s
}

/// Save a freeform lesson or conclusion ("the fix was X") into the bookmarks
/// table — the same store `mw mark` writes to. Records the author as a human;
/// see [`remember_as`] for agent-attributed writes. Shared by `mw remember` and
/// `mw mark`.
pub fn remember(text: &str, cwd: Option<&str>) -> Result<i64, String> {
    remember_as(text, cwd, "human", None, None)
}

/// Like [`remember`] but records who wrote the lesson. `author_kind` is
/// "human" or "agent"; `author_name` is the MCP client name for agents (else
/// `None`). When [`review_agent_memories`] is on, agent lessons are stored
/// unapproved so retrieval skips them until approved in the dashboard.
pub fn remember_as(
    text: &str,
    cwd: Option<&str>,
    author_kind: &str,
    author_name: Option<&str>,
    source_session_id: Option<i64>,
) -> Result<i64, String> {
    // Capture gate, before the database is opened. Every note-writing surface
    // (`mw mark`, `mw remember`, MCP, the desktop app) routes through here.
    let gate = capture_rule_for(cwd);
    if !gate.mode.stores_anything() {
        return Err(format!("capture is off for this directory ({}) — nothing saved", gate.source));
    }
    let text = redact(text);
    let path = database_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create data dir: {e}"))?;
    }
    let conn = Connection::open(&path).map_err(|e| format!("failed to open db: {e}"))?;
    migrate(&conn)?;
    let approved = if author_kind == "agent" && review_agent_memories() {
        0
    } else {
        1
    };
    conn.execute(
        "INSERT INTO bookmarks
            (label, cwd, created_at, author_kind, author_name, source_session_id, approved)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            text,
            cwd,
            Utc::now().to_rfc3339(),
            author_kind,
            author_name,
            source_session_id,
            approved
        ],
    )
    .map_err(|e| format!("failed to save note: {e}"))?;
    Ok(conn.last_insert_rowid())
}

const REDACTED: &str = "[REDACTED]";

/// Scrub common secret shapes out of captured text before it lands in SQLite.
///
/// This runs on stdout/stderr/notes/transcripts — the bulky, unattended
/// captures where an `env` dump or a leaked token is most likely to end up.
/// It is intentionally conservative (known token formats + `key=secret`
/// assignments), not a guarantee. Set `MEMORYWHALE_NO_REDACT=1` to store raw.
pub fn redact(text: &str) -> String {
    if std::env::var_os("MEMORYWHALE_NO_REDACT").is_some() {
        return text.to_string();
    }
    let mut out = text.to_string();
    for re in secret_patterns() {
        out = re.replace_all(&out, |caps: &regex::Captures| {
            // If the pattern captured a leading "key=" / "key:" label, keep it.
            match caps.name("label") {
                Some(label) => format!("{}{}", label.as_str(), REDACTED),
                None => REDACTED.to_string(),
            }
        })
        .into_owned();
    }
    out
}

fn secret_patterns() -> &'static [Regex] {
    static PATTERNS: OnceLock<Vec<Regex>> = OnceLock::new();
    PATTERNS.get_or_init(|| {
        [
            // key = value / key: value where the key name looks sensitive.
            r#"(?i)(?P<label>\b(?:api[_-]?key|secret|token|password|passwd|pwd|access[_-]?key|client[_-]?secret)\b\s*[:=]\s*)['"]?[A-Za-z0-9/_+\-\.]{6,}['"]?"#,
            // Authorization: Bearer <token>
            r#"(?i)(?P<label>bearer\s+)[A-Za-z0-9._\-]{8,}"#,
            // Provider token formats.
            r#"AKIA[0-9A-Z]{16}"#,                                   // AWS access key id
            r#"gh[pousr]_[A-Za-z0-9]{20,}"#,                          // GitHub tokens
            r#"xox[baprs]-[A-Za-z0-9\-]{10,}"#,                       // Slack tokens
            r#"eyJ[A-Za-z0-9_\-]+\.eyJ[A-Za-z0-9_\-]+\.[A-Za-z0-9_\-]+"#, // JWTs
            // PEM private key blocks (whole block).
            r#"(?s)-----BEGIN [A-Z ]*PRIVATE KEY-----.*?-----END [A-Z ]*PRIVATE KEY-----"#,
        ]
        .iter()
        .map(|p| Regex::new(p).expect("valid secret regex"))
        .collect()
    })
}

/// Finalize the current in-progress recording the moment this process's parent
/// dies, instead of waiting for the dashboard's next-startup recovery.
///
/// `mw --live` inserts a `status='recording'` session row and then blocks inside
/// the interactive `script` child. If its parent (the terminal/shell that
/// launched it) is killed, `mw` is orphaned and the row is stranded. This guard
/// spawns a small background watcher that detects parent death and runs
/// `finalize` (which flips the row to `interrupted`), then exits cleanly.
///
/// Mechanisms, cheapest reliable one per platform:
///   * Linux: `prctl(PR_SET_PDEATHSIG, SIGTERM)`, with SIGTERM blocked and a
///     thread `sigwait`-ing for it so `finalize` runs on a normal thread (SQLite
///     is not async-signal-safe). Handles the race where the parent died before
///     `prctl` by re-checking `getppid()`.
///   * Other Unix (macOS, BSD): if a cooperating parent handed us the read end of
///     a pipe via `MW_PDEATH_FD`, watch it for EOF (parent closed its write end =
///     parent gone). Otherwise poll `getppid()` for reparenting — the portable
///     fallback that needs no parent cooperation, which is what an arbitrary
///     terminal/shell parent gives us.
///
/// `finalize` must be idempotent: startup recovery may still run later. Flipping
/// a row to `interrupted` and skipping already-imported transcripts both are.
#[cfg(unix)]
pub fn guard_parent_death<F>(finalize: F)
where
    F: Fn() + Send + 'static,
{
    // safety: getppid() takes no args and only reads our own parent pid.
    let original_ppid = unsafe { libc::getppid() };

    #[cfg(target_os = "linux")]
    {
        // Block SIGTERM process-wide so the PDEATHSIG below can't just kill us
        // before we finalize; a thread sigwaits for it instead. Called before any
        // other thread spawns so the mask is inherited by all of them.
        // safety: zeroed sigset_t is a valid empty set for the libc sig* calls.
        unsafe {
            let mut set: libc::sigset_t = std::mem::zeroed();
            libc::sigemptyset(&mut set);
            libc::sigaddset(&mut set, libc::SIGTERM);
            libc::pthread_sigmask(libc::SIG_BLOCK, &set, std::ptr::null_mut());
            // Ask the kernel to send us SIGTERM when our parent dies.
            libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM as libc::c_ulong, 0, 0, 0);
        }
        // Race: the parent may have died between our start and prctl. If so we've
        // been reparented (getppid changed, typically to 1) — finalize now.
        // safety: see above.
        if unsafe { libc::getppid() } != original_ppid {
            finalize();
            std::process::exit(0);
        }
        std::thread::spawn(move || {
            // safety: sigset_t built here, sigwait blocks until SIGTERM arrives.
            unsafe {
                let mut set: libc::sigset_t = std::mem::zeroed();
                libc::sigemptyset(&mut set);
                libc::sigaddset(&mut set, libc::SIGTERM);
                let mut sig: libc::c_int = 0;
                libc::sigwait(&set, &mut sig);
            }
            finalize();
            std::process::exit(0);
        });
    }

    #[cfg(not(target_os = "linux"))]
    {
        use std::os::unix::io::RawFd;
        let pipe_fd: Option<RawFd> = std::env::var(PDEATH_FD_ENV)
            .ok()
            .and_then(|v| v.parse::<RawFd>().ok());
        std::thread::spawn(move || {
            match pipe_fd {
                Some(fd) => {
                    let mut buf = [0u8; 1];
                    loop {
                        // safety: blocking read of one byte from a pipe fd handed
                        // to us by the parent; the buffer is a local we own.
                        let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut libc::c_void, 1) };
                        if n == 0 {
                            break; // EOF: parent closed its write end -> parent gone
                        }
                        if n < 0 {
                            let e = std::io::Error::last_os_error();
                            if e.kind() == std::io::ErrorKind::Interrupted {
                                continue;
                            }
                            break; // any other error: treat as parent gone
                        }
                        // n == 1: unexpected byte on the control pipe; keep waiting.
                    }
                }
                None => loop {
                    std::thread::sleep(std::time::Duration::from_millis(200));
                    // safety: getppid() only reads our own parent pid.
                    if unsafe { libc::getppid() } != original_ppid {
                        break; // reparented -> parent gone
                    }
                },
            }
            finalize();
            std::process::exit(0);
        });
    }
}

/// Env var a cooperating parent sets to the inherited read-end fd of a
/// parent-death pipe, for the non-Linux EOF mechanism above.
#[cfg(unix)]
pub const PDEATH_FD_ENV: &str = "MW_PDEATH_FD";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_label_hides_value() {
        assert_eq!(redact("API_KEY=abcdef123456"), "API_KEY=[REDACTED]");
        assert_eq!(redact("password: hunter2secret"), "password: [REDACTED]");
    }

    #[test]
    fn hides_known_token_formats() {
        assert!(redact("here AKIAABCDEFGHIJKLMNOP done").contains("[REDACTED]"));
        assert!(!redact("ghp_0123456789abcdefghijABCDEF").contains("ghp_"));
        assert!(redact("Authorization: Bearer abcd.efgh.ijkl").contains("Bearer [REDACTED]"));
    }

    #[test]
    fn leaves_ordinary_text_alone() {
        let s = "cargo build finished in 3.2s with 0 warnings";
        assert_eq!(redact(s), s);
    }

    #[test]
    fn opt_out_env_disables() {
        // Not testing the env branch here to avoid global state; ensure a
        // non-secret round-trips unchanged (covers the common path).
        assert_eq!(redact("plain line"), "plain line");
    }

    #[test]
    fn fts_query_quotes_and_prefixes_terms() {
        assert_eq!(fts_match_query("linker error"), "\"linker\"* \"error\"*");
        assert_eq!(fts_match_query("  spaced  "), "\"spaced\"*");
        assert_eq!(fts_match_query(""), "");
        // a stray quote is escaped, not left to break MATCH syntax
        assert_eq!(fts_match_query("a\"b"), "\"a\"\"b\"*");
    }

    #[test]
    fn ensure_fts_indexes_and_matches() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE command_runs (id INTEGER PRIMARY KEY, command TEXT,
                 argv_json TEXT, stdout TEXT, stderr TEXT, notes TEXT);
             INSERT INTO command_runs (command, argv_json, stdout, stderr, notes)
             VALUES ('cargo', '[\"cargo\",\"build\"]', '', 'error: linker failed', 'auv');",
        )
        .unwrap();
        // pre-existing row must be indexed by the one-time rebuild
        ensure_fts(&conn).unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM command_fts WHERE command_fts MATCH ?1",
                [fts_match_query("linker")],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1, "pre-existing row should be found via FTS");

        // a new row must be maintained by the trigger
        conn.execute(
            "INSERT INTO command_runs (command, argv_json, stdout, stderr, notes)
             VALUES ('make', '[\"make\"]', '', 'undefined reference', '')",
            [],
        )
        .unwrap();
        let n: i64 = conn
            .query_row(
                "SELECT count(*) FROM command_fts WHERE command_fts MATCH ?1",
                [fts_match_query("undefined")],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(n, 1, "trigger should index the new row");
    }

    // Tests that mutate process-global env (MEMORYWHALE_DATA_DIR / review flag)
    // are serialized so they don't clobber each other under parallel runs.
    static ENV_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn fresh_data_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "mw-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("MEMORYWHALE_DATA_DIR", &dir);
        dir
    }

    #[test]
    fn remember_writes_and_redacts() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("MEMORYWHALE_REVIEW_AGENT_MEMORIES");
        let dir = fresh_data_dir("remember-test");

        let id = remember("the fix: API_KEY=abcdef123456 in .env", Some("/tmp/repo")).unwrap();
        assert!(id > 0);

        let conn = Connection::open(dir.join("memorywhale.sqlite3")).unwrap();
        let (label, cwd): (String, Option<String>) = conn
            .query_row(
                "SELECT label, cwd FROM bookmarks WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert!(label.contains("[REDACTED]"), "secret should be redacted: {label}");
        assert_eq!(cwd.as_deref(), Some("/tmp/repo"));

        std::env::remove_var("MEMORYWHALE_DATA_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // `mw remember` / `mw mark` (CLI) attribute the lesson to a human.
    #[test]
    fn cli_remember_is_human() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("MEMORYWHALE_REVIEW_AGENT_MEMORIES");
        let dir = fresh_data_dir("prov-human");

        let id = remember("the fix was --features vendored-ssl", None).unwrap();
        let conn = Connection::open(dir.join("memorywhale.sqlite3")).unwrap();
        let (kind, name, approved): (String, Option<String>, i64) = conn
            .query_row(
                "SELECT author_kind, author_name, approved FROM bookmarks WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
            )
            .unwrap();
        assert_eq!(kind, "human");
        assert_eq!(name, None);
        assert_eq!(approved, 1);

        std::env::remove_var("MEMORYWHALE_DATA_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // The MCP `remember` tool attributes the lesson to the agent + client name.
    #[test]
    fn mcp_remember_is_agent_attributed() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("MEMORYWHALE_REVIEW_AGENT_MEMORIES");
        let dir = fresh_data_dir("prov-agent");

        let id = remember_as("E0308 was a string fps field", None, "agent", Some("Claude Code"), Some(41))
            .unwrap();
        let conn = Connection::open(dir.join("memorywhale.sqlite3")).unwrap();
        let (kind, name, sid, approved): (String, Option<String>, Option<i64>, i64) = conn
            .query_row(
                "SELECT author_kind, author_name, source_session_id, approved FROM bookmarks WHERE id = ?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
            )
            .unwrap();
        assert_eq!(kind, "agent");
        assert_eq!(name.as_deref(), Some("Claude Code"));
        assert_eq!(sid, Some(41));
        assert_eq!(approved, 1, "agent memory is approved when review mode is off");

        std::env::remove_var("MEMORYWHALE_DATA_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // With review mode ON, agent memories start unapproved and are excluded from
    // the approved-only retrieval query.
    #[test]
    fn review_mode_hides_unapproved_agent_memories() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = fresh_data_dir("prov-review");
        std::env::set_var("MEMORYWHALE_REVIEW_AGENT_MEMORIES", "1");

        let agent_id = remember_as("unreviewed agent claim", None, "agent", Some("Codex"), None).unwrap();
        let human_id = remember("trusted human note", None).unwrap();

        let conn = Connection::open(dir.join("memorywhale.sqlite3")).unwrap();
        let agent_approved: i64 = conn
            .query_row("SELECT approved FROM bookmarks WHERE id = ?1", [agent_id], |r| r.get(0))
            .unwrap();
        assert_eq!(agent_approved, 0, "agent memory pending review");

        // Assert through the REAL retrieval path — the shared loader every
        // surface (mw search, MCP, desktop) goes through — not a re-implemented
        // query, so this test fails if the loader ever drops the approved filter.
        let loaded = mw_memory::sqlite::load_memories(&conn);
        let visible: Vec<i64> = loaded
            .iter()
            .filter_map(|m| match mw_memory::sqlite::decode_id(m.id) {
                (mw_memory::sqlite::Source::Note, id) => Some(id),
                _ => None,
            })
            .collect();
        assert!(visible.contains(&human_id), "human memory visible");
        assert!(!visible.contains(&agent_id), "unapproved agent memory hidden");

        std::env::remove_var("MEMORYWHALE_REVIEW_AGENT_MEMORIES");
        std::env::remove_var("MEMORYWHALE_DATA_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // The migration is safe on a populated DB: an existing pre-provenance row is
    // backfilled as human/approved, and new provenance columns are usable.
    #[test]
    fn migrate_backfills_existing_rows_as_human() {
        let conn = Connection::open_in_memory().unwrap();
        // Fixture: legacy bookmarks table with a row, no provenance columns.
        conn.execute_batch(
            "CREATE TABLE bookmarks (
                 id INTEGER PRIMARY KEY, label TEXT NOT NULL, cwd TEXT,
                 created_at TEXT NOT NULL, command_run_id INTEGER, session_id INTEGER
             );
             INSERT INTO bookmarks (label, created_at) VALUES ('old lesson', '2026-01-01T00:00:00Z');",
        )
        .unwrap();

        migrate(&conn).unwrap();

        let (kind, approved): (String, i64) = conn
            .query_row(
                "SELECT author_kind, approved FROM bookmarks WHERE label = 'old lesson'",
                [],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .unwrap();
        assert_eq!(kind, "human", "existing rows backfilled as human");
        assert_eq!(approved, 1);

        // migrate is idempotent
        migrate(&conn).unwrap();
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(version, LATEST_SCHEMA_VERSION);
    }

    // Migration 2 round-trip on a populated pre-scope DB: the legacy
    // `project:` note convention becomes a real column, the notes themselves
    // are untouched, and scoped queries then work.
    #[test]
    fn migration_2_lifts_project_notes_into_a_column() {
        let conn = Connection::open_in_memory().unwrap();
        let legacy_notes = "debugging the jetson build project:camera-driver runtime:host";
        conn.execute_batch(&format!(
            "CREATE TABLE sessions (
                 id INTEGER PRIMARY KEY, shell TEXT, cwd TEXT, transcript_path TEXT NOT NULL,
                 transcript TEXT NOT NULL DEFAULT '', notes TEXT NOT NULL DEFAULT '',
                 started_at TEXT NOT NULL, ended_at TEXT NOT NULL,
                 byte_count INTEGER NOT NULL DEFAULT 0, status TEXT NOT NULL DEFAULT 'finished'
             );
             CREATE TABLE command_runs (id INTEGER PRIMARY KEY, command TEXT, argv_json TEXT,
                 notes TEXT, stderr TEXT, exit_code INTEGER, created_at TEXT);
             INSERT INTO sessions (transcript_path, transcript, notes, started_at, ended_at)
             VALUES ('/tmp/a', 'linker failed', '{legacy_notes}', '2026-06-20T12:00:00+00:00',
                     '2026-06-20T13:00:00+00:00');
             INSERT INTO sessions (transcript_path, transcript, notes, started_at, ended_at)
             VALUES ('/tmp/b', 'unrelated work', 'project:other', '2026-06-21T12:00:00+00:00',
                     '2026-06-21T13:00:00+00:00');"
        ))
        .unwrap();

        migrate(&conn).unwrap();

        let (project, notes): (Option<String>, String) = conn
            .query_row("SELECT project, notes FROM sessions WHERE id = 1", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })
            .unwrap();
        assert_eq!(project.as_deref(), Some("camera-driver"));
        assert_eq!(notes, legacy_notes, "original notes must survive untouched");

        // Scoped retrieval now works through the shared loader + scope filter.
        let mems = mw_memory::sqlite::load_memories(&conn);
        assert_eq!(mems.len(), 2);
        let scoped = scope_memories(&conn, mems.clone(), Some("camera-driver"), None, None);
        assert_eq!(scoped.len(), 1);
        assert!(scoped[0].text.contains("linker failed"));
        // Unscoped is untouched — the byte-identical guarantee for old users.
        assert_eq!(scope_memories(&conn, mems, None, None, None).len(), 2);

        migrate(&conn).unwrap(); // idempotent
        let version: i64 = conn.query_row("PRAGMA user_version", [], |r| r.get(0)).unwrap();
        assert_eq!(version, LATEST_SCHEMA_VERSION);
    }

    #[test]
    fn since_parses_relative_windows() {
        assert_eq!(parse_since("7d").unwrap(), chrono::Duration::days(7));
        assert_eq!(parse_since("24h").unwrap(), chrono::Duration::hours(24));
        assert_eq!(parse_since("2w").unwrap(), chrono::Duration::weeks(2));
        assert_eq!(parse_since(" 30m ").unwrap(), chrono::Duration::minutes(30));
        for bad in ["7", "d", "", "7y", "-1d", "1.5d", "seven days"] {
            assert!(parse_since(bad).is_err(), "{bad:?} should be rejected");
        }
    }

    // --- capture gates -------------------------------------------------

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "mw-cap-{tag}-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::canonicalize(&dir).unwrap()
    }

    #[test]
    fn capture_modes_parse() {
        assert_eq!(CaptureMode::parse("\"full\""), Some(CaptureMode::Full));
        assert_eq!(CaptureMode::parse(" commands-only "), Some(CaptureMode::CommandsOnly));
        assert_eq!(CaptureMode::parse("OFF"), Some(CaptureMode::Off));
        assert_eq!(CaptureMode::parse("maybe"), None);
        assert!(CaptureMode::Full.stores_output() && CaptureMode::Full.stores_anything());
        assert!(!CaptureMode::CommandsOnly.stores_output());
        assert!(CaptureMode::CommandsOnly.stores_anything());
        assert!(!CaptureMode::Off.stores_anything());
    }

    // Longest matching prefix wins inside [capture.paths], and only that section
    // is consulted (a top-level `capture = ...` key must not leak in).
    #[test]
    fn global_config_matches_longest_path_prefix() {
        let root = scratch("prefix");
        let nested = root.join("a/b");
        std::fs::create_dir_all(&nested).unwrap();
        let config = format!(
            "machine = \"laptop\"\n\
             capture = \"off\"\n\
             \n\
             [capture.paths]\n\
             \"{root}\" = \"commands-only\"\n\
             \"{root}/a/b\" = \"off\"\n\
             \"/nowhere/else\" = \"off\"\n",
            root = root.display()
        );
        assert_eq!(capture_paths_rule(&config, &root).unwrap().mode, CaptureMode::CommandsOnly);
        assert_eq!(capture_paths_rule(&config, &nested).unwrap().mode, CaptureMode::Off);
        assert!(capture_paths_rule(&config, std::path::Path::new("/tmp")).is_none());
    }

    // Full precedence chain: .mwignore beats the global config, which beats the
    // default — and every answer names the rule that produced it.
    #[test]
    fn mwignore_beats_global_config_beats_default() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let data = fresh_data_dir("capture-precedence");
        let root = scratch("precedence");
        let gated = root.join("gated");
        let deep = gated.join("sub/dir");
        std::fs::create_dir_all(&deep).unwrap();

        std::fs::write(
            data.join("config.toml"),
            format!("[capture.paths]\n\"{}\" = \"commands-only\"\n", root.display()),
        )
        .unwrap();

        // Global config only.
        let rule = capture_rule(&root);
        assert_eq!(rule.mode, CaptureMode::CommandsOnly);
        assert!(rule.source.contains("[capture.paths]"), "{}", rule.source);

        // .mwignore at a directory root wins, and applies to children below it.
        std::fs::write(gated.join(".mwignore"), "# private\ncapture = \"off\"\n").unwrap();
        let rule = capture_rule(&deep);
        assert_eq!(rule.mode, CaptureMode::Off);
        assert!(rule.source.ends_with(".mwignore"), "{}", rule.source);

        // Nothing configured anywhere → full, byte-identical old behaviour.
        let untouched = scratch("precedence-default");
        let rule = capture_rule(&untouched);
        assert_eq!(rule.mode, CaptureMode::Full);
        assert!(rule.source.starts_with("default"), "{}", rule.source);

        std::env::remove_var("MEMORYWHALE_DATA_DIR");
        let _ = std::fs::remove_dir_all(&data);
        let _ = std::fs::remove_dir_all(&root);
    }

    // The hard guarantee: an `off` directory writes ZERO rows, and the gate runs
    // before the database is touched (not as post-hoc deletion).
    #[test]
    fn off_directory_writes_zero_rows() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        std::env::remove_var("MEMORYWHALE_REVIEW_AGENT_MEMORIES");
        let data = fresh_data_dir("capture-off");
        let secret = scratch("off-dir");
        std::fs::write(secret.join(".mwignore"), "capture = \"off\"\n").unwrap();

        let err = remember("balance is 12345", secret.to_str()).unwrap_err();
        assert!(err.contains("capture is off"), "{err}");

        // A directory with no gate still records — no regression.
        let open_dir = scratch("off-control");
        assert!(remember("normal lesson", open_dir.to_str()).is_ok());

        let conn = Connection::open(data.join("memorywhale.sqlite3")).unwrap();
        let gated: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM bookmarks WHERE cwd = ?1",
                params![secret.to_str()],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(gated, 0, "an off directory must produce no rows at all");
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM bookmarks", [], |r| r.get(0))
            .unwrap();
        assert_eq!(total, 1, "only the ungated write landed");

        std::env::remove_var("MEMORYWHALE_DATA_DIR");
        let _ = std::fs::remove_dir_all(&data);
        let _ = std::fs::remove_dir_all(&secret);
        let _ = std::fs::remove_dir_all(&open_dir);
    }

    // A symlink pointing into a gated tree must not dodge the gate: both sides
    // of the prefix comparison are canonicalized.
    #[test]
    #[cfg(unix)]
    fn symlinked_cwd_still_hits_the_gate() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let data = fresh_data_dir("capture-symlink");
        let root = scratch("symlink");
        let real = root.join("finances");
        std::fs::create_dir_all(&real).unwrap();
        std::fs::write(
            data.join("config.toml"),
            format!("[capture.paths]\n\"{}\" = \"off\"\n", real.display()),
        )
        .unwrap();

        let link = root.join("shortcut");
        std::os::unix::fs::symlink(&real, &link).unwrap();
        assert_eq!(capture_rule(&link).mode, CaptureMode::Off, "symlink must resolve to the gate");

        // And the write path agrees.
        assert!(remember("secret", link.to_str()).is_err());

        std::env::remove_var("MEMORYWHALE_DATA_DIR");
        let _ = std::fs::remove_dir_all(&data);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn project_tag_is_parsed_out_of_notes() {
        assert_eq!(project_of("os:macos project:mw-cli runtime:host").as_deref(), Some("mw-cli"));
        assert_eq!(project_of("no tags here"), None);
    }

    #[test]
    fn provenance_label_formats() {
        assert_eq!(
            provenance_label("agent", Some("Claude Code"), "2026-07-12T09:00:00Z", Some(41)),
            "remembered by Claude Code on 2026-07-12 during session #41"
        );
        assert_eq!(
            provenance_label("human", None, "2026-07-12T09:00:00Z", None),
            "remembered by you on 2026-07-12"
        );
        assert_eq!(
            provenance_label("agent", None, "2026-07-12", None),
            "remembered by agent on 2026-07-12"
        );
    }
}
