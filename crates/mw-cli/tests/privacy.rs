//! End-to-end privacy check for the command-capture write path.
//!
//! `redact()` has unit tests, and `remember()` (the bookmarks path) is covered
//! in the lib test module. The gap this closes: a captured command's
//! stdout/stderr flows through the `mw-remember` binary into the `command_runs`
//! table — prove the scrub actually fires on that path, so a secret printed by
//! a recorded command never lands raw in the DB. (`mw-run` shares the identical
//! `redact()`-before-INSERT into `command_runs`.)

use rusqlite::Connection;
use std::process::Command;

// Hand-authored fake credentials — never real. One per shape `secret_patterns`
// handles: an assignment, a GitHub token, and an AWS access-key id.
const SECRETS: [&str; 3] = [
    "hunter2secret",                 // password: <value>
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
