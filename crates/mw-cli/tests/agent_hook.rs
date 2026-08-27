use std::io::Write;
use std::process::{Command, Stdio};

fn sandbox(name: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "mw-agent-hook-{name}-{}-{}",
        std::process::id(),
        std::thread::current().name().unwrap_or("test")
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).unwrap();
    path
}

fn remember_from_hook(
    data_dir: &std::path::Path,
    agent: &str,
    payload: &str,
) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .args(["--from-hook", agent])
        .env("MEMORYWHALE_DATA_DIR", data_dir)
        .env("PATH", "")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mw-remember --from-hook");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(payload.as_bytes())
        .unwrap();
    child.wait_with_output().expect("wait mw-remember")
}

#[test]
fn from_hook_records_a_claude_bash_payload() {
    let data_dir = sandbox("claude");
    let output = remember_from_hook(
        &data_dir,
        "claude",
        r#"{
            "hook_event_name": "PostToolUse",
            "tool_name": "Bash",
            "cwd": "/work",
            "tool_input": {"command": "cargo test --from-hook-claude"},
            "tool_response": {"stdout": "ok", "stderr": ""}
        }"#,
    );
    assert!(output.status.success(), "{output:?}");
    assert!(
        output.stdout.is_empty(),
        "hook mode must stay silent: {output:?}"
    );

    let conn = memorywhale_cli::storage::open_path(&data_dir.join("memorywhale.sqlite3")).unwrap();
    let command: String = conn
        .query_row(
            "SELECT command FROM command_runs WHERE notes LIKE '%agent:claude-code%'",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert_eq!(command, "cargo test --from-hook-claude");
}

#[test]
fn from_hook_records_a_rho_failure_without_command_text() {
    let data_dir = sandbox("rho");
    let output = remember_from_hook(
        &data_dir,
        "rho",
        r#"{
            "event": "after_tool_use",
            "workspace": {"root": "/work"},
            "payload": {
                "tool": {"name": "bash"},
                "status": "failed",
                "failure": {"kind": "tool", "message": "exit 1"}
            }
        }"#,
    );
    assert!(output.status.success(), "{output:?}");

    let conn = memorywhale_cli::storage::open_path(&data_dir.join("memorywhale.sqlite3")).unwrap();
    let (command, exit_code, notes): (String, Option<i64>, String) = conn
        .query_row(
            "SELECT command, exit_code, notes FROM command_runs",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .unwrap();
    assert_eq!(command, "[rho:after_tool_use]");
    assert!(exit_code.is_none());
    assert!(notes.contains("agent:rho"), "{notes}");
    assert!(notes.contains("status:failed"), "{notes}");
    assert!(notes.contains("command:unknown"), "{notes}");
}

#[test]
fn from_hook_ignores_unknown_json_and_exits_zero() {
    let data_dir = sandbox("skip");
    let output = remember_from_hook(&data_dir, "claude", r#"{"tool_name":"Read"}"#);
    assert!(output.status.success(), "{output:?}");
    assert!(!data_dir.join("memorywhale.sqlite3").exists());
}

#[test]
fn from_hook_requires_a_named_client() {
    let output = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .arg("--from-hook")
        .env("PATH", "")
        .output()
        .expect("run mw-remember --from-hook");
    assert!(!output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("requires claude or rho"),
        "{output:?}"
    );
}

#[test]
fn from_hook_rejects_mixed_options() {
    let output = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .args(["--from-hook", "claude", "--cwd", "/tmp"])
        .env("PATH", "")
        .output()
        .expect("run mixed mw-remember");
    assert!(!output.status.success(), "{output:?}");
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("cannot be mixed"),
        "{output:?}"
    );
}
