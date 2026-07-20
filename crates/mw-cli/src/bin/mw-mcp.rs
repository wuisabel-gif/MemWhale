// mw-mcp: a Model Context Protocol server over stdio, so an AI agent (Claude
// Code, Codex, Cursor, …) can query your MemoryWhale memory directly instead of
// pasting it in. Speaks newline-delimited JSON-RPC 2.0 on stdin/stdout.
//
// Register with Claude Code:
//   claude mcp add memorywhale -- mw-mcp
//
// Tools exposed: recent_errors, search_memory, get_context, remember.

use chrono::Utc;
use mw_memory::engine::MemoryEngine;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

fn main() {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    // Client name from the `initialize` handshake (e.g. "Claude Code"), used to
    // attribute agent-written memories. One process serves one client, so a
    // single mutable slot is enough.
    let mut client_name: Option<String> = None;
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        if line.trim().is_empty() {
            continue;
        }
        let msg: Value = match serde_json::from_str(&line) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let method = msg.get("method").and_then(Value::as_str).unwrap_or("");
        let params = msg.get("params").cloned().unwrap_or(json!({}));
        if method == "initialize" {
            client_name = params
                .get("clientInfo")
                .and_then(|c| c.get("name"))
                .and_then(Value::as_str)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
        }
        // Notifications (no `id`) get no reply.
        let Some(id) = msg.get("id").cloned() else {
            continue;
        };
        let reply = match handle(method, &params, client_name.as_deref()) {
            Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
            Err(msg) => json!({"jsonrpc": "2.0", "id": id,
                "error": {"code": -32603, "message": msg}}),
        };
        let _ = writeln!(stdout, "{reply}");
        let _ = stdout.flush();
    }
}

fn handle(method: &str, params: &Value, client_name: Option<&str>) -> Result<Value, String> {
    match method {
        "initialize" => Ok(json!({
            "protocolVersion": "2024-11-05",
            "capabilities": {"tools": {}},
            "serverInfo": {"name": "memorywhale", "version": env!("CARGO_PKG_VERSION")}
        })),
        "tools/list" => Ok(json!({"tools": tool_defs()})),
        "tools/call" => {
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let args = params.get("arguments").cloned().unwrap_or(json!({}));
            let text = call_tool(name, &args, client_name)?;
            Ok(json!({"content": [{"type": "text", "text": text}]}))
        }
        // Unknown method: return empty result rather than erroring the session.
        _ => Ok(json!({})),
    }
}

fn tool_defs() -> Value {
    json!([
        {
            "name": "recent_errors",
            "description": "Recent failed commands (non-zero exit) with their error output. Use this first when debugging a recurring failure.",
            "inputSchema": {"type": "object", "properties": {
                "limit": {"type": "integer", "description": "max results (default 8)"}
            }}
        },
        {
            "name": "search_memory",
            "description": "Search remembered commands, sessions, and notes for a term. Results are ranked by an explainable engine and each includes the reasons it ranked where it did.",
            "inputSchema": {"type": "object", "properties": {
                "query": {"type": "string", "description": "text to search for"},
                "project": {"type": "string", "description": "optional: only memory recorded for this project, e.g. demo"},
                "machine": {"type": "string", "description": "optional: only memory recorded on this machine"}
            }, "required": ["query"]}
        },
        {
            "name": "get_context",
            "description": "The most relevant remembered memory, engine-ranked, optionally scoped to a project or machine. Each result includes the reasons it was ranked where it did.",
            "inputSchema": {"type": "object", "properties": {
                "project": {"type": "string", "description": "project tag, e.g. project:demo"},
                "machine": {"type": "string", "description": "optional: only memory recorded on this machine"}
            }}
        },
        {
            "name": "remember",
            "description": "Save a freeform lesson or conclusion for later — e.g. 'the E0308 in camera-driver was the fps field being a string; fix: parse it as i32'. Use this once you've figured out *why* something failed or *how* a fix worked, so future sessions (yours or a teammate's) don't have to re-derive it. Findable later via search_memory and get_context.",
            "inputSchema": {"type": "object", "properties": {
                "text": {"type": "string", "description": "the lesson or conclusion to remember"}
            }, "required": ["text"]}
        }
    ])
}

fn open() -> Result<Connection, String> {
    let path = memorywhale_cli::database_path()?;
    let conn =
        Connection::open(&path).map_err(|e| format!("failed to open {}: {e}", path.display()))?;
    // Ensure bookmarks provenance columns exist so reads below can rely on them.
    let _ = memorywhale_cli::migrate(&conn);
    Ok(conn)
}

fn call_tool(name: &str, args: &Value, client_name: Option<&str>) -> Result<String, String> {
    match name {
        "recent_errors" => {
            let limit = args.get("limit").and_then(Value::as_i64).unwrap_or(8);
            recent_errors(limit)
        }
        "search_memory" => {
            let q = args
                .get("query")
                .and_then(Value::as_str)
                .ok_or_else(|| "search_memory needs a 'query'".to_string())?;
            search_memory(q, scope_arg(args, "project"), scope_arg(args, "machine"))
        }
        "get_context" => {
            let project = args.get("project").and_then(Value::as_str);
            get_context(project, scope_arg(args, "machine"))
        }
        "remember" => {
            let text = args
                .get("text")
                .and_then(Value::as_str)
                .ok_or_else(|| "remember needs a 'text'".to_string())?;
            remember_tool(text, client_name)
        }
        other => Err(format!("unknown tool: {other}")),
    }
}

fn recent_errors(limit: i64) -> Result<String, String> {
    let conn = open()?;
    let mut stmt = conn
        .prepare(
            "SELECT argv_json, cwd, exit_code, stderr, notes, created_at
             FROM command_runs
             WHERE exit_code IS NOT NULL AND exit_code != 0
             ORDER BY id DESC LIMIT ?1",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![limit], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?,
                r.get::<_, Option<i64>>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, String>(5)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut out = String::new();
    for row in rows {
        let (argv_json, cwd, exit_code, stderr, notes, created_at) = row.map_err(|e| e.to_string())?;
        let argv: Vec<String> = serde_json::from_str(&argv_json).unwrap_or_default();
        out.push_str(&format!(
            "- `{}` (exit {}, {})\n  cwd: {}\n  err: {}\n  note: {}\n",
            argv.join(" "),
            exit_code.unwrap_or(-1),
            created_at,
            cwd.unwrap_or_default(),
            last_line(&stderr, 240),
            notes.trim()
        ));
    }
    Ok(if out.is_empty() {
        "(no failed commands recorded)".to_string()
    } else {
        out
    })
}

/// One ranked hit rendered for an agent: score, source, a snippet, the
/// `reasons` the engine ranked it where it did, and — for remembered notes —
/// who wrote it, so an agent can weigh a peer's lesson differently from a
/// human's (additive; the tool's input schema is unchanged).
fn render_hit(conn: &Connection, sm: &mw_memory::ScoredMemory) -> String {
    let (source, real_id) = mw_memory::sqlite::decode_id(sm.memory.id);
    let prov = match source {
        mw_memory::sqlite::Source::Note => note_provenance(conn, real_id)
            .map(|p| format!("\n  {p}"))
            .unwrap_or_default(),
        _ => String::new(),
    };
    let snippet: String = sm
        .memory
        .text
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim()
        .chars()
        .take(160)
        .collect();
    let reasons = sm.reasons();
    let reasons = if reasons.is_empty() {
        "(low-signal match)".to_string()
    } else {
        reasons.join("; ")
    };
    format!(
        "- [{} #{}] {}% — {}\n  reasons: {}{}\n",
        source.tag(),
        real_id,
        sm.percent(),
        snippet,
        reasons,
        prov
    )
}

/// Provenance for one remembered note, e.g. "remembered by Claude Code on
/// 2026-07-12 during session #41".
fn note_provenance(conn: &Connection, id: i64) -> Option<String> {
    conn.query_row(
        "SELECT author_kind, author_name, created_at, source_session_id
         FROM bookmarks WHERE id = ?1",
        params![id],
        |r| {
            Ok(memorywhale_cli::provenance_label(
                &r.get::<_, String>(0)?,
                r.get::<_, Option<String>>(1)?.as_deref(),
                &r.get::<_, Option<String>>(2)?.unwrap_or_default(),
                r.get::<_, Option<i64>>(3)?,
            ))
        },
    )
    .ok()
}

/// Rank all memories (commands, sessions, notes, …) for the query via the
/// explainable engine — the same loader + scorer the CLI and desktop use.
/// "now" is supplied by this caller so ranking is deterministic in tests.
fn search_memory(
    query: &str,
    project: Option<&str>,
    machine: Option<&str>,
) -> Result<String, String> {
    let conn = open()?;
    let now = Utc::now();
    // Unscoped (no project/machine) leaves the memory set untouched.
    let mems = memorywhale_cli::scope_memories(
        &conn,
        mw_memory::sqlite::load_memories(&conn),
        project,
        machine,
        None,
    );
    let engine = mw_memory::engine::BuiltinEngine::new(mems);
    let mut q = mw_memory::Query::new(query, now);
    let tags = task_tags(&[project, machine]);
    if !tags.is_empty() {
        q = q.with_task(tags);
    }
    let hits = engine.retrieve(&q, 20);
    if hits.is_empty() {
        return Ok(format!("(no matches for {query:?})"));
    }
    let mut out = String::new();
    for sm in &hits {
        out.push_str(&render_hit(&conn, sm));
    }
    Ok(out)
}

/// Non-empty scope values as engine task tags, so task-relevance scoring can
/// fire on memories that mention the project or machine.
fn task_tags(values: &[Option<&str>]) -> Vec<String> {
    values.iter().flatten().map(|s| s.to_string()).collect()
}

/// Optional string argument, treating empty/blank as absent.
fn scope_arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn get_context(project: Option<&str>, machine: Option<&str>) -> Result<String, String> {
    let conn = open()?;
    // Callers have always passed the tag form ("project:demo"); the column
    // holds the bare name, so accept either.
    let scope = project.map(|p| p.trim_start_matches("project:"));
    let mems = memorywhale_cli::scope_memories(
        &conn,
        mw_memory::sqlite::load_memories(&conn),
        scope.filter(|s| !s.is_empty()),
        machine,
        None,
    );
    let engine = mw_memory::engine::BuiltinEngine::new(mems);
    // Scope by project tag when given: it's both the query text and a task tag
    // so the task-relevance signal can fire when a memory carries the tag.
    let query = project.unwrap_or("");
    let mut q = mw_memory::Query::new(query, Utc::now());
    let tags = task_tags(&[project, machine]);
    if !tags.is_empty() {
        q = q.with_task(tags);
    }
    let hits = engine.retrieve(&q, 8);
    let mut out = String::from("Most relevant memory (engine-ranked):\n");
    if hits.is_empty() {
        out.push_str("(none)\n");
        return Ok(out);
    }
    for sm in &hits {
        out.push_str(&render_hit(&conn, sm));
    }
    Ok(out)
}

fn remember_tool(text: &str, client_name: Option<&str>) -> Result<String, String> {
    let id = memorywhale_cli::remember_as(text, None, "agent", client_name, None)?;
    Ok(format!(
        "Saved as memory #{id}. Future search_memory/get_context calls (yours or a teammate's) will find it."
    ))
}

/// Last non-empty line, char-capped (safe on UTF-8).
fn last_line(text: &str, max: usize) -> String {
    let t = text
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim();
    let chars: Vec<char> = t.chars().collect();
    if chars.len() > max {
        format!("…{}", chars[chars.len() - max..].iter().collect::<String>())
    } else {
        t.to_string()
    }
}
