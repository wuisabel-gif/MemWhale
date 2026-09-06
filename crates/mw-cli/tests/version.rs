use std::path::Path;
use std::process::Command;

fn sandbox_data_dir(kind: &str, name: &str) -> std::path::PathBuf {
    let data_dir =
        std::env::temp_dir().join(format!("mw-{kind}-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&data_dir);
    data_dir
}

fn run_mw(flag: &str, data_dir: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_mw"))
        .arg(flag)
        .env("MEMORYWHALE_DATA_DIR", data_dir)
        .output()
        .unwrap_or_else(|err| panic!("run mw {flag}: {err}"))
}

/// Help/version must not open the store. Prefer "dir never created"; if a
/// SQLite file somehow appears, every user table must still be empty.
fn assert_no_database_rows(data_dir: &Path, flag: &str) {
    let db_path = data_dir.join("memorywhale.sqlite3");
    if !db_path.exists() {
        assert!(
            !data_dir.exists(),
            "{flag} created MEMORYWHALE_DATA_DIR without a database: {}",
            data_dir.display()
        );
        return;
    }

    let conn = rusqlite::Connection::open(&db_path)
        .unwrap_or_else(|err| panic!("open db after {flag}: {err}"));
    let mut stmt = conn
        .prepare(
            "SELECT name FROM sqlite_master \
             WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        )
        .expect("list user tables");
    let tables: Vec<String> = stmt
        .query_map([], |row| row.get(0))
        .expect("query tables")
        .collect::<Result<_, _>>()
        .expect("read table names");

    for table in tables {
        let count: i64 = conn
            .query_row(&format!("SELECT COUNT(*) FROM \"{table}\""), [], |row| {
                row.get(0)
            })
            .unwrap_or_else(|err| panic!("count {table} after {flag}: {err}"));
        assert_eq!(
            count, 0,
            "{flag} wrote {count} row(s) to {table} under {}",
            data_dir.display()
        );
    }
}

fn assert_version_flag(flag: &str, sandbox_name: &str) {
    let data_dir = sandbox_data_dir("version", sandbox_name);
    let output = run_mw(flag, &data_dir);

    assert!(output.status.success(), "{flag} failed: {output:?}");
    let stdout = std::str::from_utf8(&output.stdout).expect("version output is UTF-8");
    assert_eq!(
        stdout.trim_end(),
        format!("mw {}", env!("CARGO_PKG_VERSION"))
    );
    assert_eq!(
        stdout.lines().count(),
        1,
        "unexpected version output: {stdout:?}"
    );
    assert!(output.stderr.is_empty(), "unexpected stderr: {output:?}");
    assert_no_database_rows(&data_dir, flag);
}

fn assert_help_flag(flag: &str, sandbox_name: &str) {
    let data_dir = sandbox_data_dir("help", sandbox_name);
    let output = run_mw(flag, &data_dir);

    assert!(output.status.success(), "{flag} failed: {output:?}");
    let stdout = std::str::from_utf8(&output.stdout).expect("help output is UTF-8");
    assert!(
        !stdout.trim().is_empty(),
        "{flag} produced empty help output"
    );
    assert!(output.stderr.is_empty(), "unexpected stderr: {output:?}");

    // Principal command groups called out in #254.
    for needle in ["capture", "search", "memory", "doctor", "integrate"] {
        assert!(
            stdout.contains(needle),
            "{flag} help missing {needle:?}: {stdout}"
        );
    }

    assert_no_database_rows(&data_dir, flag);
}

#[test]
fn long_version_flag_prints_version_without_initializing_database() {
    assert_version_flag("--version", "long");
}

#[test]
fn short_version_flag_prints_version_without_initializing_database() {
    assert_version_flag("-V", "short");
}

#[test]
fn long_help_flag_lists_principal_commands_without_writing_database() {
    assert_help_flag("--help", "long");
}

#[test]
fn short_help_flag_lists_principal_commands_without_writing_database() {
    assert_help_flag("-h", "short");
}
