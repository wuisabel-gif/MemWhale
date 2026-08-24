use rusqlite::Connection;
use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn imported_relative_transcript_path_cannot_authorize_compaction() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!("mw-compaction-{unique}"));
    let source_dir = root.join("source");
    let data_dir = root.join("data");
    let work_dir = root.join("work");
    fs::create_dir_all(&source_dir).unwrap();
    fs::create_dir_all(&work_dir).unwrap();

    let source_db = source_dir.join("memorywhale.sqlite3");
    let source = Connection::open(&source_db).unwrap();
    source
        .execute_batch(
            "CREATE TABLE sessions (
                id INTEGER PRIMARY KEY,
                transcript_path TEXT NOT NULL,
                transcript TEXT NOT NULL,
                started_at TEXT NOT NULL,
                ended_at TEXT NOT NULL,
                byte_count INTEGER NOT NULL,
                status TEXT NOT NULL);
             INSERT INTO sessions
                (transcript_path, transcript, started_at, ended_at, byte_count, status)
             VALUES
                ('backing.log', 'recoverable imported transcript',
                 '2020-01-01T00:00:00Z', '2020-01-01T01:00:00Z', 31, 'finished');",
        )
        .unwrap();
    drop(source);

    fs::write(work_dir.join("backing.log"), "unrelated local file").unwrap();

    let import = Command::new(env!("CARGO_BIN_EXE_mw"))
        .args(["import", source_db.to_str().unwrap()])
        .env("MEMORYWHALE_DATA_DIR", &data_dir)
        .current_dir(&work_dir)
        .output()
        .unwrap();
    assert!(import.status.success(), "mw import failed: {import:?}");

    let compact = Command::new(env!("CARGO_BIN_EXE_mw"))
        .args([
            "memory",
            "compact",
            "--apply",
            "--min-session-bytes",
            "0",
            "--stale-days",
            "1",
        ])
        .env("MEMORYWHALE_DATA_DIR", &data_dir)
        .current_dir(&work_dir)
        .output()
        .unwrap();
    assert!(compact.status.success(), "mw compact failed: {compact:?}");

    let destination = Connection::open(data_dir.join("memorywhale.sqlite3")).unwrap();
    let transcript: String = destination
        .query_row("SELECT transcript FROM sessions", [], |row| row.get(0))
        .unwrap();
    assert_eq!(transcript, "recoverable imported transcript");

    drop(destination);
    fs::remove_dir_all(root).unwrap();
}
