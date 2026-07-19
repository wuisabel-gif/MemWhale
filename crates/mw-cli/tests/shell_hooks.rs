//! End-to-end check of the lightweight shell-hook capture tier.
//!
//! Installs hooks into a sandbox HOME, runs a real `bash -i`, and asserts what
//! landed (and what didn't) in the sandbox database.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

fn sandbox(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("mw-hooks-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("bin")).unwrap();
    fs::create_dir_all(dir.join("data")).unwrap();
    fs::create_dir_all(dir.join("work")).unwrap();
    // mw-remember must be reachable as that exact name on PATH.
    let src = PathBuf::from(env!("CARGO_BIN_EXE_mw-remember"));
    fs::copy(&src, dir.join("bin").join("mw-remember")).unwrap();
    dir
}

fn install_hooks(home: &Path) {
    let out = Command::new(env!("CARGO_BIN_EXE_mw"))
        .args(["hooks", "install"])
        .env("HOME", home)
        .env("SHELL", "/bin/bash")
        .env("MEMORYWHALE_DATA_DIR", home.join("data"))
        .output()
        .unwrap();
    assert!(out.status.success(), "install failed: {out:?}");
}

/// Run one command in an interactive bash with the sandbox rc loaded.
fn run_bash(home: &Path, cwd: &Path, script: &str, extra: &[(&str, &str)]) {
    let mut cmd = Command::new("bash");
    cmd.arg("-i")
        .current_dir(cwd)
        .env("HOME", home)
        .env("MEMORYWHALE_DATA_DIR", home.join("data"))
        .env(
            "PATH",
            format!("{}:{}", home.join("bin").display(), std::env::var("PATH").unwrap()),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    for (k, v) in extra {
        cmd.env(k, v);
    }
    let mut child = cmd.spawn().unwrap();
    {
        use std::io::Write;
        // Trailing `true` gives the prompt hook a chance to fire for `script`.
        write!(child.stdin.as_mut().unwrap(), "{script}\ntrue\nexit\n").unwrap();
    }
    child.wait().unwrap();
}

fn hook_rows(home: &Path) -> Vec<(String, String, i64)> {
    let db = home.join("data").join("memorywhale.sqlite3");
    if !db.exists() {
        return Vec::new();
    }
    let conn = rusqlite::Connection::open(db).unwrap();
    let exists: bool = conn
        .prepare("SELECT 1 FROM pragma_table_info('command_runs') WHERE name='capture_kind'")
        .and_then(|mut s| s.exists([]))
        .unwrap_or(false);
    if !exists {
        return Vec::new();
    }
    let mut stmt = conn
        .prepare("SELECT command, cwd, exit_code FROM command_runs WHERE capture_kind = 'hook'")
        .unwrap();
    let rows = stmt
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?.unwrap_or_default(),
                r.get::<_, Option<i64>>(2)?.unwrap_or(-1),
            ))
        })
        .unwrap()
        .flatten()
        .collect();
    rows
}

/// The hook writes from a detached subshell, so rows land asynchronously.
/// Poll rather than guess a sleep.
fn wait_for_hook_row(home: &Path, command: &str) -> Option<(String, String, i64)> {
    for _ in 0..60 {
        if let Some(row) = hook_rows(home).into_iter().find(|(c, _, _)| c == command) {
            return Some(row);
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }
    None
}

/// Negative assertions have nothing to poll for: wait out the async write.
fn settle() {
    std::thread::sleep(std::time::Duration::from_secs(3));
}

#[test]
fn hook_records_command_cwd_and_exit_code() {
    let home = sandbox("basic");
    install_hooks(&home);

    let rc = fs::read_to_string(home.join(".bashrc")).unwrap();
    assert!(rc.contains("memorywhale shell hooks"), "rc block missing: {rc}");
    // Idempotent: installing twice must not duplicate the block.
    install_hooks(&home);
    let rc2 = fs::read_to_string(home.join(".bashrc")).unwrap();
    assert_eq!(rc2.matches(">>> memorywhale shell hooks >>>").count(), 1);

    let work = home.join("work");
    run_bash(&home, &work, "sh -c 'exit 7'", &[]);

    let hit = wait_for_hook_row(&home, "sh")
        .unwrap_or_else(|| panic!("no hook row for `sh`; got {:?}", hook_rows(&home)));
    let rows = hook_rows(&home);
    assert_eq!(hit.2, 7, "wrong exit code in {rows:?}");
    assert!(
        Path::new(&hit.1).ends_with("work"),
        "wrong cwd {:?} in {rows:?}",
        hit.1
    );
}

#[test]
fn wrapper_session_suppresses_hook_rows() {
    let home = sandbox("guard");
    install_hooks(&home);
    run_bash(
        &home,
        &home.join("work"),
        "sh -c 'exit 7'",
        &[("MW_RECORDING", "1")],
    );
    settle();
    assert!(
        hook_rows(&home).is_empty(),
        "hook logged inside an mw capture session: {:?}",
        hook_rows(&home)
    );
}

#[test]
fn off_gated_directory_records_nothing() {
    let home = sandbox("gate");
    install_hooks(&home);
    let work = home.join("work");
    fs::write(work.join(".mwignore"), "capture = \"off\"\n").unwrap();
    run_bash(&home, &work, "sh -c 'exit 7'", &[]);
    settle();
    assert!(
        hook_rows(&home).is_empty(),
        "hook wrote in an off-gated directory: {:?}",
        hook_rows(&home)
    );
}
