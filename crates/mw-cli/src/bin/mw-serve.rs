// mw-serve: serve MemoryWhale's local memory as a web dashboard.
//
// Starts a small HTTP server (no external dependencies) that reads the local
// SQLite store and serves a browsable page of your previous command runs and
// recorded sessions. Designed for headless machines (e.g. a Jetson): run it on
// the machine that has the data, then open it from a laptop browser over the LAN
// at http://<machine-ip>:<port>/. Everything stays local; nothing is uploaded.
//
// Usage:
//   mw-serve                 serve on 127.0.0.1:7071
//   mw-serve --lan           serve on the LAN; mints serve.token if needed
//   mw-serve --lan --print-token   print the LAN token this process would use, then exit
//   mw-serve --port 8080     serve on a different port

use chrono::{DateTime, FixedOffset, Local, Utc};
use regex::Regex;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Serialize;
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

static STARTUP_NOTICE: OnceLock<String> = OnceLock::new();
// Optional shared token gating the dashboard. Empty = open (no auth).
static AUTH_TOKEN: OnceLock<String> = OnceLock::new();
/// Whether the server bound a loopback address (drives Host-header checks).
static LOOPBACK_BIND: OnceLock<bool> = OnceLock::new();
/// Whether the versioned JSON API is enabled for this server process.
static API_ENABLED: OnceLock<bool> = OnceLock::new();

const IO_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_CONNECTIONS: usize = 64;
const MAX_REQUEST_LINE_BYTES: usize = 8 * 1024;
const MAX_HEADER_LINE_BYTES: usize = 8 * 1024;
const MAX_HEADER_BYTES: usize = 32 * 1024;
const MAX_HEADER_COUNT: usize = 100;
const MAX_BODY_BYTES: usize = 4096;
/// MCP JSON-RPC body cap. Receipt: same as one captured text field
/// (`DEFAULT_MAX_CAPTURE_BYTES` = 1 MiB). A `remember` / `similar_failures`
/// argument cannot usefully exceed what the store will keep.
const MCP_MAX_BODY_BYTES: usize = memorywhale_core::privacy::DEFAULT_MAX_CAPTURE_BYTES;
static ACTIVE_CONNECTIONS: AtomicUsize = AtomicUsize::new(0);

struct ConnectionGuard;

impl ConnectionGuard {
    fn acquire() -> Option<Self> {
        ACTIVE_CONNECTIONS
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                (n < MAX_CONNECTIONS).then_some(n + 1)
            })
            .ok()
            .map(|_| Self)
    }
}

impl Drop for ConnectionGuard {
    fn drop(&mut self) {
        ACTIVE_CONNECTIONS.fetch_sub(1, Ordering::AcqRel);
    }
}

fn main() {
    if let Err(err) = run() {
        eprintln!("mw-serve: {err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let mut config = parse_server_args(std::env::args().skip(1))?;
    if config.help {
        println!(
            "mw-serve [--lan | --host <addr>] [--port <n>] [--token <secret>] [--print-token] [--api]  — serve memory locally"
        );
        return Ok(());
    }
    let mut token_source = None;
    if config.print_token {
        println!(
            "{}",
            memorywhale_cli::serve_auth::load_or_mint_serve_token(&config.token)?.value
        );
        return Ok(());
    }
    if !is_loopback_host(&config.host) && config.token.is_empty() {
        let loaded = memorywhale_cli::serve_auth::load_or_mint_serve_token("")?;
        token_source = Some(loaded.source);
        config.token = loaded.value;
    } else if !config.token.is_empty() {
        token_source = Some(memorywhale_cli::serve_auth::TokenSource::Explicit);
    }
    validate_server_config(&config)?;
    if !config.token.is_empty() {
        let _ = AUTH_TOKEN.set(config.token.clone());
    }
    let _ = LOOPBACK_BIND.set(is_loopback_host(&config.host));
    let _ = API_ENABLED.set(config.api);

    let db = database_path()?;

    // Self-heal: import any session transcripts whose recording was interrupted
    // before it could write its database row.
    match recover_orphans() {
        Ok(report) => {
            if report.recovered > 0 {
                println!(
                    "Recovered {} interrupted session(s) from transcripts.",
                    report.recovered
                );
            }
            if report.deleted_empty > 0 {
                println!(
                    "Removed {} empty 0-byte transcript(s).",
                    report.deleted_empty
                );
            }
            if report.recovered > 0 || report.deleted_empty > 0 {
                let mut parts = Vec::new();
                if report.recovered > 0 {
                    parts.push(format!(
                        "{} interrupted session(s) recovered",
                        report.recovered
                    ));
                }
                if report.deleted_empty > 0 {
                    parts.push(format!(
                        "{} empty 0-byte transcript(s) cleaned",
                        report.deleted_empty
                    ));
                }
                let _ = STARTUP_NOTICE.set(parts.join(" · "));
            }
        }
        Err(e) => eprintln!("mw-serve: recovery skipped: {e}"),
    }

    let listener = TcpListener::bind((config.host.as_str(), config.port))
        .map_err(|e| format!("failed to bind {}:{}: {e}", config.host, config.port))?;

    println!("MemoryWhale dashboard serving from {}", db.display());
    println!("  local:   http://localhost:{}/", config.port);
    println!("  mcp:     http://localhost:{}/mcp", config.port);
    if !is_loopback_host(&config.host) {
        println!(
            "  network: http://<this-machine-ip>:{}/  (find it with: hostname -I)",
            config.port
        );
        println!("  mcp:     http://<this-machine-ip>:{}/mcp", config.port);
    }
    if AUTH_TOKEN.get().is_some() {
        let from_file = matches!(
            token_source,
            Some(memorywhale_cli::serve_auth::TokenSource::File)
                | Some(memorywhale_cli::serve_auth::TokenSource::Minted)
        );
        match memorywhale_cli::serve_auth::serve_token_path() {
            Ok(path) if from_file => {
                println!(
                    "  auth:    token stored at {} — dashboard sign-in uses the raw token; MCP uses Authorization: Bearer …",
                    path.display()
                );
            }
            _ => {
                println!("  auth:    token required — enter it in the dashboard sign-in form; MCP uses Authorization: Bearer …");
            }
        }
    }
    println!("Press Ctrl-C to stop.");

    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                let _ = s.set_read_timeout(Some(IO_TIMEOUT));
                let _ = s.set_write_timeout(Some(IO_TIMEOUT));
                let Some(connections) = ConnectionGuard::acquire() else {
                    let _ = write_error(&s, "503 Service Unavailable");
                    continue;
                };
                std::thread::spawn(move || {
                    let _connections = connections;
                    handle(s);
                });
            }
            Err(e) => eprintln!("mw-serve: connection error: {e}"),
        }
    }
    Ok(())
}

#[derive(Debug, PartialEq, Eq)]
struct ServerConfig {
    host: String,
    port: u16,
    token: String,
    help: bool,
    print_token: bool,
    api: bool,
}

fn parse_server_args<I>(args: I) -> Result<ServerConfig, String>
where
    I: IntoIterator<Item = String>,
{
    let mut host = "127.0.0.1".to_string();
    let mut port = 7071;
    let mut token = std::env::var("MEMORYWHALE_TOKEN").unwrap_or_default();
    let mut help = false;
    let mut print_token = false;
    let mut api = false;
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--help" | "-h" => {
                help = true;
                break;
            }
            "--lan" => host = "0.0.0.0".to_string(),
            "--api" => api = true,
            "--host" => host = args.next().ok_or("--host needs an address")?,
            "--port" => {
                port = args
                    .next()
                    .and_then(|v| v.parse().ok())
                    .ok_or("--port needs a number")?;
            }
            "--token" => token = args.next().unwrap_or_default(),
            "--print-token" => print_token = true,
            other => return Err(format!("unknown option {other:?}; run mw-serve --help")),
        }
    }
    Ok(ServerConfig {
        host,
        port,
        token,
        help,
        print_token,
        api,
    })
}

fn request_path(raw_path: &str) -> &str {
    raw_path.split('?').next().unwrap_or(raw_path)
}

fn bearer_token(authorization: &str) -> Option<&str> {
    let value = authorization.trim();
    let (scheme, token) = value.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("bearer") {
        return None;
    }
    Some(token.trim()).filter(|token| !token.is_empty())
}

fn is_loopback_host(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "::1" | "localhost")
}

/// True when the request's Host header matches the address the server bound.
/// Loopback binds accept only loopback hostnames (with optional port), which
/// blocks DNS-rebinding attacks: a rebound public hostname cannot make the
/// browser-origin request resolve to this server under an attacker's origin.
/// Non-loopback binds always require a token, so host checking is not needed
/// there — the token gate protects the data.
fn host_header_allowed(host_header: &str, loopback_bind: bool) -> bool {
    if !loopback_bind {
        return true;
    }
    let host = host_header.trim();
    // Strip an optional :port (careful with [::1]:port bracket form).
    let hostname = if let Some(rest) = host.strip_prefix('[') {
        match rest.split_once(']') {
            Some((name, _tail)) => name.to_string(),
            None => return false,
        }
    } else {
        // Split on the LAST colon: separates a trailing :port from a hostname.
        // A bare (unbracketed) IPv6 literal is invalid in a Host header.
        match host.rsplit_once(':') {
            Some((name, port)) if !port.is_empty() && port.chars().all(|c| c.is_ascii_digit()) => {
                name.to_string()
            }
            _ => host.to_string(),
        }
    };
    matches!(
        hostname.to_ascii_lowercase().as_str(),
        "localhost" | "127.0.0.1" | "::1"
    )
}

/// Constant-time equality for secret comparison (avoids timing oracles on the
/// shared token). Both length and byte differences fold into one result.
fn ct_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    let mut diff = (a.len() ^ b.len()) as u8;
    for i in 0..a.len().max(b.len()) {
        let x = a.get(i).copied().unwrap_or(0);
        let y = b.get(i).copied().unwrap_or(0);
        diff |= x ^ y;
    }
    diff == 0
}

/// Security headers appended to every response (normal, error, auth).
const SECURITY_HEADERS: &str = "X-Content-Type-Options: nosniff\r\n\
Referrer-Policy: no-referrer\r\n\
Cache-Control: no-store\r\n\
Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'; form-action 'self'; frame-ancestors 'none'\r\n";

/// Build a full HTTP response with the security headers applied.
fn response(status: &str, body: &str, extra_headers: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\n{SECURITY_HEADERS}{extra_headers}Connection: close\r\n\r\n",
        body.len()
    )
}

fn json_http(status: &str, body: &str, extra_headers: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\n{SECURITY_HEADERS}{extra_headers}Connection: close\r\n\r\n{body}",
        body.len()
    )
}

fn json_response(status: &str, body: &str, extra_headers: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\n{SECURITY_HEADERS}{extra_headers}Connection: close\r\n\r\n",
        body.len()
    )
}

const API_VERSION: &str = "v1";
const API_DEFAULT_LIMIT: usize = 20;
const API_MAX_LIMIT: usize = 50;

fn redact_json(value: &mut Value) {
    match value {
        Value::String(text) => *text = redact_secrets(text),
        Value::Array(values) => values.iter_mut().for_each(redact_json),
        Value::Object(values) => values.values_mut().for_each(redact_json),
        Value::Null | Value::Bool(_) | Value::Number(_) => {}
    }
}

fn api_envelope(mut data: Value) -> String {
    redact_json(&mut data);
    serde_json::to_string(&json!({ "api_version": API_VERSION, "data": data }))
        .expect("JSON API envelope is serializable")
}

fn api_error(status: &'static str, code: &str, message: &str) -> (&'static str, String) {
    (
        status,
        serde_json::to_string(&json!({
            "api_version": API_VERSION,
            "error": {"code": code, "message": redact_secrets(message)}
        }))
        .expect("JSON API error is serializable"),
    )
}

fn api_http_error(status: &'static str) -> (&'static str, String) {
    let (code, message) = match status {
        "408 Request Timeout" => ("request_timeout", "request timed out"),
        "413 Payload Too Large" => ("body_too_large", "request body is too large"),
        "431 Request Header Fields Too Large" => {
            ("headers_too_large", "request headers are too large")
        }
        _ => ("bad_request", "request could not be parsed"),
    };
    api_error(status, code, message)
}

fn api_route(raw_path: &str) -> (&'static str, String) {
    let path = raw_path.split('?').next().unwrap_or("/");
    match path {
        "/api/v1" | "/api/v1/" => {
            api_error("404 Not Found", "not_found", "use a versioned API endpoint")
        }
        "/api/v1/health" => api_health(),
        "/api/v1/search" => api_search(raw_path),
        "/api/v1/repositories" => api_repositories(),
        "/api/v1/sessions" => api_sessions(raw_path),
        _ if path.starts_with("/api/v1/memories/") => {
            api_memory(path.trim_start_matches("/api/v1/memories/"))
        }
        _ if path.starts_with("/api/v1/commands/") => {
            api_command(path.trim_start_matches("/api/v1/commands/"))
        }
        _ => api_error("404 Not Found", "not_found", "unknown API endpoint"),
    }
}

fn api_open() -> Result<Connection, (&'static str, String)> {
    open_db().map_err(|error| api_error("500 Internal Server Error", "database", &error))
}

fn api_health() -> (&'static str, String) {
    let conn = match api_open() {
        Ok(conn) => conn,
        Err(response) => return response,
    };
    let memory_count = match memorywhale_core::sqlite::load_memories(&conn) {
        Ok(memories) => memories.len(),
        Err(error) => {
            return api_error("503 Service Unavailable", "memory_load", &error.to_string())
        }
    };
    (
        "200 OK",
        api_envelope(json!({
            "status": "ok",
            "version": env!("CARGO_PKG_VERSION"),
            "memory_count": memory_count
        })),
    )
}

fn api_limit(raw_path: &str) -> Result<usize, (&'static str, String)> {
    let Some(value) = query_param(raw_path, "limit") else {
        return Ok(API_DEFAULT_LIMIT);
    };
    let limit = value.parse::<usize>().map_err(|_| {
        api_error(
            "400 Bad Request",
            "invalid_limit",
            "limit must be a positive integer",
        )
    })?;
    if !(1..=API_MAX_LIMIT).contains(&limit) {
        return Err(api_error(
            "400 Bad Request",
            "invalid_limit",
            "limit must be between 1 and 50",
        ));
    }
    Ok(limit)
}

fn api_search(raw_path: &str) -> (&'static str, String) {
    let query = query_param(raw_path, "q").unwrap_or_default();
    if query.trim().is_empty() {
        return api_error("400 Bad Request", "missing_query", "q is required");
    }
    let limit = match api_limit(raw_path) {
        Ok(limit) => limit,
        Err(response) => return response,
    };
    let conn = match api_open() {
        Ok(conn) => conn,
        Err(response) => return response,
    };
    let memories = match memorywhale_core::sqlite::load_memories(&conn) {
        Ok(memories) => memories,
        Err(error) => {
            return api_error("503 Service Unavailable", "memory_load", &error.to_string())
        }
    };
    let engine = memorywhale_core::engine::BuiltinEngine::new(memories);
    let hits = memorywhale_core::engine::MemoryEngine::retrieve(
        &engine,
        &memorywhale_core::Query::new(&query, Utc::now()),
        limit,
    );
    let results: Vec<Value> = hits
        .into_iter()
        .map(|hit| {
            let reasons = hit.reasons();
            let (source, source_id) = memorywhale_core::sqlite::decode_id(hit.memory.id);
            json!({
                "id": hit.memory.id,
                "source": source.tag(),
                "source_id": source_id,
                "command_id": (source == memorywhale_core::sqlite::Source::Command)
                    .then_some(source_id),
                "score": hit.score,
                "memory": hit.memory,
                "signals": hit.signals,
                "reasons": reasons
            })
        })
        .collect();
    (
        "200 OK",
        api_envelope(json!({"query": query, "results": results})),
    )
}

fn api_memory(raw_id: &str) -> (&'static str, String) {
    let Ok(id) = percent_decode(raw_id).parse::<i64>() else {
        return api_error(
            "400 Bad Request",
            "invalid_id",
            "memory id must be an integer",
        );
    };
    let conn = match api_open() {
        Ok(conn) => conn,
        Err(response) => return response,
    };
    let memories = match memorywhale_core::sqlite::load_memories(&conn) {
        Ok(memories) => memories,
        Err(error) => {
            return api_error("503 Service Unavailable", "memory_load", &error.to_string())
        }
    };
    match memories.into_iter().find(|memory| memory.id == id) {
        Some(memory) => ("200 OK", api_envelope(json!({"memory": memory}))),
        None => api_error("404 Not Found", "not_found", "memory was not found"),
    }
}

type ApiCommandRow = (
    String,
    String,
    Option<String>,
    Option<i64>,
    String,
    String,
    String,
    String,
);

fn api_command(raw_id: &str) -> (&'static str, String) {
    let Ok(id) = percent_decode(raw_id).parse::<i64>() else {
        return api_error(
            "400 Bad Request",
            "invalid_id",
            "command id must be an integer",
        );
    };
    let conn = match api_open() {
        Ok(conn) => conn,
        Err(response) => return response,
    };
    let row: Option<ApiCommandRow> = match conn
        .query_row(
            "SELECT command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at
                 FROM command_runs WHERE id = ?1",
            params![id],
            |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                    row.get(5)?,
                    row.get(6)?,
                    row.get(7)?,
                ))
            },
        )
        .optional()
    {
        Ok(row) => row,
        Err(error) => {
            return api_error("500 Internal Server Error", "database", &error.to_string())
        }
    };
    let Some((command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at)) = row else {
        return api_error("404 Not Found", "not_found", "command was not found");
    };
    let argv = serde_json::from_str::<Value>(&argv_json).unwrap_or(Value::Null);
    (
        "200 OK",
        api_envelope(json!({
            "id": id,
            "command": command,
            "argv": argv,
            "cwd": cwd,
            "exit_code": exit_code,
            "stdout": stdout,
            "stderr": stderr,
            "notes": notes,
            "created_at": created_at
        })),
    )
}

fn api_sessions(raw_path: &str) -> (&'static str, String) {
    let limit = match api_limit(raw_path) {
        Ok(limit) => limit,
        Err(response) => return response,
    };
    let conn = match api_open() {
        Ok(conn) => conn,
        Err(response) => return response,
    };
    let mut stmt = match conn.prepare(
        "SELECT id, started_at, ended_at, status, byte_count, notes, cwd
         FROM sessions ORDER BY id DESC LIMIT ?1",
    ) {
        Ok(stmt) => stmt,
        Err(error) => {
            return api_error("500 Internal Server Error", "database", &error.to_string())
        }
    };
    let rows = match stmt.query_map(params![limit as i64], |row| {
        Ok(json!({
            "id": row.get::<_, i64>(0)?,
            "started_at": row.get::<_, String>(1)?,
            "ended_at": row.get::<_, String>(2)?,
            "status": row.get::<_, String>(3)?,
            "byte_count": row.get::<_, i64>(4)?,
            "notes": row.get::<_, String>(5)?,
            "cwd": row.get::<_, Option<String>>(6)?
        }))
    }) {
        Ok(rows) => rows,
        Err(error) => {
            return api_error("500 Internal Server Error", "database", &error.to_string())
        }
    };
    let mut sessions = Vec::new();
    for row in rows {
        match row {
            Ok(row) => sessions.push(row),
            Err(error) => {
                return api_error("500 Internal Server Error", "database", &error.to_string())
            }
        }
    }
    ("200 OK", api_envelope(json!({"sessions": sessions})))
}

fn api_repositories() -> (&'static str, String) {
    let conn = match api_open() {
        Ok(conn) => conn,
        Err(response) => return response,
    };
    let mut stmt = match conn.prepare(
        "SELECT repository_id, repository_name, worktree_root FROM command_runs
         WHERE repository_id IS NOT NULL
         UNION
         SELECT repository_id, repository_name, worktree_root FROM sessions
         WHERE repository_id IS NOT NULL
         ORDER BY repository_name, worktree_root",
    ) {
        Ok(stmt) => stmt,
        Err(error) => {
            return api_error("500 Internal Server Error", "database", &error.to_string())
        }
    };
    let rows = match stmt.query_map([], |row| {
        Ok(json!({
            "id": row.get::<_, String>(0)?,
            "name": row.get::<_, Option<String>>(1)?,
            "worktree_root": row.get::<_, Option<String>>(2)?
        }))
    }) {
        Ok(rows) => rows,
        Err(error) => {
            return api_error("500 Internal Server Error", "database", &error.to_string())
        }
    };
    let mut repositories = Vec::new();
    for row in rows {
        match row {
            Ok(row) => repositories.push(row),
            Err(error) => {
                return api_error("500 Internal Server Error", "database", &error.to_string())
            }
        }
    }
    (
        "200 OK",
        api_envelope(json!({"repositories": repositories})),
    )
}

fn handle_mcp(stream: &mut TcpStream, method: &str, authorization: &str, body: &str) {
    if method != "POST" {
        let response = json_http(
            "405 Method Not Allowed",
            "{\"error\":\"method not allowed\"}",
            "Allow: POST\r\n",
        );
        let _ = stream.write_all(response.as_bytes());
        return;
    }
    if let Some(want) = AUTH_TOKEN.get() {
        let supplied = bearer_token(authorization).unwrap_or("");
        if !ct_eq(supplied, want) {
            let response = json_http(
                "401 Unauthorized",
                "{\"error\":\"unauthorized\"}",
                "WWW-Authenticate: Bearer\r\n",
            );
            let _ = stream.write_all(response.as_bytes());
            return;
        }
    }
    let reply = memorywhale_cli::mcp::handle_http_rpc(body);
    if reply.body.is_empty() {
        let response = format!(
            "HTTP/1.1 {}\r\nContent-Length: 0\r\n{SECURITY_HEADERS}Connection: close\r\n\r\n",
            reply.status
        );
        let _ = stream.write_all(response.as_bytes());
        return;
    }
    let response = json_http(reply.status, &reply.body, "");
    let _ = stream.write_all(response.as_bytes());
}

fn validate_server_config(config: &ServerConfig) -> Result<(), String> {
    if !config.help && !is_loopback_host(&config.host) && config.token.is_empty() {
        return Err(
            "refusing unauthenticated non-loopback bind; set MEMORYWHALE_TOKEN or --token"
                .to_string(),
        );
    }
    Ok(())
}

struct HttpMessage {
    cookie: String,
    host_header: String,
    authorization: String,
    body: String,
}

fn host_ok(host_header: &str) -> bool {
    let loopback_bind = *LOOPBACK_BIND.get().unwrap_or(&true);
    host_header_allowed(host_header, loopback_bind)
}

fn read_http_message<R: BufRead>(
    reader: &mut R,
    max_body: usize,
) -> Result<HttpMessage, &'static str> {
    let mut cookie = String::new();
    let mut host_header = String::new();
    let mut authorization = String::new();
    let mut content_length = 0usize;
    let mut saw_content_length = false;
    let mut header_bytes = 0usize;
    let mut header_count = 0usize;
    loop {
        let line = match read_limited_line(reader, MAX_HEADER_LINE_BYTES) {
            Ok(Some(line)) => line,
            Ok(None) => return Err("400 Bad Request"),
            Err(error) => return Err(error.status()),
        };
        header_bytes = match header_bytes.checked_add(line.len()) {
            Some(n) if n <= MAX_HEADER_BYTES => n,
            _ => return Err("431 Request Header Fields Too Large"),
        };
        if line == "\r\n" || line == "\n" || line.is_empty() {
            break;
        }
        header_count += 1;
        if header_count > MAX_HEADER_COUNT {
            return Err("431 Request Header Fields Too Large");
        }
        let Some((header_name, header_value)) = line.trim_end_matches(['\r', '\n']).split_once(':')
        else {
            return Err("400 Bad Request");
        };
        if header_name.eq_ignore_ascii_case("cookie") {
            cookie = header_value.trim().to_string();
        }
        if header_name.eq_ignore_ascii_case("host") {
            host_header = header_value.trim().to_string();
        }
        if header_name.eq_ignore_ascii_case("authorization") {
            authorization = header_value.trim().to_string();
        }
        if header_name.eq_ignore_ascii_case("content-length") {
            if saw_content_length {
                return Err("400 Bad Request");
            }
            saw_content_length = true;
            content_length = match parse_content_length(header_value.trim()) {
                Ok(n) if n <= max_body => n,
                Err(_) => return Err("400 Bad Request"),
                _ => return Err("413 Payload Too Large"),
            };
        }
        if header_name.eq_ignore_ascii_case("transfer-encoding") {
            return Err("400 Bad Request");
        }
    }
    let mut request_body = vec![0; content_length];
    if let Err(error) = reader.read_exact(&mut request_body) {
        let status = if matches!(
            error.kind(),
            std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
        ) {
            "408 Request Timeout"
        } else {
            "400 Bad Request"
        };
        return Err(status);
    }
    Ok(HttpMessage {
        cookie,
        host_header,
        authorization,
        body: String::from_utf8_lossy(&request_body).into_owned(),
    })
}

fn handle(mut stream: TcpStream) {
    let read_stream = match stream.try_clone() {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut reader = BufReader::new(read_stream);
    let request_line = match read_limited_line(&mut reader, MAX_REQUEST_LINE_BYTES) {
        Ok(Some(line)) => line,
        Ok(None) => return,
        Err(error) => {
            let _ = write_error(&stream, error.status());
            return;
        }
    };
    let (method, raw_path) = match parse_request_line(&request_line) {
        Ok(parts) => parts,
        Err(_) => {
            let _ = write_error(&stream, "400 Bad Request");
            return;
        }
    };
    if request_path(&raw_path) == "/mcp" {
        serve_mcp(&mut stream, &mut reader, &method);
        return;
    }
    serve_dashboard(&mut stream, &mut reader, method, raw_path);
}

fn serve_mcp<R: BufRead>(stream: &mut TcpStream, reader: &mut R, method: &str) {
    let msg = match read_http_message(reader, MCP_MAX_BODY_BYTES) {
        Ok(msg) => msg,
        Err(status) => {
            let _ = write_error(stream, status);
            return;
        }
    };
    if !host_ok(&msg.host_header) {
        let _ = write_error(stream, "403 Forbidden");
        return;
    }
    handle_mcp(stream, method, &msg.authorization, &msg.body);
}

fn serve_dashboard<R: BufRead>(
    stream: &mut TcpStream,
    reader: &mut R,
    method: String,
    raw_path: String,
) {
    let path = request_path(&raw_path);
    let is_api_path = path == "/api/v1" || path.starts_with("/api/v1/");
    let msg = match read_http_message(reader, MAX_BODY_BYTES) {
        Ok(msg) => msg,
        Err(status) => {
            if is_api_path {
                let (status, body) = api_http_error(status);
                let response = json_response(status, &body, "");
                let _ = stream.write_all(response.as_bytes());
                if method != "HEAD" {
                    let _ = stream.write_all(body.as_bytes());
                }
            } else {
                let _ = write_error(stream, status);
            }
            return;
        }
    };
    // DNS-rebinding protection: on loopback binds, only loopback Host names
    // may reach the dashboard. A rebound attacker hostname gets 403.
    if !host_ok(&msg.host_header) {
        if is_api_path {
            let (status, body) = api_error("403 Forbidden", "forbidden", "host is not allowed");
            let response = json_response(status, &body, "");
            let _ = stream.write_all(response.as_bytes());
            if method != "HEAD" {
                let _ = stream.write_all(body.as_bytes());
            }
        } else {
            let _ = write_error(stream, "403 Forbidden");
        }
        return;
    }

    let cookie = msg.cookie;
    let request_body = msg.body;
    let is_head = method == "HEAD";
    let method_allowed = method == "GET" || is_head || (method == "POST" && path == "/login");
    if !method_allowed {
        let allow = if path == "/login" {
            "GET, HEAD, POST"
        } else {
            "GET, HEAD"
        };
        let response = if is_api_path {
            let (status, body) = api_error(
                "405 Method Not Allowed",
                "method_not_allowed",
                "the JSON API accepts GET and HEAD",
            );
            json_response(status, &body, &format!("Allow: {allow}\r\n"))
        } else {
            response("405 Method Not Allowed", "", &format!("Allow: {allow}\r\n"))
        };
        let _ = stream.write_all(response.as_bytes());
        if is_api_path && !is_head {
            let (_, body) = api_error(
                "405 Method Not Allowed",
                "method_not_allowed",
                "the JSON API accepts GET and HEAD",
            );
            let _ = stream.write_all(body.as_bytes());
        }
        return;
    }

    let mut cookies: Vec<String> = Vec::new();

    // Display timezone: `?tz=` selects it (and remembers it in a cookie);
    // otherwise fall back to the cookie, else the server's local time.
    let cookie_tz = cookie
        .split(';')
        .find_map(|c| c.trim().strip_prefix("mw_tz=").map(str::to_string))
        .filter(|tz| cookie_value_is_safe(tz));
    match query_param(&raw_path, "tz").filter(|tz| cookie_value_is_safe(tz)) {
        Some(tz) => {
            set_display_tz(parse_tz(&tz));
            if let Some(c) = set_cookie("mw_tz", &tz, "; Path=/; SameSite=Strict; Max-Age=31536000")
            {
                cookies.push(c);
            }
        }
        None => set_display_tz(
            cookie_tz
                .as_deref()
                .map(parse_tz)
                .unwrap_or(DisplayTz::Local),
        ),
    }

    // Optional shared-token gate. Sign in with a POST body so the token never
    // appears in browser history, server logs, or copied dashboard URLs.
    if let Some(want) = AUTH_TOKEN.get() {
        let via_cookie = cookie
            .split(';')
            .filter_map(|c| c.trim().strip_prefix("mw_token="))
            .any(|v| ct_eq(v, want));
        let via_bearer =
            is_api_path && bearer_token(&msg.authorization).is_some_and(|token| ct_eq(token, want));
        let login_attempt = method == "POST" && raw_path == "/login";
        let supplied = form_param(&request_body, "token");
        if login_attempt && supplied.as_deref().is_some_and(|s| ct_eq(s, want)) {
            if let Some(c) = set_cookie("mw_token", want, "; Path=/; HttpOnly; SameSite=Strict") {
                cookies.push(c);
            }
            let Some(token_cookie) = cookies.last() else {
                return;
            };
            let response = format!(
                "HTTP/1.1 303 See Other\r\nLocation: /\r\nSet-Cookie: {token_cookie}\r\n{SECURITY_HEADERS}Connection: close\r\n\r\n"
            );
            let _ = stream.write_all(response.as_bytes());
            return;
        } else if !via_cookie && !via_bearer {
            if is_api_path {
                let (status, body) = api_error(
                    "401 Unauthorized",
                    "unauthorized",
                    "authentication required",
                );
                let response = json_response(status, &body, "WWW-Authenticate: Bearer\r\n");
                let _ = stream.write_all(response.as_bytes());
                if !is_head {
                    let _ = stream.write_all(body.as_bytes());
                }
                return;
            }
            let message = if login_attempt {
                "<p>That token was not accepted.</p>"
            } else {
                "<p>This dashboard requires a token.</p>"
            };
            let body = page(
                "Sign in",
                &format!(
                    "{message}<form method=\"post\" action=\"/login\"><label>Shared token <input type=\"password\" name=\"token\" autocomplete=\"current-password\" required></label> <button type=\"submit\">Sign in</button></form>"
                ),
            );
            let response = response("401 Unauthorized", &body, "");
            let _ = stream.write_all(response.as_bytes());
            if !is_head {
                let _ = stream.write_all(body.as_bytes());
            }
            return;
        }
    }

    let cookie_header: String = cookies
        .iter()
        .map(|c| format!("Set-Cookie: {c}\r\n"))
        .collect();
    if is_api_path {
        let (status, body) = if *API_ENABLED.get().unwrap_or(&false) {
            api_route(&raw_path)
        } else {
            api_error("404 Not Found", "api_disabled", "the JSON API is disabled")
        };
        let response = json_response(status, &body, &cookie_header);
        let _ = stream.write_all(response.as_bytes());
        if !is_head {
            let _ = stream.write_all(body.as_bytes());
        }
        return;
    }
    let (status, body) = route(&raw_path);
    let response = response(status, &body, &cookie_header);
    let _ = stream.write_all(response.as_bytes());
    if !is_head {
        let _ = stream.write_all(body.as_bytes());
    }
}

#[derive(Debug)]
enum LineError {
    Io(std::io::Error),
    TooLong,
    InvalidUtf8,
    Unterminated,
}

impl LineError {
    fn status(&self) -> &'static str {
        match self {
            Self::Io(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) =>
            {
                "408 Request Timeout"
            }
            Self::TooLong => "431 Request Header Fields Too Large",
            Self::Io(_) | Self::InvalidUtf8 | Self::Unterminated => "400 Bad Request",
        }
    }
}

fn read_limited_line<R: BufRead>(
    reader: &mut R,
    limit: usize,
) -> Result<Option<String>, LineError> {
    let mut bytes = Vec::new();
    let read = reader
        .take((limit + 1) as u64)
        .read_until(b'\n', &mut bytes)
        .map_err(LineError::Io)?;
    if bytes.len() > limit {
        return Err(LineError::TooLong);
    }
    if read == 0 {
        return Ok(None);
    }
    if bytes.last() != Some(&b'\n') {
        return Err(LineError::Unterminated);
    }
    String::from_utf8(bytes)
        .map(Some)
        .map_err(|_| LineError::InvalidUtf8)
}

fn write_error(stream: &TcpStream, status: &str) -> std::io::Result<()> {
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Length: 0\r\n{SECURITY_HEADERS}Connection: close\r\n\r\n"
    );
    stream.try_clone()?.write_all(response.as_bytes())
}

fn parse_request_line(line: &str) -> Result<(String, String), ()> {
    let mut parts = line.split_whitespace();
    let method = parts.next().ok_or(())?;
    let path = parts.next().ok_or(())?;
    let version = parts.next().ok_or(())?;
    if parts.next().is_some() || version != "HTTP/1.1" || method.is_empty() || path.is_empty() {
        return Err(());
    }
    Ok((method.to_string(), path.to_string()))
}

fn parse_content_length(value: &str) -> Result<usize, ()> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(());
    }
    value.parse::<usize>().map_err(|_| ())
}

fn form_param(body: &str, key: &str) -> Option<String> {
    query_param(&format!("?{body}"), key)
}

fn route(raw_path: &str) -> (&'static str, String) {
    let path = raw_path.split('?').next().unwrap_or("/");
    if path == "/" {
        return ("200 OK", dashboard(raw_path));
    }
    if path == "/graph" {
        return ("200 OK", graph_page());
    }
    if let Some(rest) = path.strip_prefix("/project/") {
        return ("200 OK", project_page(rest));
    }
    if let Some(rest) = path.strip_prefix("/repo/") {
        return (
            "200 OK",
            repo_page(
                rest,
                query_param(raw_path, "worktree"),
                &query_param(raw_path, "q").unwrap_or_default(),
            ),
        );
    }
    if let Some(rest) = path.strip_prefix("/runs/") {
        return ("200 OK", runs_page(rest));
    }
    if path == "/favicon.ico" {
        return ("204 No Content", String::new());
    }
    if let Some(rest) = path.strip_prefix("/command/") {
        if let Ok(id) = rest.parse::<i64>() {
            return match command_page(id) {
                Ok(html) => ("200 OK", html),
                Err(e) => (
                    "404 Not Found",
                    page("Not found", &format!("<p>{}</p>", esc(&e))),
                ),
            };
        }
    }
    if let Some(rest) = path.strip_prefix("/session/") {
        if let Ok(id) = rest.parse::<i64>() {
            return match session_page(id) {
                Ok(html) => ("200 OK", html),
                Err(e) => (
                    "404 Not Found",
                    page("Not found", &format!("<p>{}</p>", esc(&e))),
                ),
            };
        }
    }
    (
        "404 Not Found",
        page(
            "Not found",
            "<p>Nothing here. <a href=\"/\">Back to dashboard</a></p>",
        ),
    )
}

fn dashboard(raw_path: &str) -> String {
    let query = query_param(raw_path, "q").unwrap_or_default();
    let distinguish_worktrees = query_param(raw_path, "worktrees").as_deref() == Some("1");
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => {
            return page(
                "MemoryWhale",
                &format!("<p>Could not open database: {}</p>", esc(&e)),
            )
        }
    };

    let mut body =
        String::from("<div class=\"eyebrow\">MemoryWhale</div>\n<h1>Terminal memory</h1>\n");
    if let Some(notice) = STARTUP_NOTICE.get() {
        body.push_str(&format!("<div class=\"notice\">{}</div>\n", esc(notice)));
    }
    body.push_str("<p class=\"sub\">Your previous commands and recorded sessions, served locally. <a class=\"glink\" href=\"/graph\">open graph view →</a></p>\n");
    body.push_str(&format!(
        "<form class=\"search\" method=\"get\" action=\"/\"><input name=\"q\" value=\"{}\" placeholder=\"Search commands, logs, notes, sessions, cwd, tags\"/><button type=\"submit\">Search</button></form>\n",
        esc(&query)
    ));
    body.push_str(&tz_selector());

    if !query.trim().is_empty() {
        body.push_str(&search_results(&conn, &query));
    }

    let repos = repo_counts(&conn, distinguish_worktrees);
    if !repos.is_empty() {
        let mut entries: Vec<(&RepoKey, &i64)> = repos.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1).then(a.0.name.cmp(&b.0.name)));
        body.push_str("<h2>Repos</h2>\n");
        body.push_str(if distinguish_worktrees {
            "<p class=\"sub\">Grouped by worktree. <a href=\"/\">Group linked worktrees together</a></p>\n"
        } else {
            "<p class=\"sub\">Grouped by canonical repository. <a href=\"/?worktrees=1\">Distinguish worktrees</a></p>\n"
        });
        body.push_str("<div class=\"chips\">\n");
        for (repo, n) in entries {
            let mut href = format!("/repo/{}", percent_encode(&repo.id));
            let label = if let Some(worktree) = &repo.worktree {
                let leaf = Path::new(worktree)
                    .file_name()
                    .map(|name| name.to_string_lossy().into_owned())
                    .unwrap_or_else(|| worktree.clone());
                href.push_str(&format!("?worktree={}", percent_encode(worktree)));
                format!("{} · {}", repo.name, leaf)
            } else {
                repo.name.clone()
            };
            body.push_str(&format!(
                "<a class=\"chip\" href=\"{}\" title=\"{}\">{} <span>{}</span></a>\n",
                esc(&href),
                esc(&repo.id),
                esc(&label),
                n
            ));
        }
        body.push_str("</div>\n");
    }

    body.push_str("<h2>Command runs</h2>\n");
    let mut rows = 0;
    let mut cur_date = String::new();
    let mut group = String::new();
    let mut gcount = 0usize;
    let mut first_group = true;
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, command, argv_json, exit_code, created_at, notes FROM command_runs ORDER BY created_at DESC, id DESC LIMIT 200",
    ) {
        if let Ok(iter) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        }) {
            for row in iter.flatten() {
                let (id, cmd, argv_json, code, at, notes) = row;
                let day = fmt_date(&at);
                if day != cur_date {
                    if !cur_date.is_empty() {
                        push_day_group(&mut body, &cur_date, &group, gcount, first_group);
                        first_group = false;
                        group.clear();
                        gcount = 0;
                    }
                    cur_date = day.to_string();
                }
                let full = full_command(&argv_json, &cmd);
                let ok = code == Some(0);
                group.push_str(&format!(
                    "<a class=\"row\" href=\"/command/{id}\"><span class=\"badge {}\">{}</span>\
                     <span class=\"cmd\">{}</span><span class=\"when\">{}</span><span class=\"note\">{}</span></a>\n",
                    if ok { "ok" } else { "bad" },
                    match code { Some(c) => format!("exit {c}"), None => "—".into() },
                    esc_redacted(&full),
                    esc(&fmt_time(&at)),
                    esc_redacted(&notes)
                ));
                gcount += 1;
                rows += 1;
            }
        }
    }
    if !cur_date.is_empty() {
        push_day_group(&mut body, &cur_date, &group, gcount, first_group);
    }
    if rows == 0 {
        body.push_str("<div class=\"list\"><p class=\"empty\">No command runs yet. Record one with <code>mw-remember</code>.</p></div>\n");
    }

    body.push_str("<h2>Sessions</h2>\n");
    let mut srows = 0;
    let mut cur_date = String::new();
    let mut group = String::new();
    let mut gcount = 0usize;
    let mut first_group = true;
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, started_at, ended_at, byte_count, notes, status FROM sessions ORDER BY started_at DESC, id DESC LIMIT 200",
    ) {
        if let Ok(iter) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        }) {
            for row in iter.flatten() {
                let (id, at, ended_at, bytes, notes, status) = row;
                let day = fmt_date(&at);
                if day != cur_date {
                    if !cur_date.is_empty() {
                        push_day_group(&mut body, &cur_date, &group, gcount, first_group);
                        first_group = false;
                        group.clear();
                        gcount = 0;
                    }
                    cur_date = day.to_string();
                }
                group.push_str(&session_row(id, &at, &ended_at, bytes, &notes, &status));
                gcount += 1;
                srows += 1;
            }
        }
    }
    if !cur_date.is_empty() {
        push_day_group(&mut body, &cur_date, &group, gcount, first_group);
    }
    if srows == 0 {
        body.push_str("<div class=\"list\"><p class=\"empty\">No sessions yet. Record one with <code>mw</code>.</p></div>\n");
    }

    body.push_str("<h2>Bookmarks</h2>\n<div class=\"list\">\n");
    let mut brows = 0;
    if let Ok(mut stmt) =
        conn.prepare("SELECT id, label, cwd, created_at FROM bookmarks ORDER BY id DESC LIMIT 80")
    {
        if let Ok(iter) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
            ))
        }) {
            for (id, label, cwd, created_at) in iter.flatten() {
                body.push_str(&format!(
                    "<div class=\"row\"><span class=\"badge sess\">mark</span><span class=\"cmd\">#{id}</span><span class=\"when\">{}</span><span class=\"note\">{} {}</span></div>\n",
                    esc(&fmt_datetime(&created_at)),
                    esc_redacted(&label),
                    cwd.map(|c| format!("· {}", esc_redacted(&c))).unwrap_or_default()
                ));
                brows += 1;
            }
        }
    }
    if brows == 0 {
        body.push_str("<p class=\"empty\">No bookmarks yet. Mark one with <code>mw mark \"important moment\"</code>.</p>\n");
    }
    body.push_str("</div>\n");

    body.push_str(&integrations_section());

    page("MemoryWhale — terminal memory", &body)
}

/// Supported client integrations, mirroring the capability matrix in
/// `integrations/README.md`. Keep the two in sync when adding a client.
/// (display name, matrix slug for the guide link, MCP support badge)
const INTEGRATIONS: &[(&str, &str, &str)] = &[
    ("Claude Code", "claude-code", "MCP · capture"),
    ("Claude Desktop", "claude-desktop", "MCP"),
    ("Cline", "cline", "MCP"),
    ("CodeWhale", "codewhale", "MCP"),
    ("Codex CLI", "codex", "MCP"),
    ("Continue", "continue", "MCP"),
    ("CrowClaw", "crowclaw", "MCP"),
    ("Cursor", "cursor", "MCP"),
    ("Gemini CLI", "gemini-cli", "MCP"),
    ("Goose", "goose", "MCP"),
    ("Hermes Agent", "hermes", "MCP"),
    ("Jan Desktop", "jan", "MCP"),
    ("OpenClaw", "openclaw", "MCP"),
    ("OpenCode", "opencode", "MCP"),
    ("Pi coding agent", "pi", "unverified"),
    ("Rho", "rho", "MCP · capture"),
    ("VS Code / Copilot", "vscode", "MCP"),
    ("Windsurf", "windsurf", "MCP"),
    ("Zed", "zed", "MCP"),
    ("Pullfrog", "pullfrog", "PR workflow"),
    ("CLIProxyAPI", "cliproxyapi", "model proxy"),
    ("OpenRouter", "openrouter", "model gateway"),
    ("Neovim plugin", "neovim", "CLI"),
    ("Any stdio MCP client", "generic-mcp", "MCP"),
];

/// The integration-support grid on the dashboard: one cell per client with a
/// letter-mark icon and its MCP support level, linking to the setup guide.
fn integrations_section() -> String {
    let mut cells = String::new();
    for (name, slug, badge) in INTEGRATIONS {
        // Letter-mark icon: first alnum characters of the display name.
        let mark: String = name
            .chars()
            .filter(|c| c.is_ascii_alphanumeric())
            .take(2)
            .collect::<String>()
            .to_uppercase();
        cells.push_str(&format!(
            "<a class=\"icell\" href=\"https://github.com/wuisabel-gif/MemWhale/tree/main/integrations/{slug}\" title=\"{name} setup guide\">\
<span class=\"imark\" aria-hidden=\"true\">{mark}</span><span class=\"iname\">{}</span><span class=\"ibadge{}\">{badge}</span></a>\n",
            esc(name),
            if *badge == "unverified" { " off" } else { "" },
        ));
    }
    format!(
        "<h2>Integrations</h2>\n<p class=\"sub\">MemoryWhale works alongside these coding agents, editors, and model-routing tools — \
each cell links to its setup guide in the repository.</p>\n<div class=\"igrid\">\n{cells}</div>\n"
    )
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct RepoKey {
    id: String,
    name: String,
    worktree: Option<String>,
}

fn repo_key(
    id: Option<String>,
    name: Option<String>,
    worktree: Option<String>,
    distinguish_worktrees: bool,
) -> Option<RepoKey> {
    Some(RepoKey {
        id: id?,
        name: name?,
        worktree: if distinguish_worktrees {
            Some(worktree?)
        } else {
            None
        },
    })
}

/// Unique persisted worktree roots. Unlike the old cwd-based discovery, these
/// remain useful after a worktree is deleted or the dashboard moves machines.
fn discovered_repositories(conn: &Connection) -> Vec<RepoKey> {
    let mut seen = HashSet::new();
    for sql in [
        "SELECT repository_id, repository_name, worktree_root FROM command_runs",
        "SELECT repository_id, repository_name, worktree_root FROM sessions",
    ] {
        if let Ok(mut stmt) = conn.prepare(sql) {
            if let Ok(rows) = stmt.query_map([], |row| {
                Ok(repo_key(row.get(0)?, row.get(1)?, row.get(2)?, true))
            }) {
                seen.extend(rows.flatten().flatten());
            }
        }
    }
    seen.into_iter().collect()
}

fn session_repos(
    own: Option<RepoKey>,
    transcript: &str,
    roots: &[RepoKey],
    distinguish_worktrees: bool,
) -> HashSet<RepoKey> {
    let canonical = |mut repo: RepoKey| {
        if !distinguish_worktrees {
            repo.worktree = None;
        }
        repo
    };
    let mut repos = HashSet::new();
    if let Some(repo) = own {
        repos.insert(canonical(repo));
    }
    for repo in roots {
        if repo
            .worktree
            .as_deref()
            .is_some_and(|root| transcript.contains(root))
        {
            repos.insert(canonical(repo.clone()));
        }
    }
    repos
}

/// Command-runs + sessions per repository (a session can count under several).
fn repo_counts(conn: &Connection, distinguish_worktrees: bool) -> HashMap<RepoKey, i64> {
    let mut counts = HashMap::new();
    if let Ok(mut stmt) =
        conn.prepare("SELECT repository_id, repository_name, worktree_root FROM command_runs")
    {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok(repo_key(
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
                distinguish_worktrees,
            ))
        }) {
            for repo in rows.flatten().flatten() {
                *counts.entry(repo).or_insert(0) += 1;
            }
        }
    }
    let roots = discovered_repositories(conn);
    if let Ok(mut stmt) = conn
        .prepare("SELECT repository_id, repository_name, worktree_root, transcript FROM sessions")
    {
        if let Ok(rows) = stmt.query_map([], |row| {
            Ok((
                repo_key(row.get(0)?, row.get(1)?, row.get(2)?, true),
                row.get::<_, String>(3)?,
            ))
        }) {
            for (own, transcript) in rows.flatten() {
                for repo in session_repos(own, &transcript, &roots, distinguish_worktrees) {
                    *counts.entry(repo).or_insert(0) += 1;
                }
            }
        }
    }
    counts
}

fn project_of(notes: &str) -> Option<String> {
    let re = Regex::new(r"project:([\w.\-]+)").ok()?;
    re.captures(notes).map(|c| c[1].to_string())
}

// Count how many command runs + sessions belong to each project tag.
fn search_results(conn: &Connection, query: &str) -> String {
    let needle = format!("%{}%", query.trim());
    let mut out = String::new();
    let mut rows = 0;
    out.push_str("<h2>Search results</h2>\n<div class=\"list\">\n");

    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, command, argv_json, exit_code, created_at, notes, stdout, stderr
         FROM command_runs
         WHERE command LIKE ?1 OR argv_json LIKE ?1 OR IFNULL(cwd, '') LIKE ?1
            OR stdout LIKE ?1 OR stderr LIKE ?1 OR notes LIKE ?1
         ORDER BY id DESC LIMIT 40",
    ) {
        if let Ok(iter) = stmt.query_map(params![needle.as_str()], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
                r.get::<_, String>(7)?,
            ))
        }) {
            for row in iter.flatten() {
                let (id, cmd, argv_json, code, at, notes, stdout, stderr) = row;
                let ok = code == Some(0);
                let tags = error_tags(&format!("{stdout}\n{stderr}"));
                out.push_str(&format!(
                    "<a class=\"row\" href=\"/command/{id}\"><span class=\"badge {}\">{}</span>\
                     <span class=\"cmd\">{}</span><span class=\"when\">{}</span><span class=\"note\">{} {}</span></a>\n",
                    if ok { "ok" } else { "bad" },
                    match code { Some(c) => format!("exit {c}"), None => "—".into() },
                    esc_redacted(&full_command(&argv_json, &cmd)),
                    esc(&fmt_datetime(&at)),
                    tag_pills(&tags),
                    esc_redacted(&notes)
                ));
                rows += 1;
            }
        }
    }

    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, started_at, ended_at, byte_count, notes, status, transcript
         FROM sessions
         WHERE IFNULL(shell, '') LIKE ?1 OR IFNULL(cwd, '') LIKE ?1 OR transcript LIKE ?1
            OR notes LIKE ?1 OR started_at LIKE ?1 OR status LIKE ?1
         ORDER BY id DESC LIMIT 40",
    ) {
        if let Ok(iter) = stmt.query_map(params![needle.as_str()], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, String>(6)?,
            ))
        }) {
            for row in iter.flatten() {
                let (id, started_at, ended_at, bytes, notes, status, transcript) = row;
                let tags = error_tags(&transcript);
                let mut row_html = session_row(id, &started_at, &ended_at, bytes, &notes, &status);
                if !tags.is_empty() {
                    row_html = row_html
                        .replace("</span></a>", &format!(" {}</span></a>", tag_pills(&tags)));
                }
                out.push_str(&row_html);
                rows += 1;
            }
        }
    }

    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, label, cwd, created_at FROM bookmarks
         WHERE label LIKE ?1 OR IFNULL(cwd, '') LIKE ?1 OR created_at LIKE ?1
         ORDER BY id DESC LIMIT 40",
    ) {
        if let Ok(iter) = stmt.query_map(params![needle.as_str()], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
                r.get::<_, String>(3)?,
            ))
        }) {
            for (id, label, cwd, created_at) in iter.flatten() {
                out.push_str(&format!(
                    "<div class=\"row\"><span class=\"badge sess\">mark</span><span class=\"cmd\">#{id}</span><span class=\"when\">{}</span><span class=\"note\">{} {}</span></div>\n",
                    esc(&fmt_datetime(&created_at)),
                    esc_redacted(&label),
                    cwd.map(|c| format!("· {}", esc_redacted(&c))).unwrap_or_default()
                ));
                rows += 1;
            }
        }
    }

    if rows == 0 {
        out.push_str("<p class=\"empty\">No matching terminal memory found.</p>\n");
    }
    out.push_str("</div>\n");
    out
}

/// Timezone the dashboard renders timestamps in, per-request (thread-local so it
/// doesn't have to be threaded through every render function).
#[derive(Clone, Copy)]
enum DisplayTz {
    Local,
    Fixed(FixedOffset),
}

thread_local! {
    static DISPLAY_TZ: std::cell::Cell<DisplayTz> = const { std::cell::Cell::new(DisplayTz::Local) };
}

fn set_display_tz(tz: DisplayTz) {
    DISPLAY_TZ.with(|c| c.set(tz));
}
fn cur_tz() -> DisplayTz {
    DISPLAY_TZ.with(|c| c.get())
}

/// Parse a tz selection: "local", "utc", or a `±HH:MM` / `±HHMM` / `±H` offset.
fn parse_tz(s: &str) -> DisplayTz {
    let s = s.trim();
    if s.is_empty() || s.eq_ignore_ascii_case("local") {
        return DisplayTz::Local;
    }
    if s.eq_ignore_ascii_case("utc") {
        return DisplayTz::Fixed(FixedOffset::east_opt(0).unwrap());
    }
    let sign = match s.as_bytes().first() {
        Some(b'+') => 1,
        Some(b'-') => -1,
        _ => return DisplayTz::Local,
    };
    let rest = &s[1..];
    let (h, m) = if let Some((h, m)) = rest.split_once(':') {
        (h.parse::<i32>().ok(), m.parse::<i32>().ok())
    } else if rest.len() == 4 && rest.is_ascii() {
        (rest[..2].parse().ok(), rest[2..].parse().ok())
    } else {
        (rest.parse::<i32>().ok(), Some(0))
    };
    match (h, m) {
        (Some(h), Some(m)) if (0..24).contains(&h) && (0..60).contains(&m) => {
            FixedOffset::east_opt(sign * (h * 3600 + m * 60))
                .map(DisplayTz::Fixed)
                .unwrap_or(DisplayTz::Local)
        }
        _ => DisplayTz::Local,
    }
}

fn cookie_value_is_safe(s: &str) -> bool {
    !s.bytes().any(|b| b.is_ascii_control())
}

/// Build a Set-Cookie value only if both name and value are free of control
/// characters. This prevents response splitting/header injection through
/// cookie values.
fn set_cookie(name: &str, value: &str, attrs: &str) -> Option<String> {
    if !cookie_value_is_safe(name) || !cookie_value_is_safe(value) {
        return None;
    }
    Some(format!("{name}={value}{attrs}"))
}

fn parse_ts(ts: &str) -> Option<DateTime<FixedOffset>> {
    DateTime::parse_from_rfc3339(ts).ok()
}

/// Format a stored (UTC) timestamp in the request's display timezone.
fn fmt_with(ts: &str, pattern: &str, fallback: &str) -> String {
    match parse_ts(ts) {
        Some(dt) => match cur_tz() {
            DisplayTz::Local => dt.with_timezone(&Local).format(pattern).to_string(),
            DisplayTz::Fixed(o) => dt.with_timezone(&o).format(pattern).to_string(),
        },
        None => fallback.to_string(),
    }
}

/// Local calendar date `YYYY-MM-DD` (for grouping headers).
fn fmt_date(ts: &str) -> String {
    fmt_with(ts, "%Y-%m-%d", date_of(ts))
}
/// Local `YYYY-MM-DD HH:MM` for row timestamps.
fn fmt_datetime(ts: &str) -> String {
    fmt_with(ts, "%Y-%m-%d %H:%M", ts)
}
/// Local `HH:MM:SS` for rows shown under a date header.
fn fmt_time(ts: &str) -> String {
    fmt_with(ts, "%H:%M:%S", time_of(ts))
}

/// The value string for the active tz (matches a selector option).
fn cur_tz_value() -> String {
    match cur_tz() {
        DisplayTz::Local => "local".to_string(),
        DisplayTz::Fixed(o) if o.local_minus_utc() == 0 => "utc".to_string(),
        DisplayTz::Fixed(o) => {
            let secs = o.local_minus_utc();
            let sign = if secs < 0 { '-' } else { '+' };
            let a = secs.abs();
            format!("{sign}{:02}:{:02}", a / 3600, (a % 3600) / 60)
        }
    }
}

/// A no-JS timezone picker: pick a zone and hit Set. "Local" tracks your
/// computer's clock (with daylight saving); the rest are fixed UTC offsets.
fn tz_selector() -> String {
    const OPTS: &[(&str, &str)] = &[
        ("local", "Local (your computer)"),
        ("utc", "UTC"),
        ("-10:00", "UTC-10:00 · Hawaii"),
        ("-09:00", "UTC-09:00 · Alaska"),
        ("-08:00", "UTC-08:00 · US Pacific (PST)"),
        ("-07:00", "UTC-07:00 · US Pacific (PDT) / Mountain"),
        ("-06:00", "UTC-06:00 · US Central"),
        ("-05:00", "UTC-05:00 · US Eastern"),
        ("-03:00", "UTC-03:00"),
        ("+01:00", "UTC+01:00 · Central Europe"),
        ("+02:00", "UTC+02:00"),
        ("+03:00", "UTC+03:00"),
        ("+05:30", "UTC+05:30 · India"),
        ("+08:00", "UTC+08:00 · China / Singapore"),
        ("+09:00", "UTC+09:00 · Japan / Korea"),
        ("+10:00", "UTC+10:00 · Sydney"),
    ];
    let cur = cur_tz_value();
    let mut s = String::from(
        "<form class=\"tzbar\" method=\"get\" action=\"/\"><label for=\"tz\">times shown in</label> <select id=\"tz\" name=\"tz\">",
    );
    for (v, l) in OPTS {
        let sel = if *v == cur { " selected" } else { "" };
        s.push_str(&format!("<option value=\"{v}\"{sel}>{}</option>", esc(l)));
    }
    s.push_str("</select> <button type=\"submit\">Set</button></form>");
    s
}

/// The `YYYY-MM-DD` part of an RFC3339 timestamp, used to group rows by day.
/// Falls back to the whole string if it isn't a recognizable date.
fn date_of(ts: &str) -> &str {
    if ts.len() >= 10 && ts.as_bytes().get(4) == Some(&b'-') {
        &ts[..10]
    } else {
        ts
    }
}

/// The `HH:MM:SS` part of an RFC3339 timestamp (whole string if not one).
fn time_of(ts: &str) -> &str {
    if ts.len() >= 19 && ts.as_bytes().get(10) == Some(&b'T') {
        &ts[11..19]
    } else {
        ts
    }
}

/// Emit one collapsible day group: a `<details>` whose summary is the date and
/// count, wrapping the already-rendered rows. `open` expands it by default.
fn push_day_group(body: &mut String, date: &str, rows_html: &str, count: usize, open: bool) {
    body.push_str(&format!(
        "<details class=\"daygroup\"{}><summary class=\"datehead\">{} \
         <span class=\"gcount\">{}</span></summary>\n<div class=\"list\">\n{}</div>\n</details>\n",
        if open { " open" } else { "" },
        esc(date),
        count,
        rows_html
    ));
}

fn session_row(
    id: i64,
    started_at: &str,
    ended_at: &str,
    bytes: i64,
    notes: &str,
    status: &str,
) -> String {
    let badge_class = match status {
        "recording" if session_age_seconds(ended_at).is_some_and(|age| age <= 30) => "live",
        "recording" | "interrupted" => "warn",
        _ => "sess",
    };
    let label = session_label(status, ended_at);
    let badge_text = match badge_class {
        "live" => "live",
        "warn" => "interrupted",
        _ => "session",
    };
    format!(
        "<a class=\"row\" href=\"/session/{id}\"><span class=\"badge {badge_class}\">{}</span>\
         <span class=\"cmd\">#{id}</span><span class=\"when\">{}</span><span class=\"note\">{} · {bytes} bytes</span></a>\n",
        esc(badge_text),
        esc(&fmt_datetime(started_at)),
        esc_redacted(&format!("{label} · {notes}"))
    )
}

fn session_label(status: &str, ended_at: &str) -> String {
    match status {
        "recording" => match session_age_seconds(ended_at) {
            Some(age) if age <= 30 => format!("Recording now · last autosaved {age}s ago"),
            Some(age) => format!("Interrupted or stale · last autosaved {}", human_age(age)),
            None => "Recording now".to_string(),
        },
        "interrupted" => "Recovered interrupted".to_string(),
        _ => "session".to_string(),
    }
}

fn session_age_seconds(ended_at: &str) -> Option<i64> {
    DateTime::parse_from_rfc3339(ended_at)
        .ok()
        .map(|dt| (Utc::now() - dt.with_timezone(&Utc)).num_seconds().max(0))
}

fn human_age(seconds: i64) -> String {
    if seconds < 60 {
        format!("{seconds}s ago")
    } else if seconds < 3600 {
        format!("{}m ago", seconds / 60)
    } else {
        format!("{}h ago", seconds / 3600)
    }
}

/// A combined, time-ordered view of every command run and session tagged with a
/// given project — even if they were recorded in different terminals.
fn project_page(raw_name: &str) -> String {
    let name = raw_name.trim_end_matches('/').to_string();
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => return page("Project", &format!("<p>{}</p>", esc(&e))),
    };

    let mut items: Vec<(String, String)> = Vec::new(); // (timestamp, row html)

    if let Ok(mut stmt) = conn
        .prepare("SELECT id, command, argv_json, exit_code, created_at, notes FROM command_runs")
    {
        if let Ok(it) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        }) {
            for (id, cmd, argv_json, code, at, notes) in it.flatten() {
                if project_of(&notes).as_deref() != Some(name.as_str()) {
                    continue;
                }
                let ok = code == Some(0);
                let row = format!(
                    "<a class=\"row\" href=\"/command/{id}\"><span class=\"badge {}\">{}</span>\
                     <span class=\"cmd\">{}</span><span class=\"when\">{}</span><span class=\"note\">{}</span></a>",
                    if ok { "ok" } else { "bad" },
                    match code { Some(c) => format!("exit {c}"), None => "—".into() },
                    esc_redacted(&full_command(&argv_json, &cmd)), esc(&fmt_datetime(&at)), esc_redacted(&notes)
                );
                items.push((at, row));
            }
        }
    }

    if let Ok(mut stmt) =
        conn.prepare("SELECT id, started_at, ended_at, byte_count, notes, status FROM sessions")
    {
        if let Ok(it) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        }) {
            for (id, at, ended_at, bytes, notes, status) in it.flatten() {
                if project_of(&notes).as_deref() != Some(name.as_str()) {
                    continue;
                }
                let row = session_row(id, &at, &ended_at, bytes, &notes, &status);
                items.push((at, row));
            }
        }
    }

    items.sort_by(|a, b| b.0.cmp(&a.0)); // newest first

    let mut body = String::from("<a class=\"back\" href=\"/\">← all memory</a>\n");
    body.push_str(&format!(
        "<div class=\"eyebrow\">project</div>\n<h1>{}</h1>\n",
        esc(&name)
    ));
    body.push_str(&format!(
        "<p class=\"sub\">{} memory item(s) across all terminals, newest first.</p>\n",
        items.len()
    ));
    body.push_str("<div class=\"list\">\n");
    if items.is_empty() {
        body.push_str("<p class=\"empty\">No memory tagged <code>project:");
        body.push_str(&esc(&name));
        body.push_str("</code> yet. Record with <code>--notes \"project:");
        body.push_str(&esc(&name));
        body.push_str("\"</code>.</p>");
    }
    for (_, row) in items {
        body.push_str(&row);
        body.push('\n');
    }
    body.push_str("</div>\n");
    page(&format!("{} · MemoryWhale", name), &body)
}

/// Everything that happened in a canonical repository, optionally narrowed to
/// one worktree, newest first.
fn repo_page(raw_id: &str, worktree: Option<String>, query: &str) -> String {
    let repository_id = percent_decode(raw_id.trim_end_matches('/'));
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => return page("Repo", &format!("<p>{}</p>", esc(&e))),
    };
    let roots = discovered_repositories(&conn);
    let name = roots
        .iter()
        .find(|repo| repo.id == repository_id)
        .map(|repo| repo.name.clone())
        .unwrap_or_else(|| repository_id.clone());

    let mut items: Vec<(String, String)> = Vec::new(); // (timestamp, row html)

    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, command, argv_json, exit_code, created_at, notes, cwd,
                repository_id, repository_name, worktree_root
         FROM command_runs",
    ) {
        if let Ok(it) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, Option<i64>>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, Option<String>>(7)?,
                r.get::<_, Option<String>>(8)?,
                r.get::<_, Option<String>>(9)?,
            ))
        }) {
            for (id, cmd, argv_json, code, at, notes, cwd, repo_id, _, worktree_root) in
                it.flatten()
            {
                if repo_id.as_deref() != Some(repository_id.as_str())
                    || worktree
                        .as_deref()
                        .is_some_and(|wanted| worktree_root.as_deref() != Some(wanted))
                    || !matches_repo_query(
                        query,
                        &[&cmd, &argv_json, &notes, cwd.as_deref().unwrap_or("")],
                    )
                {
                    continue;
                }
                let ok = code == Some(0);
                let row = format!(
                    "<a class=\"row\" href=\"/command/{id}\"><span class=\"badge {}\">{}</span>\
                     <span class=\"cmd\">{}</span><span class=\"when\">{}</span><span class=\"note\">{}</span></a>",
                    if ok { "ok" } else { "bad" },
                    match code { Some(c) => format!("exit {c}"), None => "—".into() },
                    esc_redacted(&full_command(&argv_json, &cmd)), esc(&fmt_datetime(&at)), esc_redacted(&notes)
                );
                items.push((at, row));
            }
        }
    }

    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, started_at, ended_at, byte_count, notes, status, cwd, transcript,
                repository_id, repository_name, worktree_root
         FROM sessions",
    ) {
        if let Ok(it) = stmt.query_map([], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, i64>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
                r.get::<_, Option<String>>(6)?,
                r.get::<_, String>(7)?,
                r.get::<_, Option<String>>(8)?,
                r.get::<_, Option<String>>(9)?,
                r.get::<_, Option<String>>(10)?,
            ))
        }) {
            for (
                id,
                at,
                ended_at,
                bytes,
                notes,
                status,
                cwd,
                transcript,
                repo_id,
                repo_name,
                worktree_root,
            ) in it.flatten()
            {
                let own = repo_key(repo_id, repo_name, worktree_root, true);
                let belongs = session_repos(own, &transcript, &roots, worktree.is_some())
                    .into_iter()
                    .any(|repo| {
                        repo.id == repository_id
                            && worktree
                                .as_deref()
                                .is_none_or(|wanted| repo.worktree.as_deref() == Some(wanted))
                    });
                if !belongs
                    || !matches_repo_query(
                        query,
                        &[&notes, &transcript, cwd.as_deref().unwrap_or("")],
                    )
                {
                    continue;
                }
                items.push((
                    at.clone(),
                    session_row(id, &at, &ended_at, bytes, &notes, &status),
                ));
            }
        }
    }

    items.sort_by(|a, b| b.0.cmp(&a.0)); // newest first

    let mut body = String::from("<a class=\"back\" href=\"/\">← all memory</a>\n");
    body.push_str(&format!(
        "<div class=\"eyebrow\">repo</div>\n<h1>{}</h1>\n",
        esc(&name),
    ));
    if let Some(worktree) = &worktree {
        body.push_str(&format!(
            "<p class=\"sub\">Worktree: <code>{}</code> · <a href=\"/repo/{}\">all worktrees</a></p>\n",
            esc(worktree),
            percent_encode(&repository_id),
        ));
    }
    body.push_str(&format!(
        "<form class=\"search\" method=\"get\" action=\"/repo/{}\">",
        percent_encode(&repository_id)
    ));
    if let Some(worktree) = &worktree {
        body.push_str(&format!(
            "<input type=\"hidden\" name=\"worktree\" value=\"{}\"/>",
            esc(worktree)
        ));
    }
    body.push_str(&format!(
        "<input name=\"q\" value=\"{}\" placeholder=\"Search this repository\"/><button type=\"submit\">Search</button></form>\n",
        esc(query)
    ));
    body.push_str(&format!(
        "<p class=\"sub\">{} matching memory item(s), newest first. A session that also touched another repo appears under that one too.</p>\n",
        items.len()
    ));
    body.push_str("<div class=\"list\">\n");
    if items.is_empty() {
        body.push_str(
            "<p class=\"empty\">Nothing recorded in a working directory under this repo yet.</p>",
        );
    }
    for (_, row) in items {
        body.push_str(&row);
        body.push('\n');
    }
    body.push_str("</div>\n");
    page(&format!("{} · MemoryWhale", name), &body)
}

fn matches_repo_query(query: &str, values: &[&str]) -> bool {
    let query = query.trim().to_ascii_lowercase();
    query.is_empty()
        || values
            .iter()
            .any(|value| value.to_ascii_lowercase().contains(&query))
}

fn command_page(id: i64) -> Result<String, String> {
    let conn = open_db()?;
    let row = conn
        .query_row(
            "SELECT command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at
             FROM command_runs WHERE id = ?1",
            params![id],
            |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, Option<String>>(2)?,
                    r.get::<_, Option<i64>>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                ))
            },
        )
        .optional()
        .map_err(|e| format!("read command run: {e}"))?
        .ok_or_else(|| format!("no command run #{id}"))?;
    let (command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at) = row;
    let argv: Vec<String> =
        serde_json::from_str(&argv_json).unwrap_or_else(|_| vec![command.clone()]);
    let ok = exit_code == Some(0);

    let mut body = String::from("<a class=\"back\" href=\"/\">← all memory</a>\n");
    body.push_str(&format!(
        "<div class=\"eyebrow\">command run · #{id}</div>\n<h1>{}</h1>\n",
        esc(&command)
    ));
    body.push_str(&format!(
        "<div class=\"badge {}\">{}</div>\n",
        if ok { "ok" } else { "bad" },
        match exit_code {
            Some(0) => "exit 0 · success".to_string(),
            Some(c) => format!("exit {c} · failed"),
            None => "no exit code".to_string(),
        }
    ));
    body.push_str(&tag_pills(&error_tags(&format!(
        "{stdout}\n{stderr}\n{notes}"
    ))));
    body.push_str("<div class=\"meta\">");
    if let Some(cwd) = &cwd {
        body.push_str(&format!("<div><span>cwd</span>{}</div>", esc(cwd)));
    }
    body.push_str(&format!(
        "<div><span>when</span>{}</div></div>\n",
        esc(&fmt_datetime(&created_at))
    ));

    body.push_str("<h2>Command</h2>\n");
    body.push_str(&code_block(&argv.join(" ")));
    if !stdout.trim().is_empty() {
        body.push_str("<h2>Output</h2>\n");
        body.push_str(&format!(
            "<pre class=\"out\">{}</pre>\n",
            esc_redacted(&stdout)
        ));
    }
    if !stderr.trim().is_empty() {
        body.push_str("<h2>Error log</h2>\n");
        body.push_str(&format!(
            "<pre class=\"err\">{}</pre>\n",
            esc_redacted(&stderr)
        ));
    }
    if !notes.trim().is_empty() {
        body.push_str(&format!(
            "<h2>Note</h2>\n<p class=\"noteblock\">{}</p>\n",
            esc_redacted(&notes)
        ));
    }
    body.push_str(&debug_summary(&argv.join(" "), &stdout, &stderr, &notes));
    body.push_str(&hints(&conn, id, &command, ok));
    Ok(page(&format!("{} · MemoryWhale", command), &body))
}

fn session_page(id: i64) -> Result<String, String> {
    let conn = open_db()?;
    let row = conn
        .query_row(
            "SELECT shell, cwd, notes, started_at, ended_at, byte_count, transcript, status FROM sessions WHERE id = ?1",
            params![id],
            |r| {
                Ok((
                    r.get::<_, Option<String>>(0)?,
                    r.get::<_, Option<String>>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, i64>(5)?,
                    r.get::<_, String>(6)?,
                    r.get::<_, String>(7)?,
                ))
            },
        )
        .optional()
        .map_err(|e| format!("read session: {e}"))?
        .ok_or_else(|| format!("no session #{id}"))?;
    let (shell, cwd, notes, started_at, ended_at, byte_count, transcript, status) = row;

    let mut body = String::from("<a class=\"back\" href=\"/\">← all memory</a>\n");
    body.push_str(&format!(
        "<div class=\"eyebrow\">recorded session · #{id}</div>\n<h1>Session {id}</h1>\n"
    ));
    body.push_str("<div class=\"meta\">");
    if let Some(shell) = &shell {
        body.push_str(&format!("<div><span>shell</span>{}</div>", esc(shell)));
    }
    if let Some(cwd) = &cwd {
        body.push_str(&format!("<div><span>cwd</span>{}</div>", esc(cwd)));
    }
    body.push_str(&format!(
        "<div><span>started</span>{}</div>",
        esc(&fmt_datetime(&started_at))
    ));
    body.push_str(&format!(
        "<div><span>status</span>{}</div>",
        esc(&session_label(&status, &ended_at))
    ));
    body.push_str(&format!(
        "<div><span>size</span>{byte_count} bytes</div></div>\n"
    ));
    if !notes.trim().is_empty() {
        body.push_str(&format!(
            "<p class=\"noteblock\">{}</p>\n",
            esc_redacted(&notes)
        ));
    }
    body.push_str(&tag_pills(&error_tags(&transcript)));
    body.push_str(&session_debug_summary(&transcript, &notes));
    body.push_str("<h2>Transcript</h2>\n");
    body.push_str(&format!(
        "<pre class=\"out\">{}</pre>\n",
        esc_redacted(&transcript)
    ));
    Ok(page(&format!("Session {id} · MemoryWhale"), &body))
}

fn hints(conn: &Connection, id: i64, command: &str, ok: bool) -> String {
    let mut out = String::new();
    let mut items: Vec<(String, Option<String>)> = Vec::new();

    if let Ok(total) = conn.query_row(
        "SELECT COUNT(*) FROM command_runs WHERE command = ?1",
        params![command],
        |r| r.get::<_, i64>(0),
    ) {
        if total > 1 {
            let failures: i64 = conn
                .query_row(
                    "SELECT COUNT(*) FROM command_runs WHERE command = ?1 AND exit_code <> 0",
                    params![command],
                    |r| r.get(0),
                )
                .unwrap_or(0);
            items.push((
                format!(
                    "You've run `{command}` {total} time(s) — {} succeeded, {failures} failed.",
                    total - failures
                ),
                None,
            ));
        }
    }

    if !ok {
        if let Ok(Some(argv_json)) = conn
            .query_row(
                "SELECT argv_json FROM command_runs WHERE command = ?1 AND exit_code = 0 AND id <> ?2 ORDER BY created_at DESC LIMIT 1",
                params![command, id],
                |r| r.get::<_, String>(0),
            )
            .optional()
        {
            let argv: Vec<String> = serde_json::from_str(&argv_json).unwrap_or_default();
            if !argv.is_empty() {
                items.push((format!("A previous run of `{command}` succeeded — try that exact command:"), Some(argv.join(" "))));
            }
        }
        if let Ok(Some(prev_at)) = conn
            .query_row(
                "SELECT created_at FROM command_runs WHERE command = ?1 AND exit_code <> 0 AND id <> ?2 ORDER BY created_at DESC LIMIT 1",
                params![command, id],
                |r| r.get::<_, String>(0),
            )
            .optional()
        {
            if let Ok(Some((next_cmd, next_argv))) = conn
                .query_row(
                    "SELECT command, argv_json FROM command_runs WHERE created_at > ?1 ORDER BY created_at ASC LIMIT 1",
                    params![prev_at],
                    |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)),
                )
                .optional()
            {
                let argv: Vec<String> = serde_json::from_str(&next_argv).unwrap_or_default();
                let line = if argv.is_empty() { next_cmd } else { argv.join(" ") };
                items.push(("Last time this command failed, the next thing you ran was:".to_string(), Some(line)));
            }
        }
    }

    if items.is_empty() {
        return out;
    }
    out.push_str("<h2>Suggested next steps</h2>\n<div class=\"hints\">\n");
    for (text, snippet) in items {
        out.push_str("<div class=\"hint\"><p>");
        out.push_str(&esc(&text));
        out.push_str("</p>");
        if let Some(s) = snippet {
            out.push_str(&code_block(&s));
        }
        out.push_str("</div>\n");
    }
    out.push_str("</div>\n");
    out
}

#[derive(Serialize)]
struct GNode {
    id: String,
    label: String,
    kind: String,
    weight: i64,
    name: Option<String>,
}

#[derive(Serialize)]
struct GLink {
    source: String,
    target: String,
}

#[derive(Serialize)]
struct Graph {
    nodes: Vec<GNode>,
    links: Vec<GLink>,
}

/// Build a graph aggregated by command name and argument value. Nodes carry a
/// weight (how often they appear); arguments used by two or more distinct
/// commands are marked as bridges. One node per command (not per run).
fn graph_json() -> Result<String, String> {
    let conn = open_db()?;

    let mut run_cmd: HashMap<i64, String> = HashMap::new();
    let mut cmd_count: HashMap<String, i64> = HashMap::new();
    {
        let mut stmt = conn
            .prepare("SELECT id, command FROM command_runs")
            .map_err(|e| format!("query runs: {e}"))?;
        let it = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| format!("read runs: {e}"))?;
        for (id, cmd) in it.flatten() {
            run_cmd.insert(id, cmd.clone());
            *cmd_count.entry(cmd).or_insert(0) += 1;
        }
    }

    let mut arg_count: HashMap<String, i64> = HashMap::new();
    let mut arg_cmds: HashMap<String, HashSet<String>> = HashMap::new();
    let mut pairs: HashSet<(String, String)> = HashSet::new();
    {
        let mut stmt = conn
            .prepare("SELECT command_run_id, value FROM command_arguments WHERE position >= 1")
            .map_err(|e| format!("query args: {e}"))?;
        let it = stmt
            .query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))
            .map_err(|e| format!("read args: {e}"))?;
        for (run_id, value) in it.flatten() {
            if value.trim().is_empty() {
                continue;
            }
            let cmd = match run_cmd.get(&run_id) {
                Some(c) => c.clone(),
                None => continue,
            };
            *arg_count.entry(value.clone()).or_insert(0) += 1;
            arg_cmds
                .entry(value.clone())
                .or_default()
                .insert(cmd.clone());
            pairs.insert((cmd, value));
        }
    }

    let mut nodes: Vec<GNode> = Vec::new();
    for (cmd, count) in &cmd_count {
        nodes.push(GNode {
            id: format!("cmd:{cmd}"),
            label: cmd.clone(),
            kind: "cmd".into(),
            weight: *count,
            name: Some(cmd.clone()),
        });
    }
    for (val, count) in &arg_count {
        let shared = arg_cmds.get(val).map(|s| s.len() >= 2).unwrap_or(false);
        nodes.push(GNode {
            id: format!("arg:{val}"),
            label: val.clone(),
            kind: if shared {
                "bridge".into()
            } else {
                "arg".into()
            },
            weight: *count,
            name: None,
        });
    }
    let mut links: Vec<GLink> = Vec::new();
    for (cmd, val) in &pairs {
        links.push(GLink {
            source: format!("cmd:{cmd}"),
            target: format!("arg:{val}"),
        });
    }

    serde_json::to_string(&Graph { nodes, links }).map_err(|e| format!("serialize graph: {e}"))
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(high), Some(low)) = (hex_value(bytes[i + 1]), hex_value(bytes[i + 2])) {
                out.push((high << 4) | low);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

fn percent_encode(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn query_param(raw_path: &str, key: &str) -> Option<String> {
    let query = raw_path.split_once('?')?.1;
    for pair in query.split('&') {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        if percent_decode(k) == key {
            return Some(percent_decode(&v.replace('+', " ")));
        }
    }
    None
}

/// List all command runs for one command name (where a graph command node links).
fn runs_page(raw: &str) -> String {
    let name = percent_decode(raw.trim_end_matches('/'));
    let conn = match open_db() {
        Ok(c) => c,
        Err(e) => return page("Runs", &format!("<p>{}</p>", esc(&e))),
    };

    let mut body = String::from("<a class=\"back\" href=\"/graph\">← graph</a>\n");
    body.push_str(&format!(
        "<div class=\"eyebrow\">command</div>\n<h1>{}</h1>\n",
        esc(&name)
    ));
    body.push_str("<div class=\"list\">\n");
    let mut rows = 0;
    if let Ok(mut stmt) = conn.prepare(
        "SELECT id, argv_json, exit_code, created_at, notes FROM command_runs WHERE command = ?1 ORDER BY id DESC",
    ) {
        if let Ok(it) = stmt.query_map(params![name], |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
            ))
        }) {
            for (id, argv_json, code, at, notes) in it.flatten() {
                let ok = code == Some(0);
                body.push_str(&format!(
                    "<a class=\"row\" href=\"/command/{id}\"><span class=\"badge {}\">{}</span>\
                     <span class=\"cmd\">{}</span><span class=\"when\">{}</span><span class=\"note\">{}</span></a>\n",
                    if ok { "ok" } else { "bad" },
                    match code { Some(c) => format!("exit {c}"), None => "—".into() },
                    esc_redacted(&full_command(&argv_json, &name)), esc(&fmt_datetime(&at)), esc_redacted(&notes)
                ));
                rows += 1;
            }
        }
    }
    if rows == 0 {
        body.push_str("<p class=\"empty\">No runs found for this command.</p>");
    }
    body.push_str("</div>\n");
    page(&format!("{} runs · MemoryWhale", name), &body)
}

fn graph_page() -> String {
    let data = match graph_json() {
        Ok(d) => d.replace("</", "<\\/"), // keep an arg value of "</script>" from breaking the page
        Err(e) => return page("Graph", &format!("<p>{}</p>", esc(&e))),
    };
    let mut body = String::new();
    body.push_str("<a class=\"back\" href=\"/\">← all memory</a>\n");
    body.push_str("<div class=\"eyebrow\">knowledge graph</div>\n<h1>Command graph</h1>\n");
    body.push_str("<p class=\"sub\">Commands sized by how often you ran them, linked to their arguments. Orange arguments are shared by two or more commands (bridges). Click a command to see its runs.</p>\n");
    body.push_str("<div class=\"legend\"><span class=\"dot run\"></span>command<span class=\"dot arg\"></span>argument<span class=\"dot bridge\"></span>shared</div>\n");
    body.push_str("<canvas id=\"g\" width=\"920\" height=\"560\"></canvas>\n");
    body.push_str("<script>\nconst DATA = ");
    body.push_str(&data);
    body.push_str(";\n");
    body.push_str(GRAPH_JS);
    body.push_str("\n</script>\n");
    page("Command graph · MemoryWhale", &body)
}

const GRAPH_JS: &str = include_str!("mw-serve/graph.js");

fn code_block(text: &str) -> String {
    format!(
        "<div class=\"codeblock\"><code>{}</code></div>\n",
        esc_redacted(text)
    )
}

fn page(title: &str, body: &str) -> String {
    format!(
        "<!DOCTYPE html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"/>\
<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"/>\
<title>{}</title>\n<style>{CSS}</style></head>\n<body><main>{body}\
<footer>MemoryWhale — served locally from SQLite · nothing is uploaded</footer></main></body></html>\n",
        esc(title)
    )
}

const CSS: &str = include_str!("mw-serve/styles.css");

/// The full command line from stored argv (e.g. "npm run tauri:dev"), falling
/// back to the bare command name if argv can't be parsed.
fn full_command(argv_json: &str, fallback: &str) -> String {
    serde_json::from_str::<Vec<String>>(argv_json)
        .ok()
        .filter(|a| !a.is_empty())
        .map(|a| a.join(" "))
        .unwrap_or_else(|| fallback.to_string())
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn esc_redacted(s: &str) -> String {
    esc(&redact_secrets(s))
}

fn redact_secrets(input: &str) -> String {
    let mut out = input.to_string();
    let patterns = [
        (
            r#"(?i)(api[_-]?key|token|password|passwd|secret|authorization)\s*[:=]\s*['"]?([^\s'"&]+)"#,
            "$1=<redacted>",
        ),
        (r"(?i)Bearer\s+[A-Za-z0-9._\-]+", "Bearer <redacted>"),
        (r"gh[pousr]_[A-Za-z0-9_]+", "<redacted-github-token>"),
        (r"sk-[A-Za-z0-9_\-]{20,}", "<redacted-api-key>"),
    ];
    for (pattern, replacement) in patterns {
        if let Ok(re) = Regex::new(pattern) {
            out = re.replace_all(&out, replacement).into_owned();
        }
    }
    out
}

fn error_tags(text: &str) -> Vec<&'static str> {
    let lower = text.to_lowercase();
    let mut tags = Vec::new();
    let patterns = [
        ("command not found", "command not found"),
        ("permission denied", "permission denied"),
        ("package not found", "package not found"),
        ("no package", "package not found"),
        ("failed to build", "failed to build"),
        ("failed to compile", "failed to build"),
        ("could not compile", "failed to build"),
        ("address already in use", "port already in use"),
        ("port already in use", "port already in use"),
        ("no such file or directory", "no such file"),
        ("not found", "not found"),
    ];
    for (needle, label) in patterns {
        if lower.contains(needle) && !tags.contains(&label) {
            tags.push(label);
        }
    }
    tags
}

fn tag_pills(tags: &[&str]) -> String {
    if tags.is_empty() {
        return String::new();
    }
    let mut out = String::from("<span class=\"tags\">");
    for tag in tags {
        out.push_str(&format!("<span class=\"tag\">{}</span>", esc(tag)));
    }
    out.push_str("</span>");
    out
}

fn debug_summary(command: &str, stdout: &str, stderr: &str, notes: &str) -> String {
    let combined = format!("{stdout}\n{stderr}\n{notes}");
    let tags = error_tags(&combined);
    let mut next = Vec::new();
    if tags.contains(&"command not found") {
        next.push("Verify the command is installed and available on PATH.");
    }
    if tags.contains(&"permission denied") {
        next.push("Check file permissions, executable bits, or whether sudo is actually required.");
    }
    if tags.contains(&"package not found") {
        next.push("Update package indexes and confirm the package name for this OS/architecture.");
    }
    if tags.contains(&"failed to build") {
        next.push("Read the first compiler error above, then rerun the narrowest build command.");
    }
    if tags.contains(&"port already in use") {
        next.push("Find the process using the port or choose a different port.");
    }
    if next.is_empty() {
        next.push("Search for the most recent related failed command, then compare it with the next successful run.");
    }

    format!(
        "<h2>Local debug summary</h2><div class=\"hint\">\
         <p><strong>What happened?</strong> MemoryWhale recorded <code>{}</code> with its output, error log, notes, and timestamp.</p>\
         <p><strong>What failed?</strong> {}</p>\
         <p><strong>What was tried?</strong> {}</p>\
         <p><strong>Try next:</strong> {}</p></div>\n",
        esc_redacted(command),
        if tags.is_empty() { "No common failure pattern was detected.".to_string() } else { esc(&tags.join(", ")) },
        if notes.trim().is_empty() { "No note was attached to this run.".to_string() } else { esc_redacted(notes) },
        esc(&next.join(" "))
    )
}

fn session_debug_summary(transcript: &str, notes: &str) -> String {
    let tags = error_tags(transcript);
    format!(
        "<h2>Local debug summary</h2><div class=\"hint\">\
         <p><strong>What happened?</strong> MemoryWhale saved a terminal session transcript for later debugging.</p>\
         <p><strong>Detected issues:</strong> {}</p>\
         <p><strong>Context:</strong> {}</p></div>\n",
        if tags.is_empty() { "No common failure pattern was detected.".to_string() } else { esc(&tags.join(", ")) },
        if notes.trim().is_empty() { "No session note was attached.".to_string() } else { esc_redacted(notes) }
    )
}

/// Import every session `.log` that has no row yet (interrupted recordings).
struct RecoveryReport {
    recovered: usize,
    deleted_empty: usize,
}

fn recover_orphans() -> Result<RecoveryReport, String> {
    let sessions_dir = memorywhale_dir()?.join("sessions");
    if !sessions_dir.exists() {
        return Ok(RecoveryReport {
            recovered: 0,
            deleted_empty: 0,
        });
    }
    let conn = open_db()?;

    let mut entries: Vec<PathBuf> = fs::read_dir(&sessions_dir)
        .map_err(|e| format!("read sessions dir: {e}"))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().map(|x| x == "log").unwrap_or(false))
        .collect();
    entries.sort();

    let mut recovered = 0;
    let mut deleted_empty = 0;
    for path in entries {
        let path_str = match path.to_str() {
            Some(s) => s.to_string(),
            None => continue,
        };
        let already: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM sessions WHERE transcript_path = ?1",
                params![path_str],
                |r| r.get(0),
            )
            .unwrap_or(0);
        if already > 0 {
            continue;
        }
        let raw = match fs::read(&path) {
            Ok(b) => b,
            Err(_) => continue,
        };
        if raw.is_empty() {
            if fs::remove_file(&path).is_ok() {
                deleted_empty += 1;
            }
            continue;
        }
        let cleaned =
            memorywhale_cli::sanitize_capture(&clean_transcript(&String::from_utf8_lossy(&raw)));
        let started = started_from_filename(&path).unwrap_or_else(|| mtime_rfc3339(&path));
        let ended = mtime_rfc3339(&path);
        conn.execute(
            "INSERT INTO sessions (shell, cwd, transcript_path, transcript, notes, started_at, ended_at, byte_count, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, 'interrupted')",
            params![
                Option::<String>::None, Option::<String>::None, path_str, cleaned,
                "recovered from transcript (recording was interrupted before saving)",
                started, ended, cleaned.len() as i64
            ],
        )
        .map_err(|e| format!("insert recovered session: {e}"))?;
        recovered += 1;
    }
    Ok(RecoveryReport {
        recovered,
        deleted_empty,
    })
}

fn started_from_filename(path: &Path) -> Option<String> {
    let stamp = path.file_stem()?.to_str()?.strip_prefix("session-")?;
    let re =
        Regex::new(r"^(\d{4}-\d{2}-\d{2})T(\d{2})-(\d{2})-(\d{2})(\.\d+)?([+-]\d{2})-(\d{2})$")
            .ok()?;
    let c = re.captures(stamp)?;
    Some(format!(
        "{}T{}:{}:{}{}{}:{}",
        &c[1],
        &c[2],
        &c[3],
        &c[4],
        c.get(5).map(|m| m.as_str()).unwrap_or(""),
        &c[6],
        &c[7]
    ))
}

fn mtime_rfc3339(path: &Path) -> String {
    let t = fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH);
    DateTime::<Utc>::from(t).to_rfc3339()
}

fn clean_transcript(input: &str) -> String {
    let osc = Regex::new(r"\x1b\][^\x07\x1b]*(?:\x07|\x1b\\)").unwrap();
    let csi = Regex::new(r"\x1b[@-Z\\-_]|\x1b\[[0-?]*[ -/]*[@-~]").unwrap();
    let ctrl = Regex::new(r"[\x00-\x08\x0b\x0c\x0e-\x1f\x7f]").unwrap();
    let s = osc.replace_all(input, "");
    let s = csi.replace_all(&s, "");
    let s = s.replace('\r', "");
    ctrl.replace_all(&s, "").into_owned()
}

fn open_db() -> Result<Connection, String> {
    let path = database_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("create data dir: {e}"))?;
    }
    memorywhale_cli::storage::open_path(&path)
}

fn database_path() -> Result<PathBuf, String> {
    Ok(memorywhale_dir()?.join("memorywhale.sqlite3"))
}

fn data_base() -> Result<PathBuf, String> {
    dirs::data_local_dir()
        .or_else(dirs::home_dir)
        .ok_or_else(|| "could not resolve local data directory".to_string())
}

fn memorywhale_dir() -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os("MEMORYWHALE_DATA_DIR") {
        return Ok(PathBuf::from(path));
    }
    Ok(data_base()?.join("MemoryWhale"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Shutdown;

    #[test]
    fn repository_counts_group_canonical_repos_and_can_split_worktrees() {
        let conn = Connection::open_in_memory().unwrap();
        memorywhale_cli::storage::initialize(&conn).unwrap();
        for (id, name, root) in [
            ("remote:example.com/acme/project", "project", "/gone/main"),
            (
                "remote:example.com/acme/project",
                "project",
                "/gone/feature",
            ),
            (
                "remote:example.com/other/project",
                "project",
                "/gone/unrelated",
            ),
        ] {
            conn.execute(
                "INSERT INTO command_runs
                    (command, argv_json, created_at, repository_id, repository_name, worktree_root)
                 VALUES ('cargo', '[\"cargo\"]', '2026-01-01T00:00:00Z', ?1, ?2, ?3)",
                params![id, name, root],
            )
            .unwrap();
        }
        conn.execute(
            "INSERT INTO sessions
                (transcript_path, transcript, started_at, ended_at, repository_id,
                 repository_name, worktree_root)
             VALUES ('', 'cd /gone/feature', '2026-01-01T00:00:00Z',
                     '2026-01-01T00:01:00Z', ?1, 'project', '/gone/main')",
            ["remote:example.com/acme/project"],
        )
        .unwrap();

        let canonical = repo_counts(&conn, false);
        assert_eq!(canonical.len(), 2);
        assert_eq!(
            canonical
                .iter()
                .find(|(repo, _)| repo.id == "remote:example.com/acme/project")
                .map(|(_, count)| *count),
            Some(3)
        );

        let worktrees = repo_counts(&conn, true);
        assert_eq!(worktrees.len(), 3);
        for root in ["/gone/main", "/gone/feature"] {
            assert_eq!(
                worktrees
                    .iter()
                    .find(|(repo, _)| repo.worktree.as_deref() == Some(root))
                    .map(|(_, count)| *count),
                Some(2)
            );
        }
    }

    #[test]
    fn repository_route_ids_round_trip() {
        let id = "remote:github.com/acme/a repo";
        assert_eq!(percent_decode(&percent_encode(id)), id);
    }

    #[test]
    fn dashboard_defaults_to_loopback() {
        let config = parse_server_args(Vec::<String>::new()).unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 7071);
        assert!(!config.print_token);
        assert!(validate_server_config(&config).is_ok());
    }

    #[test]
    fn print_token_flag_does_not_bind() {
        let config = parse_server_args(["--print-token".to_string()]).unwrap();
        assert!(config.print_token);
    }

    #[test]
    fn bearer_header_strips_scheme() {
        assert_eq!(bearer_token("Bearer secret"), Some("secret"));
        assert_eq!(bearer_token("bearer secret"), Some("secret"));
        assert_eq!(bearer_token("BEARER secret"), Some("secret"));
        assert!(bearer_token("secret").is_none());
    }

    #[test]
    fn mcp_post_discovers_on_loopback_without_a_token() {
        let body = r#"{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"test","version":"1"},"io.modelcontextprotocol/clientCapabilities":{}}}}"#;
        let request = format!(
            "POST /mcp HTTP/1.1\r\nHost: localhost\r\nContent-Length: {}\r\n\r\n{body}",
            body.len()
        );
        let response = raw_response(request.as_bytes());
        assert!(
            response.starts_with("HTTP/1.1 200 OK"),
            "unexpected MCP response: {response}"
        );
        assert!(response.contains("application/json"));
        assert!(response.contains("2026-07-28"));
    }

    #[test]
    fn mcp_get_is_not_allowed() {
        let response = raw_response(b"GET /mcp HTTP/1.1\r\nHost: localhost\r\n\r\n");
        assert!(response.starts_with("HTTP/1.1 405 Method Not Allowed"));
        assert!(response.contains("Allow: POST\r\n"));
    }

    #[test]
    fn lan_requires_authentication() {
        let mut config = parse_server_args(["--lan".to_string()]).unwrap();
        config.token.clear();
        assert!(validate_server_config(&config).is_err());
        config.token = "shared-secret".to_string();
        assert!(validate_server_config(&config).is_ok());
    }

    #[test]
    fn token_is_read_from_form_body_not_query_string() {
        assert_eq!(
            form_param("token=shared%20secret", "token").as_deref(),
            Some("shared secret")
        );
        assert!(form_param("other=value", "token").is_none());
    }

    #[test]
    fn limited_lines_reject_oversized_input() {
        let mut reader = &b"abcdef\n"[..];
        assert!(matches!(
            read_limited_line(&mut reader, 5),
            Err(LineError::TooLong)
        ));
    }

    #[test]
    fn limited_lines_accept_complete_input() {
        let mut reader = &b"GET / HTTP/1.1\r\n"[..];
        assert_eq!(
            read_limited_line(&mut reader, 64).unwrap().as_deref(),
            Some("GET / HTTP/1.1\r\n")
        );
    }

    #[test]
    fn limited_lines_reject_unterminated_input() {
        let mut reader = &b"GET / HTTP/1.1"[..];
        assert!(matches!(
            read_limited_line(&mut reader, 64),
            Err(LineError::Unterminated)
        ));
    }

    #[test]
    fn request_line_requires_http11_shape() {
        assert_eq!(
            parse_request_line("GET / HTTP/1.1\r\n").unwrap(),
            ("GET".into(), "/".into())
        );
        assert!(parse_request_line("GET /\r\n").is_err());
        assert!(parse_request_line("GET / HTTP/2\r\n").is_err());
        assert!(parse_request_line("GET / HTTP/1.1 extra\r\n").is_err());
    }

    fn raw_response(request: &[u8]) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let worker = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            handle(stream);
        });
        let mut client = TcpStream::connect(address).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        client.write_all(request).unwrap();
        client.shutdown(Shutdown::Write).unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        worker.join().unwrap();
        response
    }

    fn raw_response_with_server_timeout(request: &[u8], timeout: Duration) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let worker = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            stream.set_read_timeout(Some(timeout)).unwrap();
            stream.set_write_timeout(Some(timeout)).unwrap();
            handle(stream);
        });
        let mut client = TcpStream::connect(address).unwrap();
        client
            .set_read_timeout(Some(Duration::from_secs(2)))
            .unwrap();
        client.write_all(request).unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        worker.join().unwrap();
        response
    }

    #[test]
    fn parser_returns_bounded_errors_for_hostile_requests() {
        let malformed = raw_response(b"GET /\r\n\r\n");
        assert!(malformed.starts_with("HTTP/1.1 400 Bad Request"));

        let oversized = format!(
            "GET / HTTP/1.1\r\nX: {}\r\n\r\n",
            "x".repeat(MAX_HEADER_LINE_BYTES)
        );
        assert!(raw_response(oversized.as_bytes()).starts_with("HTTP/1.1 431 "));

        let transfer = raw_response(b"GET / HTTP/1.1\r\nTransfer-Encoding: chunked\r\n\r\n");
        assert!(transfer.starts_with("HTTP/1.1 400 Bad Request"));

        let too_large = format!(
            "POST /login HTTP/1.1\r\nContent-Length: {}\r\n\r\n",
            MAX_BODY_BYTES + 1
        );
        assert!(raw_response(too_large.as_bytes()).starts_with("HTTP/1.1 413 Payload Too Large"));
    }

    #[test]
    fn parser_rejects_header_ambiguity_and_missing_loopback_host() {
        let missing_host = raw_response(b"GET / HTTP/1.1\r\n\r\n");
        assert!(missing_host.starts_with("HTTP/1.1 403 Forbidden"));

        let duplicate_length = raw_response(
            b"GET / HTTP/1.1\r\nHost: localhost\r\nContent-Length: 0\r\nContent-Length: 0\r\n\r\n",
        );
        assert!(duplicate_length.starts_with("HTTP/1.1 400 Bad Request"));

        let invalid_length =
            raw_response(b"GET / HTTP/1.1\r\nHost: localhost\r\nContent-Length: +1\r\n\r\n");
        assert!(invalid_length.starts_with("HTTP/1.1 400 Bad Request"));

        let too_many_headers = format!(
            "GET / HTTP/1.1\r\nHost: localhost\r\n{}\r\n",
            (0..=MAX_HEADER_COUNT)
                .map(|i| format!("X-{i}: value\r\n"))
                .collect::<String>()
        );
        assert!(raw_response(too_many_headers.as_bytes()).starts_with("HTTP/1.1 431 "));
    }

    #[test]
    fn parser_rejects_unsupported_methods_and_malformed_lengths() {
        let method = raw_response(b"PUT / HTTP/1.1\r\nHost: localhost\r\n\r\n");
        assert!(method.starts_with("HTTP/1.1 405 Method Not Allowed"));
        assert!(method.contains("Allow: GET, HEAD\r\n"));
        let login_method = raw_response(b"PUT /login HTTP/1.1\r\nHost: localhost\r\n\r\n");
        assert!(login_method.contains("Allow: GET, HEAD, POST\r\n"));
        for value in ["+1", "1.0", "0x10", "-1", ""] {
            assert!(parse_content_length(value).is_err(), "accepted {value:?}");
        }
        assert_eq!(parse_content_length("00012"), Ok(12));
    }

    #[test]
    fn head_requests_match_get_headers_without_a_body() {
        let get = raw_response(b"GET /not-a-real-route HTTP/1.1\r\nHost: localhost\r\n\r\n");
        let head = raw_response(b"HEAD /not-a-real-route HTTP/1.1\r\nHost: localhost\r\n\r\n");
        let (get_headers, get_body) = get.split_once("\r\n\r\n").unwrap();
        let (head_headers, head_body) = head.split_once("\r\n\r\n").unwrap();

        assert_eq!(head_headers, get_headers);
        assert!(!get_body.is_empty());
        assert!(head_body.is_empty());
    }

    #[test]
    fn incomplete_client_gets_bounded_timeout_response() {
        let response = raw_response_with_server_timeout(
            b"GET / HTTP/1.1\r\nHost: localhost\r\n",
            Duration::from_millis(50),
        );
        assert!(
            response.starts_with("HTTP/1.1 408 Request Timeout"),
            "unexpected timeout response: {response:?}"
        );
    }

    #[test]
    fn safe_cookie_values_reject_control_characters() {
        assert!(cookie_value_is_safe("utc"));
        assert!(cookie_value_is_safe("-08:00"));
        assert!(!cookie_value_is_safe("utc\r\nX:1"));
        assert!(!cookie_value_is_safe("foo\x7f"));
    }

    #[test]
    fn malicious_tz_does_not_emit_cookie() {
        let mut cookies: Vec<String> = Vec::new();
        let tz = "UTC\r\nX-Injected: yes";
        if let Some(c) = set_cookie("mw_tz", tz, "; Path=/; SameSite=Strict; Max-Age=31536000") {
            cookies.push(c);
        }
        assert!(cookies.is_empty());
        let header: String = cookies
            .iter()
            .map(|c| format!("Set-Cookie: {c}\r\n"))
            .collect();
        assert!(!header.contains("X-Injected"));
        assert!(!header.contains("\r\n\r\n"));
    }

    #[test]
    fn valid_tz_emits_safe_cookie() {
        let c = set_cookie(
            "mw_tz",
            "utc",
            "; Path=/; SameSite=Strict; Max-Age=31536000",
        )
        .unwrap();
        assert_eq!(c, "mw_tz=utc; Path=/; SameSite=Strict; Max-Age=31536000");
    }

    #[test]
    fn timezone_parser_rejects_malformed_and_accepts_supported_offsets() {
        assert!(matches!(parse_tz(""), DisplayTz::Local));
        assert!(matches!(parse_tz("LOCAL"), DisplayTz::Local));
        assert!(matches!(parse_tz("utc"), DisplayTz::Fixed(_)));
        assert!(matches!(parse_tz("-08:00"), DisplayTz::Fixed(_)));
        assert!(matches!(parse_tz("+0530"), DisplayTz::Fixed(_)));
        assert!(matches!(parse_tz("+5"), DisplayTz::Fixed(_)));
        assert!(matches!(parse_tz("+99:99"), DisplayTz::Local));
        assert!(matches!(parse_tz("5"), DisplayTz::Local));
        assert!(matches!(parse_tz("+01:60"), DisplayTz::Local));
        assert!(matches!(parse_tz("+💥"), DisplayTz::Local));
        assert!(matches!(parse_tz("+01:00\r\nX"), DisplayTz::Local));
    }

    #[test]
    fn query_and_route_decoding_does_not_turn_encoded_input_into_html() {
        assert_eq!(percent_decode("a%2Fb+two"), "a/b+two");
        assert_eq!(
            query_param("/?q=a%3Cscript%3E%26b", "q").as_deref(),
            Some("a<script>&b")
        );
        assert_eq!(query_param("/?q=one+two", "q").as_deref(), Some("one two"));
        assert_eq!(route("/not-a-real-route").0, "404 Not Found");

        let rendered = page(
            "</script><script>alert(1)</script>",
            &code_block("<img src=x onerror=alert(1)>"),
        );
        assert!(!rendered.contains("<script>alert(1)</script>"));
        assert!(rendered.contains("&lt;img src=x onerror=alert(1)&gt;"));
        assert!(!esc_redacted("password: hunter2secret").contains("hunter2secret"));
    }

    #[test]
    fn response_framing_matches_body_and_has_no_injection() {
        let response = raw_response(b"GET / HTTP/1.1\r\nHost: localhost\r\n\r\n");
        let header_end = response.find("\r\n\r\n").unwrap();
        let (headers, payload) = response.split_at(header_end + 4);
        assert!(headers.contains(&format!("Content-Length: {}", payload.len())));
        assert!(!payload.is_empty());
        assert!(headers.ends_with("\r\n\r\n"));
        assert!(!response.contains("\r\nX-Injected:"));
    }

    #[test]
    fn percent_decode_rejects_malformed_utf8_sequences_without_panicking() {
        assert_eq!(percent_decode("%💥"), "%💥");
    }

    #[test]
    fn loopback_host_header_accepts_localhost_names() {
        for host in [
            "localhost",
            "LOCALHOST:7071",
            "127.0.0.1",
            "127.0.0.1:7071",
            "[::1]",
            "[::1]:7071",
        ] {
            assert!(host_header_allowed(host, true), "should allow {host}");
        }
    }

    #[test]
    fn loopback_host_header_rejects_rebound_hostnames() {
        for host in [
            "attacker.example",
            "attacker.example:7071",
            "127.0.0.1.evil.com",
            "185.199.108.153",
            "",
        ] {
            assert!(!host_header_allowed(host, true), "should reject {host}");
        }
    }

    #[test]
    fn lan_bind_accepts_any_host_header() {
        // Non-loopback binds require a token; the token gate protects data,
        // and LAN clients reach the server under arbitrary interface IPs.
        assert!(host_header_allowed("192.168.1.20:7071", false));
        assert!(host_header_allowed("myhost.local", false));
    }

    #[test]
    fn constant_time_equality_behaves_like_equality() {
        assert!(ct_eq("shared-secret", "shared-secret"));
        assert!(!ct_eq("shared-secret", "shared-secret2"));
        assert!(!ct_eq("", "x"));
        assert!(ct_eq("", ""));
        assert!(!ct_eq("aaaa", "aaab"));
    }

    #[test]
    fn responses_carry_security_headers() {
        let r = response("200 OK", "<html></html>", "");
        assert!(r.contains("X-Content-Type-Options: nosniff"));
        assert!(r.contains("Referrer-Policy: no-referrer"));
        assert!(r.contains("Cache-Control: no-store"));
        assert!(r.contains("Content-Security-Policy: default-src 'none'"));
        assert!(r.contains("frame-ancestors 'none'"));
        // Set-Cookie extras are still appended for normal routes.
        let r2 = response("200 OK", "x", "Set-Cookie: mw_tz=utc\r\n");
        assert!(r2.contains("Set-Cookie: mw_tz=utc"));
    }

    #[test]
    fn api_search_requires_a_query_and_returns_json() {
        let _ = API_ENABLED.set(true);
        let response = raw_response(b"GET /api/v1/search HTTP/1.1\r\nHost: localhost\r\n\r\n");
        assert!(response.starts_with("HTTP/1.1 400 Bad Request"));
        assert!(response.contains("Content-Type: application/json; charset=utf-8"));
        assert!(response.contains("\"api_version\":\"v1\""));
        assert!(response.contains("\"code\":\"missing_query\""));
    }

    #[test]
    fn api_limit_rejects_unbounded_requests() {
        assert!(api_limit("/api/v1/search?limit=0").is_err());
        assert!(api_limit("/api/v1/search?limit=51").is_err());
        assert!(api_limit("/api/v1/search?limit=not-a-number").is_err());
    }

    #[test]
    fn integrations_section_has_a_guide_link_for_every_client() {
        let html = integrations_section();
        assert_eq!(INTEGRATIONS.len(), 24);
        for (name, slug, badge) in INTEGRATIONS {
            assert!(html.contains(&format!(
                "href=\"https://github.com/wuisabel-gif/MemWhale/tree/main/integrations/{slug}\""
            )));
            assert!(html.contains(&format!("<span class=\"iname\">{}</span>", esc(name))));
            if *badge == "unverified" {
                assert!(html.contains("class=\"ibadge off\">unverified"));
            }
        }
        assert_eq!(html.matches("class=\"icell\"").count(), INTEGRATIONS.len());
        assert_eq!(
            html.matches("aria-hidden=\"true\"").count(),
            INTEGRATIONS.len()
        );
    }
}
