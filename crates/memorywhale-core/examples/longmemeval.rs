//! LongMemEval retrieval benchmark (session level), offline and deterministic.
//!
//!   cargo run --release -p memorywhale-core --example longmemeval -- \
//!       path/to/longmemeval_s.json benchmarks/longmemeval/
//!
//! LongMemEval (Wu et al., ICLR 2025, MIT) asks questions about a long chat
//! history. Each question comes with its own haystack of ~50 timestamped
//! sessions and labels the session(s) that hold the answer. This measures only
//! the retrieval step: does a ranker put an answer session near the top? No
//! model is called, so there is no reader accuracy here.
//!
//! Protocol (per question, abstention questions excluded as in the paper):
//!   * corpus = that question's haystack; one memory per session, text = every
//!     turn (user and assistant), created/last-used at the session's date;
//!   * query = the question, "now" = the question date;
//!   * metrics = recall_any@k (an answer session in the top k), recall_all@k
//!     (every answer session in the top k), and NDCG@10.
//!
//! Systems, all over the same corpus: builtin (default ranking), bayesian
//! (opt-in `Ranking::Bayesian`), fts5 (plain SQLite bm25), keyword (term overlap).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use memorywhale_core::engine::{BuiltinEngine, MemoryEngine};
use memorywhale_core::{Memory, Query, Ranking, Weights};
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
struct Turn {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct Item {
    question_id: String,
    question_type: String,
    question: String,
    question_date: String,
    haystack_dates: Vec<String>,
    haystack_session_ids: Vec<String>,
    haystack_sessions: Vec<Vec<Turn>>,
    answer_session_ids: Vec<String>,
}

const SYSTEMS: [&str; 6] = [
    "builtin",
    "bayesian",
    "no_recency",
    "similarity_only",
    "fts5",
    "keyword",
];

// Ablations of the default blend. Importance, reinforcement, and task are
// constant (or not applicable) here, so only similarity and recency can reorder.
fn no_recency() -> Weights {
    Weights {
        recency: 0.0,
        ..Weights::default()
    }
}
fn similarity_only() -> Weights {
    Weights {
        similarity: 1.0,
        recency: 0.0,
        importance: 0.0,
        reinforcement: 0.0,
        task: 0.0,
    }
}

#[derive(Default, Clone, Serialize)]
struct Score {
    n: usize,
    recall_any_5: f64,
    recall_all_5: f64,
    recall_any_10: f64,
    recall_all_10: f64,
    ndcg_10: f64,
}

impl Score {
    fn add(&mut self, ranked: &[i64], rel: &BTreeSet<i64>) {
        let hits = |k: usize| ranked.iter().take(k).filter(|id| rel.contains(id)).count();
        self.n += 1;
        self.recall_any_5 += (hits(5) > 0) as u8 as f64;
        self.recall_all_5 += (hits(5) == rel.len()) as u8 as f64;
        self.recall_any_10 += (hits(10) > 0) as u8 as f64;
        self.recall_all_10 += (hits(10) == rel.len()) as u8 as f64;
        let dcg: f64 = ranked
            .iter()
            .take(10)
            .enumerate()
            .filter(|(_, id)| rel.contains(id))
            .map(|(i, _)| 1.0 / ((i + 2) as f64).log2())
            .sum();
        let ideal: f64 = (0..rel.len().min(10))
            .map(|i| 1.0 / ((i + 2) as f64).log2())
            .sum();
        self.ndcg_10 += if ideal > 0.0 { dcg / ideal } else { 0.0 };
    }

    fn mean(&self) -> Score {
        let n = self.n.max(1) as f64;
        Score {
            n: self.n,
            recall_any_5: self.recall_any_5 / n,
            recall_all_5: self.recall_all_5 / n,
            recall_any_10: self.recall_any_10 / n,
            recall_all_10: self.recall_all_10 / n,
            ndcg_10: self.ndcg_10 / n,
        }
    }
}

// Dates look like "2023/05/20 (Sat) 02:21".
fn parse_date(s: &str) -> DateTime<Utc> {
    let naive = NaiveDateTime::parse_from_str(s, "%Y/%m/%d (%a) %H:%M")
        .unwrap_or_else(|e| panic!("unparseable LongMemEval date {s:?}: {e}"));
    Utc.from_utc_datetime(&naive)
}

fn terms(text: &str) -> Vec<String> {
    let mut seen = BTreeSet::new();
    text.split(|c: char| !c.is_alphanumeric())
        .map(str::to_lowercase)
        .filter(|t| t.chars().count() >= 2 && seen.insert(t.clone()))
        .collect()
}

fn rank_builtin(
    memories: &[Memory],
    q: &str,
    now: DateTime<Utc>,
    ranking: Ranking,
    weights: Weights,
) -> Vec<i64> {
    let query = Query::new(q, now).with_ranking(ranking);
    BuiltinEngine::new(memories.to_vec())
        .with_weights(weights)
        .retrieve(&query, memories.len())
        .into_iter()
        .map(|s| s.memory.id)
        .collect()
}

fn rank_keyword(memories: &[Memory], q: &str) -> Vec<i64> {
    let terms = terms(q);
    let mut scored: Vec<(usize, i64)> = memories
        .iter()
        .map(|m| {
            let hay = m.text.to_lowercase();
            (
                terms.iter().filter(|t| hay.contains(t.as_str())).count(),
                m.id,
            )
        })
        .collect();
    scored.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    scored.into_iter().map(|(_, id)| id).collect()
}

fn rank_fts5(memories: &[Memory], q: &str) -> rusqlite::Result<Vec<i64>> {
    let conn = Connection::open_in_memory()?;
    conn.execute_batch("CREATE VIRTUAL TABLE mem_fts USING fts5(text);")?;
    for m in memories {
        conn.execute(
            "INSERT INTO mem_fts(rowid, text) VALUES (?1, ?2)",
            (m.id, &m.text),
        )?;
    }
    let expr = terms(q)
        .iter()
        .map(|t| format!("\"{}\"", t.replace('"', "")))
        .collect::<Vec<_>>()
        .join(" OR ");
    let mut ranked = Vec::new();
    if !expr.is_empty() {
        let mut stmt = conn.prepare(
            "SELECT rowid FROM mem_fts WHERE mem_fts MATCH ?1 ORDER BY bm25(mem_fts), rowid",
        )?;
        for id in stmt.query_map([&expr], |r| r.get::<_, i64>(0))? {
            ranked.push(id?);
        }
    }
    let matched: BTreeSet<i64> = ranked.iter().copied().collect();
    ranked.extend(
        memories
            .iter()
            .map(|m| m.id)
            .filter(|id| !matched.contains(id)),
    );
    Ok(ranked)
}

#[derive(Serialize)]
struct Report {
    dataset: String,
    questions: usize,
    excluded_abstention: usize,
    overall: BTreeMap<&'static str, Score>,
    by_type: BTreeMap<String, BTreeMap<&'static str, Score>>,
}

fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let input = PathBuf::from(
        args.next()
            .expect("usage: longmemeval <longmemeval_s.json> <out_dir>"),
    );
    let out_dir = PathBuf::from(
        args.next()
            .unwrap_or_else(|| "benchmarks/longmemeval".into()),
    );
    let items: Vec<Item> = serde_json::from_slice(&fs::read(&input)?)?;

    let mut overall: BTreeMap<&'static str, Score> = BTreeMap::new();
    let mut by_type: BTreeMap<String, BTreeMap<&'static str, Score>> = BTreeMap::new();
    let mut excluded = 0;

    for item in &items {
        if item.question_id.ends_with("_abs") {
            excluded += 1;
            continue;
        }
        // Sessions, dates, and ids are parallel arrays; reject a malformed
        // item instead of silently mis-aligning labels and scoring it.
        let n = item.haystack_sessions.len();
        anyhow::ensure!(
            item.haystack_dates.len() == n && item.haystack_session_ids.len() == n,
            "{}: {} sessions, {} dates, {} session ids",
            item.question_id,
            n,
            item.haystack_dates.len(),
            item.haystack_session_ids.len()
        );
        let memories: Vec<Memory> = item
            .haystack_sessions
            .iter()
            .zip(&item.haystack_dates)
            .enumerate()
            .map(|(i, (turns, date))| {
                let at = parse_date(date);
                let text = turns
                    .iter()
                    .map(|t| format!("{}: {}", t.role, t.content))
                    .collect::<Vec<_>>()
                    .join("\n");
                Memory {
                    id: i as i64 + 1,
                    text,
                    created_at: at,
                    last_used: at,
                    mentions: 0,
                    importance: 0.5,
                    tags: Vec::new(),
                    embedding: None,
                    agent: None,
                }
            })
            .collect();
        let answers: BTreeSet<&str> = item.answer_session_ids.iter().map(String::as_str).collect();
        let rel: BTreeSet<i64> = item
            .haystack_session_ids
            .iter()
            .enumerate()
            .filter(|(_, sid)| answers.contains(sid.as_str()))
            .map(|(i, _)| i as i64 + 1)
            .collect();
        anyhow::ensure!(
            rel.len() == answers.len(),
            "{}: answer session ids missing from the haystack",
            item.question_id
        );
        let now = parse_date(&item.question_date);
        let rankings = [
            rank_builtin(
                &memories,
                &item.question,
                now,
                Ranking::Default,
                Weights::default(),
            ),
            rank_builtin(
                &memories,
                &item.question,
                now,
                Ranking::Bayesian,
                Weights::default(),
            ),
            rank_builtin(
                &memories,
                &item.question,
                now,
                Ranking::Default,
                no_recency(),
            ),
            rank_builtin(
                &memories,
                &item.question,
                now,
                Ranking::Default,
                similarity_only(),
            ),
            rank_fts5(&memories, &item.question)?,
            rank_keyword(&memories, &item.question),
        ];
        for (system, ranked) in SYSTEMS.iter().zip(&rankings) {
            overall.entry(system).or_default().add(ranked, &rel);
            by_type
                .entry(item.question_type.clone())
                .or_default()
                .entry(system)
                .or_default()
                .add(ranked, &rel);
        }
    }

    let report = Report {
        dataset: input.file_name().unwrap().to_string_lossy().into_owned(),
        questions: overall.values().next().map_or(0, |s| s.n),
        excluded_abstention: excluded,
        overall: overall.iter().map(|(k, v)| (*k, v.mean())).collect(),
        by_type: by_type
            .iter()
            .map(|(t, m)| (t.clone(), m.iter().map(|(k, v)| (*k, v.mean())).collect()))
            .collect(),
    };
    fs::create_dir_all(&out_dir)?;
    fs::write(
        out_dir.join("results.json"),
        serde_json::to_string_pretty(&report)? + "\n",
    )?;

    println!(
        "LongMemEval ({}) retrieval, {} questions ({} abstention excluded)",
        report.dataset, report.questions, excluded
    );
    println!(
        "{:<16} {:>9} {:>9} {:>10} {:>10} {:>8}",
        "system", "any@5", "all@5", "any@10", "all@10", "ndcg@10"
    );
    for (system, s) in &report.overall {
        println!(
            "{:<16} {:>9.3} {:>9.3} {:>10.3} {:>10.3} {:>8.3}",
            system, s.recall_any_5, s.recall_all_5, s.recall_any_10, s.recall_all_10, s.ndcg_10
        );
    }
    println!("\nrecall_any@5 by question type:");
    for (t, m) in &report.by_type {
        let row: Vec<String> = SYSTEMS
            .iter()
            .map(|s| format!("{s} {:.3}", m[s].recall_any_5))
            .collect();
        println!("  {:<26} n={:<4} {}", t, m["builtin"].n, row.join("  "));
    }
    Ok(())
}
