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
    let candidates = fix_candidates(&conn, &failures).unwrap();
    let v = candidates[0].verification.as_ref().unwrap();
    assert_eq!(v.exit_code, Some(0));
    assert!(v.summary().contains("synthetic-repo"));
    assert!(v.summary().contains("commit unknown"));
    assert!(v.summary().contains("2026-11-01"));
    let regression = run(r#"["cargo","test"]"#, Some(2), "/synthetic");
    run(r#"["cargo","test"]"#, Some(0), "/unrelated");
    let candidates = fix_candidates(&conn, &failures).unwrap();
    let v = candidates[0].verification.as_ref().unwrap();
    assert_eq!(v.run_id, regression);
    assert_eq!(v.exit_code, Some(2));
    assert!(v.summary().contains("contradicts"));
    run(r#"["cargo","test"]"#, None, "/synthetic");
    let candidates = fix_candidates(&conn, &failures).unwrap();
    assert_eq!(candidates[0].verification.as_ref().unwrap().exit_code, None);
}
