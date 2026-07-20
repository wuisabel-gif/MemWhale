//! A minimal MCP *client* over stdio — the mirror image of `mw-mcp` (the server
//! in `crates/mw-cli/src/bin/mw-mcp.rs`). Newline-delimited JSON-RPC 2.0 on the
//! child's stdin/stdout; only the three methods we need: `initialize`,
//! `tools/list`, `tools/call`.
//!
//! ponytail: no MCP client crate — the wire format is three JSON objects.
//! Swap in an official client if we ever need SSE/HTTP transports or sampling.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

pub struct McpClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
}

impl McpClient {
    /// Spawn `command args…` and complete the MCP handshake. Errors if the
    /// binary is missing or the server never answers `initialize`.
    pub fn spawn(command: &str, args: &[String]) -> Result<Self> {
        let mut child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| format!("failed to start MCP server `{command}`"))?;
        let stdin = child.stdin.take().expect("piped stdin");
        let stdout = BufReader::new(child.stdout.take().expect("piped stdout"));
        let mut client = Self {
            child,
            stdin,
            stdout,
            next_id: 0,
        };
        client
            .request(
                "initialize",
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {},
                    "clientInfo": {"name": "memorywhale", "version": env!("CARGO_PKG_VERSION")}
                }),
            )
            .with_context(|| format!("MCP handshake with `{command}` failed"))?;
        client.notify("notifications/initialized", json!({}))?;
        Ok(client)
    }

    pub fn list_tools(&mut self) -> Result<Value> {
        self.request("tools/list", json!({}))
    }

    /// `tools/call`, returning the concatenated text of the result content.
    pub fn call_tool(&mut self, name: &str, arguments: Value) -> Result<String> {
        let result = self.request("tools/call", json!({"name": name, "arguments": arguments}))?;
        let text: String = result
            .get("content")
            .and_then(Value::as_array)
            .map(|items| {
                items
                    .iter()
                    .filter_map(|c| c.get("text").and_then(Value::as_str))
                    .collect::<Vec<_>>()
                    .join("\n")
            })
            .unwrap_or_default();
        if text.is_empty() {
            return Err(anyhow!("MCP tool `{name}` returned no text content"));
        }
        Ok(text)
    }

    fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        let msg = json!({"jsonrpc": "2.0", "method": method, "params": params});
        writeln!(self.stdin, "{msg}")?;
        self.stdin.flush()?;
        Ok(())
    }

    fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        self.next_id += 1;
        let id = self.next_id;
        let msg = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        writeln!(self.stdin, "{msg}").context("MCP server closed its input")?;
        self.stdin.flush()?;

        // Skip notifications and stray lines until our id comes back.
        let mut line = String::new();
        loop {
            line.clear();
            let n = self
                .stdout
                .read_line(&mut line)
                .context("reading MCP server output")?;
            if n == 0 {
                return Err(anyhow!(
                    "MCP server exited without answering `{method}` (id {id})"
                ));
            }
            let Ok(msg) = serde_json::from_str::<Value>(line.trim()) else {
                continue;
            };
            if msg.get("id").and_then(Value::as_i64) != Some(id) {
                continue;
            }
            if let Some(err) = msg.get("error") {
                let text = err
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown error");
                return Err(anyhow!("MCP `{method}` failed: {text}"));
            }
            return Ok(msg.get("result").cloned().unwrap_or(json!({})));
        }
    }
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
