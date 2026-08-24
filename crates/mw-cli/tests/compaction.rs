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
