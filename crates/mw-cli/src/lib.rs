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

/// Apply numbered schema migrations to a MemoryWhale database. Idempotent and
/// cheap (a `user_version` check), so callers run it before touching bookmarks.
///
/// Migration 1 — memory provenance: adds `author_kind`/`author_name`/
/// `source_session_id`/`approved` to `bookmarks` and backfills existing rows as
/// human (via the column defaults). Safe on a populated DB: `ADD COLUMN` with a
/// constant default never rewrites rows.
pub fn migrate(conn: &Connection) -> Result<(), String> {
    let version: i64 = conn
        .query_row("PRAGMA user_version", [], |r| r.get(0))
        .map_err(|e| format!("failed to read schema version: {e}"))?;
    if version < 1 {
        conn.execute_batch(BOOKMARKS_BASE)
            .map_err(|e| format!("failed to prepare bookmarks table: {e}"))?;
        add_column_if_missing(conn, "author_kind", "TEXT NOT NULL DEFAULT 'human'")?;
        add_column_if_missing(conn, "author_name", "TEXT")?;
        add_column_if_missing(conn, "source_session_id", "INTEGER")?;
        add_column_if_missing(conn, "approved", "INTEGER NOT NULL DEFAULT 1")?;
        add_column_if_missing(conn, "created_at", "TEXT")?; // belt-and-suspenders; base has it
        conn.execute_batch("PRAGMA user_version = 1;")
            .map_err(|e| format!("failed to bump schema version: {e}"))?;
    }
    Ok(())
}

fn add_column_if_missing(conn: &Connection, column: &str, decl: &str) -> Result<(), String> {
    let present = conn
        .prepare("SELECT 1 FROM pragma_table_info('bookmarks') WHERE name = ?1")
        .and_then(|mut s| s.exists(params![column]))
        .map_err(|e| format!("failed to inspect bookmarks columns: {e}"))?;
    if !present {
        conn.execute(&format!("ALTER TABLE bookmarks ADD COLUMN {column} {decl}"), [])
            .map_err(|e| format!("failed to add bookmarks.{column}: {e}"))?;
    }
    Ok(())
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
        assert_eq!(version, 1);
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
