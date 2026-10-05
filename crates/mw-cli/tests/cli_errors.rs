//! Friendly errors for missing, malformed, or unknown `mw` arguments: each
//! exits non-zero with a stable message and records nothing.

use std::path::Path;
use std::process::Command;

fn mw_fails(args: &[&str], fragment: &str) {
    let data_dir = std::env::temp_dir().join(format!(
        "mw-cli-errors-{}-{}",
        args.join("-").replace(['/', ' '], "_"),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&data_dir);

    let output = Command::new(env!("CARGO_BIN_EXE_mw"))
        .args(args)
        .env("MEMORYWHALE_DATA_DIR", &data_dir)
        .output()
        .expect("run mw");

    assert!(
        !output.status.success(),
        "mw {args:?} succeeded: {output:?}"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains(fragment),
        "mw {args:?} stderr lacks {fragment:?}: {stderr}"
    );
    assert_eq!(rows(&data_dir), 0, "mw {args:?} recorded something");
    let _ = std::fs::remove_dir_all(&data_dir);
}

/// Commands and sessions recorded under `data_dir`; 0 when no database exists.
fn rows(data_dir: &Path) -> i64 {
    let db = data_dir.join("memorywhale.sqlite3");
    if !db.exists() {
        return 0;
    }
    let conn = rusqlite::Connection::open(db).unwrap();
    conn.query_row(
        "SELECT (SELECT COUNT(*) FROM command_runs) + (SELECT COUNT(*) FROM sessions)",
        [],
        |r| r.get(0),
    )
    .unwrap()
}

#[test]
fn unknown_option_points_to_help() {
    mw_fails(
        &["--unknown"],
        "unknown option \"--unknown\"; run mw --help",
    );
}

#[test]
fn show_needs_a_numeric_id() {
    mw_fails(&["show"], "usage: mw show <id>");
    mw_fails(&["show", "abc"], "invalid session id \"abc\"");
}

#[test]
fn memory_needs_a_lifecycle_action() {
    mw_fails(&["memory"], "usage: mw memory stale <id>");
}

#[test]
fn ask_chat_needs_a_target() {
    mw_fails(&["ask", "--chat"], "--chat needs a value");
}

#[test]
fn import_needs_a_bundle_path() {
    mw_fails(
        &["import"],
        "usage: mw import <bundle-dir|memorywhale.sqlite3>",
    );
}
