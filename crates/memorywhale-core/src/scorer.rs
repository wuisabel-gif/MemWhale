//! The explainable scorer: turn a (memory, query) pair into a blended score plus
//! the interpretable signals behind it.
//!
//! # The signal model
//!
//! Relevance is a **weighted mean over the *applicable* signals**. Each signal
//! yields a strength in `[0,1]`, a mixing [`Weights`] weight, and a
//! human-readable `detail`; the blend is `Σ(weight × score) / Σ(weight)` taken
//! only over signals that apply. So a missing context (e.g. no task tags) drops
//! out of the average — it neither helps nor unfairly penalizes — instead of
//! counting as a zero. This is what makes every result explainable: the score is
//! literally the sum of named, inspectable contributions.
//!
//! The five signals:
//!
//! - **similarity** — how well the text matches the query. Semantic cosine over
//!   embeddings when available, else FTS5 BM25 keyword relevance, else a
//!   term-overlap fallback.
//! - **recency** — exponential decay from `last_used`, halving every
//!   `query.half_life_days`.
//! - **importance** — the memory's stored importance weight, used directly.
//! - **reinforcement** — mention count, log-scaled and capped.
//! - **task-relevance** — overlap between the memory's tags and the query's task
//!   tags; *inapplicable* (drops out) when there is no task context.
//!
//! Each contributes a reason string, surfaced via [`ScoredMemory::reasons`] and
//! [`ScoredMemory::explain`].

use std::collections::HashSet;

use crate::{Memory, Query, ScoredMemory, Signal, Weights};

/// Score one memory against a query. `query_embedding` enables semantic
/// similarity (cosine) when the memory is also embedded; otherwise similarity
/// falls back to lexical term overlap.
///
/// This is the thin wrapper used by callers that have no corpus-wide keyword
/// index handy (unit tests, the `eval` example). The production path goes
/// through [`score_with_lexical`], which feeds a precomputed FTS5 BM25
/// similarity in place of the crude term-overlap fallback.
pub fn score(
    memory: &Memory,
    query: &Query,
    w: &Weights,
    query_embedding: Option<&[f32]>,
) -> ScoredMemory {
    score_with_lexical(memory, query, w, query_embedding, None)
}

/// Like [`score`], but with `lexical_sim`: a precomputed lexical relevance in
/// `[0,1]` (the engine derives it from an in-memory SQLite FTS5 BM25 index over
/// the whole memory set). When present it *is* the similarity signal in the
/// non-semantic path — proper keyword relevance instead of Jaccard term
/// overlap. `None` falls back to term overlap for callers without an index.
pub fn score_with_lexical(
    memory: &Memory,
    query: &Query,
    w: &Weights,
    query_embedding: Option<&[f32]>,
    lexical_sim: Option<f32>,
) -> ScoredMemory {
    let signals = vec![
        similarity_signal(memory, query, w.similarity, query_embedding, lexical_sim),
        recency_signal(memory, query, w.recency),
        importance_signal(memory, w.importance),
        reinforcement_signal(memory, w.reinforcement),
        task_signal(memory, query, w.task),
    ];

    let numerator: f32 = signals.iter().map(Signal::contribution).sum();
    let denominator: f32 = signals
        .iter()
        .filter(|s| s.applicable)
        .map(|s| s.weight)
        .sum();
    let score = if denominator > 0.0 {
        (numerator / denominator).clamp(0.0, 1.0)
    } else {
        0.0
    };

    ScoredMemory {
        memory: memory.clone(),
        score,
        signals,
    }
}

// ── signals ──────────────────────────────────────────────────────────────

/// Similarity, in priority order:
///   1. Semantic (cosine over embeddings) when both query and memory are embedded.
///   2. Keyword relevance (SQLite FTS5 BM25) when the engine supplies `lexical_sim`.
///   3. Lexical term overlap (Jaccard) as a last-resort fallback for callers
///      without a corpus-wide index (unit tests, the `eval` example).
fn similarity_signal(
    memory: &Memory,
    query: &Query,
    weight: f32,
    query_embedding: Option<&[f32]>,
    lexical_sim: Option<f32>,
) -> Signal {
    // Semantic path.
    if let (Some(qe), Some(me)) = (query_embedding, memory.embedding.as_deref()) {
        let score = crate::embed::cosine(qe, me);
        return Signal {
            name: "similarity".into(),
            weight,
            score,
            applicable: true,
            detail: format!("{}% semantic match (embeddings)", pct(score)),
        };
    }
    // Keyword path: precomputed FTS5 BM25 relevance from the engine.
    if let Some(sim) = lexical_sim {
        let score = sim.clamp(0.0, 1.0);
        return Signal {
            name: "similarity".into(),
            weight,
            score,
            applicable: true,
            detail: format!(
                "{}% keyword match ({})",
                pct(score),
                bm25_source(query, memory)
            ),
        };
    }
    // Lexical fallback (no index available).
    let q = tokenize(&query.text);
    let m = tokenize(&memory.text);
    let score = if q.is_empty() || m.is_empty() {
        0.0
    } else {
        let inter = q.intersection(&m).count() as f32;
        let union = q.union(&m).count() as f32;
        inter / union
    };
    Signal {
        name: "similarity".into(),
        weight,
        score,
        applicable: true,
        detail: format!("{}% term overlap (lexical)", pct(score)),
    }
}

/// Exponential recency from `last_used`, halving every `half_life_days`.
fn recency_signal(memory: &Memory, query: &Query, weight: f32) -> Signal {
    let age_days = (query.now - memory.last_used).num_seconds() as f32 / 86_400.0;
    let age_days = age_days.max(0.0);
    let hl = query.half_life_days.max(0.1);
    let score = 0.5f32.powf(age_days / hl).clamp(0.0, 1.0);
    Signal {
        name: "recency".into(),
        weight,
        score,
        applicable: true,
        detail: format!("last used {}", human_ago(age_days)),
    }
}

/// Stored importance weight, used directly.
fn importance_signal(memory: &Memory, weight: f32) -> Signal {
    let score = memory.importance.clamp(0.0, 1.0);
    Signal {
        name: "importance".into(),
        weight,
        score,
        applicable: true,
        detail: format!("importance {score:.2}"),
    }
}

/// Reinforcement from mention count, log-scaled and capped.
fn reinforcement_signal(memory: &Memory, weight: f32) -> Signal {
    const CAP: f32 = 30.0;
    let n = memory.mentions as f32;
    let score = ((1.0 + n).ln() / (1.0 + CAP).ln()).clamp(0.0, 1.0);
    Signal {
        name: "reinforcement".into(),
        weight,
        score,
        applicable: true,
        detail: format!("mentioned {}×", memory.mentions),
    }
}

/// Task relevance: overlap between the memory's tags and the current task tags.
/// Inert (not applicable) when there is no task context.
fn task_signal(memory: &Memory, query: &Query, weight: f32) -> Signal {
    if query.task_tags.is_empty() {
        return Signal {
            name: "task".into(),
            weight,
            score: 0.0,
            applicable: false,
            detail: "no task context".into(),
        };
    }
    let task: HashSet<String> = query.task_tags.iter().map(|t| t.to_lowercase()).collect();
    let tags: HashSet<String> = memory.tags.iter().map(|t| t.to_lowercase()).collect();
    let matched: Vec<String> = task.intersection(&tags).cloned().collect();
    let score = (matched.len() as f32 / task.len() as f32).clamp(0.0, 1.0);
    let detail = if matched.is_empty() {
        "not linked to the current task".into()
    } else {
        format!("linked to current task ({})", matched.join(", "))
    };
    Signal {
        name: "task".into(),
        weight,
        score,
        applicable: true,
        detail,
    }
}

/// Classify which FTS column(s) the BM25 match came from, for the reason string:
/// `"BM25 · tag+body: compiler, error"`, `"BM25 · tag: flaky"`, or
/// `"BM25 · body: pizza"`. Uses the engine's shared tokenizer so the label
/// reflects exactly what the index matched. Falls back to a bare `"BM25"` if the
/// query/memory tokenization can't attribute it (not expected in practice).
fn bm25_source(query: &Query, memory: &Memory) -> String {
    use crate::engine::fts_tokens;
    let qterms: HashSet<String> = fts_tokens(&query.text).collect();
    let body: HashSet<String> = fts_tokens(&memory.text).collect();
    let tags: HashSet<String> = memory.tags.iter().flat_map(|t| fts_tokens(t)).collect();

    // Sorted so the reason string is deterministic.
    let hit = |col: &HashSet<String>| -> Vec<String> {
        let mut v: Vec<String> = qterms.intersection(col).cloned().collect();
        v.sort();
        v
    };
    let in_tags = hit(&tags);
    let in_body = hit(&body);

    let (label, terms): (&str, Vec<String>) = match (in_tags.is_empty(), in_body.is_empty()) {
        (false, false) => {
            let mut u = in_tags;
            u.extend(in_body);
            u.sort();
            u.dedup();
            ("tag+body", u)
        }
        (false, true) => ("tag", in_tags),
        (true, false) => ("body", in_body),
        (true, true) => return "BM25".into(),
    };
    format!("BM25 · {label}: {}", terms.join(", "))
}

// ── helpers ──────────────────────────────────────────────────────────────

fn tokenize(text: &str) -> HashSet<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() >= 3 && !is_stopword(w))
        .collect()
}

fn is_stopword(w: &str) -> bool {
    matches!(
        w,
        "the"
            | "and"
            | "for"
            | "are"
            | "you"
            | "your"
            | "with"
            | "что"
            | "this"
            | "that"
            | "what"
            | "use"
            | "using"
            | "have"
            | "has"
            | "was"
            | "were"
            | "from"
            | "into"
            | "out"
            | "how"
            | "why"
            | "when"
            | "where"
            | "who"
            | "can"
            | "does"
            | "did"
            | "but"
            | "not"
            | "all"
            | "any"
            | "our"
    )
}

fn pct(x: f32) -> u32 {
    (x * 100.0).round().clamp(0.0, 100.0) as u32
}

fn human_ago(days: f32) -> String {
    if days < 1.0 {
        "today".into()
    } else if days < 2.0 {
        "yesterday".into()
    } else if days < 14.0 {
        format!("{} days ago", days.round() as i64)
    } else if days < 60.0 {
        format!("{} weeks ago", (days / 7.0).round() as i64)
    } else {
        format!("{} months ago", (days / 30.0).round() as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};

    fn mem(
        id: i64,
        text: &str,
        days_old: i64,
        mentions: u32,
        importance: f32,
        tags: &[&str],
    ) -> Memory {
        let now = Utc.with_ymd_and_hms(2026, 6, 27, 12, 0, 0).unwrap();
        Memory {
            id,
            text: text.into(),
            created_at: now - Duration::days(days_old + 10),
            last_used: now - Duration::days(days_old),
            mentions,
            importance,
            tags: tags.iter().map(|s| s.to_string()).collect(),
            embedding: None,
            agent: None,
        }
    }

    fn now() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 27, 12, 0, 0).unwrap()
    }

    // NB: these `score(.., None)` tests exercise the *term-overlap fallback*
    // (no embeddings, no engine-supplied BM25). The production ranking path runs
    // through `BuiltinEngine`, which feeds real FTS5 BM25 as the similarity
    // signal — see `engine::tests::bm25_similarity_signal_fires`.
    #[test]
    fn rust_beats_pizza_for_systems_query() {
        let w = Weights::default();
        let q = Query::new("what language do I use for systems programming?", now());
        let rust = score(
            &mem(
                1,
                "I use Rust for systems programming.",
                0,
                27,
                0.98,
                &["rust"],
            ),
            &q,
            &w,
            None,
        );
        let pizza = score(&mem(2, "I ate pizza.", 40, 1, 0.01, &[]), &q, &w, None);
        assert!(
            rust.score > pizza.score,
            "rust {} should beat pizza {}",
            rust.score,
            pizza.score
        );
        assert!(rust.percent() > 50);
    }

    #[test]
    fn recency_recent_beats_old() {
        let w = Weights::default();
        let q = Query::new("rust tooling", now());
        let recent = score(&mem(1, "rust tooling notes", 1, 5, 0.5, &[]), &q, &w, None);
        let old = score(&mem(2, "rust tooling notes", 90, 5, 0.5, &[]), &q, &w, None);
        assert!(recent.score > old.score);
    }

    #[test]
    fn task_signal_inert_without_context() {
        let w = Weights::default();
        let q = Query::new("rust", now()); // no task tags
        let s = score(&mem(1, "rust", 0, 1, 0.5, &["rust"]), &q, &w, None);
        let task = s.signals.iter().find(|x| x.name == "task").unwrap();
        assert!(!task.applicable);
    }

    #[test]
    fn task_signal_boosts_when_linked() {
        let w = Weights::default();
        let base = Query::new("auth changes", now());
        let with_task = Query::new("auth changes", now()).with_task(vec!["delphin".into()]);
        let m = mem(1, "auth changes", 0, 1, 0.5, &["delphin"]);
        let s0 = score(&m, &base, &w, None);
        let s1 = score(&m, &with_task, &w, None);
        assert!(s1.score > s0.score, "task link should raise score");
    }

    #[test]
    fn reasons_are_ordered_and_nonempty() {
        let w = Weights::default();
        let q = Query::new("rust systems", now());
        let s = score(
            &mem(1, "I use Rust for systems work.", 0, 27, 0.98, &["rust"]),
            &q,
            &w,
            None,
        );
        let reasons = s.reasons();
        assert!(!reasons.is_empty());
    }

    #[test]
    fn lexical_override_replaces_term_overlap() {
        // When the engine supplies a BM25 similarity, it drives the signal
        // (and the detail names BM25), regardless of literal token overlap.
        let w = Weights::default();
        let q = Query::new("anything", now());
        let m = mem(1, "totally unrelated text", 0, 1, 0.5, &[]);
        let s = score_with_lexical(&m, &q, &w, None, Some(0.9));
        let sim = s.signals.iter().find(|x| x.name == "similarity").unwrap();
        assert!(
            (sim.score - 0.9).abs() < 1e-6,
            "override should win: {}",
            sim.score
        );
        assert!(sim.detail.contains("BM25"));
    }

    #[test]
    fn semantic_similarity_matches_without_shared_words() {
        // "database" shares no literal token with "postgres", but embeddings
        // place them close — cosine should score high where lexical scores 0.
        let w = Weights::default();
        let q = Query::new("which database did we choose", now());
        // Hand-built 2-D "concept space": axis 0 = db-ness, axis 1 = food-ness.
        let q_emb = [1.0f32, 0.0];
        let mut postgres = mem(1, "we chose Postgres for storage", 0, 1, 0.5, &[]);
        postgres.embedding = Some(vec![0.96, 0.10]); // very db
        let mut pizza = mem(2, "I ate pizza", 0, 1, 0.5, &[]);
        pizza.embedding = Some(vec![0.05, 0.99]); // very food

        let sp = score(&postgres, &q, &w, Some(&q_emb));
        let sz = score(&pizza, &q, &w, Some(&q_emb));
        let sim_p = sp.signals.iter().find(|s| s.name == "similarity").unwrap();
        assert!(
            sim_p.score > 0.9,
            "postgres semantic sim should be high: {}",
            sim_p.score
        );
        assert!(sp.score > sz.score);
        assert!(sim_p.detail.contains("semantic"));
    }
}
