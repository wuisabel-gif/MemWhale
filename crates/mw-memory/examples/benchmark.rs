//! Reproducible, offline recall benchmark for the built-in scorer.
//!
//!   cargo run -p mw-memory --example benchmark -- benchmarks/
//!
//! No network, no API keys, no embedding downloads. Everything is deterministic:
//! a fixed corpus, a fixed "now" for recency, and a fixed question set with
//! hand-labeled relevant ids. Re-running regenerates benchmarks/results/*.json
//! byte-identically.
//!
//! Compares three retrieval systems over the same corpus:
//!   1. builtin  — mw-memory's BuiltinEngine (default weights; similarity is now
//!                 SQLite FTS5 BM25 blended with recency/importance/reinforcement/task).
//!   2. keyword  — a plain substring/keyword-overlap baseline.
//!   3. fts5     — an in-memory SQLite FTS5 (bm25) index over the same text.
//!
//! Two gold sets, each written to its own results directory:
//!   - questions.json        → results/         — pure *term-overlap* recall
//!     (the set the lexical baselines are built to win).
//!   - questions_intent.json → results_intent/  — *intent* recall that needs
//!     recency/importance/reinforcement/task context (the set the blend wins).
//!
//! (No fourth "local embedding" row: embed.rs only ships an Ollama-backed
//! embedder, which needs a running local server — not offline/deterministic —
//! so it is intentionally excluded from the committed numbers.)

use std::collections::BTreeSet;

use chrono::{DateTime, Duration, TimeZone, Utc};
use mw_memory::engine::{BuiltinEngine, MemoryEngine};
use mw_memory::{Memory, Query};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

// Fixed "now" so recency scoring (and therefore ranking) is reproducible.
fn fixed_now() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2026, 7, 19, 12, 0, 0).unwrap()
}

// ── dataset ────────────────────────────────────────────────────────────────
#[derive(Deserialize)]
struct Corpus {
    memories: Vec<MemSpec>,
}
#[derive(Deserialize)]
struct MemSpec {
    id: i64,
    text: String,
    tags: Vec<String>,
    importance: f32,
    mentions: u32,
    days_old: i64,
}
#[derive(Deserialize)]
struct Questions {
    questions: Vec<QuerySpec>,
}
#[derive(Deserialize)]
struct QuerySpec {
    id: String,
    text: String,
    #[serde(default)]
    task_tags: Vec<String>,
    relevant_ids: Vec<i64>,
}

// ── per-question output (committed under results/) ───────────────────────────
#[derive(Serialize)]
struct SystemResult {
    system: &'static str,
    top5: Vec<i64>,
    recall_at_1: f64,
    recall_at_5: f64,
    reciprocal_rank: f64,
}
#[derive(Serialize)]
struct QuestionResult {
    query_id: String,
    query: String,
    relevant_ids: Vec<i64>,
    systems: Vec<SystemResult>,
}

// ── metrics (binary relevance over a full ranking) ───────────────────────────
fn recall_at_k(ranked: &[i64], rel: &BTreeSet<i64>, k: usize) -> f64 {
    if rel.is_empty() {
        return 0.0;
    }
    let hits = ranked.iter().take(k).filter(|id| rel.contains(id)).count();
    hits as f64 / rel.len() as f64
}
fn reciprocal_rank(ranked: &[i64], rel: &BTreeSet<i64>) -> f64 {
    ranked
        .iter()
        .position(|id| rel.contains(id))
        .map(|i| 1.0 / (i as f64 + 1.0))
        .unwrap_or(0.0)
}

// ── rankers: each returns the full corpus ranked best→worst by id ────────────

fn rank_builtin(memories: &[Memory], q: &QuerySpec, now: DateTime<Utc>) -> Vec<i64> {
    let query = if q.task_tags.is_empty() {
        Query::new(&q.text, now)
    } else {
        Query::new(&q.text, now).with_task(q.task_tags.clone())
    };
    let eng = BuiltinEngine::new(memories.to_vec());
    eng.retrieve(&query, memories.len())
        .into_iter()
        .map(|s| s.memory.id)
        .collect()
}

/// Plain baseline: rank by how many distinct query terms appear (substring) in
/// the memory text. Ties break by id (ascending) so the ranking is stable.
fn rank_keyword(memories: &[Memory], q: &QuerySpec) -> Vec<i64> {
    let terms = terms(&q.text);
    let mut scored: Vec<(usize, i64)> = memories
        .iter()
        .map(|m| {
            let hay = m.text.to_lowercase();
            let hits = terms.iter().filter(|t| hay.contains(*t)).count();
            (hits, m.id)
        })
        .collect();
    // Higher hit count first; then smaller id first (deterministic tie-break).
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, id)| id).collect()
}

/// In-memory SQLite FTS5 (bm25). Ranks by relevance; unmatched rows are appended
/// by id so the ranking covers the whole corpus (matches the other two systems).
fn rank_fts5(conn: &Connection, memories: &[Memory], q: &QuerySpec) -> rusqlite::Result<Vec<i64>> {
    let terms = terms(&q.text);
    // Build a safe OR-of-quoted-terms MATCH string; quoting treats each token as
    // a literal, so FTS5 special chars can never break the query syntax.
    let match_expr = terms
        .iter()
        .map(|t| format!("\"{}\"", t.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" OR ");

    let mut ranked: Vec<i64> = Vec::new();
    if !match_expr.is_empty() {
        let mut stmt =
            conn.prepare("SELECT rowid FROM mem_fts WHERE mem_fts MATCH ?1 ORDER BY bm25(mem_fts), rowid")?;
        let rows = stmt.query_map([&match_expr], |r| r.get::<_, i64>(0))?;
        for id in rows {
            ranked.push(id?);
        }
    }
    // Append the rest of the corpus (no match) in id order so recall/MRR are
    // always defined over the full set.
    let matched: BTreeSet<i64> = ranked.iter().copied().collect();
    for m in memories {
        if !matched.contains(&m.id) {
            ranked.push(m.id);
        }
    }
    Ok(ranked)
}

/// Shared tokenizer for the two lexical baselines: lowercase alphanumeric tokens
/// of length >= 2. Kept deliberately dumb — that is the point of a baseline.
fn terms(text: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    text.split(|c: char| !c.is_alphanumeric())
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() >= 2)
        .filter(|w| seen.insert(w.clone()))
        .collect()
}

#[derive(Serialize)]
struct Row {
    system: String,
    recall_at_1: f64,
    recall_at_5: f64,
    mrr: f64,
}

/// Run one gold set end-to-end: score every question with all three systems,
/// write per-question JSON + summary.json under `results_subdir`, and return the
/// averaged rows for the printed table.
fn run_set(
    dir: &str,
    results_subdir: &str,
    questions_file: &str,
    conn: &Connection,
    memories: &[Memory],
    now: DateTime<Utc>,
) -> anyhow::Result<Vec<Row>> {
    let questions: Questions =
        serde_json::from_str(&std::fs::read_to_string(format!("{dir}/{questions_file}"))?)?;

    let results_dir = format!("{dir}/{results_subdir}");
    std::fs::create_dir_all(&results_dir)?;

    const SYSTEMS: [&str; 3] = ["builtin", "keyword", "fts5"];
    let mut sum_r1 = [0.0f64; 3];
    let mut sum_r5 = [0.0f64; 3];
    let mut sum_mrr = [0.0f64; 3];

    for q in &questions.questions {
        let rel: BTreeSet<i64> = q.relevant_ids.iter().copied().collect();
        let rankings = [
            rank_builtin(memories, q, now),
            rank_keyword(memories, q),
            rank_fts5(conn, memories, q)?,
        ];

        let mut systems = Vec::with_capacity(3);
        for (i, ranked) in rankings.iter().enumerate() {
            let r1 = recall_at_k(ranked, &rel, 1);
            let r5 = recall_at_k(ranked, &rel, 5);
            let rr = reciprocal_rank(ranked, &rel);
            sum_r1[i] += r1;
            sum_r5[i] += r5;
            sum_mrr[i] += rr;
            systems.push(SystemResult {
                system: SYSTEMS[i],
                top5: ranked.iter().take(5).copied().collect(),
                recall_at_1: r1,
                recall_at_5: r5,
                reciprocal_rank: rr,
            });
        }

        let out = QuestionResult {
            query_id: q.id.clone(),
            query: q.text.clone(),
            relevant_ids: q.relevant_ids.clone(),
            systems,
        };
        // Pretty + trailing newline → stable, diff-friendly, byte-identical.
        let json = serde_json::to_string_pretty(&out)? + "\n";
        std::fs::write(format!("{results_dir}/{}.json", q.id), json)?;
    }

    let n = questions.questions.len() as f64;
    let rows: Vec<Row> = (0..3)
        .map(|i| Row {
            system: SYSTEMS[i].to_string(),
            recall_at_1: sum_r1[i] / n,
            recall_at_5: sum_r5[i] / n,
            mrr: sum_mrr[i] / n,
        })
        .collect();

    #[derive(Serialize)]
    struct Summary<'a> {
        corpus_items: usize,
        questions: usize,
        fixed_now: String,
        rows: &'a [Row],
    }
    let summary = Summary {
        corpus_items: memories.len(),
        questions: questions.questions.len(),
        fixed_now: now.to_rfc3339(),
        rows: &rows,
    };
    std::fs::write(
        format!("{results_dir}/summary.json"),
        serde_json::to_string_pretty(&summary)? + "\n",
    )?;
    Ok(rows)
}

fn print_table(label: &str, questions: usize, now: DateTime<Utc>, rows: &[Row]) {
    println!(
        "\n{label} · {} queries · now={}\n",
        questions,
        now.date_naive()
    );
    println!("{:<10} {:>10} {:>10} {:>8}", "system", "recall@1", "recall@5", "MRR");
    for r in rows {
        println!(
            "{:<10} {:>10.3} {:>10.3} {:>8.3}",
            r.system, r.recall_at_1, r.recall_at_5, r.mrr
        );
    }
}

fn main() -> anyhow::Result<()> {
    let dir = std::env::args().nth(1).unwrap_or_else(|| "benchmarks".into());
    let dir = dir.trim_end_matches('/');
    let corpus: Corpus = serde_json::from_str(&std::fs::read_to_string(format!("{dir}/corpus.json"))?)?;

    let now = fixed_now();
    let memories: Vec<Memory> = corpus
        .memories
        .iter()
        .map(|m| Memory {
            id: m.id,
            text: m.text.clone(),
            created_at: now - Duration::days(m.days_old + 10),
            last_used: now - Duration::days(m.days_old),
            mentions: m.mentions,
            importance: m.importance,
            tags: m.tags.clone(),
            embedding: None,
        })
        .collect();

    // Build the FTS5 index once (deterministic: insert in corpus order).
    let conn = Connection::open_in_memory()?;
    conn.execute("CREATE VIRTUAL TABLE mem_fts USING fts5(text)", [])?;
    {
        let mut ins = conn.prepare("INSERT INTO mem_fts(rowid, text) VALUES (?1, ?2)")?;
        for m in &memories {
            ins.execute(rusqlite::params![m.id, m.text])?;
        }
    }

    // Set 1: pure term-overlap gold set → results/.
    let text_rows = run_set(dir, "results", "questions.json", &conn, &memories, now)?;
    // Set 2: intent gold set → results_intent/.
    let intent_rows = run_set(dir, "results_intent", "questions_intent.json", &conn, &memories, now)?;

    let count = |file: &str| -> usize {
        std::fs::read_to_string(format!("{dir}/{file}"))
            .ok()
            .and_then(|s| serde_json::from_str::<Questions>(&s).ok())
            .map(|q| q.questions.len())
            .unwrap_or(0)
    };

    println!("recall benchmark · {} memories · now={}", memories.len(), now.date_naive());
    print_table("term-overlap set (results/)", count("questions.json"), now, &text_rows);
    print_table("intent set (results_intent/)", count("questions_intent.json"), now, &intent_rows);
    println!("\nper-question results written to {dir}/results/*.json and {dir}/results_intent/*.json");
    Ok(())
}
