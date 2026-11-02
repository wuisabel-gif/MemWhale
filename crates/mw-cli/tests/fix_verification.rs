//! Synthetic-only outcome/provenance regressions. No user store is opened.
use memorywhale_cli::{fix_candidates, program_stem, storage, Failure};
use rusqlite::{params, Connection};

#[test]
fn latest_rerun_overrides_old_success_and_keeps_recorded_provenance() {
    let conn = Connection::open_in_memory().unwrap();
    storage::initialize(&conn).unwrap();
    let run = |argv: &str, exit: Option<i64>, cwd: &str| {
        conn.execute("INSERT INTO command_runs (command, argv_json, cwd, exit_code, created_at, repository_id) VALUES ('synthetic', ?1, ?2, ?3, '2026-11-01T00:00:00Z', 'synthetic-repo')", params![argv, cwd, exit]).unwrap();
        conn.last_insert_rowid()
    };
    let failed = run(r#"["cargo","test"]"#, Some(1), "/synthetic");
    run(r#"["cargo","clean"]"#, Some(0), "/synthetic");
    run(r#"["cargo","test"]"#, Some(0), "/synthetic");
    let failures = vec![Failure {
        id: failed,
        stem: program_stem(r#"["cargo","test"]"#),
        cwd: Some("/synthetic".into()),
    }];
    let candidates = fix_candidates(&conn, &failures, i64::MAX).unwrap();
    let v = candidates[0].verification.as_ref().unwrap();
    assert_eq!(v.exit_code, Some(0));
    assert!(v.summary().contains("synthetic-repo"));
    assert!(v.summary().contains("commit unknown"));
    assert!(v.summary().contains("2026-11-01"));
    let regression = run(r#"["cargo","test"]"#, Some(2), "/synthetic");
    run(r#"["cargo","test"]"#, Some(0), "/unrelated");
    let candidates = fix_candidates(&conn, &failures, i64::MAX).unwrap();
    let v = candidates[0].verification.as_ref().unwrap();
    assert_eq!(v.run_id, regression);
    assert_eq!(v.exit_code, Some(2));
    assert!(v.summary().contains("contradicts"));
    run(r#"["cargo","test"]"#, None, "/synthetic");
    let candidates = fix_candidates(&conn, &failures, i64::MAX).unwrap();
    assert_eq!(candidates[0].verification.as_ref().unwrap().exit_code, None);
}

/// One synthetic episode: the error, a fix, the failing program passing.
fn episode(conn: &Connection) {
    let run = |argv: &str, exit: i64, stderr: &str| {
        conn.execute("INSERT INTO command_runs (command, argv_json, cwd, exit_code, stderr, created_at) VALUES ('synthetic', ?1, '/synthetic', ?2, ?3, '2026-11-01T00:00:00Z')", params![argv, exit, stderr]).unwrap();
    };
    run(r#"["cargo","build"]"#, 101, ERR);
    run(r#"["xcode-select","--install"]"#, 0, "");
    run(r#"["cargo","build"]"#, 0, "");
}

const ERR: &str = "error: linker `cc` not found";

#[test]
fn the_error_coming_back_does_not_contradict_its_fix() {
    let conn = Connection::open_in_memory().unwrap();
    storage::initialize(&conn).unwrap();
    episode(&conn);
    conn.execute("INSERT INTO command_runs (command, argv_json, cwd, exit_code, stderr, created_at) VALUES ('cargo', '[\"cargo\",\"build\"]', '/synthetic', 101, ?1, '2026-11-02T00:00:00Z')", [ERR]).unwrap();
    let hint = memorywhale_cli::error_hint(&conn, ERR).unwrap().unwrap();
    assert!(
        hint.ends_with("possible fix: xcode-select --install"),
        "{hint}"
    );
}

#[test]
fn a_failure_after_the_fix_and_before_the_error_returns_drops_the_marker() {
    let conn = Connection::open_in_memory().unwrap();
    storage::initialize(&conn).unwrap();
    episode(&conn);
    for (exit, stderr) in [(2, "error: something else broke"), (101, ERR)] {
        conn.execute("INSERT INTO command_runs (command, argv_json, cwd, exit_code, stderr, created_at) VALUES ('cargo', '[\"cargo\",\"build\"]', '/synthetic', ?1, ?2, '2026-11-02T00:00:00Z')", params![exit, stderr]).unwrap();
    }
    let hint = memorywhale_cli::error_hint(&conn, ERR).unwrap().unwrap();
    assert!(hint.contains("later rerun failed, exit 2"), "{hint}");
    assert!(
        hint.ends_with("historical candidate: xcode-select --install"),
        "{hint}"
    );
    assert!(!hint.contains("fix: xcode-select"), "{hint}");
}
