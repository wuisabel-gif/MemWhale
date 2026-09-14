//! Receipt-contract replay, not a claim of live Codewhale compatibility.
use memorywhale_cli::agent_hook::codewhale::{record_from_slice, MAX_HOOK_BYTES};
use serde_json::{json, Value};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    process::{Command, Output, Stdio},
};

fn event() -> Value {
    json!({"schema_version":1,"event":"tool_call_after","tool_name":"bash",
        "session_id":"synthetic-session","tool_call_id":"synthetic-call",
        "session_id_truncated":false,"tool_call_id_truncated":false,"tool_name_truncated":false,
        "execution_receipt":{"schema_version":1,"command":"printf MW_CODEWHALE_CAPTURE",
        "cwd":"/fixture/project","command_truncated":false,"cwd_truncated":false,
        "execution":"started","completion":"completed","exit_code":0,
        "stdout":"MW_CODEWHALE_CAPTURE\n","stderr":"","stdout_truncated":false,
        "stderr_truncated":false,"output_mode":"separate"}})
}
fn parse(value: &Value) -> Option<memorywhale_cli::remember::CommandRecord> {
    record_from_slice(&serde_json::to_vec(value).unwrap())
        .ok()
        .flatten()
}

#[test]
fn optional_bundle_contains_only_a_nonsteering_completion_observer() {
    let manifest: Value = serde_json::from_str(include_str!(
        "../../../integrations/codewhale/capture-plugin/plugin.json"
    ))
    .unwrap();
    assert_eq!(manifest["name"], "memorywhale-capture");
    let extension = &manifest["extensions"]["net.codewhale"];
    assert_eq!(extension["hooks"]["path"], "hooks.toml");
    for unwanted in ["native", "skills", "commands", "mcp_servers"] {
        assert!(extension.get(unwanted).is_none());
    }
    let document = include_str!("../../../integrations/codewhale/capture-plugin/hooks.toml")
        .parse::<toml_edit::DocumentMut>()
        .unwrap();
    assert!(document.get("working_dir").is_none());
    assert!(document.get("default_timeout_secs").is_none());
    let hooks = document["hooks"].as_array_of_tables().unwrap();
    assert_eq!(hooks.len(), 1);
    let hook = hooks.get(0).unwrap();
    assert_eq!(hook["event"].as_str(), Some("tool_call_after"));
    assert_eq!(
        hook["command"].as_str(),
        Some("mw-remember --from-hook codewhale")
    );
    assert_eq!(hook["timeout_secs"].as_integer(), Some(5));
    assert_eq!(hook["continue_on_error"].as_bool(), Some(true));
    assert_eq!(hook["background"].as_bool(), Some(true));
    let conditions = hook["condition"]["conditions"].as_array().unwrap();
    let names: Vec<_> = conditions
        .iter()
        .map(|v| {
            v.as_inline_table()
                .unwrap()
                .get("name")
                .unwrap()
                .as_str()
                .unwrap()
        })
        .collect();
    assert_eq!(names, vec!["exec_shell", "Bash", "bash"]);
    let base: Value = serde_json::from_str(include_str!(
        "../../../integrations/codewhale/plugin/plugin.json"
    ))
    .unwrap();
    assert!(base["extensions"]["net.codewhale"].get("hooks").is_none());
}

#[test]
fn receipt_identity_is_authoritative_and_exit_codes_are_not_synthesized() {
    for name in ["exec_shell", "Bash", "bash"] {
        for exit in [json!(0), json!(7), json!(3221225477_i64), Value::Null] {
            let mut v = event();
            v["tool_name"] = json!(name);
            v["execution_receipt"]["exit_code"] = exit.clone();
            v["requested_command"] = json!("NOT_THE_EXECUTED_COMMAND");
            v["workspace"] = json!("/not-the-execution-cwd");
            let r = parse(&v).unwrap();
            assert_eq!(r.command_parts, vec!["printf MW_CODEWHALE_CAPTURE"]);
            assert_eq!(r.cwd.as_deref(), Some("/fixture/project"));
            assert_eq!(r.exit_code, exit.as_i64());
            assert_eq!(r.agent.as_deref(), Some("codewhale"));
        }
    }
    for completion in ["failed", "killed", "timed_out"] {
        let mut v = event();
        v["execution_receipt"]["completion"] = json!(completion);
        v["execution_receipt"]["exit_code"] = Value::Null;
        v["execution_receipt"]["stderr"] = json!("text says exit 7; not a numeric field");
        let r = parse(&v).unwrap();
        assert_eq!(r.exit_code, None);
        assert!(r.notes.contains(completion));
    }
}

#[test]
fn legacy_control_incomplete_and_truncated_identity_events_are_not_execution_records() {
    let mut variants = vec![json!({}), Value::Null];
    for name in ["tool_call_before", "on_error", "session_end"] {
        let mut v = event();
        v["event"] = json!(name);
        variants.push(v);
    }
    for name in ["read_file", "task_shell_wait", "mcp__memory__search_memory"] {
        let mut v = event();
        v["tool_name"] = json!(name);
        variants.push(v);
    }
    for field in [
        "session_id_truncated",
        "tool_call_id_truncated",
        "tool_name_truncated",
    ] {
        let mut v = event();
        v[field] = json!(true);
        variants.push(v);
    }
    for field in ["command_truncated", "cwd_truncated"] {
        let mut v = event();
        v["execution_receipt"][field] = json!(true);
        variants.push(v);
    }
    for completion in ["running", "unknown"] {
        let mut v = event();
        v["execution_receipt"]["completion"] = json!(completion);
        variants.push(v);
    }
    for cwd in ["relative", "", "/bad\0cwd"] {
        let mut v = event();
        v["execution_receipt"]["cwd"] = json!(cwd);
        variants.push(v);
    }
    let mut v = event();
    v["execution_receipt"] = Value::Null;
    variants.push(v);
    let mut v = event();
    v["execution_receipt"]["execution"] = json!("not_started");
    variants.push(v);
    let mut v = event();
    v["execution_receipt"]["exit_code"] = json!("0");
    variants.push(v);
    let mut v = event();
    v["execution_receipt"]["command"] = json!("x".repeat(4097));
    variants.push(v);
    for v in variants {
        assert!(parse(&v).is_none(), "{v}");
    }
    assert!(record_from_slice(b"{broken").is_err());
    assert!(record_from_slice(&vec![b' '; MAX_HOOK_BYTES as usize + 1]).is_err());
    let duplicate = event().to_string().replacen(
        "\"schema_version\":1",
        "\"schema_version\":1,\"schema_version\":1",
        1,
    );
    assert!(record_from_slice(duplicate.as_bytes()).is_err());
}

#[test]
fn combined_unavailable_and_truncated_outputs_are_honest() {
    let mut v = event();
    v["execution_receipt"]["output_mode"] = json!("combined");
    let r = parse(&v).unwrap();
    assert!(r
        .stdout
        .starts_with("[codewhale: combined stdout/stderr preview]"));
    assert!(r.stderr.is_empty());
    assert!(r.notes.contains("combined"));
    v["execution_receipt"]["output_mode"] = json!("unavailable");
    let r = parse(&v).unwrap();
    assert!(r.stdout.is_empty());
    assert!(r.notes.contains("unavailable"));
    v["execution_receipt"]["output_mode"] = json!("separate");
    v["execution_receipt"]["stdout"] = json!("鲸🐋".repeat(4000));
    let r = parse(&v).unwrap();
    assert!(r.stdout.len() < 20_100);
    assert!(r.stdout.contains("truncated]"));
    assert!(!r.stdout.contains('\u{fffd}'));
    assert!(r.notes.contains("\"stdout_truncated\":true"));
}

struct Sandbox {
    root: PathBuf,
    project: PathBuf,
    data: PathBuf,
}
impl Sandbox {
    fn new() -> Self {
        let mut bytes = [0u8; 16];
        getrandom::getrandom(&mut bytes).unwrap();
        let root =
            std::env::temp_dir().join(format!("mw-codewhale-{:032x}", u128::from_ne_bytes(bytes)));
        fs::create_dir(&root).unwrap();
        let root = root.canonicalize().unwrap();
        let project = root.join("project");
        let data = root.join("data");
        fs::create_dir(&project).unwrap();
        fs::create_dir(&data).unwrap();
        Self {
            root,
            project,
            data,
        }
    }
    fn command(&self, bin: &str) -> Command {
        let mut c = Command::new(bin);
        c.env_clear()
            .env("HOME", &self.root)
            .env("USERPROFILE", &self.root)
            .env("RHO_HOME", self.root.join("rho"))
            .env("XDG_CONFIG_HOME", self.root.join("config"))
            .env("PATH", "")
            .env("MEMORYWHALE_DATA_DIR", &self.data)
            .current_dir(&self.project);
        c
    }
    fn payload(&self) -> Value {
        let mut v = event();
        v["execution_receipt"]["cwd"] = json!(self.project);
        v
    }
    fn pipe(mut command: Command, bytes: &[u8]) -> Output {
        let mut c = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        if let Err(e) = c.stdin.take().unwrap().write_all(bytes) {
            assert_eq!(e.kind(), std::io::ErrorKind::BrokenPipe);
        }
        let output = c.wait_with_output().unwrap();
        assert!(output.status.success(), "{output:?}");
        output
    }
    fn hook(&self, v: &Value) {
        let mut c = self.command(env!("CARGO_BIN_EXE_mw-remember"));
        c.args(["--from-hook", "codewhale"]);
        let r = Self::pipe(c, v.to_string().as_bytes());
        assert!(r.stdout.is_empty());
        assert!(r.stderr.is_empty(), "{r:?}");
    }
    fn rows(&self) -> Vec<(String, Option<i64>, String, String, String)> {
        if !self.data.join("memorywhale.sqlite3").exists() {
            return vec![];
        }
        let conn = rusqlite::Connection::open(self.data.join("memorywhale.sqlite3")).unwrap();
        let mut statement = conn.prepare("SELECT command,exit_code,stdout,stderr,notes FROM command_runs WHERE agent='codewhale' ORDER BY id").unwrap();
        let rows = statement
            .query_map([], |r| {
                Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            })
            .unwrap()
            .collect::<Result<_, _>>()
            .unwrap();
        rows
    }
}
impl Drop for Sandbox {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[test]
fn capture_is_silent_requires_explicit_store_and_never_falls_back_to_workspace() {
    let s = Sandbox::new();
    for selected in [None, Some("relative")] {
        let mut c = s.command(env!("CARGO_BIN_EXE_mw-remember"));
        c.args(["--from-hook", "codewhale"]);
        c.env_remove("MEMORYWHALE_DATA_DIR");
        if let Some(value) = selected {
            c.env("MEMORYWHALE_DATA_DIR", value);
        }
        let r = Sandbox::pipe(c, s.payload().to_string().as_bytes());
        assert!(r.stdout.is_empty() && r.stderr.is_empty());
    }
    let mut v = s.payload();
    v["execution_receipt"]["cwd"] = json!(s.root.join("missing"));
    v["workspace"] = json!(s.project);
    s.hook(&v);
    assert!(s.rows().is_empty());
    assert!(!s.root.join("relative").exists());
    let mut c = s.command(env!("CARGO_BIN_EXE_mw-remember"));
    c.args(["--from-hook", "codewhale"])
        .env("MEMORYWHALE_HOOK_DIAGNOSTICS", "1");
    let r = Sandbox::pipe(c, b"private-secret-invalid");
    assert!(r.stdout.is_empty());
    assert!(!String::from_utf8_lossy(&r.stderr).contains("private-secret"));
    assert!(!r.stderr.is_empty());
}

#[test]
fn exclusions_redaction_and_commands_only_apply_to_effective_cwd() {
    for mode in ["off", "commands-only", "full"] {
        let s = Sandbox::new();
        fs::write(
            s.project.join(".mwignore"),
            format!("capture = \"{mode}\"\n"),
        )
        .unwrap();
        let mut v = s.payload();
        let secret = "ghp_0123456789abcdefghijABCDEF";
        v["execution_receipt"]["command"] = json!(format!("printf '{secret}'"));
        v["execution_receipt"]["stdout"] = json!(secret);
        v["execution_receipt"]["stderr"] = json!(secret);
        v["execution_receipt"]["exit_code"] = Value::Null;
        s.hook(&v);
        let rows = s.rows();
        if mode == "off" {
            assert!(rows.is_empty());
            continue;
        }
        assert_eq!(rows.len(), 1);
        assert!(!rows[0].0.contains(secret));
        assert!(rows[0].0.contains("[REDACTED]"));
        assert_eq!(rows[0].1, None);
        if mode == "commands-only" {
            assert!(rows[0].2.is_empty() && rows[0].3.is_empty());
        } else {
            assert!(rows[0].2.contains("[REDACTED]"));
            assert!(rows[0].3.contains("[REDACTED]"));
        }
    }
}

#[test]
fn repeated_executions_are_preserved_without_double_recording_after_error_surfaces() {
    let s = Sandbox::new();
    let mut v = s.payload();
    v["execution_receipt"]["exit_code"] = json!(3221225477_i64);
    s.hook(&v);
    v["event"] = json!("on_error");
    s.hook(&v);
    v["event"] = json!("tool_call_after");
    s.hook(&v);
    v["tool_call_id"] = json!("second-execution");
    s.hook(&v);
    assert_eq!(s.rows().len(), 2);
    assert_eq!(s.rows()[0].1, Some(3221225477));
    let output = s
        .command(env!("CARGO_BIN_EXE_mw"))
        .args(["search", "MW_CODEWHALE_CAPTURE", "agent:codewhale"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("MW_CODEWHALE_CAPTURE"));
    let meta = json!({"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"codewhale-receipt-test","version":"1"},"io.modelcontextprotocol/clientCapabilities":{}});
    let req = json!({"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"search_memory","arguments":{"query":"MW_CODEWHALE_CAPTURE","agent":"codewhale"},"_meta":meta}});
    let result = Sandbox::pipe(
        s.command(env!("CARGO_BIN_EXE_mw-mcp")),
        format!("{req}\n").as_bytes(),
    );
    let reply: Value = serde_json::from_slice(&result.stdout).unwrap();
    assert!(reply.get("error").is_none(), "{reply}");
    assert_eq!(reply["result"]["isError"], false);
    assert!(reply.to_string().contains("MW_CODEWHALE_CAPTURE"));
}
