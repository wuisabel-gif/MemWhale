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
    let (agent, command): (Option<String>, String) = conn
        .query_row(
            "SELECT agent, command FROM command_runs WHERE notes LIKE '%agent:claude-code%'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!(agent.as_deref(), Some("claude"));
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
    let (agent, command, exit_code, notes): (Option<String>, String, Option<i64>, String) = conn
        .query_row(
            "SELECT agent, command, exit_code, notes FROM command_runs",
            [],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        )
        .unwrap();
    assert_eq!(agent.as_deref(), Some("rho"));
    assert_eq!(command, "[rho:after_tool_use]");
    assert!(exit_code.is_none());
    assert!(notes.contains("agent:rho"), "{notes}");
    assert!(notes.contains("status:failed"), "{notes}");
    assert!(notes.contains("command:unknown"), "{notes}");
}

#[test]
fn repeated_claude_hook_writes_keep_each_row_attributed() {
    let data_dir = sandbox("claude-repeated");
    let payload = r#"{
        "hook_event_name": "PostToolUse",
        "tool_name": "Bash",
        "cwd": "/work",
        "tool_input": {"command": "cargo test --repeated"},
        "tool_response": {"stdout": "ok", "stderr": ""}
    }"#;

    for _ in 0..2 {
        let output = remember_from_hook(&data_dir, "claude", payload);
        assert!(output.status.success(), "{output:?}");
    }

    let conn = memorywhale_cli::storage::open_path(&data_dir.join("memorywhale.sqlite3")).unwrap();
    let (rows, attributed): (i64, i64) = conn
        .query_row(
            "SELECT COUNT(*), COUNT(agent) FROM command_runs",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert_eq!((rows, attributed), (2, 2));
}

#[test]
fn manual_capture_keeps_agent_null() {
    let data_dir = sandbox("manual");
    let output = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .args([
            "--cwd",
            "/work",
            "--exit-code",
            "0",
            "--stdout",
            "manual stdout",
            "--stderr",
            "manual stderr",
            "--notes",
            "manual capture",
            "--",
            "cargo",
            "test",
        ])
        .env("MEMORYWHALE_DATA_DIR", &data_dir)
        .env("PATH", "")
        .output()
        .expect("run manual mw-remember capture");
    assert!(output.status.success(), "{output:?}");

    let conn = memorywhale_cli::storage::open_path(&data_dir.join("memorywhale.sqlite3")).unwrap();
    let (agent, cwd, exit_code, stdout, stderr, notes): (
        Option<String>,
        Option<String>,
        Option<i64>,
        String,
        String,
        String,
    ) = conn
        .query_row(
            "SELECT agent, cwd, exit_code, stdout, stderr, notes FROM command_runs",
            [],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                ))
            },
        )
        .unwrap();
    assert!(agent.is_none());
    assert_eq!(cwd.as_deref(), Some("/work"));
    assert_eq!(exit_code, Some(0));
    assert_eq!(stdout, "manual stdout");
    assert_eq!(stderr, "manual stderr");
    assert!(notes.contains("manual capture"));
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
        String::from_utf8_lossy(&output.stderr)
            .contains("requires claude, codex, rho, cursor, or codewhale"),
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

#[test]
fn from_hook_records_codex_bash_and_names_a_past_fix() {
    let data_dir = sandbox("codex");
    let conn = memorywhale_cli::storage::open_path(&data_dir.join("memorywhale.sqlite3")).unwrap();
    for (argv, exit, stderr, at) in [
        (
            r#"["cargo","build"]"#,
            101,
            "error: linker `cc` not found",
            "2026-01-01T00:00:00Z",
        ),
        (
            r#"["xcode-select","--install"]"#,
            0,
            "",
            "2026-01-01T00:01:00Z",
        ),
    ] {
        conn.execute(
            "INSERT INTO command_runs (command, argv_json, cwd, exit_code, stderr, created_at)
             VALUES (json_extract(?1, '$[0]'), ?1, '/work', ?2, ?3, ?4)",
            rusqlite::params![argv, exit, stderr, at],
        )
        .unwrap();
    }

    // Current Codex sends the combined output as a string and no exit code.
    let output = remember_from_hook(
        &data_dir,
        "codex",
        r#"{
            "session_id": "s1",
            "turn_id": "t1",
            "cwd": "/work",
            "hook_event_name": "PostToolUse",
            "permission_mode": "default",
            "tool_name": "Bash",
            "tool_use_id": "call_1",
            "tool_input": {"command": "cargo build"},
            "tool_response": "   Compiling app v0.1.0\nerror: linker `cc` not found\n"
        }"#,
    );
    assert!(output.status.success(), "{output:?}");
    let reply: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(reply["hookSpecificOutput"]["hookEventName"], "PostToolUse");
    let note = reply["hookSpecificOutput"]["additionalContext"]
        .as_str()
        .unwrap();
    assert!(
        note.contains("the fix was: xcode-select --install"),
        "{note}"
    );

    let (notes, exit): (String, Option<i64>) = conn
        .query_row(
            "SELECT notes, exit_code FROM command_runs WHERE agent = 'codex'",
            [],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .unwrap();
    assert!(notes.contains("agent:codex"), "{notes}");
    assert_eq!(exit, None);
}
