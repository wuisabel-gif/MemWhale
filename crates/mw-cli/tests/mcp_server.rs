//! End-to-end check of the `mw-mcp` server over real stdio.
//!
//! Spawns the actual binary against a hermetic temp data dir (via
//! `MEMORYWHALE_DATA_DIR`, the same env the server reads to resolve its DB),
//! speaks newline-delimited JSON-RPC 2.0 on its stdin/stdout, and asserts that
//! `tools/list` advertises all six tools and that each one returns a non-error
//! response — including on a fresh, empty database.
#![cfg(unix)]

use serde_json::{json, Value};
use std::io::{BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const TOOLS: [&str; 6] = [
    "recent_errors",
    "search_memory",
    "get_context",
    "remember",
    "similar_failures",
    "stats",
];

fn modern_meta() -> Value {
    json!({
        "io.modelcontextprotocol/protocolVersion": "2026-07-28",
        "io.modelcontextprotocol/clientInfo": {
            "name": "integration-test",
            "version": "1"
        },
        "io.modelcontextprotocol/clientCapabilities": {}
    })
}

/// Minimal valid arguments for each tool — enough to pass its required-field
/// check. Everything here must succeed against an empty store.
fn args_for(tool: &str) -> Value {
    match tool {
        "search_memory" => json!({"query": "linker error"}),
        "remember" => json!({"text": "integration-test lesson"}),
        "similar_failures" => json!({"error_text": "error: linking with cc failed"}),
        _ => json!({}),
    }
}

#[test]
fn server_lists_and_calls_all_six_tools_on_empty_db() {
    let dir = std::env::temp_dir().join(format!("mw-mcp-it-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();

    let mut child = Command::new(env!("CARGO_BIN_EXE_mw-mcp"))
        .env("MEMORYWHALE_DATA_DIR", &dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mw-mcp");

    // Build the request batch: discovery, tools/list, then a call per tool.
    // ids: 1 = discover, 2 = tools/list, 3.. = one per tool (in TOOLS order).
    let mut requests = vec![
        json!({"jsonrpc": "2.0", "id": 1, "method": "server/discover",
               "params": {"_meta": modern_meta()}}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list",
               "params": {"_meta": modern_meta()}}),
    ];
    for (i, tool) in TOOLS.iter().enumerate() {
        requests.push(json!({
            "jsonrpc": "2.0", "id": 3 + i as i64, "method": "tools/call",
            "params": {"name": tool, "arguments": args_for(tool), "_meta": modern_meta()}
        }));
    }

    // Write every request line-delimited, then close stdin so the server hits
    // EOF and exits its loop after replying.
    let mut stdin = child.stdin.take().unwrap();
    for req in &requests {
        writeln!(stdin, "{req}").unwrap();
    }
    drop(stdin);

    // Read all response lines on a worker thread so a hang fails the test via
    // the recv timeout instead of blocking CI forever.
    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines() {
            let Ok(line) = line else { break };
            if line.trim().is_empty() {
                continue;
            }
            if tx.send(line).is_err() {
                break;
            }
        }
    });

    // Collect one response per request id (8 total: discover + list + 6 calls).
    let mut by_id: std::collections::HashMap<i64, Value> = std::collections::HashMap::new();
    while by_id.len() < requests.len() {
        let line = rx
            .recv_timeout(Duration::from_secs(15))
            .expect("timed out waiting for an mw-mcp response");
        let v: Value = serde_json::from_str(&line).expect("valid JSON-RPC line");
        if let Some(id) = v.get("id").and_then(Value::as_i64) {
            by_id.insert(id, v);
        }
    }
    let _ = child.wait();

    let discover = &by_id[&1];
    assert_eq!(discover["result"]["resultType"], "complete");
    assert_eq!(discover["result"]["supportedVersions"][0], "2026-07-28");
    assert!(discover["result"]["capabilities"]["tools"].is_object());

    // tools/list must advertise all six tool names using the current result shape.
    let list = &by_id[&2];
    assert!(list.get("error").is_none(), "tools/list errored: {list}");
    assert_eq!(list["result"]["resultType"], "complete");
    let names: Vec<&str> = list["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t.get("name").and_then(Value::as_str))
        .collect();
    for tool in TOOLS {
        assert!(
            names.contains(&tool),
            "tools/list missing {tool}: {names:?}"
        );
    }
    assert_eq!(names.len(), 6, "expected exactly 6 tools, got {names:?}");

    // Every tools/call — on the empty DB — echoes its id, has a result, no error.
    for (i, tool) in TOOLS.iter().enumerate() {
        let id = 3 + i as i64;
        let resp = &by_id[&id];
        assert_eq!(
            resp["id"].as_i64(),
            Some(id),
            "{tool}: id not echoed: {resp}"
        );
        assert!(
            resp.get("error").is_none(),
            "{tool} returned an error: {resp}"
        );
        assert!(resp.get("result").is_some(), "{tool} has no result: {resp}");
        assert_eq!(resp["result"]["resultType"], "complete", "{tool}: {resp}");
        assert_eq!(resp["result"]["isError"], false, "{tool}: {resp}");
    }

    let _ = std::fs::remove_dir_all(&dir);
}

// --- recall block of `stats`, end to end (issue #426) -----------------------

const ERR: &str = "error: linker `cc` not found";
const CWD: &str = "/mw-recall-it";

fn recall_sandbox(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("mw-mcp-recall-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

/// Record one command run with the real `mw-remember` binary.
fn remember(dir: &std::path::Path, exit: i64, stderr: &str, cmd: &[&str]) {
    let out = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .env("MEMORYWHALE_DATA_DIR", dir)
        .args([
            "--cwd",
            CWD,
            "--exit-code",
            &exit.to_string(),
            "--stderr",
            stderr,
            "--",
        ])
        .args(cmd)
        .output()
        .expect("spawn mw-remember");
    assert!(out.status.success(), "mw-remember failed: {out:?}");
}

/// Send a failing Claude `PostToolUse` Bash payload through
/// `mw-remember --from-hook claude`; this is the path that shows (and
/// records) a recall note. Returns the hook's stdout.
fn claude_hook_failure(dir: &std::path::Path, command: &str) -> String {
    let payload = json!({
        "hook_event_name": "PostToolUse",
        "tool_name": "Bash",
        "cwd": CWD,
        "tool_input": {"command": command},
        "tool_response": {"stdout": "", "stderr": ERR, "is_error": true}
    });
    let mut child = Command::new(env!("CARGO_BIN_EXE_mw-remember"))
        .args(["--from-hook", "claude"])
        .env("MEMORYWHALE_DATA_DIR", dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn mw-remember --from-hook");
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(payload.to_string().as_bytes())
        .unwrap();
    let out = child.wait_with_output().expect("wait mw-remember");
    assert!(out.status.success(), "{out:?}");
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// Call the `stats` tool on the real `mw-mcp` binary and return its `recall` block.
fn mcp_stats_recall(dir: &std::path::Path) -> Value {
    let mut child = Command::new(env!("CARGO_BIN_EXE_mw-mcp"))
        .env("MEMORYWHALE_DATA_DIR", dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("spawn mw-mcp");
    let req = json!({"jsonrpc": "2.0", "id": 1, "method": "tools/call",
        "params": {"name": "stats", "arguments": {}, "_meta": modern_meta()}});
    let mut stdin = child.stdin.take().unwrap();
    writeln!(stdin, "{req}").unwrap();
    drop(stdin);

    let stdout = child.stdout.take().unwrap();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            let Ok(line) = line else { break };
            if !line.trim().is_empty() && tx.send(line).is_err() {
                break;
            }
        }
    });
    let resp: Value = loop {
        let line = rx
            .recv_timeout(Duration::from_secs(15))
            .expect("timed out waiting for stats");
        let v: Value = serde_json::from_str(&line).expect("valid JSON-RPC line");
        if v.get("id").and_then(Value::as_i64) == Some(1) {
            break v;
        }
    };
    let _ = child.wait();
    assert!(resp.get("error").is_none(), "stats errored: {resp}");
    assert_eq!(resp["result"]["isError"], false, "{resp}");
    let stats = resp["result"]
        .get("structuredContent")
        .cloned()
        .or_else(|| {
            resp["result"]["content"][0]["text"]
                .as_str()
                .and_then(|t| serde_json::from_str(t).ok())
        })
        .unwrap_or_else(|| panic!("no stats payload: {resp}"));
    stats["recall"].clone()
}

fn assert_recall(recall: &Value, shown: i64, followed: i64, worked: i64) {
    assert_eq!(recall["hints_shown"].as_i64(), Some(shown), "{recall}");
    assert_eq!(recall["followed"].as_i64(), Some(followed), "{recall}");
    assert_eq!(recall["worked"].as_i64(), Some(worked), "{recall}");
}

/// Fail, fix, pass: gives the store a recorded fix for `ERR`.
fn seed_known_fix(dir: &std::path::Path) {
    remember(dir, 101, ERR, &["cargo", "build"]);
    remember(dir, 0, "", &["xcode-select", "--install"]);
    remember(dir, 0, "", &["cargo", "build"]);
}

#[test]
fn stats_recall_is_all_zero_on_empty_db() {
    let dir = recall_sandbox("empty");
    assert_recall(&mcp_stats_recall(&dir), 0, 0, 0);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn stats_recall_counts_a_hint_that_was_followed_and_worked() {
    let dir = recall_sandbox("followed");
    seed_known_fix(&dir);
    let note = claude_hook_failure(&dir, "cargo build");
    assert!(
        note.contains("xcode-select --install"),
        "hook note: {note:?}"
    );
    remember(&dir, 0, "", &["xcode-select", "--install"]);
    remember(&dir, 0, "", &["cargo", "build"]);
    assert_recall(&mcp_stats_recall(&dir), 1, 1, 1);
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn stats_recall_counts_a_hint_that_was_shown_but_not_followed() {
    let dir = recall_sandbox("ignored");
    seed_known_fix(&dir);
    let note = claude_hook_failure(&dir, "cargo build");
    assert!(
        note.contains("xcode-select --install"),
        "hook note: {note:?}"
    );
    assert_recall(&mcp_stats_recall(&dir), 1, 0, 0);
    let _ = std::fs::remove_dir_all(&dir);
}
