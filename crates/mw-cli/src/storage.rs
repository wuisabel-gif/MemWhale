//! Canonical SQLite connection and schema ownership for every CLI surface.

use rusqlite::{params, Connection};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Open MemoryWhale's default database and bring it to the current schema.
pub fn open() -> Result<Connection, String> {
    open_path(&crate::database_path()?)
}

/// Open a database at an explicit path and bring it to the current schema.
pub fn open_path(path: &Path) -> Result<Connection, String> {
    let conn =
        Connection::open(path).map_err(|e| format!("failed to open {}: {e}", path.display()))?;
    initialize(&conn)?;
    Ok(conn)
}

/// Apply the connection policy and canonical base schema, then run migrations.
///
/// Keeping the complete latest shape here lets every binary safely open either
/// a fresh database or one created by an older MemoryWhale release.
pub fn initialize(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        PRAGMA journal_mode = WAL;
        PRAGMA foreign_keys = ON;
        PRAGMA busy_timeout = 3000;

        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY,
            shell TEXT,
            cwd TEXT,
            transcript_path TEXT NOT NULL DEFAULT '',
            transcript TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            started_at TEXT NOT NULL DEFAULT '',
            ended_at TEXT NOT NULL DEFAULT '',
            byte_count INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'finished',
            project TEXT,
            machine TEXT
        );

        CREATE TABLE IF NOT EXISTS command_runs (
            id INTEGER PRIMARY KEY,
            command TEXT NOT NULL,
            argv_json TEXT NOT NULL,
            cwd TEXT,
            exit_code INTEGER,
            stdout TEXT NOT NULL DEFAULT '',
            stderr TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            created_at TEXT NOT NULL,
            capture_kind TEXT NOT NULL DEFAULT 'full',
            error_fingerprint TEXT
        );

        CREATE TABLE IF NOT EXISTS command_arguments (
            id INTEGER PRIMARY KEY,
            command_run_id INTEGER NOT NULL,
            position INTEGER NOT NULL,
            value TEXT NOT NULL,
            FOREIGN KEY(command_run_id) REFERENCES command_runs(id) ON DELETE CASCADE
        );

        CREATE TABLE IF NOT EXISTS bookmarks (
            id INTEGER PRIMARY KEY,
            label TEXT NOT NULL,
            cwd TEXT,
            created_at TEXT NOT NULL,
            command_run_id INTEGER,
            session_id INTEGER,
            author_kind TEXT NOT NULL DEFAULT 'human',
            author_name TEXT,
            source_session_id INTEGER,
            approved INTEGER NOT NULL DEFAULT 1
        );

        CREATE TABLE IF NOT EXISTS screenshots (
            id INTEGER PRIMARY KEY,
            command_run_id INTEGER,
            file_path TEXT NOT NULL,
            cwd TEXT,
            notes TEXT NOT NULL DEFAULT '',
            captured_at TEXT NOT NULL,
            FOREIGN KEY(command_run_id) REFERENCES command_runs(id) ON DELETE SET NULL
        );

        ",
    )
    .map_err(|e| format!("failed to initialize database: {e}"))?;
    crate::migrate(conn)?;
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_sessions_started_at ON sessions(started_at);
         CREATE INDEX IF NOT EXISTS idx_command_runs_command ON command_runs(command);
         CREATE INDEX IF NOT EXISTS idx_command_runs_exit_code ON command_runs(exit_code);
         CREATE INDEX IF NOT EXISTS idx_command_runs_fingerprint ON command_runs(error_fingerprint);
         CREATE INDEX IF NOT EXISTS idx_command_arguments_value ON command_arguments(value);
         CREATE INDEX IF NOT EXISTS idx_bookmarks_created_at ON bookmarks(created_at);
         CREATE INDEX IF NOT EXISTS idx_screenshots_command_run_id ON screenshots(command_run_id);
         CREATE INDEX IF NOT EXISTS idx_screenshots_captured_at ON screenshots(captured_at);",
    )
    .map_err(|e| format!("failed to initialize database indexes: {e}"))
}

/// Rows and managed capture files associated with one repository tree.
#[derive(Debug)]
pub struct RepositoryDeletionPlan {
    pub root: PathBuf,
    pub sessions: usize,
    pub command_runs: usize,
    pub bookmarks: usize,
    pub screenshots: usize,
    pub managed_files: Vec<PathBuf>,
    session_ids: HashSet<i64>,
    command_run_ids: HashSet<i64>,
    bookmark_ids: HashSet<i64>,
    screenshot_ids: HashSet<i64>,
    sync_ids: HashSet<i64>,
}

impl RepositoryDeletionPlan {
    pub fn total_rows(&self) -> usize {
        self.sessions + self.command_runs + self.bookmarks + self.screenshots
    }
}

fn cwd_is_within(cwd: Option<&str>, root: &Path) -> bool {
    cwd.filter(|cwd| !cwd.is_empty())
        .map(Path::new)
        .is_some_and(|cwd| {
            let resolved = cwd.canonicalize().unwrap_or_else(|_| cwd.to_path_buf());
            resolved == root || resolved.starts_with(root)
        })
}

/// Preview every local-memory row associated with `root`.
///
/// Linked bookmarks and screenshots are included even when their own `cwd` is
/// empty. Only transcript and screenshot paths inside MemoryWhale's data
/// directory are returned as managed files, so a tampered database cannot turn
/// repository cleanup into arbitrary file deletion.
pub fn plan_repository_deletion(
    conn: &Connection,
    root: &Path,
) -> Result<RepositoryDeletionPlan, String> {
    let mut session_ids = HashSet::new();
    let mut command_run_ids = HashSet::new();
    let mut bookmark_ids = HashSet::new();
    let mut screenshot_ids = HashSet::new();
    let mut managed_files = Vec::new();
    let data_dir = crate::data_dir()?;

    let mut session_stmt = conn
        .prepare("SELECT id, cwd, transcript_path FROM sessions")
        .map_err(|e| format!("failed to inspect sessions: {e}"))?;
    let sessions = session_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })
        .map_err(|e| format!("failed to inspect sessions: {e}"))?;
    for row in sessions {
        let (id, cwd, transcript_path) = row.map_err(|e| format!("failed to read session: {e}"))?;
        if cwd_is_within(cwd.as_deref(), root) {
            session_ids.insert(id);
            if let Some(path) = transcript_path
                .filter(|path| !path.is_empty())
                .map(PathBuf::from)
                .filter(|path| path.starts_with(&data_dir))
            {
                managed_files.push(path);
            }
        }
    }

    let mut command_stmt = conn
        .prepare("SELECT id, cwd FROM command_runs")
        .map_err(|e| format!("failed to inspect command runs: {e}"))?;
    let commands = command_stmt
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Option<String>>(1)?))
        })
        .map_err(|e| format!("failed to inspect command runs: {e}"))?;
    for row in commands {
        let (id, cwd) = row.map_err(|e| format!("failed to read command run: {e}"))?;
        if cwd_is_within(cwd.as_deref(), root) {
            command_run_ids.insert(id);
        }
    }

    let mut bookmark_stmt = conn
        .prepare("SELECT id, cwd, command_run_id, session_id FROM bookmarks")
        .map_err(|e| format!("failed to inspect bookmarks: {e}"))?;
    let bookmarks = bookmark_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, Option<i64>>(3)?,
            ))
        })
        .map_err(|e| format!("failed to inspect bookmarks: {e}"))?;
    for row in bookmarks {
        let (id, cwd, command_run_id, session_id) =
            row.map_err(|e| format!("failed to read bookmark: {e}"))?;
        if cwd_is_within(cwd.as_deref(), root)
            || command_run_id.is_some_and(|id| command_run_ids.contains(&id))
            || session_id.is_some_and(|id| session_ids.contains(&id))
        {
            bookmark_ids.insert(id);
        }
    }

    let mut screenshot_stmt = conn
        .prepare("SELECT id, cwd, command_run_id, file_path FROM screenshots")
        .map_err(|e| format!("failed to inspect screenshots: {e}"))?;
    let screenshots = screenshot_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, String>(3)?,
            ))
        })
        .map_err(|e| format!("failed to inspect screenshots: {e}"))?;
    for row in screenshots {
        let (id, cwd, command_run_id, file_path) =
            row.map_err(|e| format!("failed to read screenshot: {e}"))?;
        if cwd_is_within(cwd.as_deref(), root)
            || command_run_id.is_some_and(|id| command_run_ids.contains(&id))
        {
            screenshot_ids.insert(id);
            let path = PathBuf::from(file_path);
            if path.starts_with(&data_dir) {
                managed_files.push(path);
            }
        }
    }

    let mut sync_ids = HashSet::new();
    let mut sync_stmt = conn
        .prepare("SELECT mw_id FROM mempalace_sync")
        .map_err(|e| format!("failed to inspect sync mappings: {e}"))?;
    let mappings = sync_stmt
        .query_map([], |row| row.get::<_, i64>(0))
        .map_err(|e| format!("failed to inspect sync mappings: {e}"))?;
    for mapping in mappings {
        let mw_id = mapping.map_err(|e| format!("failed to read sync mapping: {e}"))?;
        let (source, id) = memorywhale_core::sqlite::decode_id(mw_id);
        let selected = match source {
            memorywhale_core::sqlite::Source::Command => command_run_ids.contains(&id),
            memorywhale_core::sqlite::Source::Note => bookmark_ids.contains(&id),
            memorywhale_core::sqlite::Source::Session => session_ids.contains(&id),
            _ => false,
        };
        if selected {
            sync_ids.insert(mw_id);
        }
    }

    managed_files.sort();
    managed_files.dedup();
    Ok(RepositoryDeletionPlan {
        root: root.to_path_buf(),
        sessions: session_ids.len(),
        command_runs: command_run_ids.len(),
        bookmarks: bookmark_ids.len(),
        screenshots: screenshot_ids.len(),
        managed_files,
        session_ids,
        command_run_ids,
        bookmark_ids,
        screenshot_ids,
        sync_ids,
    })
}

/// Delete a previously reviewed repository plan in one database transaction.
pub fn execute_repository_deletion(
    conn: &mut Connection,
    plan: &RepositoryDeletionPlan,
) -> Result<(), String> {
    let transaction = conn
        .transaction()
        .map_err(|e| format!("failed to start repository deletion: {e}"))?;
    for id in &plan.sync_ids {
        transaction
            .execute("DELETE FROM mempalace_sync WHERE mw_id = ?1", params![id])
            .map_err(|e| format!("failed to delete sync mapping: {e}"))?;
    }
    for id in &plan.screenshot_ids {
        transaction
            .execute("DELETE FROM screenshots WHERE id = ?1", params![id])
            .map_err(|e| format!("failed to delete screenshot: {e}"))?;
    }
    for id in &plan.bookmark_ids {
        transaction
            .execute("DELETE FROM bookmarks WHERE id = ?1", params![id])
            .map_err(|e| format!("failed to delete bookmark: {e}"))?;
    }
    for id in &plan.command_run_ids {
        transaction
            .execute("DELETE FROM command_runs WHERE id = ?1", params![id])
            .map_err(|e| format!("failed to delete command run: {e}"))?;
    }
    for id in &plan.session_ids {
        transaction
            .execute("DELETE FROM sessions WHERE id = ?1", params![id])
            .map_err(|e| format!("failed to delete session: {e}"))?;
    }
    transaction
        .commit()
        .map_err(|e| format!("failed to commit repository deletion: {e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_database_has_the_canonical_schema_and_connection_policy() {
        let conn = Connection::open_in_memory().unwrap();
        initialize(&conn).unwrap();

        for table in [
            "sessions",
            "command_runs",
            "command_arguments",
            "bookmarks",
            "screenshots",
            "mempalace_sync",
        ] {
            let exists: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
                    [table],
                    |r| r.get(0),
                )
                .unwrap();
            assert_eq!(exists, 1, "missing canonical table {table}");
        }
        assert_eq!(
            conn.query_row("PRAGMA foreign_keys", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            1
        );
        assert_eq!(
            conn.query_row("PRAGMA busy_timeout", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            3000
        );
        assert_eq!(
            conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            crate::LATEST_SCHEMA_VERSION
        );
    }

    #[test]
    fn legacy_partial_database_converges_without_losing_rows() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE command_runs (
                id INTEGER PRIMARY KEY, command TEXT NOT NULL, argv_json TEXT NOT NULL,
                cwd TEXT, exit_code INTEGER, stdout TEXT NOT NULL DEFAULT '',
                stderr TEXT NOT NULL DEFAULT '', notes TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL
             );
             INSERT INTO command_runs (command, argv_json, created_at)
             VALUES ('cargo', '[\"cargo\",\"test\"]', '2026-07-01T00:00:00Z');",
        )
        .unwrap();

        initialize(&conn).unwrap();
        let command: String = conn
            .query_row("SELECT command FROM command_runs WHERE id = 1", [], |r| {
                r.get(0)
            })
            .unwrap();
        assert_eq!(command, "cargo");
        let capture_kind: String = conn
            .query_row(
                "SELECT capture_kind FROM command_runs WHERE id = 1",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(capture_kind, "full");
    }

    #[test]
    fn v0_6_2_database_upgrades_without_data_loss() {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "
            CREATE TABLE sessions (
                id INTEGER PRIMARY KEY,
                shell TEXT,
                cwd TEXT,
                transcript_path TEXT NOT NULL DEFAULT '',
                transcript TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                started_at TEXT NOT NULL DEFAULT '',
                ended_at TEXT NOT NULL DEFAULT '',
                byte_count INTEGER NOT NULL DEFAULT 0,
                status TEXT NOT NULL DEFAULT 'finished',
                project TEXT,
                machine TEXT
            );
            CREATE TABLE command_runs (
                id INTEGER PRIMARY KEY,
                command TEXT NOT NULL,
                argv_json TEXT NOT NULL,
                cwd TEXT,
                exit_code INTEGER,
                stdout TEXT NOT NULL DEFAULT '',
                stderr TEXT NOT NULL DEFAULT '',
                notes TEXT NOT NULL DEFAULT '',
                created_at TEXT NOT NULL,
                capture_kind TEXT NOT NULL DEFAULT 'full',
                error_fingerprint TEXT
            );
            CREATE TABLE command_arguments (
                id INTEGER PRIMARY KEY,
                command_run_id INTEGER NOT NULL,
                position INTEGER NOT NULL,
                value TEXT NOT NULL,
                FOREIGN KEY(command_run_id) REFERENCES command_runs(id) ON DELETE CASCADE
            );
            CREATE TABLE bookmarks (
                id INTEGER PRIMARY KEY,
                label TEXT NOT NULL,
                cwd TEXT,
                created_at TEXT NOT NULL,
                command_run_id INTEGER,
                session_id INTEGER,
                author_kind TEXT NOT NULL DEFAULT 'human',
                author_name TEXT,
                source_session_id INTEGER,
                approved INTEGER NOT NULL DEFAULT 1
            );
            CREATE TABLE screenshots (
                id INTEGER PRIMARY KEY,
                command_run_id INTEGER,
                file_path TEXT NOT NULL,
                cwd TEXT,
                notes TEXT NOT NULL DEFAULT '',
                captured_at TEXT NOT NULL,
                FOREIGN KEY(command_run_id) REFERENCES command_runs(id) ON DELETE SET NULL
            );
            CREATE TABLE mempalace_sync (
                mw_id INTEGER PRIMARY KEY,
                wing TEXT NOT NULL,
                drawer_id TEXT NOT NULL,
                content_hash TEXT NOT NULL,
                synced_at TEXT NOT NULL
            );

            INSERT INTO sessions
                (id, shell, cwd, transcript_path, transcript, notes, started_at,
                 ended_at, byte_count, status, project, machine)
            VALUES
                (11, 'zsh', '/work/project', '/tmp/session.log',
                 'cargo test failed, then passed', 'project:upgrade-test',
                 '2026-07-20T10:00:00Z', '2026-07-20T10:05:00Z', 30,
                 'finished', 'upgrade-test', 'laptop');
            INSERT INTO command_runs
                (id, command, argv_json, cwd, exit_code, stdout, stderr, notes,
                 created_at, capture_kind, error_fingerprint)
            VALUES
                (12, 'cargo', '[\"cargo\",\"test\"]', '/work/project', 1, '',
                 'linker failed', 'before upgrade',
                 '2026-07-20T10:01:00Z', 'full', 'linker-fingerprint');
            INSERT INTO command_arguments
                (id, command_run_id, position, value)
            VALUES (13, 12, 0, 'test');
            INSERT INTO bookmarks
                (id, label, cwd, created_at, command_run_id, session_id,
                 author_kind, author_name, source_session_id, approved)
            VALUES
                (14, 'install the linker before building', '/work/project',
                 '2026-07-20T10:02:00Z', 12, 11, 'human', NULL, NULL, 1);
            INSERT INTO screenshots
                (id, command_run_id, file_path, cwd, notes, captured_at)
            VALUES
                (15, 12, '/tmp/failure.png', '/work/project', 'failure state',
                 '2026-07-20T10:03:00Z');
            INSERT INTO mempalace_sync
                (mw_id, wing, drawer_id, content_hash, synced_at)
            VALUES
                (16, 'terminal', 'drawer-16', 'abc123',
                 '2026-07-20T10:04:00Z');
            PRAGMA user_version = 5;
            ",
        )
        .unwrap();

        initialize(&conn).unwrap();

        assert_eq!(
            conn.query_row("PRAGMA user_version", [], |row| row.get::<_, i64>(0))
                .unwrap(),
            crate::LATEST_SCHEMA_VERSION
        );
        let lifecycle: (String, Option<i64>) = conn
            .query_row(
                "SELECT status, superseded_by_id FROM bookmarks WHERE id = 14",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(lifecycle, ("active".to_string(), None));

        for (table, expected) in [
            ("sessions", 1_i64),
            ("command_runs", 1),
            ("command_arguments", 1),
            ("bookmarks", 1),
            ("screenshots", 1),
            ("mempalace_sync", 1),
        ] {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(count, expected, "{table} rows changed during upgrade");
        }

        let memories = memorywhale_core::sqlite::load_memories(&conn);
        assert!(
            memories
                .iter()
                .any(|memory| memory.text.contains("install the linker")),
            "the released bookmark remains retrievable after migration"
        );
        assert!(
            memories
                .iter()
                .any(|memory| memory.text.contains("linker failed")),
            "the released command failure remains retrievable after migration"
        );

        initialize(&conn).unwrap();
        let bookmark_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM bookmarks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(bookmark_count, 1, "reopening the upgraded DB is idempotent");
    }

    #[test]
    fn repository_deletion_is_scoped_and_removes_linked_rows() {
        let mut conn = Connection::open_in_memory().unwrap();
        initialize(&conn).unwrap();
        let managed_transcript = crate::data_dir().unwrap().join("sessions/repo.log");
        conn.execute_batch(&format!(
            "
            INSERT INTO sessions
                (id, cwd, transcript_path, transcript, started_at, ended_at)
            VALUES
                (1, '/work/repo', '{}', 'secret session', '2026-07-20T10:00:00Z',
                 '2026-07-20T10:01:00Z'),
                (2, '/work/repo-other', '/tmp/keep.log', 'keep session',
                 '2026-07-20T10:00:00Z', '2026-07-20T10:01:00Z');
            INSERT INTO command_runs
                (id, command, argv_json, cwd, created_at)
            VALUES
                (3, 'cargo', '[\"cargo\"]', '/work/repo/crate',
                 '2026-07-20T10:00:00Z'),
                (4, 'git', '[\"git\"]', '/work/repo-other',
                 '2026-07-20T10:00:00Z');
            INSERT INTO command_arguments
                (command_run_id, position, value)
            VALUES (3, 0, 'test'), (4, 0, 'status');
            INSERT INTO bookmarks
                (id, label, cwd, created_at, command_run_id, session_id)
            VALUES
                (5, 'linked secret', NULL, '2026-07-20T10:00:00Z', 3, 1),
                (6, 'keep note', '/work/repo-other', '2026-07-20T10:00:00Z', 4, 2);
            INSERT INTO screenshots
                (id, command_run_id, file_path, cwd, captured_at)
            VALUES
                (7, 3, '/tmp/outside-data-dir.png', NULL, '2026-07-20T10:00:00Z'),
                (8, 4, '/tmp/keep.png', '/work/repo-other', '2026-07-20T10:00:00Z');
            INSERT INTO mempalace_sync
                (mw_id, wing, drawer_id, content_hash, synced_at)
            VALUES
                (1000000003, 'terminal', 'run-3', 'a', '2026-07-20T10:00:00Z'),
                (3000000005, 'terminal', 'note-5', 'b', '2026-07-20T10:00:00Z'),
                (4000000001, 'terminal', 'session-1', 'c', '2026-07-20T10:00:00Z'),
                (1000000004, 'terminal', 'run-4', 'd', '2026-07-20T10:00:00Z');
            ",
            managed_transcript.display()
        ))
        .unwrap();

        let plan = plan_repository_deletion(&conn, Path::new("/work/repo")).unwrap();
        assert_eq!(plan.sessions, 1);
        assert_eq!(plan.command_runs, 1);
        assert_eq!(plan.bookmarks, 1);
        assert_eq!(plan.screenshots, 1);
        assert_eq!(plan.total_rows(), 4);
        assert_eq!(plan.managed_files, vec![managed_transcript]);
        execute_repository_deletion(&mut conn, &plan).unwrap();

        for table in [
            "sessions",
            "command_runs",
            "command_arguments",
            "bookmarks",
            "screenshots",
            "mempalace_sync",
        ] {
            let count: i64 = conn
                .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |row| {
                    row.get(0)
                })
                .unwrap();
            assert_eq!(count, 1, "unrelated {table} row should remain");
        }
        let cwd: String = conn
            .query_row("SELECT cwd FROM sessions", [], |row| row.get(0))
            .unwrap();
        assert_eq!(cwd, "/work/repo-other", "prefix collisions must not match");
    }
}
