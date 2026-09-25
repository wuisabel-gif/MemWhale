//! End-to-end privacy check for the command-capture write path.
//!
//! `redact()` has unit tests, and `remember()` (the bookmarks path) is covered
//! in the lib test module. The gap this closes: a captured command's
//! stdout/stderr/notes/argv flow through the capture binaries into SQLite —
//! prove the scrub fires before a secret lands raw in the DB.

use rusqlite::Connection;
use std::process::Command;

// Hand-authored fake credentials — never real. One per shape the shared privacy
// module handles: an assignment, a GitHub token, and an AWS access-key id.
const SECRETS: [&str; 3] = [
    "hunter2secret",                  // password: <value>
    "ghp_0123456789abcdefghijABCDEF", // GitHub token
    "AKIAABCDEFGHIJKLMNOP",           // AWS access key id
];

#[test]
fn command_capture_stdout_stderr_is_redacted_in_db() {
    let dir = std::env::temp_dir().join(format!("mw-privacy-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let stdout = format!("logging in with password: {} then done", SECRETS[0]);
    let stderr = format!("token {} and key {} leaked", SECRETS[1], SECRETS[2]);

    let out = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .env("MEMORYWHALE_DATA_DIR", &dir)
        .args([
            "--cwd",
            dir.to_str().unwrap(),
            "--exit-code",
            "0",
            "--stdout",
            &stdout,
            "--stderr",
            &stderr,
            "--",
            "printenv",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "mw-remember failed: {out:?}");

    let conn = Connection::open(dir.join("memorywhale.sqlite3")).unwrap();
    let (db_stdout, db_stderr): (String, String) = conn
        .query_row(
            "SELECT stdout, stderr FROM command_runs ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();

    let stored = format!("{db_stdout}\n{db_stderr}");
    assert!(stored.contains("[REDACTED]"), "scrub never fired: {stored}");
    for secret in SECRETS {
        assert!(
            !stored.contains(secret),
            "raw secret {secret:?} landed in command_runs: {stored}"
        );
    }

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn command_capture_notes_and_arguments_are_redacted_in_db() {
    let dir = std::env::temp_dir().join(format!("mw-privacy-args-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let split_secret = "hunter2secret";
    let equals_secret = "equal-secret-99";
    let notes = format!("password: {split_secret}");
    let split_argument = "--token";
    let equals_argument = format!("--password={equals_secret}");

    let out = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .env("MEMORYWHALE_DATA_DIR", &dir)
        .args([
            "--notes",
            &notes,
            "--",
            "deploy",
            split_argument,
            split_secret,
            &equals_argument,
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "mw-remember failed: {out:?}");

    let conn = Connection::open(dir.join("memorywhale.sqlite3")).unwrap();
    let (argv_json, stored_notes): (String, String) = conn
        .query_row(
            "SELECT argv_json, notes FROM command_runs ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .unwrap();
    let stored_arguments: Vec<String> = conn
        .prepare("SELECT value FROM command_arguments ORDER BY position")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();

    for stored in [&argv_json, &stored_notes] {
        assert!(stored.contains("[REDACTED]"), "not redacted: {stored}");
    }
    assert!(stored_arguments.contains(&"[REDACTED]".to_string()));
    assert!(stored_arguments.contains(&"--password=[REDACTED]".to_string()));
    let stored = stored_arguments.join(" ");
    for secret in [split_secret, equals_secret] {
        assert!(
            !stored.contains(secret),
            "raw secret landed in DB: {stored}"
        );
    }
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn control_split_secrets_are_redacted_in_command_stdout_and_notes() {
    let dir = std::env::temp_dir().join(format!("mw-privacy-controls-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let secret = "abc123secret";

    let out = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .env("MEMORYWHALE_DATA_DIR", &dir)
        .args([
            "--cwd",
            dir.to_str().unwrap(),
            "--stdout",
            &format!("\x1b[32mok\x1b[0m pass\x1b[1mword={secret}"),
            "--notes",
            &format!("api\x1b]0;x\x07_key: {secret}"),
            "--",
            "deploy",
            &format!("tok\x1ben={secret}"),
            "--to\x1b[0mken",
            secret,
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "mw-remember failed: {out:?}");

    let conn = Connection::open(dir.join("memorywhale.sqlite3")).unwrap();
    let (command, argv_json, stdout, notes): (String, String, String, String) = conn
        .query_row(
            "SELECT command, argv_json, stdout, notes FROM command_runs ORDER BY id DESC LIMIT 1",
            [],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)),
        )
        .unwrap();
    let arguments: Vec<String> = conn
        .prepare("SELECT value FROM command_arguments ORDER BY position")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();

    for stored in [&argv_json, &stdout, &notes, &arguments.join(" ")] {
        assert!(stored.contains("[REDACTED]"), "not redacted: {stored:?}");
        assert!(!stored.contains(secret), "raw secret landed: {stored:?}");
    }
    assert!(
        !command.contains(secret),
        "raw secret in command: {command:?}"
    );
    // Legitimate color output outside the secret survives capture.
    assert!(stdout.starts_with("\x1b[32mok\x1b[0m"), "{stdout:?}");
    let _ = std::fs::remove_dir_all(&dir);
}
