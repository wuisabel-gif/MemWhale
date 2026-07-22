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

    // Build the request batch: initialize, tools/list, then a call per tool.
    // ids: 1 = initialize, 2 = tools/list, 3.. = one per tool (in TOOLS order).
    let mut requests = vec![
        json!({"jsonrpc": "2.0", "id": 1, "method": "initialize",
               "params": {"clientInfo": {"name": "integration-test"}}}),
        json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list"}),
    ];
    for (i, tool) in TOOLS.iter().enumerate() {
        requests.push(json!({
            "jsonrpc": "2.0", "id": 3 + i as i64, "method": "tools/call",
            "params": {"name": tool, "arguments": args_for(tool)}
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

    // Collect one response per request id (8 total: initialize + list + 6 calls).
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

    // tools/list must advertise all six tool names.
    let list = &by_id[&2];
    assert!(list.get("error").is_none(), "tools/list errored: {list}");
    let names: Vec<&str> = list["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t.get("name").and_then(Value::as_str))
        .collect();
    for tool in TOOLS {
        assert!(names.contains(&tool), "tools/list missing {tool}: {names:?}");
    }
    assert_eq!(names.len(), 6, "expected exactly 6 tools, got {names:?}");

    // Every tools/call — on the empty DB — echoes its id, has a result, no error.
    for (i, tool) in TOOLS.iter().enumerate() {
        let id = 3 + i as i64;
        let resp = &by_id[&id];
        assert_eq!(resp["id"].as_i64(), Some(id), "{tool}: id not echoed: {resp}");
        assert!(resp.get("error").is_none(), "{tool} returned an error: {resp}");
        assert!(resp.get("result").is_some(), "{tool} has no result: {resp}");
    }

    let _ = std::fs::remove_dir_all(&dir);
}
