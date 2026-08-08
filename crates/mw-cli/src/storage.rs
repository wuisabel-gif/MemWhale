//! Canonical SQLite connection and schema ownership for every CLI surface.

use rusqlite::Connection;
use std::path::Path;

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
    .map_err(|e| format!("failed to initialize database indexes: {e}"))?;

    // Sweep TTL-expired notes on every open so retrieval stops surfacing them
    // (their rows are preserved). Best-effort — never fails an open.
    crate::expire_due_notes(conn, &chrono::Utc::now().to_rfc3339());
    Ok(())
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
}
