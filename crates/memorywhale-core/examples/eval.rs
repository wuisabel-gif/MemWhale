// crates/memorywhale-core/examples/eval.rs
// Lexical:  cargo run -p memorywhale-core --example eval -- eval_data.json
// Semantic: start Ollama (`ollama pull nomic-embed-text`), same command.
//           Falls back to lexical-only automatically if Ollama is unreachable.
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use chrono::{Duration, Utc};
use memorywhale_core::embed::{Embedder, OllamaEmbedder};
use memorywhale_core::{scorer::score, Memory, Query, Weights};
use serde::{Deserialize, Serialize};

// ── dataset ──────────────────────────────────────────────────────────────
#[derive(Deserialize)]
struct Dataset { memories: Vec<MemSpec>, queries: Vec<QuerySpec> }
#[derive(Deserialize)]
struct MemSpec { id: i64, text: String, tags: Vec<String>, importance: f32, mentions: u32, days_old: i64 }
#[derive(Deserialize)]
struct QuerySpec { text: String, #[serde(default)] task_tags: Vec<String>, relevant_ids: Vec<i64> }

// ── embedding cache (keyed by text, namespaced by model) ─────────────────
#[derive(Default, Serialize, Deserialize)]
struct EmbedCache { model: String, vectors: HashMap<String, Vec<f32>> }

impl EmbedCache {
    fn load(path: &str, model: &str) -> Self {
        match std::fs::read_to_string(path) {
            Ok(s) => match serde_json::from_str::<EmbedCache>(&s) {
                // Model changed → old vectors are meaningless, start fresh.
                Ok(c) if c.model == model => c,
                _ => EmbedCache { model: model.into(), vectors: HashMap::new() },
            },
            Err(_) => EmbedCache { model: model.into(), vectors: HashMap::new() },
        }
    }
    fn save(&self, path: &str) {
        if let Ok(s) = serde_json::to_string(self) {
            let _ = std::fs::write(path, s);
        }
    }
}

#[derive(Default)]
struct EmbedStats { cold_calls: usize, cache_hits: usize, total_cold_ms: u128 }
impl EmbedStats {
    fn mean_cold_ms(&self) -> f64 {
        if self.cold_calls == 0 { 0.0 } else { self.total_cold_ms as f64 / self.cold_calls as f64 }
    }
}

/// Embed every text, using the cache. Any error (Ollama down, etc.) aborts the
/// whole semantic path — the caller falls back to lexical-only.
fn embed_all(
    embedder: &dyn Embedder,
    texts: &[String],
    cache: &mut EmbedCache,
    stats: &mut EmbedStats,
) -> anyhow::Result<HashMap<String, Vec<f32>>> {
    let mut out = HashMap::new();
    for t in texts {
        if let Some(v) = cache.vectors.get(t) {
            stats.cache_hits += 1;
            out.insert(t.clone(), v.clone());
            continue;
        }
        let start = Instant::now();
        let v = embedder.embed(t)?; // network call to Ollama
        stats.total_cold_ms += start.elapsed().as_millis();
        stats.cold_calls += 1;
        cache.vectors.insert(t.clone(), v.clone());
        out.insert(t.clone(), v);
    }
    Ok(out)
}

// ── metrics (binary relevance) ───────────────────────────────────────────
fn precision_at_k(rel: &[bool], k: usize) -> f64 {
    rel.iter().take(k).filter(|&&r| r).count() as f64 / k as f64
}
fn recall_at_k(rel: &[bool], num_relevant: usize, k: usize) -> f64 {
    if num_relevant == 0 { return 0.0; }
    rel.iter().take(k).filter(|&&r| r).count() as f64 / num_relevant as f64
}
fn reciprocal_rank(rel: &[bool]) -> f64 {
    rel.iter().position(|&r| r).map(|i| 1.0 / (i as f64 + 1.0)).unwrap_or(0.0)
}
fn dcg_at_k(rel: &[bool], k: usize) -> f64 {
    rel.iter().take(k).enumerate()
        .map(|(i, &r)| if r { 1.0 / (i as f64 + 2.0).log2() } else { 0.0 }).sum()
}
fn ndcg_at_k(rel: &[bool], num_relevant: usize, k: usize) -> f64 {
    let ideal: Vec<bool> = std::iter::repeat(true).take(num_relevant).collect();
    let idcg = dcg_at_k(&ideal, k);
    if idcg == 0.0 { 0.0 } else { dcg_at_k(rel, k) / idcg }
}

struct Config { name: &'static str, weights: Weights, semantic: bool }

const K: usize = 5;

fn main() -> anyhow::Result<()> {
    let path = std::env::args().nth(1).unwrap_or_else(|| "eval_data.json".into());
    let ds: Dataset = serde_json::from_str(&std::fs::read_to_string(&path)?)?;
    let now = Utc::now();

    // Base memories (no embeddings yet).
    let mut memories: Vec<Memory> = ds.memories.iter().map(|m| Memory {
        id: m.id,
        text: m.text.clone(),
        created_at: now - Duration::days(m.days_old + 10),
        last_used: now - Duration::days(m.days_old),
        mentions: m.mentions,
        importance: m.importance,
        tags: m.tags.clone(),
        embedding: None,
    }).collect();

    // ── try the semantic path ────────────────────────────────────────────
    let embedder = OllamaEmbedder::default(); // nomic-embed-text @ localhost:11434
    let cache_path = "embed_cache.json";
    let mut cache = EmbedCache::load(cache_path, embedder.name());
    let mut stats = EmbedStats::default();

    let mut mem_texts: Vec<String> = ds.memories.iter().map(|m| m.text.clone()).collect();
    let query_texts: Vec<String> = ds.queries.iter().map(|q| q.text.clone()).collect();
    mem_texts.extend(query_texts.iter().cloned());

    let mut query_embeddings: HashMap<String, Vec<f32>> = HashMap::new();
    let semantic_available = match embed_all(&embedder, &mem_texts, &mut cache, &mut stats) {
        Ok(map) => {
            // Attach memory embeddings; stash query embeddings for lookup.
            for m in memories.iter_mut() {
                if let Some(v) = map.get(&m.text) { m.embedding = Some(v.clone()); }
            }
            for q in &ds.queries {
                if let Some(v) = map.get(&q.text) { query_embeddings.insert(q.text.clone(), v.clone()); }
            }
            cache.save(cache_path);
            true
        }
        Err(e) => {
            eprintln!("[eval] semantic path disabled ({e}); running lexical-only.");
            false
        }
    };

    if semantic_available {
        eprintln!(
            "[eval] embedded {} texts | {} cold calls, {} cache hits | mean cold embed = {:.1} ms/text",
            mem_texts.len(), stats.cold_calls, stats.cache_hits, stats.mean_cold_ms(),
        );
    }

    // ── configs ──────────────────────────────────────────────────────────
    let d = Weights::default();
    let mut configs = vec![
        Config { name: "lexical-full",      weights: d.clone(), semantic: false },
        Config { name: "lexical-simonly",   weights: Weights { similarity: 1.0, recency: 0.0, importance: 0.0, reinforcement: 0.0, task: 0.0 }, semantic: false },
        Config { name: "lexical-norecency", weights: Weights { recency: 0.0, ..d.clone() }, semantic: false },
    ];
    if semantic_available {
        configs.push(Config { name: "semantic-full",    weights: d.clone(), semantic: true });
        configs.push(Config { name: "semantic-simonly", weights: Weights { similarity: 1.0, recency: 0.0, importance: 0.0, reinforcement: 0.0, task: 0.0 }, semantic: true });
    }

    // ── run ──────────────────────────────────────────────────────────────
    println!("{:<20} {:>6} {:>6} {:>6} {:>6} {:>10}", "config", "P@5", "R@5", "MRR", "nDCG5", "score_us/q");
    for cfg in &configs {
        let (mut p, mut r, mut mrr, mut ndcg) = (0.0, 0.0, 0.0, 0.0);
        let n = ds.queries.len() as f64;
        let t0 = Instant::now();
        for q in &ds.queries {
            let relset: HashSet<i64> = q.relevant_ids.iter().copied().collect();
            let query = if q.task_tags.is_empty() {
                Query::new(&q.text, now)
            } else {
                Query::new(&q.text, now).with_task(q.task_tags.clone())
            };
            // Semantic run passes the query embedding; lexical passes None
            // (the scorer then ignores memory embeddings and uses term overlap).
            let qemb: Option<&[f32]> = if cfg.semantic {
                query_embeddings.get(&q.text).map(|v| v.as_slice())
            } else { None };

            let mut scored: Vec<_> = memories.iter()
                .map(|m| score(m, &query, &cfg.weights, qemb))
                .collect();
            scored.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());
            let ranked: Vec<bool> = scored.iter().map(|s| relset.contains(&s.memory.id)).collect();
            let num_rel = q.relevant_ids.len();

            p    += precision_at_k(&ranked, K);
            r    += recall_at_k(&ranked, num_rel, K);
            mrr  += reciprocal_rank(&ranked);
            ndcg += ndcg_at_k(&ranked, num_rel, K);
        }
        let score_us_per_q = t0.elapsed().as_micros() as f64 / n;
        println!("{:<20} {:>6.3} {:>6.3} {:>6.3} {:>6.3} {:>10.1}",
            cfg.name, p/n, r/n, mrr/n, ndcg/n, score_us_per_q);
    }

    if semantic_available {
        println!(
            "\n[cost] semantic pays ~{:.1} ms once per *new* query to embed it (Ollama), \
             then scoring is the µs/q above. Lexical pays 0 ms embedding.",
            stats.mean_cold_ms(),
        );
    }
    Ok(())
}
