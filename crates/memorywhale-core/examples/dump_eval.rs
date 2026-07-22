// crates/memorywhale-core/examples/dump_eval.rs
//
// Build an eval_data.json *skeleton* from your real memorywhale.sqlite3, so the
// eval benchmark (examples/eval.rs) runs against your actual memory instead of a
// hand-written dataset — where you'd unconsciously write queries your scorer
// already wins.
//
//   cargo run -p memorywhale-core --example dump_eval > eval_data.json
//   cargo run -p memorywhale-core --example dump_eval -- --db /path/to/memorywhale.sqlite3 --limit 100 --queries 8
//
// It emits every memory (remembered lessons + command runs) fully, and a set of
// EMPTY query stubs. You then fill in each query's `text` and, crucially, its
// `relevant_ids` — the human-labeled part that makes the P@5/nDCG numbers mean
// something. The memory id → text map is printed to stderr to help you label.
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::Connection;
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize)]
struct Dataset {
    memories: Vec<MemSpec>,
    queries: Vec<QueryStub>,
}
#[derive(Serialize)]
struct MemSpec {
    id: i64,
    text: String,
    tags: Vec<String>,
    importance: f32,
    mentions: u32,
    days_old: i64,
}
#[derive(Serialize)]
struct QueryStub {
    text: String,
    task_tags: Vec<String>,
    relevant_ids: Vec<i64>,
}

fn default_db_path() -> Option<PathBuf> {
    if let Some(dir) = std::env::var_os("MEMORYWHALE_DATA_DIR") {
        return Some(PathBuf::from(dir).join("memorywhale.sqlite3"));
    }
    let base = dirs::data_local_dir().or_else(dirs::home_dir)?;
    Some(base.join("MemoryWhale").join("memorywhale.sqlite3"))
}

fn days_old(ts: &str, now: DateTime<Utc>) -> i64 {
    DateTime::parse_from_rfc3339(ts)
        .map(|d| (now - d.with_timezone(&Utc)).num_days().max(0))
        .unwrap_or(0)
}

/// The `project:<name>` tag from a notes string, if present.
fn project_tag(notes: &str) -> Option<String> {
    notes
        .split_whitespace()
        .find_map(|w| w.strip_prefix("project:").map(|p| p.to_string()))
        .filter(|p| !p.is_empty())
}

/// Last non-empty line, char-capped.
fn tail(text: &str, max: usize) -> String {
    let t = text
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("")
        .trim();
    let chars: Vec<char> = t.chars().collect();
    if chars.len() > max {
        chars[chars.len() - max..].iter().collect()
    } else {
        t.to_string()
    }
}

fn main() -> Result<()> {
    let mut db: Option<PathBuf> = None;
    let mut limit: i64 = 200;
    let mut num_queries: usize = 6;
    let mut args = std::env::args().skip(1);
    while let Some(a) = args.next() {
        match a.as_str() {
            "--db" => db = args.next().map(PathBuf::from),
            "--limit" => limit = args.next().and_then(|v| v.parse().ok()).unwrap_or(limit),
            "--queries" => {
                num_queries = args.next().and_then(|v| v.parse().ok()).unwrap_or(num_queries)
            }
            other => anyhow::bail!("unknown arg {other:?}; use --db PATH --limit N --queries K"),
        }
    }
    let path = db
        .or_else(default_db_path)
        .context("could not resolve a database path; pass --db PATH")?;
    let conn = Connection::open(&path)
        .with_context(|| format!("failed to open {}", path.display()))?;
    let now = Utc::now();

    let mut memories: Vec<MemSpec> = Vec::new();
    let mut mapping: Vec<(i64, String)> = Vec::new(); // (id, short text) for stderr
    let mut next_id: i64 = 1;

    // Remembered lessons (bookmarks) first — the highest-value memories. The
    // table may not exist on a DB that's only seen command runs.
    if let Ok(mut stmt) =
        conn.prepare("SELECT label, created_at FROM bookmarks ORDER BY id DESC")
    {
        if let Ok(rows) = stmt.query_map([], |r| {
            Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))
        }) {
            for (label, created_at) in rows.flatten() {
                let short = label.chars().take(64).collect::<String>();
                memories.push(MemSpec {
                    id: next_id,
                    text: label,
                    tags: vec!["lesson".into()],
                    importance: 0.7,
                    mentions: 1,
                    days_old: days_old(&created_at, now),
                });
                mapping.push((next_id, short));
                next_id += 1;
            }
        }
    }

    // Command runs. `mentions` = how many runs share the same command (a real
    // reinforcement signal); tags come from the project: note tag if present.
    let mut stmt = conn
        .prepare(
            "SELECT argv_json, exit_code, stderr, notes, created_at,
                    (SELECT COUNT(*) FROM command_runs c2 WHERE c2.command = command_runs.command)
             FROM command_runs ORDER BY id DESC LIMIT ?1",
        )
        .context("failed to query command_runs (is this a MemoryWhale database?)")?;
    let rows = stmt
        .query_map([limit], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, Option<i64>>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
                r.get::<_, String>(4)?,
                r.get::<_, i64>(5)?,
            ))
        })
        .context("failed to read command_runs")?;
    for row in rows {
        let (argv_json, exit_code, stderr, notes, created_at, freq) =
            row.context("row error")?;
        let argv: Vec<String> = serde_json::from_str(&argv_json).unwrap_or_default();
        let failed = matches!(exit_code, Some(c) if c != 0);
        // Memory text = the command, plus the error tail if it failed (that's
        // the searchable signal a query would match on).
        let mut text = argv.join(" ");
        if failed {
            let t = tail(&stderr, 160);
            if !t.is_empty() {
                text.push_str(" — ");
                text.push_str(&t);
            }
        }
        let mut tags = Vec::new();
        if let Some(p) = project_tag(&notes) {
            tags.push(p);
        }
        tags.push(if failed { "failed".into() } else { "ok".into() });

        let short = text.chars().take(64).collect::<String>();
        memories.push(MemSpec {
            id: next_id,
            text,
            tags,
            importance: if failed { 0.6 } else { 0.3 },
            mentions: freq.max(1) as u32,
            days_old: days_old(&created_at, now),
        });
        mapping.push((next_id, short));
        next_id += 1;
    }

    // Empty query stubs for you to fill in.
    let queries: Vec<QueryStub> = (0..num_queries)
        .map(|_| QueryStub {
            text: String::new(),
            task_tags: Vec::new(),
            relevant_ids: Vec::new(),
        })
        .collect();

    let ds = Dataset { memories, queries };
    println!("{}", serde_json::to_string_pretty(&ds)?);

    eprintln!(
        "\n[dump_eval] {} memories + {} empty query stubs from {}",
        ds.memories.len(),
        num_queries,
        path.display()
    );
    eprintln!(
        "[dump_eval] Now edit the JSON: for each query set `text` (a natural-language\n\
         [dump_eval] question) and `relevant_ids` (the memory ids that *should* rank for it).\n\
         [dump_eval] That relevance labeling is the human part — it's what makes the numbers honest.\n\
         [dump_eval] Memory id -> text:"
    );
    for (id, t) in &mapping {
        eprintln!("  {id}: {t}");
    }
    Ok(())
}
