//! Recipe lifecycle: atomic saves, ordered provenance, and source-run deletion.

use rusqlite::Connection;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn data_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mw-recipes-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn mw(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_mw"))
        .env("MEMORYWHALE_DATA_DIR", dir)
        .args(args)
        .output()
        .unwrap()
}

fn text(out: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    )
}

fn remember(dir: &Path, cmd: &[&str]) -> i64 {
    let mut args = vec!["--cwd", dir.to_str().unwrap(), "--exit-code", "0", "--"];
    args.extend_from_slice(cmd);
    let out = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .env("MEMORYWHALE_DATA_DIR", dir)
        .args(&args)
        .output()
        .unwrap();
    assert!(out.status.success(), "mw-remember failed: {out:?}");
    db(dir)
        .query_row("SELECT MAX(id) FROM command_runs", [], |r| r.get(0))
        .unwrap()
}

fn db(dir: &Path) -> Connection {
    Connection::open(dir.join("memorywhale.sqlite3")).unwrap()
}

fn count(dir: &Path, table: &str) -> i64 {
    db(dir)
        .query_row(&format!("SELECT COUNT(*) FROM {table}"), [], |r| r.get(0))
        .unwrap()
}

fn save(dir: &Path, runs: &str) -> Output {
    mw(
        dir,
        &[
            "recipe",
            "save",
            "--run",
            runs,
            "--description",
            "build",
            "--criteria",
            "ok",
        ],
    )
}

#[test]
fn failed_saves_leave_no_partial_recipe() {
    let dir = data_dir("atomic");
    let a = remember(&dir, &["cargo", "build"]);
    let other = remember(&dir, &["cargo", "test"]);

    let missing = save(&dir, &format!("{a},999999"));
    assert!(!missing.status.success(), "{}", text(&missing));
    let mixed = save(&dir, &format!("{a},{other}"));
    assert!(!mixed.status.success(), "{}", text(&mixed));
    assert!(text(&mixed).contains("incompatible"), "{}", text(&mixed));
    assert_eq!(count(&dir, "command_recipes"), 0);
    assert_eq!(count(&dir, "command_recipe_sources"), 0);

    let dup = save(&dir, &format!("{a},{a}"));
    assert!(dup.status.success(), "{}", text(&dup));
    assert_eq!(count(&dir, "command_recipe_sources"), 1);
}

#[test]
fn sources_keep_selection_order_and_survive_run_deletion() {
    let dir = data_dir("order");
    let first = remember(&dir, &["make", "all"]);
    let second = remember(&dir, &["make", "all"]);
    let out = save(&dir, &format!("{second},{first}"));
    assert!(out.status.success(), "{}", text(&out));

    let show = text(&mw(&dir, &["recipe", "show", "1"]));
    assert!(
        show.contains(&format!("source runs: [{second}, {first}]")),
        "{show}"
    );

    let rm = mw(&dir, &["rm", "command", &second.to_string()]);
    assert!(rm.status.success(), "{}", text(&rm));
    let show = text(&mw(&dir, &["recipe", "show", "1"]));
    assert!(show.contains(&format!("source runs: [{first}]")), "{show}");
    assert!(show.contains("\"make\""), "{show}");
}

#[test]
fn prune_keeps_recipes_and_reports_actual_deletions() {
    let dir = data_dir("prune");
    let old = remember(&dir, &["npm", "ci"]);
    let out = save(&dir, &old.to_string());
    assert!(out.status.success(), "{}", text(&out));
    let transcript = dir.join("old-session.log");
    std::fs::write(&transcript, "evidence").unwrap();
    {
        let conn = db(&dir);
        conn.execute(
            "UPDATE command_runs SET created_at = '2000-01-01T00:00:00Z' WHERE id = ?1",
            [old],
        )
        .unwrap();
        conn.execute(
            "INSERT INTO sessions (transcript_path, started_at) VALUES (?1, '2000-01-01T00:00:00Z')",
            [transcript.to_str().unwrap()],
        )
        .unwrap();
    }

    let dry = mw(&dir, &["prune", "--older-than", "30d", "--dry-run"]);
    assert!(
        text(&dry).contains("1 session(s) and 1 command run(s)"),
        "{}",
        text(&dry)
    );
    assert!(transcript.exists());

    let pruned = mw(&dir, &["prune", "--older-than", "30d"]);
    assert!(pruned.status.success(), "{}", text(&pruned));
    assert!(
        text(&pruned).contains("1 session(s) and 1 command run(s)"),
        "{}",
        text(&pruned)
    );
    assert_eq!(count(&dir, "command_runs"), 0);
    assert_eq!(count(&dir, "command_recipes"), 1);
    assert_eq!(count(&dir, "command_recipe_sources"), 0);
    assert!(!transcript.exists());
}
