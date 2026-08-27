//! A minimal MCP *client* over stdio — the mirror image of `mw-mcp` (the server
//! in `crates/mw-cli/src/bin/mw-mcp.rs`). Newline-delimited JSON-RPC 2.0 on the
//! child's stdin/stdout; only discovery, legacy initialization, `tools/list`,
//! and `tools/call`.
//!
//! ponytail: no MCP client crate — the wire surface is deliberately small.
//! Swap in an official client if we ever need SSE/HTTP transports or sampling.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use anyhow::{anyhow, Context, Result};
use serde_json::{json, Value};

const CURRENT_PROTOCOL_VERSION: &str = "2026-07-28";
const LEGACY_PROTOCOL_VERSIONS: [&str; 2] = ["2025-11-25", "2024-11-05"];

#[derive(Clone, Copy)]
enum ProtocolEra {
    Modern,
    Legacy,
}

pub struct McpClient {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: i64,
    protocol: ProtocolEra,
}

impl McpClient {
    /// Spawn `command args…` and negotiate modern discovery or a legacy
    /// initialization handshake.
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
            protocol: ProtocolEra::Modern,
        };
        let discovery = client
            .request_raw("server/discover", modern_params(json!({})))
            .with_context(|| format!("MCP discovery with `{command}` failed"))?;
        if validate_discovery(&discovery)? {
            client.protocol = ProtocolEra::Modern;
        } else {
            client.protocol = ProtocolEra::Legacy;
            let initialized = client
                .request_raw(
                    "initialize",
                    json!({
                        "protocolVersion": LEGACY_PROTOCOL_VERSIONS[0],
                        "capabilities": {},
                        "clientInfo": {"name": "memorywhale", "version": env!("CARGO_PKG_VERSION")}
                    }),
                )
                .with_context(|| format!("MCP handshake with `{command}` failed"))?;
            let negotiated = initialized
                .pointer("/result/protocolVersion")
                .and_then(Value::as_str);
            if !negotiated.is_some_and(|version| LEGACY_PROTOCOL_VERSIONS.contains(&version)) {
                return Err(response_error("initialize", &initialized));
            }
            client.notify("notifications/initialized", json!({}))?;
        }
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
        let params = match self.protocol {
            ProtocolEra::Modern => modern_params(params),
            ProtocolEra::Legacy => params,
        };
        let response = self.request_raw(method, params)?;
        if response.get("error").is_some() {
            return Err(response_error(method, &response));
        }
        Ok(response.get("result").cloned().unwrap_or(json!({})))
    }

    fn request_raw(&mut self, method: &str, params: Value) -> Result<Value> {
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
            if msg.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
                return Err(anyhow!(
                    "MCP `{method}` returned a response without JSON-RPC 2.0"
                ));
            }
            return Ok(msg);
        }
    }
}

/// Returns false only for an unambiguously legacy response. A response that
/// uses modern discovery fields must satisfy the current DiscoverResult shape.
fn validate_discovery(response: &Value) -> Result<bool> {
    if response.get("error").is_some() {
        if response.pointer("/error/code").and_then(Value::as_i64) == Some(-32022) {
            return Err(response_error("server/discover", response));
        }
        return Ok(false);
    }
    let result = response
        .get("result")
        .and_then(Value::as_object)
        .ok_or_else(|| response_error("server/discover", response))?;
    // MemoryWhale versions predating discovery returned an empty success for
    // unknown methods instead of Method not found.
    if result.is_empty() {
        return Ok(false);
    }
    if result.get("resultType").and_then(Value::as_str) != Some("complete") {
        return Err(anyhow!(
            "MCP `server/discover` failed: missing resultType=complete"
        ));
    }
    let versions = result
        .get("supportedVersions")
        .and_then(Value::as_array)
        .ok_or_else(|| anyhow!("MCP `server/discover` failed: missing supportedVersions"))?;
    if !versions
        .iter()
        .any(|version| version.as_str() == Some(CURRENT_PROTOCOL_VERSION))
    {
        return Err(anyhow!(
            "MCP `server/discover` failed: protocol {CURRENT_PROTOCOL_VERSION} is unsupported"
        ));
    }
    if !result.get("capabilities").is_some_and(Value::is_object) {
        return Err(anyhow!(
            "MCP `server/discover` failed: missing capabilities"
        ));
    }
    Ok(true)
}

fn modern_params(mut params: Value) -> Value {
    if !params.is_object() {
        params = json!({});
    }
    params["_meta"] = json!({
        "io.modelcontextprotocol/protocolVersion": CURRENT_PROTOCOL_VERSION,
        "io.modelcontextprotocol/clientInfo": {
            "name": "memorywhale",
            "version": env!("CARGO_PKG_VERSION")
        },
        "io.modelcontextprotocol/clientCapabilities": {}
    });
    params
}

fn response_error(method: &str, response: &Value) -> anyhow::Error {
    let text = response
        .pointer("/error/message")
        .and_then(Value::as_str)
        .unwrap_or("unsupported or malformed response");
    anyhow!("MCP `{method}` failed: {text}")
}

impl Drop for McpClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn discovery_rejects_malformed_or_incompatible_modern_results() {
        let malformed = json!({"result": {
            "supportedVersions": [CURRENT_PROTOCOL_VERSION],
            "capabilities": {},
            "ttlMs": 1000,
            "cacheScope": "public"
        }});
        assert!(validate_discovery(&malformed)
            .unwrap_err()
            .to_string()
            .contains("resultType"));

        let incompatible = json!({"result": {
            "resultType": "complete",
            "supportedVersions": ["2099-01-01"],
            "capabilities": {},
            "ttlMs": 1000,
            "cacheScope": "public"
        }});
        assert!(validate_discovery(&incompatible)
            .unwrap_err()
            .to_string()
            .contains("unsupported"));
    }

    #[test]
    fn discovery_accepts_results_without_optional_cache_metadata() {
        let response = json!({"result": {
            "resultType": "complete",
            "supportedVersions": [CURRENT_PROTOCOL_VERSION],
            "capabilities": {}
        }});
        assert!(validate_discovery(&response).unwrap());
    }

    #[test]
    fn discovery_falls_back_only_for_legacy_responses() {
        assert!(!validate_discovery(&json!({"result": {}})).unwrap());
        assert!(!validate_discovery(&json!({
            "error": {"code": -32601, "message": "Method not found"}
        }))
        .unwrap());
        assert!(validate_discovery(&json!({
            "error": {"code": -32022, "message": "Unsupported protocol version"}
        }))
        .is_err());
    }
}
