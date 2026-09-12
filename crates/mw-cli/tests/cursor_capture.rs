//! Documentation-derived Cursor events, not a live Cursor/version compatibility test.
//! Exercises Capture -> Memory -> CLI/MCP Retrieval with no client or model account.
use rusqlite::Connection;
use serde_json::{json, Value};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

struct Sandbox {
    root: PathBuf,
    project: PathBuf,
    data: PathBuf,
}
impl Sandbox {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = std::env::temp_dir().join(format!(
            "mw-cursor-e2e-{}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&root).unwrap();
        let root = root.canonicalize().unwrap();
        for dir in ["project", "data", "home", "rho", "config", "empty-bin"] {
            std::fs::create_dir(root.join(dir)).unwrap();
        }
        Self {
            project: root.join("project"),
            data: root.join("data"),
            root,
        }
    }
    fn command(&self, binary: &str) -> Command {
        let mut cmd = Command::new(binary);
        cmd.env_clear()
            .env("HOME", self.root.join("home"))
            .env("RHO_HOME", self.root.join("rho"))
            .env("XDG_CONFIG_HOME", self.root.join("config"))
            .env("XDG_DATA_HOME", &self.data)
            .env("MEMORYWHALE_DATA_DIR", &self.data)
            .env("PATH", self.root.join("empty-bin"))
            .current_dir(&self.project);
        cmd
    }
    fn pipe(&self, binary: &str, args: &[&str], input: &[u8]) -> Output {
        let mut child = self
            .command(binary)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        // Oversized input may be rejected before the entire pipe has been read.
        if let Err(err) = child.stdin.take().unwrap().write_all(input) {
            assert_eq!(err.kind(), std::io::ErrorKind::BrokenPipe);
        }
        let output = child.wait_with_output().unwrap();
        assert!(output.status.success(), "{output:?}");
        output
    }
    fn hook_bytes(&self, bytes: &[u8]) {
        let output = self.pipe(
            env!("CARGO_BIN_EXE_mw-remember"),
            &["--from-hook", "cursor"],
            bytes,
        );
        assert!(
            output.stdout.is_empty(),
            "hook stdout must be empty: {output:?}"
        );
        assert!(
            output.stderr.is_empty(),
            "hook stderr must be quiet by default: {output:?}"
        );
    }
    fn hook(&self, event: &Value) {
        self.hook_bytes(&serde_json::to_vec(event).unwrap());
    }
    fn events(&self) -> Vec<Value> {
        let mut cases: Vec<Value> =
            serde_json::from_str(include_str!("fixtures/cursor/hooks.json")).unwrap();
        assert_eq!(cases.len(), 7);
        for case in &mut cases {
            case["event"]["cwd"] = json!(self.project);
        }
        cases
    }
    fn rows(&self) -> Vec<Value> {
        let path = self.data.join("memorywhale.sqlite3");
        if !path.exists() {
            return vec![];
        }
        let conn =
            Connection::open_with_flags(path, rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        let mut stmt = conn.prepare("SELECT id, command, argv_json, cwd, agent, exit_code, stdout, stderr, notes, capture_kind FROM command_runs ORDER BY id").unwrap();
        stmt.query_map([], |r| Ok(json!({"id":r.get::<_,i64>(0)?, "command":r.get::<_,String>(1)?, "argv":r.get::<_,String>(2)?, "cwd":r.get::<_,String>(3)?, "agent":r.get::<_,Option<String>>(4)?, "exit":r.get::<_,Option<i64>>(5)?, "stdout":r.get::<_,String>(6)?, "stderr":r.get::<_,String>(7)?, "notes":r.get::<_,String>(8)?, "kind":r.get::<_,String>(9)?}))).unwrap().collect::<Result<Vec<_>,_>>().unwrap()
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

#[test]
fn diagnostics_are_explicit_bounded_and_nonfatal_for_payload_and_storage_failures() {
    let s = Sandbox::new();
    let valid = serde_json::to_vec(&s.events()[0]["event"]).unwrap();
    std::fs::create_dir(s.data.join("memorywhale.sqlite3")).unwrap();
    for (input, message) in [
        (
            b"{invalid PRIVATE-PAYLOAD".as_slice(),
            "invalid Cursor hook JSON",
        ),
        (valid.as_slice(), "Cursor event could not be recorded"),
    ] {
        for flag in [None, Some("0"), Some("1")] {
            let mut command = s.command(env!("CARGO_BIN_EXE_mw-remember"));
            if let Some(flag) = flag {
                command.env("MEMORYWHALE_HOOK_DIAGNOSTICS", flag);
            }
            let mut child = command
                .args(["--from-hook", "cursor"])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .unwrap();
            child.stdin.take().unwrap().write_all(input).unwrap();
            let result = child.wait_with_output().unwrap();
            assert!(result.status.success(), "{result:?}");
            assert!(result.stdout.is_empty());
            if flag == Some("1") {
                let stderr = String::from_utf8(result.stderr).unwrap();
                assert!(stderr.contains(message), "{stderr}");
                assert!(!stderr.contains("PRIVATE-PAYLOAD"));
                assert!(!stderr.contains(s.root.to_str().unwrap()));
                assert!(stderr.len() < 256);
            } else {
                assert!(result.stderr.is_empty());
            }
        }
    }
}

#[test]
fn documented_outcomes_persist_without_inventing_exit_codes() {
    let s = Sandbox::new();
    let cases = s.events();
    for case in &cases {
        s.hook(&case["event"]);
    }
    let rows = s.rows();
    assert_eq!(rows.len(), 7);
    let statuses = [
        "exit_zero",
        "nonzero_exit",
        "completed_exit_unknown",
        "timeout",
        "permission_denied",
        "cancelled",
        "completed_exit_unknown",
    ];
    for (i, row) in rows.iter().enumerate() {
        let event = &cases[i]["event"];
        assert_eq!(row["agent"], "cursor");
        assert_eq!(row["cwd"], json!(s.project));
        assert_eq!(row["kind"], "full");
        assert_eq!(row["command"], event["tool_input"]["command"]);
        assert_eq!(
            serde_json::from_str::<Value>(row["argv"].as_str().unwrap()).unwrap(),
            json!([event["tool_input"]["command"]])
        );
        assert_eq!(
            row["exit"],
            match i {
                0 => json!(0),
                1 => json!(2),
                _ => Value::Null,
            }
        );
        let notes = row["notes"].as_str().unwrap();
        assert!(
            notes.contains(&format!("cursor_status:{} ", statuses[i])),
            "{notes}"
        );
        assert!(notes.contains(event["tool_use_id"].as_str().unwrap()));
        if i < 3 {
            let result: Value =
                serde_json::from_str(event["tool_output"].as_str().unwrap()).unwrap();
            assert_eq!(
                row["stdout"],
                result.get("stdout").unwrap_or(&json!("")).clone()
            );
            assert_eq!(
                row["stderr"],
                result.get("stderr").unwrap_or(&json!("")).clone()
            );
        } else if i < 6 {
            assert_eq!(row["stderr"], event["error_message"]);
        }
    }
    assert!(rows[4]["notes"].as_str().unwrap().contains("not_executed"));
    assert!(rows[5]["notes"].as_str().unwrap().contains("not_confirmed"));
    assert_eq!(rows[6]["stdout"], "");
    assert_eq!(rows[6]["stderr"], "");
    assert!(rows[6]["notes"]
        .as_str()
        .unwrap()
        .contains("\"output_unavailable\":true"));
    let mut failure = cases[5]["event"].clone();
    failure["is_interrupt"] = json!(false);
    s.hook(&failure);
    assert!(s.rows()[7]["notes"]
        .as_str()
        .unwrap()
        .contains("cursor_status:tool_failure"));
}

#[test]
fn ignored_invalid_and_oversized_events_are_fail_open_without_fallback() {
    let s = Sandbox::new();
    for bytes in [b"".as_slice(), b"{broken", b"null", b"[]", b"\xff"] {
        s.hook_bytes(bytes);
    }
    s.hook_bytes(&vec![
        b' ';
        memorywhale_cli::agent_hook::MAX_CURSOR_HOOK_BYTES
            as usize
            + 1
    ]);
    let base = s.events()[0]["event"].clone();
    let mut variants = Vec::new();
    for event in ["afterShellExecution", "preToolUse", "unknown"] {
        let mut v = base.clone();
        v["hook_event_name"] = json!(event);
        variants.push(v);
    }
    for tool in ["Read", "bash", "shell"] {
        let mut v = base.clone();
        v["tool_name"] = json!(tool);
        variants.push(v);
    }
    for command in [
        "".to_owned(),
        " ".to_owned(),
        "x".repeat(8193),
        "a\0b".to_owned(),
    ] {
        let mut v = base.clone();
        v["tool_input"]["command"] = json!(command);
        variants.push(v);
    }
    for cwd in [
        Value::Null,
        json!("relative"),
        json!(s.project.join("does-not-exist")),
        json!("/bad\0cwd"),
    ] {
        let mut v = base.clone();
        v["cwd"] = cwd;
        v["workspace_roots"] = json!([s.project]);
        variants.push(v);
    }
    let mut missing = base.clone();
    missing.as_object_mut().unwrap().remove("cwd");
    missing["workspace_roots"] = json!([s.project]);
    variants.push(missing);
    for v in variants {
        s.hook(&v);
    }
    assert!(
        s.rows().is_empty(),
        "invalid cwd must not fall back to real process cwd or workspace roots"
    );
    s.hook(&base);
    assert_eq!(s.rows().len(), 1, "sandbox itself allows valid capture");
}

#[test]
fn repeated_commands_are_preserved_but_overlapping_surface_is_ignored() {
    let s = Sandbox::new();
    let mut event = s.events()[0]["event"].clone();
    s.hook(&event);
    event["tool_use_id"] = json!("second-legitimate-execution");
    s.hook(&event);
    event["hook_event_name"] = json!("afterShellExecution");
    s.hook(&event);
    let rows = s.rows();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["command"], rows[1]["command"]);
    assert_ne!(rows[0]["id"], rows[1]["id"]);
    assert!(rows[1]["notes"]
        .as_str()
        .unwrap()
        .contains("second-legitimate-execution"));
}

#[test]
fn capture_policy_honors_mwignore_and_config_modes() {
    for mode in ["off", "commands-only"] {
        for local in [false, true] {
            let s = Sandbox::new();
            if local {
                std::fs::write(
                    s.project.join(".mwignore"),
                    format!("capture = \"{mode}\"\n"),
                )
                .unwrap();
            } else {
                std::fs::write(
                    s.data.join("config.toml"),
                    format!(
                        "[capture.paths]\n\"{}\" = \"{mode}\"\n",
                        s.project.display()
                    ),
                )
                .unwrap();
            }
            let mut event = s.events()[1]["event"].clone();
            event["tool_output"] = json!(
                json!({"exitCode":2,"stdout":"private stdout", "stderr":"private stderr"})
                    .to_string()
            );
            s.hook(&event);
            let rows = s.rows();
            if mode == "off" {
                assert!(rows.is_empty());
            } else {
                assert_eq!(rows.len(), 1);
                assert_eq!(rows[0]["stdout"], "");
                assert_eq!(rows[0]["stderr"], "");
                assert_eq!(rows[0]["exit"], 2);
                assert_eq!(rows[0]["command"], event["tool_input"]["command"]);
            }
        }
    }
}

#[test]
fn redaction_and_unicode_output_bounds_survive_persistence() {
    let s = Sandbox::new();
    let mut event = s.events()[0]["event"].clone();
    let secret = "ghp_0123456789abcdefghijABCDEF"; // deliberately fake
    event["tool_input"]["command"] = json!(format!("printf '{secret}'"));
    event["tool_output"] = json!(json!({"exitCode":0,"stdout":format!("{secret} stdout"),"stderr":format!("{secret} stderr")}).to_string());
    s.hook(&event);
    let row = &s.rows()[0];
    for field in ["command", "argv", "stdout", "stderr"] {
        let text = row[field].as_str().unwrap();
        assert!(!text.contains(secret), "{field}: {text}");
        assert!(text.contains("[REDACTED]"), "{field}: {text}");
    }
    let unicode = "鲸🐋".repeat(5000);
    event["tool_input"]["command"] =
        json!("printf 'legitimate unicode 鲸🐋'\n# preserve quoting && spacing");
    event["tool_output"] =
        json!(json!({"exitCode":0,"stdout":unicode,"stderr":unicode}).to_string());
    s.hook(&event);
    let row = &s.rows()[1];
    assert_eq!(row["command"], event["tool_input"]["command"]);
    for field in ["stdout", "stderr"] {
        let text = row[field].as_str().unwrap();
        assert!(text.len() <= 20_000);
        assert!(text.len() > 19_990);
        assert!(text.ends_with("\n[cursor: output truncated]"));
        assert!(!text.contains('\u{fffd}'));
        assert!(row["notes"]
            .as_str()
            .unwrap()
            .contains(&format!("\"{field}_truncated\":true")));
    }
}

#[test]
fn fresh_cli_and_mcp_retrieve_with_agent_filters() {
    let s = Sandbox::new();
    s.hook(&s.events()[0]["event"]);
    let output = s
        .command(env!("CARGO_BIN_EXE_mw-remember"))
        .args(["--", "echo cursor-fixture terminal-only-marker"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    let cli = s
        .command(env!("CARGO_BIN_EXE_mw"))
        .args(["search", "cursor-fixture", "agent:cursor"])
        .output()
        .unwrap();
    assert!(cli.status.success(), "{cli:?}");
    let text = String::from_utf8(cli.stdout).unwrap();
    assert!(text.contains("cursor-fixture success"), "{text}");
    assert!(!text.contains("terminal-only-marker"), "{text}");
    for (agent, expected, excluded) in [
        ("cursor", "cursor-fixture success", "terminal-only-marker"),
        ("terminal", "terminal-only-marker", "cursor-fixture success"),
    ] {
        let meta = json!({"io.modelcontextprotocol/protocolVersion":"2026-07-28", "io.modelcontextprotocol/clientInfo":{"name":"cursor-capture-test","version":"1"}, "io.modelcontextprotocol/clientCapabilities":{}});
        let requests = [
            json!({"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":meta}}),
            json!({"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"search_memory","arguments":{"query":"cursor-fixture","agent":agent},"_meta":meta}}),
        ];
        let input = requests
            .iter()
            .map(|r| format!("{r}\n"))
            .collect::<String>();
        let output = s.pipe(env!("CARGO_BIN_EXE_mw-mcp"), &[], input.as_bytes());
        let responses: Vec<Value> = String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|line| serde_json::from_str(line).unwrap())
            .collect();
        let response = responses
            .iter()
            .find(|r| r["id"] == 2)
            .expect("MCP tool response");
        assert!(response.get("error").is_none(), "{response}");
        assert_eq!(response["result"]["resultType"], "complete");
        assert_eq!(response["result"]["isError"], false);
        let text = response["result"].to_string();
        assert!(text.contains(expected), "{text}");
        assert!(!text.contains(excluded), "{text}");
    }
}
