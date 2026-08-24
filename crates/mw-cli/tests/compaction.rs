use rusqlite::Connection;
use std::process::Command;

fn run_mw(data_dir: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mw"))
        .args(args)
        .env("MEMORYWHALE_DATA_DIR", data_dir)
        .output()
        .unwrap()
}

#[test]
fn command_compaction_converges_on_second_apply() {
    let data_dir = std::env::temp_dir().join(format!("mw-compaction-run-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data_dir);
    std::fs::create_dir_all(&data_dir).unwrap();
    let large_output = "x".repeat(4096);
    let remembered = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .args([
            "--stdout",
            &large_output,
            "--exit-code",
            "0",
            "--",
            "successful-build",
        ])
        .env("MEMORYWHALE_DATA_DIR", &data_dir)
        .output()
        .unwrap();
    assert!(
        remembered.status.success(),
        "capture failed: {remembered:?}"
    );

    let first = run_mw(
        &data_dir,
        &["memory", "compact", "--apply", "--max-output-bytes", "256"],
    );
    assert!(first.status.success(), "first compaction failed: {first:?}");
    let first_text = String::from_utf8_lossy(&first.stdout);
    assert!(
        first_text.contains("1 command run(s)"),
        "nothing compacted: {first_text}"
    );

    let second = run_mw(
        &data_dir,
        &["memory", "compact", "--max-output-bytes", "256"],
    );
    assert!(second.status.success(), "second plan failed: {second:?}");
    let second_text = String::from_utf8_lossy(&second.stdout);
    assert!(
        second_text.contains("0 session(s), 0 command run(s)"),
        "compaction is not convergent: {second_text}"
    );

    let conn = Connection::open(data_dir.join("memorywhale.sqlite3")).unwrap();
    let (stdout, stderr): (String, String) = conn
        .query_row("SELECT stdout, stderr FROM command_runs", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .unwrap();
    assert!(stdout.contains("[COMPACTED:"));
    assert!(stdout.len() + stderr.len() <= 256);
    let _ = std::fs::remove_dir_all(data_dir);
}

#[test]
fn session_compaction_converges_and_preserves_raw_byte_count() {
    let data_dir =
        std::env::temp_dir().join(format!("mw-compaction-session-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data_dir);
    std::fs::create_dir_all(&data_dir).unwrap();
    let raw_path = data_dir.join("session.log");
    let raw = "transcript\n".repeat(30_000);
    std::fs::write(&raw_path, &raw).unwrap();

    let remembered = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .args(["--", "seed-session"])
        .env("MEMORYWHALE_DATA_DIR", &data_dir)
        .output()
        .unwrap();
    assert!(
        remembered.status.success(),
        "database setup failed: {remembered:?}"
    );
    let conn = Connection::open(data_dir.join("memorywhale.sqlite3")).unwrap();
    conn.execute(
        "INSERT INTO sessions
            (shell, cwd, transcript_path, transcript, notes, started_at, ended_at, byte_count, status)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        rusqlite::params![
            "/bin/sh",
            data_dir.to_str().unwrap(),
            raw_path.to_str().unwrap(),
            raw,
            "",
            "2020-01-01T00:00:00Z",
            "2020-01-01T01:00:00Z",
            raw.len() as i64,
            "finished"
        ],
    )
    .unwrap();
    drop(conn);

    let first = run_mw(
        &data_dir,
        &[
            "memory",
            "compact",
            "--apply",
            "--min-session-bytes",
            "1",
            "--stale-days",
            "1",
            "--max-output-bytes",
            "256",
        ],
    );
    assert!(
        first.status.success(),
        "session compaction failed: {first:?}"
    );
    assert!(String::from_utf8_lossy(&first.stdout).contains("1 session(s)"));

    let second = run_mw(
        &data_dir,
        &[
            "memory",
            "compact",
            "--min-session-bytes",
            "1",
            "--stale-days",
            "1",
        ],
    );
    assert!(
        second.status.success(),
        "second session plan failed: {second:?}"
    );
    assert!(
        String::from_utf8_lossy(&second.stdout).contains("0 session(s), 0 command run(s)"),
        "session compaction is not convergent: {}",
        String::from_utf8_lossy(&second.stdout)
    );

    let conn = Connection::open(data_dir.join("memorywhale.sqlite3")).unwrap();
    let (transcript, byte_count): (String, i64) = conn
        .query_row("SELECT transcript, byte_count FROM sessions", [], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .unwrap();
    assert!(transcript.starts_with("[COMPACTED:"));
    assert_eq!(byte_count, raw.len() as i64);
    assert_eq!(std::fs::read_to_string(raw_path).unwrap(), raw);
    let _ = std::fs::remove_dir_all(data_dir);
}
