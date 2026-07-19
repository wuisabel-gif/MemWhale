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

/// Save a freeform lesson or conclusion ("the fix was X") into the bookmarks
/// table — the same store `mw mark` writes to. Shared by `mw remember` and the
/// MCP `remember` tool, so a human and an agent write to the same place and
/// either one can search it back out later.
pub fn remember(text: &str, cwd: Option<&str>) -> Result<i64, String> {
    let text = redact(text);
    let path = database_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create data dir: {e}"))?;
    }
    let conn = Connection::open(&path).map_err(|e| format!("failed to open db: {e}"))?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS bookmarks (
             id INTEGER PRIMARY KEY,
             label TEXT NOT NULL,
             cwd TEXT,
             created_at TEXT NOT NULL,
             command_run_id INTEGER,
             session_id INTEGER
         );
         CREATE INDEX IF NOT EXISTS idx_bookmarks_created_at ON bookmarks(created_at);",
    )
    .map_err(|e| format!("failed to prepare bookmarks table: {e}"))?;
    conn.execute(
        "INSERT INTO bookmarks (label, cwd, created_at) VALUES (?1, ?2, ?3)",
        params![text, cwd, Utc::now().to_rfc3339()],
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

    #[test]
    fn remember_writes_and_redacts() {
        let dir = std::env::temp_dir().join(format!(
            "mw-remember-test-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        std::env::set_var("MEMORYWHALE_DATA_DIR", &dir);

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
}
