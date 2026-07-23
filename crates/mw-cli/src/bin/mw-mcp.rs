// mw-mcp: a Model Context Protocol server over stdio, so an AI agent (Claude
// Code, Codex, Cursor, …) can query your MemoryWhale memory directly instead of
// pasting it in. Speaks newline-delimited JSON-RPC 2.0 on stdin/stdout.
//
// Register with Claude Code:
//   claude mcp add memorywhale -- mw-mcp
//
// Tools exposed: recent_errors, search_memory, get_context, remember,
// similar_failures, stats.

use chrono::Utc;
use memorywhale_core::engine::MemoryEngine;
use rusqlite::{params, Connection};
use serde_json::{json, Value};
use std::io::{BufRead, Write};

fn main() {
    if std::env::args().nth(1).as_deref() == Some("--list-tools") {
        for tool in tool_defs().as_array().into_iter().flatten() {
            if let Some(name) = tool.get("name").and_then(Value::as_str) {
                println!("{name}");
            }
        }
        return;
    }
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
        },
        {
            "name": "similar_failures",
            "description": "Check whether an error you just hit has occurred before, and whether a later run resolved it. Pass the error output (and the command that produced it, if you have it) and get an evidence-only history — how many times this exact failure was seen and how often a later run of the same command succeeded — plus a pointer to a concrete past occurrence to go look at. Pass `command` for an exact fingerprint match; without it we fall back to a best-effort text match.",
            "inputSchema": {"type": "object", "properties": {
                "error_text": {"type": "string", "description": "the error/stderr output you hit"},
                "command": {"type": "string", "description": "optional: the command that produced it, e.g. cargo build — enables an exact fingerprint match"}
            }, "required": ["error_text"]}
        },
        {
            "name": "stats",
            "description": "Health/liveness check for the memory store: confirm it's reachable and populated before relying on the other tools. Returns total memory count, how many are recorded failures, the most-recent memory timestamp (or \"none\"), and the database file path.",
            "inputSchema": {"type": "object", "properties": {}}
        }
    ])
}

fn open() -> Result<Connection, String> {
    memorywhale_cli::storage::open()
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
        "similar_failures" => {
            let error_text = args
                .get("error_text")
                .and_then(Value::as_str)
                .ok_or_else(|| "similar_failures needs an 'error_text'".to_string())?;
            let conn = open()?;
            Ok(similar_failures_report(&conn, error_text, scope_arg(args, "command")))
        }
        "stats" => stats(),
        other => Err(format!("unknown tool: {other}")),
    }
}

fn recent_errors(limit: i64) -> Result<String, String> {
    let conn = open()?;
    // A brand-new store has only the bookmarks table; `command_runs` appears
    // once something is recorded. Treat its absence as "no failures yet" (same
    // graceful-read convention as `load_memories`) rather than erroring.
    let Ok(mut stmt) = conn.prepare(
        "SELECT argv_json, cwd, exit_code, stderr, notes, created_at
             FROM command_runs
             WHERE exit_code IS NOT NULL AND exit_code != 0
             ORDER BY id DESC LIMIT ?1",
    ) else {
        return Ok("(no failed commands recorded)".to_string());
    };
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
fn render_hit(conn: &Connection, sm: &memorywhale_core::ScoredMemory) -> String {
    let (source, real_id) = memorywhale_core::sqlite::decode_id(sm.memory.id);
    let prov = match source {
        memorywhale_core::sqlite::Source::Note => note_provenance(conn, real_id)
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
        memorywhale_core::sqlite::load_memories(&conn),
        project,
        machine,
        None,
    );
    let engine = memorywhale_core::engine::BuiltinEngine::new(mems);
    let mut q = memorywhale_core::Query::new(query, now);
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
        memorywhale_core::sqlite::load_memories(&conn),
        scope.filter(|s| !s.is_empty()),
        machine,
        None,
    );
    let engine = memorywhale_core::engine::BuiltinEngine::new(mems);
    // Scope by project tag when given: it's both the query text and a task tag
    // so the task-relevance signal can fire when a memory carries the tag.
    let query = project.unwrap_or("");
    let mut q = memorywhale_core::Query::new(query, Utc::now());
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

/// Liveness/health summary an agent can call to confirm the store is reachable
/// and populated. Reads only counts and timestamps — never memory text — so it
/// leaks nothing beyond how much is stored and when it was last written.
fn stats() -> Result<String, String> {
    let path = memorywhale_cli::database_path()?;
    let conn = open()?;
    Ok(stats_summary(&conn, &path.display().to_string()))
}

/// The stats payload as compact JSON. Counts are cheap (`COUNT(*)`, `MAX`) and
/// tolerate a fresh DB where `command_runs` doesn't exist yet: a failed query
/// (missing table) reads as zero / "none" rather than an error.
fn stats_summary(conn: &Connection, db_path: &str) -> String {
    let count = |sql: &str| conn.query_row(sql, [], |r| r.get::<_, i64>(0)).unwrap_or(0);
    let max_ts = |sql: &str| {
        conn.query_row(sql, [], |r| r.get::<_, Option<String>>(0))
            .ok()
            .flatten()
    };
    let memories =
        count("SELECT COUNT(*) FROM command_runs") + count("SELECT COUNT(*) FROM bookmarks");
    let errors =
        count("SELECT COUNT(*) FROM command_runs WHERE exit_code IS NOT NULL AND exit_code != 0");
    // ISO-8601 timestamps sort lexicographically, so the string max is the most
    // recent across both writable sources.
    let latest = [
        max_ts("SELECT MAX(created_at) FROM command_runs"),
        max_ts("SELECT MAX(created_at) FROM bookmarks"),
    ]
    .into_iter()
    .flatten()
    .max()
    .unwrap_or_else(|| "none".to_string());
    json!({
        "memories": memories,
        "errors": errors,
        "latest": latest,
        "db": db_path,
    })
    .to_string()
}

/// Report whether an error the agent just hit has been seen before.
/// With a `command` we fingerprint exactly like the stored rows and read the
/// evidence-only insight (occurrences + how often a later run resolved it).
/// Without one — or on a fingerprint miss — we fall back to matching the salient
/// stderr line against past failures with LIKE, and say so plainly.
fn similar_failures_report(conn: &Connection, error_text: &str, command: Option<&str>) -> String {
    // Exact path only helps when we know the command: stored fingerprints were
    // computed WITH the real command.
    if let Some(cmd) = command {
        if let Some(fp) = memorywhale_cli::error_fingerprint(cmd, error_text) {
            if let Ok(insight) = memorywhale_cli::error_insight(conn, &fp) {
                if insight.occurrences > 0 {
                    let mut out = insight.summary();
                    if let Some(ptr) = occurrence_pointer(conn, &fp) {
                        out.push('\n');
                        out.push_str(&ptr);
                    }
                    return out;
                }
            }
        }
    }
    // Fallback: best-effort text match on the salient stderr line.
    match salient_error_line(error_text) {
        Some(line) => like_fallback(conn, line, command.is_none()),
        None => "(no error text to match on)".to_string(),
    }
}

/// The stderr line the fingerprint keys on: first line naming an error keyword,
/// else the first non-empty line. Mirrors `error_fingerprint`'s selection.
fn salient_error_line(stderr: &str) -> Option<&str> {
    stderr
        .lines()
        .find(|l| {
            let l = l.to_lowercase();
            ["error", "failed", "fatal", "cannot", "no such", "not found", "panic", "exception"]
                .iter()
                .any(|kw| l.contains(kw))
        })
        .or_else(|| stderr.lines().find(|l| !l.trim().is_empty()))
        .map(str::trim)
}

/// A concrete past occurrence to go look at: the most recent run sharing this
/// fingerprint, by command_run id.
fn occurrence_pointer(conn: &Connection, fingerprint: &str) -> Option<String> {
    conn.query_row(
        "SELECT id, command, cwd FROM command_runs
         WHERE error_fingerprint = ?1 ORDER BY id DESC LIMIT 1",
        params![fingerprint],
        |r| {
            Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, Option<String>>(2)?,
            ))
        },
    )
    .ok()
    .map(|(id, cmd, cwd)| {
        let where_ = cwd
            .filter(|c| !c.trim().is_empty())
            .map(|c| format!(" (cwd: {c})"))
            .unwrap_or_default();
        format!("See command_run #{id}: `{cmd}`{where_}")
    })
}

/// Count failed runs whose stderr contains `line`, and point at the most recent.
/// A text match, not a fingerprint — the report says so.
fn like_fallback(conn: &Connection, line: &str, no_command: bool) -> String {
    let pattern = format!("%{}%", like_escape(line));
    let hit = conn.query_row(
        "SELECT COUNT(*), MAX(id) FROM command_runs
         WHERE exit_code IS NOT NULL AND exit_code != 0
           AND stderr LIKE ?1 ESCAPE '\\'",
        params![pattern],
        |r| Ok((r.get::<_, i64>(0)?, r.get::<_, Option<i64>>(1)?)),
    );
    let (count, last_id) = match hit {
        Ok(v) => v,
        Err(e) => return format!("(failed to search past failures: {e})"),
    };
    let why = if no_command {
        " (no command given, so this is a best-effort text match, not a fingerprint)"
    } else {
        " (no fingerprint match, so this is a best-effort text match)"
    };
    if count == 0 {
        return format!("Never seen this failure before{why}.");
    }
    let ptr = last_id
        .and_then(|id| {
            conn.query_row(
                "SELECT command, cwd FROM command_runs WHERE id = ?1",
                params![id],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, Option<String>>(1)?)),
            )
            .ok()
            .map(|(cmd, cwd)| {
                let where_ = cwd
                    .filter(|c| !c.trim().is_empty())
                    .map(|c| format!(" (cwd: {c})"))
                    .unwrap_or_default();
                format!("\nMost recent: command_run #{id}: `{cmd}`{where_}")
            })
        })
        .unwrap_or_default();
    let times = if count == 1 { "once".to_string() } else { format!("{count} times") };
    format!("Seen a similar failure {times} by stderr text{why}.{ptr}")
}

/// Escape LIKE wildcards so a stderr line matches literally (ESCAPE '\').
fn like_escape(s: &str) -> String {
    s.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_")
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

#[cfg(test)]
mod tests {
    use super::*;

    /// A repeated-then-resolved-then-regressed timeline: the tool's formatted
    /// output must report the occurrence + resolution counts and point at a
    /// concrete run. Mirrors lib.rs's `insight_counts_occurrences_and_resolutions`.
    #[test]
    fn similar_failures_reports_counts_and_pointer() {
        let conn = Connection::open_in_memory().unwrap();
        memorywhale_cli::storage::initialize(&conn).unwrap();
        let err = "error[E0308]: mismatched types";
        let fp = memorywhale_cli::error_fingerprint("cargo", err).unwrap();
        let insert = |exit: i64, fp: Option<&str>| {
            conn.execute(
                "INSERT INTO command_runs
                    (command, argv_json, cwd, exit_code, stderr, error_fingerprint, created_at)
                 VALUES ('cargo', '[\"cargo\"]', '/tmp/proj', ?1, ?2, ?3, '')",
                params![exit, err, fp],
            )
            .unwrap();
        };
        // fail, fail, success (the fix), fail again.
        insert(101, Some(&fp));
        insert(101, Some(&fp));
        insert(0, None);
        insert(101, Some(&fp));

        // Exact path (command supplied): counts come from the fingerprint insight.
        let report = similar_failures_report(&conn, err, Some("cargo"));
        assert!(report.contains("3 times"), "occurrences: {report}");
        assert!(report.contains("2 of 3"), "resolutions: {report}");
        assert!(report.contains("command_run #"), "pointer: {report}");

        // Never-seen error is reported plainly.
        let miss = similar_failures_report(&conn, "error: something brand new", Some("cargo"));
        assert!(miss.to_lowercase().contains("never seen"), "miss: {miss}");
    }

    /// stats reflects what's in the store: total count, error subset, latest ts,
    /// and the db path — and an empty DB returns zeros/"none" without erroring.
    #[test]
    fn stats_summarizes_counts_and_empty_db() {
        // Empty DB: no tables at all → graceful zeros and "none".
        let empty = Connection::open_in_memory().unwrap();
        let out = stats_summary(&empty, "/tmp/mw.sqlite3");
        let v: Value = serde_json::from_str(&out).unwrap();
        assert_eq!(v["memories"], 0);
        assert_eq!(v["errors"], 0);
        assert_eq!(v["latest"], "none");
        assert_eq!(v["db"], "/tmp/mw.sqlite3");

        // Populated: two runs, one a failure.
        let conn = Connection::open_in_memory().unwrap();
        memorywhale_cli::storage::initialize(&conn).unwrap();
        conn.execute_batch(
            "INSERT INTO command_runs (command, argv_json, exit_code, created_at)
                 VALUES ('cargo build', '[]', 0, '2026-07-20T10:00:00Z');
             INSERT INTO command_runs (command, argv_json, exit_code, created_at)
                 VALUES ('cargo build', '[]', 101, '2026-07-21T10:00:00Z');",
        )
        .unwrap();
        let v: Value = serde_json::from_str(&stats_summary(&conn, "/tmp/mw.sqlite3")).unwrap();
        assert_eq!(v["memories"], 2);
        assert_eq!(v["errors"], 1);
        assert_eq!(v["latest"], "2026-07-21T10:00:00Z");
    }
}
