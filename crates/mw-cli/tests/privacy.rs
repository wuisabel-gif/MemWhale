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
fn repository_cleanup_previews_then_deletes_only_the_selected_tree() {
    let dir = std::env::temp_dir().join(format!("mw-forget-repo-{}", std::process::id()));
    let repo = dir.join("repo");
    let other = dir.join("repo-other");
    let data = dir.join("data");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&repo).unwrap();
    std::fs::create_dir_all(&other).unwrap();
    std::fs::create_dir_all(&data).unwrap();

    for (cwd, command) in [(&repo, "cargo"), (&other, "git")] {
        let out = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
            .env("MEMORYWHALE_DATA_DIR", &data)
            .args([
                "--cwd",
                cwd.to_str().unwrap(),
                "--exit-code",
                "0",
                "--",
                command,
            ])
            .output()
            .unwrap();
        assert!(out.status.success(), "mw-remember failed: {out:?}");
    }

    let preview = Command::new(env!("CARGO_BIN_EXE_mw"))
        .env("MEMORYWHALE_DATA_DIR", &data)
        .args(["forget-repo", repo.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(preview.status.success(), "preview failed: {preview:?}");
    assert!(
        String::from_utf8_lossy(&preview.stdout).contains("dry run only"),
        "unexpected preview output: stdout={} stderr={}",
        String::from_utf8_lossy(&preview.stdout),
        String::from_utf8_lossy(&preview.stderr)
    );
    let conn = Connection::open(data.join("memorywhale.sqlite3")).unwrap();
    let before: i64 = conn
        .query_row("SELECT COUNT(*) FROM command_runs", [], |row| row.get(0))
        .unwrap();
    assert_eq!(before, 2, "preview must not delete data");
    drop(conn);

    let deletion = Command::new(env!("CARGO_BIN_EXE_mw"))
        .env("MEMORYWHALE_DATA_DIR", &data)
        .args(["forget-repo", repo.to_str().unwrap(), "--yes"])
        .output()
        .unwrap();
    assert!(deletion.status.success(), "deletion failed: {deletion:?}");

    let conn = Connection::open(data.join("memorywhale.sqlite3")).unwrap();
    let remaining: Vec<String> = conn
        .prepare("SELECT cwd FROM command_runs")
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(remaining, vec![other.to_string_lossy().into_owned()]);

    let _ = std::fs::remove_dir_all(&dir);
}
