//! Shared helpers for the MemoryWhale CLI binaries.

pub mod tui;
pub mod storage;

use chrono::Utc;
use regex::Regex;
use rusqlite::{params, Connection};
use std::collections::HashMap;
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

/// Restrict a data directory or capture file to the current user on Unix.
/// Other platforms keep their native ACL behavior.
pub fn restrict_path_permissions(path: &std::path::Path, directory: bool) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = if directory { 0o700 } else { 0o600 };
        let permissions = std::fs::Permissions::from_mode(mode);
        std::fs::set_permissions(path, permissions)
            .map_err(|e| format!("failed to restrict permissions on {}: {e}", path.display()))?;
    }
    #[cfg(not(unix))]
    let _ = (path, directory);
    Ok(())
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
pub const LATEST_SCHEMA_VERSION: i64 = 6;

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
///
/// Migration 4 — error fingerprints: adds `command_runs.error_fingerprint` (+
/// index) and backfills it for existing failed rows, so recurring failures group
/// and `mw context --last-error` can show their outcome history immediately.
///
/// Migration 5 — MemPalace id-sync: creates the local `mempalace_sync` mapping
/// table (`mw_id` → wing/drawer_id/content_hash) so `mw sync-mempalace` is
/// idempotent by memory id — edited memories replace their drawer, unchanged
/// ones are skipped. Additive; safe on a populated DB.
///
/// Migration 6 — memory lifecycle: adds `status` and `superseded_by_id` to
/// bookmarks. Existing memories remain active; stale and superseded memories
/// retain their evidence while leaving normal retrieval.
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
        conn.execute_batch("PRAGMA user_version = 3;")
            .map_err(|e| format!("failed to bump schema version: {e}"))?;
    }
    if version < 4 {
        ensure_error_fingerprint(conn)?;
        backfill_error_fingerprints(conn)?;
        conn.execute_batch("PRAGMA user_version = 4;")
            .map_err(|e| format!("failed to bump schema version: {e}"))?;
    }
    if version < 5 {
        ensure_mempalace_sync(conn)?;
        conn.execute_batch("PRAGMA user_version = 5;")
            .map_err(|e| format!("failed to bump schema version: {e}"))?;
    }
    if version < 6 {
        conn.execute_batch(BOOKMARKS_BASE)
            .map_err(|e| format!("failed to prepare bookmarks table: {e}"))?;
        add_column_if_missing(conn, "bookmarks", "status", "TEXT NOT NULL DEFAULT 'active'")?;
        add_column_if_missing(conn, "bookmarks", "superseded_by_id", "INTEGER")?;
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_bookmarks_status ON bookmarks(status);
             PRAGMA user_version = 6;",
        )
        .map_err(|e| format!("failed to migrate memory lifecycle: {e}"))?;
    }
    Ok(())
}

/// Create the local `mempalace_sync` mapping table if absent. Additive and safe
/// on a populated DB. `mw_id` is the namespaced memory id (see
/// [`memorywhale_core::sqlite::decode_id`]), stable and unique across sources.
pub fn ensure_mempalace_sync(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS mempalace_sync (
             mw_id        INTEGER PRIMARY KEY,
             wing         TEXT NOT NULL,
             drawer_id    TEXT NOT NULL,
             content_hash TEXT NOT NULL,
             synced_at    TEXT NOT NULL
         );",
    )
    .map_err(|e| format!("failed to create mempalace_sync table: {e}"))
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

/// Add `command_runs.error_fingerprint` (+ its index) if the table exists and
/// lacks it. Same rationale as [`ensure_capture_kind`]: writers create
/// `command_runs` independently, so this is callable outside migration 4.
pub fn ensure_error_fingerprint(conn: &Connection) -> Result<(), String> {
    if table_exists(conn, "command_runs")? {
        add_column_if_missing(conn, "command_runs", "error_fingerprint", "TEXT")?;
        conn.execute_batch(
            "CREATE INDEX IF NOT EXISTS idx_command_runs_fingerprint
             ON command_runs(error_fingerprint);",
        )
        .map_err(|e| format!("failed to index error_fingerprint: {e}"))?;
    }
    Ok(())
}

/// Compute fingerprints for existing failed rows that don't have one, so the
/// "you've hit this N times" history works retroactively on a populated DB the
/// first time migration 4 runs.
fn backfill_error_fingerprints(conn: &Connection) -> Result<(), String> {
    if !table_exists(conn, "command_runs")? {
        return Ok(());
    }
    let rows: Vec<(i64, String, String)> = {
        let mut stmt = conn
            .prepare(
                "SELECT id, command, stderr FROM command_runs
                 WHERE exit_code IS NOT NULL AND exit_code != 0
                   AND error_fingerprint IS NULL",
            )
            .map_err(|e| format!("failed to scan for backfill: {e}"))?;
        let mapped = stmt
            .query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
            .map_err(|e| format!("backfill query: {e}"))?;
        mapped.filter_map(Result::ok).collect()
    };
    for (id, command, stderr) in rows {
        if let Some(fp) = error_fingerprint(&command, &stderr) {
            conn.execute(
                "UPDATE command_runs SET error_fingerprint = ?1 WHERE id = ?2",
                params![fp, id],
            )
            .map_err(|e| format!("backfill update: {e}"))?;
        }
    }
    Ok(())
}

/// Mask the volatile parts of one line of error output — absolute paths,
/// hex/memory addresses, `:line:col` positions, long hashes, and bare multi-digit
/// integers (pids, byte counts, timestamps) — so the same failure normalizes
/// identically across runs, machines, and directories.
///
/// Deliberately keeps letter-prefixed codes like `E0308` intact: those *are* the
/// error identity, so `E0308` and `E0277` must fingerprint differently.
pub fn normalize_error_line(line: &str) -> String {
    fn re(pat: &str) -> &'static Regex {
        // ponytail: tiny leaked cache keyed by pattern; the set is fixed and
        // compiled once. Simpler than five named statics.
        use std::collections::HashMap;
        use std::sync::Mutex;
        static CACHE: OnceLock<Mutex<HashMap<String, &'static Regex>>> = OnceLock::new();
        let mut map = CACHE.get_or_init(|| Mutex::new(HashMap::new())).lock().unwrap();
        map.entry(pat.to_string()).or_insert_with(|| {
            Box::leak(Box::new(Regex::new(pat).expect("static regex")))
        })
    }
    let mut s = line.trim().to_string();
    // Order matters: paths and hex before the generic integer mask.
    s = re(r"(?:[A-Za-z]:)?[/\\][\w.\-/\\]+").replace_all(&s, "<path>").into_owned();
    s = re(r"\b0x[0-9a-fA-F]+\b").replace_all(&s, "<addr>").into_owned();
    s = re(r":\d+:\d+").replace_all(&s, ":<n>:<n>").into_owned();
    s = re(r":\d+\b").replace_all(&s, ":<n>").into_owned();
    s = re(r"\b[0-9a-f]{7,}\b").replace_all(&s, "<hash>").into_owned();
    s = re(r"\b\d{3,}\b").replace_all(&s, "<n>").into_owned();
    s = re(r"\s+").replace_all(&s, " ").into_owned();
    s.trim().chars().take(200).collect()
}

/// A stable, portable fingerprint for a failed command: the command's base name
/// plus the most error-ish line of its stderr, normalized and FNV-1a hashed to
/// 8 hex chars. `None` when there's nothing error-shaped to fingerprint.
///
/// Pure and deterministic — no clock, no randomness, no platform-dependent
/// hasher — so a fingerprint is comparable across machines and reproducible.
pub fn error_fingerprint(command: &str, stderr: &str) -> Option<String> {
    let salient = stderr
        .lines()
        .find(|l| {
            let l = l.to_lowercase();
            ["error", "failed", "fatal", "cannot", "no such", "not found", "panic", "exception"]
                .iter()
                .any(|kw| l.contains(kw))
        })
        .or_else(|| stderr.lines().find(|l| !l.trim().is_empty()))?;
    let cmd = command.rsplit(['/', '\\']).next().unwrap_or(command);
    let sig = format!("{cmd}\n{}", normalize_error_line(salient));
    // FNV-1a, 32-bit — small, stable, dependency-free.
    let mut hash: u32 = 0x811c_9dc5;
    for b in sig.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    Some(format!("{hash:08x}"))
}

/// Stable, portable FNV-1a hash of arbitrary content → 8 hex chars. Same style
/// as [`error_fingerprint`] (not `std`'s `DefaultHasher`, whose output isn't
/// portable), used to detect when a synced memory's drawer content changed.
pub fn content_hash(content: &str) -> String {
    let mut hash: u32 = 0x811c_9dc5;
    for b in content.bytes() {
        hash ^= b as u32;
        hash = hash.wrapping_mul(0x0100_0193);
    }
    format!("{hash:08x}")
}

/// What we know about one recurring failure, assembled from `command_runs`.
#[derive(Debug, Clone)]
pub struct ErrorInsight {
    pub fingerprint: String,
    /// How many failed runs share this fingerprint.
    pub occurrences: i64,
    /// Of those failures, how many were later followed by a *successful* run of
    /// the same command — i.e. the problem went away at least once.
    pub resolutions: i64,
    /// Whether the most recent run of this command succeeded.
    pub currently_green: bool,
}

impl ErrorInsight {
    /// One-line human summary for `mw context --last-error`.
    pub fn summary(&self) -> String {
        let times = if self.occurrences == 1 { "once".into() } else { format!("{} times", self.occurrences) };
        if self.occurrences <= 1 {
            return format!("First time hitting this ({}).", self.fingerprint);
        }
        let outcome = if self.resolutions == 0 {
            "never resolved yet".to_string()
        } else {
            format!("a later run succeeded {} of {} times", self.resolutions, self.occurrences)
        };
        let now = if self.currently_green { "; the latest run is green" } else { "" };
        format!("You've hit this {times} — {outcome}{now}. [{}]", self.fingerprint)
    }
}

/// Assemble outcome history for a fingerprint. `resolutions` counts failure rows
/// that have a later successful run of the same command — the honest, evidence-
/// only version of "did the fix work", computed from observed exit codes rather
/// than anyone's claim.
pub fn error_insight(conn: &Connection, fingerprint: &str) -> Result<ErrorInsight, String> {
    // The command name behind this fingerprint (rows sharing it share it).
    let command: Option<String> = conn
        .query_row(
            "SELECT command FROM command_runs WHERE error_fingerprint = ?1 LIMIT 1",
            params![fingerprint],
            |r| r.get(0),
        )
        .ok();
    let Some(command) = command else {
        return Ok(ErrorInsight {
            fingerprint: fingerprint.to_string(),
            occurrences: 0,
            resolutions: 0,
            currently_green: false,
        });
    };
    let occurrences: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM command_runs WHERE error_fingerprint = ?1",
            params![fingerprint],
            |r| r.get(0),
        )
        .map_err(|e| format!("count failures: {e}"))?;
    // A failure "resolved" if a later row (higher id) ran the same command and
    // exited 0.
    let resolutions: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM command_runs f
             WHERE f.error_fingerprint = ?1
               AND EXISTS (
                   SELECT 1 FROM command_runs s
                   WHERE s.command = ?2 AND s.exit_code = 0 AND s.id > f.id
               )",
            params![fingerprint, command],
            |r| r.get(0),
        )
        .map_err(|e| format!("count resolutions: {e}"))?;
    let currently_green: bool = conn
        .query_row(
            "SELECT exit_code = 0 FROM command_runs
             WHERE command = ?1 AND exit_code IS NOT NULL
             ORDER BY id DESC LIMIT 1",
            params![command],
            |r| r.get::<_, Option<bool>>(0),
        )
        .ok()
        .flatten()
        .unwrap_or(false);
    Ok(ErrorInsight { fingerprint: fingerprint.to_string(), occurrences, resolutions, currently_green })
}

/// Scoring predicate for the memory-shortcut eval: is any blind-labelled `fix`
/// among the top-`k` retrieved note ids? `ranked_note_ids` are the *decoded*
/// bookmark ids (see [`memorywhale_core::sqlite::decode_id`]) in engine rank order.
/// Trivial by design — the honest work is in retrieval, not scoring — but kept
/// here (not in the example) so it's exercised by `cargo test`.
pub fn fix_surfaced(ranked_note_ids: &[i64], fix_ids: &[i64], k: usize) -> bool {
    ranked_note_ids.iter().take(k).any(|id| fix_ids.contains(id))
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
/// When the user has opted into the external MemPalace semantic engine, returns
/// the argv to spawn its MCP server; otherwise `None` (use the builtin engine).
///
/// `<data dir>/config.toml`:
/// ```toml
/// engine = "mempalace"
/// mempalace_command = "mempalace-mcp"   # optional; may include args, e.g. "npx mempalace-mcp"
/// ```
pub fn mempalace_command() -> Option<Vec<String>> {
    if config_value("engine")?.trim() != "mempalace" {
        return None;
    }
    mempalace_argv()
}

/// The MCP server argv, regardless of whether `engine = "mempalace"` is set —
/// used by `mw sync-mempalace`, which pushes to MemPalace even when search
/// still runs on the builtin engine.
pub fn mempalace_argv() -> Option<Vec<String>> {
    let cmd = config_value("mempalace_command").unwrap_or_else(|| "mempalace-mcp".into());
    let argv: Vec<String> = cmd.split_whitespace().map(str::to_string).collect();
    (!argv.is_empty()).then_some(argv)
}

/// MemPalace's search tool name. The real server exposes `mempalace_search`
/// (not `search`), so that's the default; overridable via `mempalace_tool` in
/// config for a differently-named server.
pub fn mempalace_search_tool() -> String {
    config_value("mempalace_tool").unwrap_or_else(|| "mempalace_search".into())
}

/// MemPalace's batch-write tool name, used by `mw sync-mempalace`.
pub fn mempalace_checkpoint_tool() -> String {
    config_value("mempalace_checkpoint_tool").unwrap_or_else(|| "mempalace_checkpoint".into())
}

/// MemPalace's single-drawer add tool (returns a `drawer_id`), used by the
/// id-based `mw sync-mempalace` reconcile. Overridable via `mempalace_add_tool`.
pub fn mempalace_add_tool() -> String {
    config_value("mempalace_add_tool").unwrap_or_else(|| "mempalace_add_drawer".into())
}

/// MemPalace's delete-by-id tool, used by the reconcile to replace an edited
/// drawer. Overridable via `mempalace_delete_tool`.
pub fn mempalace_delete_tool() -> String {
    config_value("mempalace_delete_tool").unwrap_or_else(|| "mempalace_delete_drawer".into())
}

/// One memory rendered into its target MemPalace drawer for a sync pass:
/// `mw_id` is the stable namespaced id, `content` already carries its
/// provenance tag/line, and `content_hash` is the change key.
#[derive(Debug, Clone)]
pub struct DesiredDrawer {
    pub mw_id: i64,
    pub wing: String,
    pub room: String,
    pub content: String,
    pub added_by: String,
    pub content_hash: String,
}

/// A drawer to (re)file this pass. `old_drawer_id = Some` means it replaces an
/// edited drawer (delete the old one first); `None` means it's brand new.
#[derive(Debug, Clone)]
pub struct PlanItem {
    pub mw_id: i64,
    pub wing: String,
    pub room: String,
    pub content: String,
    pub added_by: String,
    pub content_hash: String,
    pub old_drawer_id: Option<String>,
}

/// The reconcile plan: drawers to file (new + updated) and how many were skipped.
#[derive(Debug, Default)]
pub struct SyncPlan {
    pub items: Vec<PlanItem>,
    pub unchanged: usize,
}

impl SyncPlan {
    /// Count of brand-new drawers.
    pub fn added(&self) -> usize {
        self.items.iter().filter(|i| i.old_drawer_id.is_none()).count()
    }
    /// Count of replaced (edited) drawers — also the number of deletes.
    pub fn updated(&self) -> usize {
        self.items.iter().filter(|i| i.old_drawer_id.is_some()).count()
    }
}

/// Load the local `mempalace_sync` mapping: `mw_id → (drawer_id, content_hash)`.
pub fn load_sync_map(conn: &Connection) -> Result<HashMap<i64, (String, String)>, String> {
    let mut stmt = conn
        .prepare("SELECT mw_id, drawer_id, content_hash FROM mempalace_sync")
        .map_err(|e| format!("failed to read mempalace_sync: {e}"))?;
    let rows = stmt
        .query_map([], |r| {
            Ok((r.get::<_, i64>(0)?, (r.get::<_, String>(1)?, r.get::<_, String>(2)?)))
        })
        .map_err(|e| format!("failed to scan mempalace_sync: {e}"))?;
    Ok(rows.filter_map(Result::ok).collect())
}

/// Diff `desired` drawers against the existing mapping into a [`SyncPlan`]:
/// not-in-map → add; in-map & hash changed → update (delete old + add); in-map &
/// hash unchanged → skip. Pure — no DB, no network — so it's directly testable.
pub fn plan_sync(existing: &HashMap<i64, (String, String)>, desired: &[DesiredDrawer]) -> SyncPlan {
    let mut plan = SyncPlan::default();
    for d in desired {
        let old_drawer_id = match existing.get(&d.mw_id) {
            Some((_, hash)) if *hash == d.content_hash => {
                plan.unchanged += 1;
                continue;
            }
            Some((drawer_id, _)) => Some(drawer_id.clone()),
            None => None,
        };
        plan.items.push(PlanItem {
            mw_id: d.mw_id,
            wing: d.wing.clone(),
            room: d.room.clone(),
            content: d.content.clone(),
            added_by: d.added_by.clone(),
            content_hash: d.content_hash.clone(),
            old_drawer_id,
        });
    }
    plan
}

/// Persist new/updated mappings in one transaction (INSERT OR REPLACE keyed on
/// the `mw_id` primary key, so an update overwrites the old drawer_id/hash).
pub fn record_sync(
    conn: &mut Connection,
    rows: &[(i64, String, String, String)], // (mw_id, wing, drawer_id, content_hash)
    synced_at: &str,
) -> Result<(), String> {
    let tx = conn.transaction().map_err(|e| format!("failed to open transaction: {e}"))?;
    for (mw_id, wing, drawer_id, hash) in rows {
        tx.execute(
            "INSERT OR REPLACE INTO mempalace_sync
                 (mw_id, wing, drawer_id, content_hash, synced_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![mw_id, wing, drawer_id, hash, synced_at],
        )
        .map_err(|e| format!("failed to record sync mapping: {e}"))?;
    }
    tx.commit().map_err(|e| format!("failed to commit sync mappings: {e}"))
}

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
    mut mems: Vec<memorywhale_core::Memory>,
    project: Option<&str>,
    machine: Option<&str>,
    since: Option<chrono::DateTime<Utc>>,
) -> Vec<memorywhale_core::Memory> {
    use memorywhale_core::sqlite::{decode_id, Source};

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
/// retrieval until approved in the dashboard. On by default. Set
/// `MEMORYWHALE_REVIEW_AGENT_MEMORIES=0` or
/// `review_agent_memories = false` in `<data dir>/config.toml` to opt into
/// automatic approval.
pub fn review_agent_memories() -> bool {
    if let Some(v) = std::env::var_os("MEMORYWHALE_REVIEW_AGENT_MEMORIES") {
        return v == "1" || v == "true";
    }
    data_dir()
        .ok()
        .map(|d| d.join("config.toml"))
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|s| {
            s.lines().find_map(|line| {
                let (key, value) = line.split_once('=')?;
                (key.trim() == "review_agent_memories")
                    .then(|| matches!(value.trim(), "true" | "\"true\"" | "1"))
            })
        })
        .unwrap_or(true)
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

/// A pending (unapproved) agent-written memory awaiting review, with the fields
/// needed to render a [`provenance_label`].
pub struct PendingNote {
    pub id: i64,
    pub label: String,
    pub created_at: String,
    pub author_kind: String,
    pub author_name: Option<String>,
    pub source_session_id: Option<i64>,
}

impl PendingNote {
    /// Human-readable provenance for this note (see [`provenance_label`]).
    pub fn provenance(&self) -> String {
        provenance_label(
            &self.author_kind,
            self.author_name.as_deref(),
            &self.created_at,
            self.source_session_id,
        )
    }
}

/// Agent-written memories still awaiting review (`approved = 0`), newest first.
/// Mirrors the dashboard's pending-list query. Human notes are always approved,
/// so this is effectively the agent-review queue.
pub fn pending_agent_notes(conn: &Connection) -> Result<Vec<PendingNote>, String> {
    conn.prepare(
        "SELECT id, label, created_at, author_kind, author_name, source_session_id
         FROM bookmarks
         WHERE approved = 0 AND author_kind = 'agent'
         ORDER BY id DESC LIMIT 500",
    )
    .and_then(|mut stmt| {
        stmt.query_map([], |r| {
            Ok(PendingNote {
                id: r.get(0)?,
                label: r.get(1)?,
                created_at: r.get(2)?,
                author_kind: r.get(3)?,
                author_name: r.get(4)?,
                source_session_id: r.get(5)?,
            })
        })?
        .collect()
    })
    .map_err(|e| format!("failed to list pending notes: {e}"))
}

/// Approve a pending note so retrieval includes it (`UPDATE approved = 1`).
pub fn approve_note(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("UPDATE bookmarks SET approved = 1 WHERE id = ?1", params![id])
        .map(|_| ())
        .map_err(|e| format!("failed to approve note: {e}"))
}

/// Reject a pending note by deleting the row.
pub fn reject_note(conn: &Connection, id: i64) -> Result<(), String> {
    conn.execute("DELETE FROM bookmarks WHERE id = ?1", params![id])
        .map(|_| ())
        .map_err(|e| format!("failed to reject note: {e}"))
}

/// Mark a memory stale without deleting its source evidence.
pub fn mark_note_stale(conn: &Connection, id: i64) -> Result<(), String> {
    let changed = conn
        .execute(
            "UPDATE bookmarks SET status = 'stale', superseded_by_id = NULL WHERE id = ?1",
            params![id],
        )
        .map_err(|e| format!("failed to mark memory stale: {e}"))?;
    if changed == 0 {
        return Err(format!("no memory #{id}"));
    }
    Ok(())
}

/// Retire an old memory in favor of a newer one while preserving both rows.
pub fn supersede_note(conn: &Connection, old_id: i64, replacement_id: i64) -> Result<(), String> {
    if old_id == replacement_id {
        return Err("a memory cannot supersede itself".to_string());
    }
    let replacement_exists = conn
        .query_row("SELECT 1 FROM bookmarks WHERE id = ?1", params![replacement_id], |_| Ok(()))
        .is_ok();
    if !replacement_exists {
        return Err(format!("no replacement memory #{replacement_id}"));
    }
    let changed = conn
        .execute(
            "UPDATE bookmarks
             SET status = 'superseded', superseded_by_id = ?2
             WHERE id = ?1",
            params![old_id, replacement_id],
        )
        .map_err(|e| format!("failed to supersede memory: {e}"))?;
    if changed == 0 {
        return Err(format!("no memory #{old_id}"));
    }
    Ok(())
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
    let text = sanitize_capture(text);
    let path = database_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create data dir: {e}"))?;
    }
    let conn = storage::open_path(&path)?;
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
pub const DEFAULT_MAX_CAPTURE_BYTES: usize = 1_048_576;

/// Maximum bytes retained for any individual captured text field.
///
/// Set `MEMORYWHALE_MAX_CAPTURE_BYTES` to a positive integer to tune the
/// limit. The default is 1 MiB per stdout, stderr, note, or transcript field.
pub fn max_capture_bytes() -> usize {
    std::env::var("MEMORYWHALE_MAX_CAPTURE_BYTES")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(DEFAULT_MAX_CAPTURE_BYTES)
}

/// Redact known secret shapes and bound the stored value, adding an explicit
/// marker when bytes were omitted.
pub fn sanitize_capture(text: &str) -> String {
    truncate_capture(&redact(text), max_capture_bytes())
}

fn truncate_capture(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_string();
    }
    let mut end = limit.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!(
        "{}\n[TRUNCATED: stored {} of {} bytes]",
        &text[..end],
        end,
        text.len()
    )
}

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
    fn fix_surfaced_hit_and_miss() {
        // Fix id present within k → hit; outside k or absent → miss.
        assert!(fix_surfaced(&[9, 2, 5], &[2], 5));
        assert!(!fix_surfaced(&[9, 5, 7, 8, 6, 2], &[2], 5)); // fix at rank 6, k=5
        assert!(!fix_surfaced(&[9, 5, 7], &[2], 5)); // fix not retrieved at all
    }

    /// End-to-end scoring over the REAL retrieval path (load_memories +
    /// BuiltinEngine): a task whose fix note is in the corpus scores a hit; a
    /// task with no matching fix scores a miss. Deterministic (fixed `now`).
    #[test]
    fn real_ranking_surfaces_fix_in_corpus() {
        use chrono::{TimeZone, Utc};
        use memorywhale_core::engine::{BuiltinEngine, MemoryEngine};
        use memorywhale_core::sqlite::{decode_id, load_memories, Source};
        use memorywhale_core::Query;

        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE bookmarks (id INTEGER PRIMARY KEY, label TEXT, created_at TEXT, approved INTEGER DEFAULT 1);
             INSERT INTO bookmarks (id, label, created_at, approved) VALUES
                 (2,  'Fixed the E0308 in the camera driver by casting the decoded pixel back to u16 with as u16.', '2026-07-17T12:00:00+00:00', 1),
                 (26, 'SQLite database is locked SQLITE_BUSY: set PRAGMA journal_mode=WAL and a busy_timeout.', '2026-07-01T12:00:00+00:00', 1),
                 (7,  'git push rejected non-fast-forward: fixed with git pull --rebase then push.', '2026-07-18T12:00:00+00:00', 1),
                 (18, 'undefined symbol sqlite3_open_v2 linker error: added rusqlite bundled feature.', '2026-07-08T12:00:00+00:00', 1),
                 (22, 'openssl-sys build failed: switched reqwest to rustls-tls.', '2026-07-05T12:00:00+00:00', 1),
                 (31, 'brew install SHA256 mismatch: recomputed the Formula sha256.', '2026-07-13T12:00:00+00:00', 1),
                 (99, 'Unrelated note about docker COPY and dockerignore patterns.', '2026-07-10T12:00:00+00:00', 1);",
        )
        .unwrap();

        let now = Utc.with_ymd_and_hms(2026, 7, 19, 12, 0, 0).unwrap();
        let rank = |q: &str| -> Vec<i64> {
            let eng = BuiltinEngine::new(load_memories(&conn));
            eng.retrieve(&Query::new(q, now), 20)
                .into_iter()
                .filter_map(|s| match decode_id(s.memory.id) {
                    (Source::Note, real) => Some(real),
                    _ => None,
                })
                .collect()
        };

        // Fix in corpus (#2) → hit within top-5 for its own query.
        assert!(fix_surfaced(&rank("camera driver expected u16 found u32 mismatch"), &[2], 5));
        // Selectivity: the camera fix is NOT the top hit for an unrelated query.
        assert!(!fix_surfaced(&rank("sqlite database is locked SQLITE_BUSY concurrent writes"), &[2], 1));
        // A failure whose fix isn't in the corpus (empty label set) → always a miss.
        assert!(!fix_surfaced(&rank("cannot find type GpuContext in this scope wgpu"), &[], 5));
    }

    #[test]
    fn mempalace_command_reads_config() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = fresh_data_dir("mempalace-cfg");

        // No config → builtin (None).
        assert!(mempalace_command().is_none());

        // engine set but not mempalace → still None.
        std::fs::write(dir.join("config.toml"), "engine = \"builtin\"\n").unwrap();
        assert!(mempalace_command().is_none());

        // Opted in, default command.
        std::fs::write(dir.join("config.toml"), "engine = \"mempalace\"\n").unwrap();
        assert_eq!(mempalace_command(), Some(vec!["mempalace-mcp".to_string()]));

        // Opted in with an override that carries args.
        std::fs::write(
            dir.join("config.toml"),
            "engine = \"mempalace\"\nmempalace_command = \"npx mempalace-mcp --stdio\"\n",
        )
        .unwrap();
        assert_eq!(
            mempalace_command(),
            Some(vec!["npx".into(), "mempalace-mcp".into(), "--stdio".into()])
        );

        std::env::remove_var("MEMORYWHALE_DATA_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn mempalace_tool_names_default_to_real_server() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = fresh_data_dir("mempalace-tools");

        // Defaults match the real MemPalace server (not the generic "search").
        assert_eq!(mempalace_search_tool(), "mempalace_search");
        assert_eq!(mempalace_checkpoint_tool(), "mempalace_checkpoint");
        assert_eq!(mempalace_add_tool(), "mempalace_add_drawer");
        assert_eq!(mempalace_delete_tool(), "mempalace_delete_drawer");

        // Overridable for a differently-named server.
        std::fs::write(
            dir.join("config.toml"),
            "mempalace_tool = \"custom_search\"\nmempalace_checkpoint_tool = \"custom_write\"\n\
             mempalace_add_tool = \"custom_add\"\nmempalace_delete_tool = \"custom_del\"\n",
        )
        .unwrap();
        assert_eq!(mempalace_search_tool(), "custom_search");
        assert_eq!(mempalace_checkpoint_tool(), "custom_write");
        assert_eq!(mempalace_add_tool(), "custom_add");
        assert_eq!(mempalace_delete_tool(), "custom_del");

        std::env::remove_var("MEMORYWHALE_DATA_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    fn desired(mw_id: i64, content: &str) -> DesiredDrawer {
        DesiredDrawer {
            mw_id,
            wing: "memorywhale".into(),
            room: "note".into(),
            content: content.into(),
            added_by: "you".into(),
            content_hash: content_hash(content),
        }
    }

    #[test]
    fn content_hash_is_stable_and_discriminating() {
        assert_eq!(content_hash("hello world"), content_hash("hello world"));
        assert_ne!(content_hash("a"), content_hash("b"));
    }

    #[test]
    fn sync_reconcile_adds_skips_and_updates() {
        let mut conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap(); // creates mempalace_sync (v5)

        let d1 = desired(3_000_000_001, "[tag]\nhello");
        let d2 = desired(1_000_000_002, "cargo build");
        let want = vec![d1.clone(), d2.clone()];

        // First sync: empty map -> both ADD, nothing to skip.
        let plan = plan_sync(&load_sync_map(&conn).unwrap(), &want);
        assert_eq!((plan.added(), plan.updated(), plan.unchanged), (2, 0, 0));
        assert!(plan.items[0].old_drawer_id.is_none());
        let rows: Vec<_> = plan
            .items
            .iter()
            .enumerate()
            .map(|(i, it)| (it.mw_id, it.wing.clone(), format!("drawer-{i}"), it.content_hash.clone()))
            .collect();
        record_sync(&mut conn, &rows, "2026-07-20T00:00:00+00:00").unwrap();

        // Re-sync with no changes -> every drawer SKIPPED, zero ops.
        let plan = plan_sync(&load_sync_map(&conn).unwrap(), &want);
        assert!(plan.items.is_empty());
        assert_eq!(plan.unchanged, 2);

        // Edit d1's content -> hash changes -> UPDATE (delete old drawer-0 + add).
        let d1b = desired(d1.mw_id, "[tag]\nhello edited");
        let want2 = vec![d1b.clone(), d2.clone()];
        let plan = plan_sync(&load_sync_map(&conn).unwrap(), &want2);
        assert_eq!((plan.added(), plan.updated(), plan.unchanged), (0, 1, 1));
        assert_eq!(plan.items[0].mw_id, d1.mw_id);
        assert_eq!(plan.items[0].old_drawer_id.as_deref(), Some("drawer-0"));

        // Persisting the replacement makes the next pass fully unchanged.
        let it = &plan.items[0];
        record_sync(
            &mut conn,
            &[(it.mw_id, it.wing.clone(), "drawer-9".into(), it.content_hash.clone())],
            "2026-07-20T01:00:00+00:00",
        )
        .unwrap();
        let plan = plan_sync(&load_sync_map(&conn).unwrap(), &want2);
        assert!(plan.items.is_empty());
        assert_eq!(plan.unchanged, 2);
    }

    #[test]
    fn fingerprint_is_stable_across_volatile_noise() {
        // Same E0308 in two builds: different path, line:col, and a build hash.
        let a = "error[E0308]: mismatched types\n --> /home/ann/proj/src/cam.rs:42:19\nbuild abcdef0123456 failed";
        let b = "error[E0308]: mismatched types\n --> /tmp/ci/9/src/cam.rs:8:3\nbuild 99fedcba00011 failed";
        let fa = error_fingerprint("cargo", a).unwrap();
        let fb = error_fingerprint("cargo", b).unwrap();
        assert_eq!(fa, fb, "path/line/hash noise must not change the fingerprint");
    }

    #[test]
    fn fingerprint_distinguishes_error_codes() {
        // E0308 vs E0277 are different failures and must not collide — the
        // letter-prefixed code is preserved through normalization.
        let e308 = error_fingerprint("cargo", "error[E0308]: mismatched types").unwrap();
        let e277 = error_fingerprint("cargo", "error[E0277]: trait not satisfied").unwrap();
        assert_ne!(e308, e277);
    }

    #[test]
    fn fingerprint_none_when_no_error_text() {
        assert!(error_fingerprint("ls", "").is_none());
        assert!(error_fingerprint("ls", "   \n  \n").is_none());
    }

    #[test]
    fn insight_counts_occurrences_and_resolutions() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE command_runs (
                id INTEGER PRIMARY KEY, command TEXT NOT NULL, argv_json TEXT NOT NULL DEFAULT '[]',
                cwd TEXT, exit_code INTEGER, stdout TEXT NOT NULL DEFAULT '',
                stderr TEXT NOT NULL DEFAULT '', notes TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL DEFAULT '', error_fingerprint TEXT);",
        )
        .unwrap();
        let err = "error[E0308]: mismatched types";
        let fp = error_fingerprint("cargo", err).unwrap();
        // Timeline: fail, fail, then a success (the fix), then fail again.
        let insert = |exit: i64, fp: Option<&str>| {
            conn.execute(
                "INSERT INTO command_runs (command, exit_code, stderr, error_fingerprint)
                 VALUES ('cargo', ?1, ?2, ?3)",
                params![exit, err, fp],
            )
            .unwrap();
        };
        insert(101, Some(&fp));
        insert(101, Some(&fp));
        insert(0, None); // the fix worked
        insert(101, Some(&fp)); // regressed later

        let insight = error_insight(&conn, &fp).unwrap();
        assert_eq!(insight.occurrences, 3, "three failing runs share the fingerprint");
        // The two failures *before* the success each have a later green run.
        assert_eq!(insight.resolutions, 2);
        assert!(!insight.currently_green, "last run failed again");
        assert!(insight.summary().contains("3 times"));
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
    fn bounded_capture_marks_truncation_without_splitting_utf8() {
        assert_eq!(truncate_capture("short", 8), "short");
        let value = truncate_capture("ab🐋cd", 5);
        assert_eq!(value, "ab\n[TRUNCATED: stored 2 of 8 bytes]");
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

        // A few distinct secret shapes in one note; the fake credentials are
        // hand-authored, never real. Assert both that the scrub fired AND that
        // no raw secret survived into the stored row — this is the end-to-end
        // guarantee `redact()`'s own unit tests can't make.
        let secrets = ["abcdef123456", "ghp_0123456789abcdefghijABCDEF", "hunter2secret"];
        let note = "the fix: API_KEY=abcdef123456, token ghp_0123456789abcdefghijABCDEF, \
                    password: hunter2secret in .env";
        let id = remember(note, Some("/tmp/repo")).unwrap();
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
        for secret in secrets {
            assert!(!label.contains(secret), "raw secret {secret:?} landed in the db row: {label}");
        }
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

    // The MCP `remember` tool attributes the lesson and defaults it to pending.
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
        assert_eq!(approved, 0, "agent memory is pending review by default");

        std::env::remove_var("MEMORYWHALE_DATA_DIR");
        let _ = std::fs::remove_dir_all(&dir);
    }

    // Users can explicitly opt into automatic approval.
    #[test]
    fn review_opt_out_auto_approves_agent_memories() {
        let _guard = ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        let dir = fresh_data_dir("prov-review");
        std::env::set_var("MEMORYWHALE_REVIEW_AGENT_MEMORIES", "0");

        let agent_id = remember_as("unreviewed agent claim", None, "agent", Some("Codex"), None).unwrap();
        let human_id = remember("trusted human note", None).unwrap();

        let conn = Connection::open(dir.join("memorywhale.sqlite3")).unwrap();
        let agent_approved: i64 = conn
            .query_row("SELECT approved FROM bookmarks WHERE id = ?1", [agent_id], |r| r.get(0))
            .unwrap();
        assert_eq!(agent_approved, 1, "explicit opt-out enables automatic approval");

        // Assert through the REAL retrieval path — the shared loader every
        // surface (mw search, MCP, desktop) goes through — not a re-implemented
        // query, so this test fails if the loader ever drops the approved filter.
        let loaded = memorywhale_core::sqlite::load_memories(&conn);
        let visible: Vec<i64> = loaded
            .iter()
            .filter_map(|m| match memorywhale_core::sqlite::decode_id(m.id) {
                (memorywhale_core::sqlite::Source::Note, id) => Some(id),
                _ => None,
            })
            .collect();
        assert!(visible.contains(&human_id), "human memory visible");
        assert!(visible.contains(&agent_id), "auto-approved memory visible");

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

    // The TUI review primitives, straight against an in-memory DB.
    #[test]
    fn review_primitives_approve_reject_and_list() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let insert = |label: &str, kind: &str, approved: i64| {
            conn.execute(
                "INSERT INTO bookmarks (label, created_at, author_kind, author_name, source_session_id, approved)
                 VALUES (?1, '2026-07-12T09:00:00Z', ?2, ?3, ?4, ?5)",
                params![label, kind, Some("Claude Code"), Some(41_i64), approved],
            )
            .unwrap();
            conn.last_insert_rowid()
        };
        let pending = insert("agent pending", "agent", 0);
        insert("agent already approved", "agent", 1);
        insert("human note", "human", 0); // human unapproved: never in the queue

        // pending list only surfaces the unapproved agent row, with provenance.
        let notes = pending_agent_notes(&conn).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].id, pending);
        assert!(notes[0].provenance().contains("Claude Code"));

        // approve flips approved to 1 and empties the queue.
        approve_note(&conn, pending).unwrap();
        let approved: i64 = conn
            .query_row("SELECT approved FROM bookmarks WHERE id = ?1", [pending], |r| r.get(0))
            .unwrap();
        assert_eq!(approved, 1);
        assert!(pending_agent_notes(&conn).unwrap().is_empty());

        // reject deletes the row.
        let doomed = insert("agent to reject", "agent", 0);
        reject_note(&conn, doomed).unwrap();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM bookmarks WHERE id = ?1", [doomed], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn stale_and_superseded_memories_leave_retrieval_but_keep_evidence() {
        let conn = Connection::open_in_memory().unwrap();
        migrate(&conn).unwrap();
        let insert = |label: &str| {
            conn.execute(
                "INSERT INTO bookmarks
                    (label, created_at, author_kind, approved, status)
                 VALUES (?1, '2026-07-12T09:00:00Z', 'human', 1, 'active')",
                params![label],
            )
            .unwrap();
            conn.last_insert_rowid()
        };
        let stale = insert("old environment advice");
        let old = insert("use the legacy flag");
        let replacement = insert("use the replacement flag");

        mark_note_stale(&conn, stale).unwrap();
        supersede_note(&conn, old, replacement).unwrap();

        let kept: (String, Option<i64>) = conn
            .query_row(
                "SELECT status, superseded_by_id FROM bookmarks WHERE id = ?1",
                [old],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(kept, ("superseded".to_string(), Some(replacement)));

        let visible: Vec<i64> = memorywhale_core::sqlite::load_memories(&conn)
            .into_iter()
            .filter_map(|memory| match memorywhale_core::sqlite::decode_id(memory.id) {
                (memorywhale_core::sqlite::Source::Note, id) => Some(id),
                _ => None,
            })
            .collect();
        assert_eq!(visible, vec![replacement]);
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM bookmarks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(total, 3, "lifecycle changes preserve every source row");
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
        let mems = memorywhale_core::sqlite::load_memories(&conn);
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
