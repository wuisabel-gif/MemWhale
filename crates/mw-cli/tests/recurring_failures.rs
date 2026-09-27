//! A failure recorded twice through the real capture path (`mw-remember`, the
//! path every agent hook uses) must be recognized as recurring. Before this was
//! fixed, `mw-remember` never stored an error fingerprint, so
//! `mw context --last-error` reported every repeat as "first time".

use std::path::Path;
use std::process::{Command, Output};

fn run(bin: &str, dir: &Path, args: &[&str]) -> Output {
    let out = Command::new(bin)
        .env("MEMORYWHALE_DATA_DIR", dir)
        .args(args)
        .output()
        .expect("run binary");
    assert!(out.status.success(), "{bin} {args:?} failed: {out:?}");
    out
}

fn fail(dir: &Path) {
    run(
        env!("CARGO_BIN_EXE_mw-remember"),
        dir,
        &[
            "--cwd",
            "/work/api",
            "--exit-code",
            "101",
            "--stderr",
            "error: linker `cc` not found",
            "--",
            "cargo",
            "build",
        ],
    );
}

#[test]
fn repeated_failure_is_recognized_as_recurring() {
    let dir = std::env::temp_dir().join(format!("mw-recurring-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    fail(&dir);
    fail(&dir);

    let db = rusqlite::Connection::open(dir.join("memorywhale.sqlite3")).unwrap();
    let fingerprints: Vec<Option<String>> = db
        .prepare("SELECT error_fingerprint FROM command_runs ORDER BY id")
        .unwrap()
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap();
    assert_eq!(fingerprints.len(), 2);
    assert!(
        fingerprints[0].is_some(),
        "capture must fingerprint failures"
    );
    assert_eq!(
        fingerprints[0], fingerprints[1],
        "same error must share a fingerprint"
    );

    let out = run(env!("CARGO_BIN_EXE_mw"), &dir, &["context", "--last-error"]);
    let text = String::from_utf8_lossy(&out.stdout);
    assert!(
        !text.contains("First time hitting this"),
        "a repeated failure was reported as new:\n{text}"
    );
    assert!(
        text.contains("2 times"),
        "expected the recurrence count:\n{text}"
    );
}

#[test]
fn successful_runs_are_not_fingerprinted() {
    let dir = std::env::temp_dir().join(format!("mw-recurring-ok-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    run(
        env!("CARGO_BIN_EXE_mw-remember"),
        &dir,
        &[
            "--exit-code",
            "0",
            "--stderr",
            "warning: unused",
            "--",
            "cargo",
            "build",
        ],
    );
    let db = rusqlite::Connection::open(dir.join("memorywhale.sqlite3")).unwrap();
    let fp: Option<String> = db
        .query_row("SELECT error_fingerprint FROM command_runs", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert!(fp.is_none(), "successful runs have no error fingerprint");
}
