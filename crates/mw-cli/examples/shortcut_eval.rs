//! Deterministic, offline **memory-shortcut** eval: what fraction of *recurring*
//! failures have their fix already retrievable via MemoryWhale's real MCP tools?
//!
//!   cargo run -p memorywhale-cli --example shortcut_eval -- benchmarks/
//!
//! No network, no LLM, no API keys. Everything is fixed: a fixed `now`, the
//! corpus in file order, the task set in file order. Re-running regenerates
//! benchmarks/shortcut_results/*.json byte-identically.
//!
//! This is a **proxy** for agent solve-rate — a *retrieval ceiling*. It measures
//! whether the fix is SURFACED by the tools, not whether an agent then uses it to
//! solve faster (that's the follow-up real-agent eval). The "without MCP" / cold
//! agent baseline is 0 by construction: with no memory there is nothing to
//! retrieve, so every recurring failure is re-derived from scratch.
//!
//! Two retrieval conditions per task, exercising the SAME paths as mw-mcp:
//!   * similar_failures — fingerprint(`command`+`error_text`) → error_insight.
//!     "Recognized" when the recurrence's fingerprint is found; "resolved" when a
//!     later successful run of the same command is on record. A shortcut when both
//!     hold: the tool tells the agent "you've hit — and fixed — this before."
//!   * search_memory — the `query` through the BuiltinEngine-ranked load path
//!     (identical to `mw search` / the MCP tool). Hit@k when a labelled `fix_id`
//!     note lands in the top-k.

use std::error::Error;

use chrono::{DateTime, Duration, TimeZone, Utc};
use memorywhale_cli as mw;
use memorywhale_core::engine::{BuiltinEngine, MemoryEngine};
use memorywhale_core::sqlite::{decode_id, load_memories, Source};
use memorywhale_core::Query;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

/// Fixed "now" — same instant the recall benchmark uses, so recency is reproducible.
fn fixed_now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 19, 12, 0, 0).unwrap()
}

// ── inputs ───────────────────────────────────────────────────────────────────
#[derive(Deserialize)]
struct Corpus {
    memories: Vec<MemSpec>,
}
#[derive(Deserialize)]
struct MemSpec {
    id: i64,
    text: String,
    days_old: i64,
}
#[derive(Deserialize)]
struct TaskSet {
    tasks: Vec<Task>,
}
#[derive(Deserialize)]
struct Task {
    id: String,
    command: String,
    stored_error: String,
    error_text: String,
    query: String,
    fix_ids: Vec<i64>,
}

// ── per-task output (committed under shortcut_results/) ───────────────────────
#[derive(Serialize)]
struct TaskResult {
    task_id: String,
    command: String,
    query: String,
    fix_ids: Vec<i64>,
    // similar_failures path (fingerprint → error_insight). Rank-free: hit is binary.
    fingerprint: Option<String>,
    occurrences: i64,
    resolutions: i64,
    similar_failures_recognized: bool,
    similar_failures_shortcut: bool,
    // search_memory path (BuiltinEngine-ranked). Top-5 note ids for auditability.
    search_top5_notes: Vec<i64>,
    search_shortcut_at_1: bool,
    search_shortcut_at_5: bool,
    // combined: fix surfaced by at least one tool.
    combined_at_1: bool,
    combined_at_5: bool,
}

/// Build the corpus once into `bookmarks` (the real note store the loader reads).
/// Ids are the corpus ids so labelled `fix_ids` decode straight back.
fn seed_corpus(conn: &Connection, corpus: &Corpus, now: DateTime<Utc>) {
    for m in &corpus.memories {
        let created = (now - Duration::days(m.days_old)).to_rfc3339();
        conn.execute(
            "INSERT INTO bookmarks (id, label, cwd, created_at, author_kind, author_name, source_session_id, approved)
             VALUES (?1, ?2, NULL, ?3, 'human', NULL, NULL, 1)",
            params![m.id, m.text, created],
        )
        .expect("seed bookmark");
    }
}

/// Seed one recurring failure into `command_runs` so a real fingerprint exists:
/// two failing runs (the recurrence) and — only when a fix is on record — a later
/// successful run (the resolution). A never-fixed failure gets no success row, so
/// both tools honestly miss it.
fn seed_failure(conn: &Connection, task: &Task, now: DateTime<Utc>) {
    let fp = mw::error_fingerprint(&task.command, &task.stored_error);
    let insert = |exit: i64, stderr: &str, fp: Option<&String>, age: i64| {
        conn.execute(
            "INSERT INTO command_runs (command, argv_json, cwd, exit_code, stdout, stderr, notes, created_at, error_fingerprint)
             VALUES (?1, '[]', '/repo', ?2, '', ?3, '', ?4, ?5)",
            params![task.command, exit, stderr, (now - Duration::days(age)).to_rfc3339(), fp],
        )
        .expect("seed command_run");
    };
    insert(101, &task.stored_error, fp.as_ref(), 6);
    insert(101, &task.stored_error, fp.as_ref(), 5);
    if !task.fix_ids.is_empty() {
        insert(0, "", None, 4); // a later run of the same command succeeded
    }
}

/// Rank a query through the same loader + engine the CLI and MCP use, returning
/// the retrieved *note* ids (decoded) in engine order.
fn ranked_notes(conn: &Connection, query: &str, now: DateTime<Utc>) -> Vec<i64> {
    let engine = BuiltinEngine::new(load_memories(conn));
    engine
        .retrieve(&Query::new(query, now), 20)
        .into_iter()
        .filter_map(|s| match decode_id(s.memory.id) {
            (Source::Note, real) => Some(real),
            _ => None,
        })
        .collect()
}

#[derive(Serialize)]
struct Row {
    path: &'static str,
    shortcut_at_1: f64,
    shortcut_at_5: f64,
}

fn main() -> Result<(), Box<dyn Error>> {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "benchmarks".into());
    let dir = dir.trim_end_matches('/');
    let corpus: Corpus =
        serde_json::from_str(&std::fs::read_to_string(format!("{dir}/corpus.json"))?)?;
    let task_set: TaskSet = serde_json::from_str(&std::fs::read_to_string(format!(
        "{dir}/shortcut_tasks.json"
    ))?)?;
    let now = fixed_now();

    // One in-memory DB seeded via the real schema + migrate(), so every retrieval
    // below runs the production code path.
    let conn = Connection::open_in_memory()?;
    conn.execute_batch(
        "CREATE TABLE command_runs (
             id INTEGER PRIMARY KEY, command TEXT NOT NULL, argv_json TEXT NOT NULL DEFAULT '[]',
             cwd TEXT, exit_code INTEGER, stdout TEXT NOT NULL DEFAULT '',
             stderr TEXT NOT NULL DEFAULT '', notes TEXT NOT NULL DEFAULT '',
             created_at TEXT NOT NULL DEFAULT '', error_fingerprint TEXT);",
    )?;
    mw::migrate(&conn)?; // creates bookmarks (+ provenance)
    seed_corpus(&conn, &corpus, now);
    for task in &task_set.tasks {
        seed_failure(&conn, task, now);
    }

    let results_dir = format!("{dir}/shortcut_results");
    std::fs::create_dir_all(&results_dir)?;

    let (mut sf, mut sm1, mut sm5, mut c1, mut c5) = (0u32, 0u32, 0u32, 0u32, 0u32);
    let n = task_set.tasks.len();

    for task in &task_set.tasks {
        // similar_failures path.
        let fp = mw::error_fingerprint(&task.command, &task.error_text);
        let insight = fp
            .as_ref()
            .and_then(|f| mw::error_insight(&conn, f).ok())
            .unwrap_or(mw::ErrorInsight {
                fingerprint: fp.clone().unwrap_or_default(),
                occurrences: 0,
                resolutions: 0,
                currently_green: false,
            });
        let recognized = insight.occurrences > 0;
        let sf_shortcut = recognized && insight.resolutions > 0;

        // search_memory path.
        let ranked = ranked_notes(&conn, &task.query, now);
        let sm_at_1 = mw::fix_surfaced(&ranked, &task.fix_ids, 1);
        let sm_at_5 = mw::fix_surfaced(&ranked, &task.fix_ids, 5);

        let comb_1 = sf_shortcut || sm_at_1;
        let comb_5 = sf_shortcut || sm_at_5;

        sf += sf_shortcut as u32;
        sm1 += sm_at_1 as u32;
        sm5 += sm_at_5 as u32;
        c1 += comb_1 as u32;
        c5 += comb_5 as u32;

        let out = TaskResult {
            task_id: task.id.clone(),
            command: task.command.clone(),
            query: task.query.clone(),
            fix_ids: task.fix_ids.clone(),
            fingerprint: fp,
            occurrences: insight.occurrences,
            resolutions: insight.resolutions,
            similar_failures_recognized: recognized,
            similar_failures_shortcut: sf_shortcut,
            search_top5_notes: ranked.into_iter().take(5).collect(),
            search_shortcut_at_1: sm_at_1,
            search_shortcut_at_5: sm_at_5,
            combined_at_1: comb_1,
            combined_at_5: comb_5,
        };
        let json = serde_json::to_string_pretty(&out)? + "\n";
        std::fs::write(format!("{results_dir}/{}.json", task.id), json)?;
    }

    let d = n as f64;
    let rate = |x: u32| x as f64 / d;
    // similar_failures is rank-free (a single fingerprint lookup), so @1 == @5.
    let rows = [
        Row {
            path: "similar_failures",
            shortcut_at_1: rate(sf),
            shortcut_at_5: rate(sf),
        },
        Row {
            path: "search_memory",
            shortcut_at_1: rate(sm1),
            shortcut_at_5: rate(sm5),
        },
        Row {
            path: "combined",
            shortcut_at_1: rate(c1),
            shortcut_at_5: rate(c5),
        },
    ];

    #[derive(Serialize)]
    struct Summary<'a> {
        tasks: usize,
        fixable_tasks: usize,
        fixed_now: String,
        headline_combined_at_5: f64,
        note: &'a str,
        rows: &'a [Row],
    }
    let fixable = task_set
        .tasks
        .iter()
        .filter(|t| !t.fix_ids.is_empty())
        .count();
    let summary = Summary {
        tasks: n,
        fixable_tasks: fixable,
        fixed_now: now.to_rfc3339(),
        headline_combined_at_5: rate(c5),
        note: "Retrieval ceiling (proxy for agent solve-rate), not a real LLM-agent eval. \
               without-MCP baseline = 0 by construction. See benchmarks/SHORTCUT_EVAL.md.",
        rows: &rows,
    };
    std::fs::write(
        format!("{results_dir}/summary.json"),
        serde_json::to_string_pretty(&summary)? + "\n",
    )?;

    println!(
        "memory-shortcut eval · {n} tasks ({fixable} with a fix in corpus) · now={}",
        now.date_naive()
    );
    println!("{:<18} {:>10} {:>10}", "path", "shortcut@1", "shortcut@5");
    for r in &rows {
        println!(
            "{:<18} {:>10.3} {:>10.3}",
            r.path, r.shortcut_at_1, r.shortcut_at_5
        );
    }
    println!("\nHEADLINE: {:.0}% of recurring failures have their fix surfaced by MemoryWhale (combined@5).", rate(c5) * 100.0);
    println!("per-task results written to {results_dir}/*.json");
    Ok(())
}
