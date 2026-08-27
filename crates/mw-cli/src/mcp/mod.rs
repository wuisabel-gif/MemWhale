//! Shared MCP protocol dispatch for `mw-mcp` (stdio) and `mw-serve` (`POST /mcp`).
//!
//! HTTP is one JSON-RPC object per POST (not an SSE session). Clients that
//! speak `2026-07-28` send `_meta` on each request. Clients that still
//! handshake (Rho `streamable_http`) send `initialize`, then `tools/list`
//! without `_meta`; HTTP accepts that path without keeping session state.

mod tools;

use serde_json::{json, Value};
use std::io::{BufRead, Write};
use tools::{call_tool, is_known_tool, tool_defs, TOOL_NAMES};

const CURRENT_PROTOCOL_VERSION: &str = "2026-07-28";
const LEGACY_PROTOCOL_VERSIONS: [&str; 2] = ["2025-11-25", "2024-11-05"];
const CACHE_TTL_MS: u64 = 3_600_000;

#[derive(Debug)]
struct RpcError {
    code: i64,
    message: String,
    data: Option<Value>,
}

impl RpcError {
    fn new(code: i64, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            data: None,
        }
    }
}

pub fn tool_names() -> Vec<String> {
    TOOL_NAMES.iter().map(|name| (*name).to_string()).collect()
}

/// JSON-RPC reply for one HTTP POST. `status` is an HTTP status line such as
/// `200 OK`. An empty `body` is a notification (`202`).
pub struct HttpMcpReply {
    pub status: &'static str,
    pub body: String,
}

/// One JSON-RPC object per POST. Modern `_meta` and a stateless initialize
/// handshake are both accepted.
pub fn handle_http_rpc(body: &str) -> HttpMcpReply {
    let msg: Value = match serde_json::from_str(body.trim()) {
        Ok(v) => v,
        Err(_) => {
            return HttpMcpReply {
                status: "400 Bad Request",
                body: error_response(Value::Null, RpcError::new(-32700, "Parse error")).to_string(),
            };
        }
    };
    if msg.is_array() {
        return HttpMcpReply {
            status: "400 Bad Request",
            body: error_response(
                Value::Null,
                RpcError::new(-32600, "JSON-RPC batches are not supported"),
            )
            .to_string(),
        };
    }
    match rpc_reply(&msg, dispatch_http) {
        None => HttpMcpReply {
            status: "202 Accepted",
            body: String::new(),
        },
        Some(reply) => HttpMcpReply {
            status: "200 OK",
            body: reply.to_string(),
        },
    }
}

struct StdioSession {
    initialized: bool,
    client_name: Option<String>,
}

pub fn serve_stdio<R: BufRead, W: Write>(input: R, mut output: W) {
    // Client name from the `initialize` handshake (e.g. "Claude Code"), used to
    // attribute agent-written memories. One process serves one client, so a
    // single mutable slot is enough.
    let mut session = StdioSession {
        initialized: false,
        client_name: None,
    };
    for line in input.lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => {
                write_error(
                    &mut output,
                    Value::Null,
                    RpcError::new(-32700, "Parse error"),
                );
                continue;
            }
        };
        if let Some(reply) = rpc_reply(&msg, |method, params| {
            dispatch_stdio(method, params, &mut session)
        }) {
            let _ = writeln!(output, "{reply}");
            let _ = output.flush();
        }
    }
}

fn rpc_reply(
    msg: &Value,
    dispatch: impl FnOnce(&str, &Value) -> Result<Value, RpcError>,
) -> Option<Value> {
    let Some(method) = msg.get("method").and_then(Value::as_str) else {
        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        return Some(error_response(
            id,
            RpcError::new(-32600, "Invalid JSON-RPC request"),
        ));
    };
    if msg.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
        let id = msg.get("id").cloned().unwrap_or(Value::Null);
        return Some(error_response(
            id,
            RpcError::new(-32600, "Invalid JSON-RPC request"),
        ));
    }
    let params = msg.get("params").cloned().unwrap_or(json!({}));
    // Notifications (no `id`) get no reply.
    let id = msg.get("id").cloned()?;
    Some(match dispatch(method, &params) {
        Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
        Err(error) => error_response(id, error),
    })
}

fn dispatch_http(method: &str, params: &Value) -> Result<Value, RpcError> {
    let params_object = require_params_object(params)?;
    if method == "initialize" {
        // HTTP is one POST at a time: the name cannot follow to later calls.
        let (result, _) = initialize_handshake(params_object)?;
        return Ok(result);
    }
    match classify_protocol(params)? {
        ProtocolClass::Modern { client_name } => handle(method, params, client_name, true),
        ProtocolClass::Legacy => handle(method, params, None, false),
    }
}

fn dispatch_stdio(
    method: &str,
    params: &Value,
    session: &mut StdioSession,
) -> Result<Value, RpcError> {
    let params_object = require_params_object(params)?;
    if method == "initialize" {
        let (result, client_name) = initialize_handshake(params_object)?;
        session.client_name = client_name;
        session.initialized = true;
        return Ok(result);
    }

    match classify_protocol(params)? {
        ProtocolClass::Modern { client_name } => handle(method, params, client_name, true),
        ProtocolClass::Legacy if session.initialized => {
            handle(method, params, session.client_name.as_deref(), false)
        }
        ProtocolClass::Legacy => Err(RpcError::new(
            -32602,
            "Request requires 2026-07-28 _meta or a legacy initialize handshake",
        )),
    }
}

fn require_params_object(params: &Value) -> Result<&serde_json::Map<String, Value>, RpcError> {
    params
        .as_object()
        .ok_or_else(|| RpcError::new(-32602, "Request params must be an object"))
}

fn initialize_handshake(
    params_object: &serde_json::Map<String, Value>,
) -> Result<(Value, Option<String>), RpcError> {
    let requested = params_object
        .get("protocolVersion")
        .and_then(Value::as_str)
        .ok_or_else(|| RpcError::new(-32602, "initialize requires protocolVersion"))?;
    let negotiated = if LEGACY_PROTOCOL_VERSIONS.contains(&requested) {
        requested
    } else {
        LEGACY_PROTOCOL_VERSIONS[0]
    };
    if params_object
        .get("capabilities")
        .is_some_and(|value| !value.is_object())
    {
        return Err(RpcError::new(
            -32602,
            "initialize capabilities must be an object",
        ));
    }
    if params_object
        .get("clientInfo")
        .is_some_and(|value| !value.is_object())
    {
        return Err(RpcError::new(
            -32602,
            "initialize clientInfo must be an object",
        ));
    }
    let client_name = params_object
        .get("clientInfo")
        .and_then(|client| client.get("name"))
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .map(str::to_string);
    Ok((
        json!({
            "protocolVersion": negotiated,
            "capabilities": {"tools": {"listChanged": false}},
            "serverInfo": server_info()
        }),
        client_name,
    ))
}

enum ProtocolClass<'a> {
    Legacy,
    Modern { client_name: Option<&'a str> },
}

fn classify_protocol(params: &Value) -> Result<ProtocolClass<'_>, RpcError> {
    let Some(requested) = requested_modern_protocol(params) else {
        return Ok(ProtocolClass::Legacy);
    };
    if requested != CURRENT_PROTOCOL_VERSION {
        return Err(unsupported_protocol(requested));
    }
    validate_modern_meta(params)?;
    Ok(ProtocolClass::Modern {
        client_name: modern_client_name(params),
    })
}

fn handle(
    method: &str,
    params: &Value,
    client_name: Option<&str>,
    modern: bool,
) -> Result<Value, RpcError> {
    match method {
        "server/discover" if modern => Ok(json!({
            "resultType": "complete",
            "supportedVersions": [
                CURRENT_PROTOCOL_VERSION,
                LEGACY_PROTOCOL_VERSIONS[0],
                LEGACY_PROTOCOL_VERSIONS[1]
            ],
            "capabilities": {"tools": {"listChanged": false}},
            "_meta": server_meta(),
            "instructions": "MemoryWhale provides local development memory retrieval and explicit note storage.",
            "ttlMs": CACHE_TTL_MS,
            "cacheScope": "public"
        })),
        "tools/list" if modern => Ok(json!({
            "resultType": "complete",
            "tools": tool_defs(),
            "_meta": server_meta(),
            "ttlMs": CACHE_TTL_MS,
            "cacheScope": "public"
        })),
        "tools/list" => Ok(json!({"tools": tool_defs()})),
        "ping" => Ok(json!({})),
        "tools/call" => {
            let name = params
                .get("name")
                .and_then(Value::as_str)
                .ok_or_else(|| RpcError::new(-32602, "tools/call requires a tool name"))?;
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            if !args.is_object() {
                return Err(RpcError::new(
                    -32602,
                    "tools/call arguments must be an object",
                ));
            }
            if !is_known_tool(name) {
                return Err(RpcError::new(-32602, format!("Unknown tool: {name}")));
            }
            match call_tool(name, &args, client_name) {
                Ok(text) if modern => Ok(json!({
                    "resultType": "complete",
                    "content": [{"type": "text", "text": text}],
                    "isError": false,
                    "_meta": server_meta()
                })),
                Ok(text) => Ok(json!({"content": [{"type": "text", "text": text}]})),
                Err(message) if modern => Ok(json!({
                    "resultType": "complete",
                    "content": [{"type": "text", "text": message}],
                    "isError": true,
                    "_meta": server_meta()
                })),
                Err(message) => Err(RpcError::new(-32603, message)),
            }
        }
        _ => Err(RpcError::new(-32601, format!("Method not found: {method}"))),
    }
}

fn requested_modern_protocol(params: &Value) -> Option<&str> {
    params
        .get("_meta")?
        .get("io.modelcontextprotocol/protocolVersion")?
        .as_str()
}

fn validate_modern_meta(params: &Value) -> Result<(), RpcError> {
    let meta = params
        .get("_meta")
        .and_then(Value::as_object)
        .ok_or_else(|| RpcError::new(-32602, "Request _meta must be an object"))?;
    if !meta
        .get("io.modelcontextprotocol/clientCapabilities")
        .is_some_and(Value::is_object)
    {
        return Err(RpcError::new(
            -32602,
            "Request _meta requires clientCapabilities object",
        ));
    }
    if meta
        .get("io.modelcontextprotocol/clientInfo")
        .is_some_and(|value| !value.is_object())
    {
        return Err(RpcError::new(
            -32602,
            "Request _meta clientInfo must be an object",
        ));
    }
    Ok(())
}

fn modern_client_name(params: &Value) -> Option<&str> {
    params
        .get("_meta")?
        .get("io.modelcontextprotocol/clientInfo")?
        .get("name")?
        .as_str()
        .filter(|name| !name.is_empty())
}

fn server_info() -> Value {
    json!({"name": "memorywhale", "version": env!("CARGO_PKG_VERSION")})
}

fn server_meta() -> Value {
    json!({"io.modelcontextprotocol/serverInfo": server_info()})
}

fn unsupported_protocol(requested: &str) -> RpcError {
    RpcError {
        code: -32022,
        message: "Unsupported protocol version".to_string(),
        data: Some(json!({
            "supported": [
                CURRENT_PROTOCOL_VERSION,
                LEGACY_PROTOCOL_VERSIONS[0],
                LEGACY_PROTOCOL_VERSIONS[1]
            ],
            "requested": requested
        })),
    }
}

fn error_response(id: Value, error: RpcError) -> Value {
    let mut body = json!({"code": error.code, "message": error.message});
    if let Some(data) = error.data {
        body["data"] = data;
    }
    json!({"jsonrpc": "2.0", "id": id, "error": body})
}

fn write_error<W: Write>(output: &mut W, id: Value, error: RpcError) {
    let _ = writeln!(output, "{}", error_response(id, error));
    let _ = output.flush();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn protocol_responses(lines: &[Value]) -> Vec<Value> {
        let mut input = lines
            .iter()
            .map(Value::to_string)
            .collect::<Vec<_>>()
            .join("\n");
        input.push('\n');
        let mut output = Vec::new();
        serve_stdio(Cursor::new(input), &mut output);
        String::from_utf8(output)
            .unwrap()
            .lines()
            .map(|line| serde_json::from_str(line).unwrap())
            .collect()
    }

    fn modern_params() -> Value {
        json!({"_meta": {
            "io.modelcontextprotocol/protocolVersion": CURRENT_PROTOCOL_VERSION,
            "io.modelcontextprotocol/clientInfo": {"name": "test", "version": "1"},
            "io.modelcontextprotocol/clientCapabilities": {"roots": {}}
        }})
    }

    #[test]
    fn current_protocol_discovers_capabilities() {
        let responses = protocol_responses(&[json!({
            "jsonrpc": "2.0", "id": 1, "method": "server/discover",
            "params": modern_params()
        })]);
        assert_eq!(responses[0]["result"]["resultType"], "complete");
        assert_eq!(
            responses[0]["result"]["supportedVersions"][0],
            CURRENT_PROTOCOL_VERSION
        );
        assert!(responses[0]["result"]["capabilities"]["tools"].is_object());
    }

    #[test]
    fn legacy_protocols_initialize_and_list_tools() {
        for protocol in LEGACY_PROTOCOL_VERSIONS {
            let responses = protocol_responses(&[
                json!({
                    "jsonrpc": "2.0", "id": 1, "method": "initialize",
                    "params": {
                        "protocolVersion": protocol,
                        "capabilities": {},
                        "clientInfo": {"name": "legacy-test", "version": "1"}
                    }
                }),
                json!({"jsonrpc": "2.0", "method": "notifications/initialized"}),
                json!({"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}}),
            ]);
            assert_eq!(responses[0]["result"]["protocolVersion"], protocol);
            assert_eq!(responses[1]["result"]["tools"].as_array().unwrap().len(), 6);
            assert!(responses[1]["result"].get("resultType").is_none());
        }
    }

    #[test]
    fn unsupported_protocol_returns_negotiation_error() {
        let mut params = modern_params();
        params["_meta"]["io.modelcontextprotocol/protocolVersion"] = json!("2099-01-01");
        let responses = protocol_responses(&[json!({
            "jsonrpc": "2.0", "id": 1, "method": "server/discover", "params": params
        })]);
        assert_eq!(responses[0]["error"]["code"], -32022);
        assert_eq!(responses[0]["error"]["data"]["requested"], "2099-01-01");
        assert_eq!(
            responses[0]["error"]["data"]["supported"][0],
            CURRENT_PROTOCOL_VERSION
        );
    }

    #[test]
    fn malformed_json_and_capabilities_return_json_rpc_errors() {
        let mut output = Vec::new();
        serve_stdio(Cursor::new("not-json\n"), &mut output);
        let parse: Value = serde_json::from_slice(&output).unwrap();
        assert_eq!(parse["id"], Value::Null);
        assert_eq!(parse["error"]["code"], -32700);

        let mut params = modern_params();
        params["_meta"]["io.modelcontextprotocol/clientCapabilities"] = json!("tools");
        let responses = protocol_responses(&[json!({
            "jsonrpc": "2.0", "id": 2, "method": "server/discover", "params": params
        })]);
        assert_eq!(responses[0]["error"]["code"], -32602);
        assert!(responses[0]["error"]["message"]
            .as_str()
            .unwrap()
            .contains("clientCapabilities"));
    }

    #[test]
    fn http_discovers_current_protocol() {
        let reply = handle_http_rpc(
            &json!({
                "jsonrpc": "2.0", "id": 1, "method": "server/discover",
                "params": modern_params()
            })
            .to_string(),
        );
        assert_eq!(reply.status, "200 OK");
        let body: Value = serde_json::from_str(&reply.body).unwrap();
        assert_eq!(body["result"]["resultType"], "complete");
    }

    #[test]
    fn http_accepts_rho_initialize_then_tools_list() {
        let init = handle_http_rpc(
            &json!({
                "jsonrpc": "2.0", "id": 1, "method": "initialize",
                "params": {
                    "protocolVersion": "2025-11-25",
                    "capabilities": {"roots": {"listChanged": false}},
                    "clientInfo": {"name": "rho", "version": "2.2.0"}
                }
            })
            .to_string(),
        );
        assert_eq!(init.status, "200 OK");
        let body: Value = serde_json::from_str(&init.body).unwrap();
        assert_eq!(body["result"]["protocolVersion"], "2025-11-25");
        assert!(body["result"]["capabilities"]["tools"].is_object());

        let initialized = handle_http_rpc(
            &json!({"jsonrpc": "2.0", "method": "notifications/initialized"}).to_string(),
        );
        assert_eq!(initialized.status, "202 Accepted");
        assert!(initialized.body.is_empty());

        let listed = handle_http_rpc(
            &json!({
                "jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}
            })
            .to_string(),
        );
        assert_eq!(listed.status, "200 OK");
        let body: Value = serde_json::from_str(&listed.body).unwrap();
        assert_eq!(body["result"]["tools"].as_array().unwrap().len(), 6);
        assert!(body["result"].get("resultType").is_none());

        let ping = handle_http_rpc(
            &json!({"jsonrpc": "2.0", "id": 3, "method": "ping", "params": {}}).to_string(),
        );
        assert_eq!(ping.status, "200 OK");
        let body: Value = serde_json::from_str(&ping.body).unwrap();
        assert_eq!(body["result"], json!({}));
    }
}
