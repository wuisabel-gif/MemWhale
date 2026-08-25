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

        let memories = memorywhale_core::sqlite::load_memories(&conn).unwrap();
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
