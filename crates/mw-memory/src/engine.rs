//! The pluggable retrieval backend.
//!
//! MemoryWhale owns the [`MemoryEngine`] interface. By default it's backed by the
//! [`BuiltinEngine`] (the explainable scorer over a local memory set). With the
//! off-by-default `mempalace` feature, `MemPalaceEngine` sits behind the same
//! interface and talks to `mempalace-mcp` as an MCP *client* — so MemPalace stays
//! an optional, swappable backend rather than a hard dependency. Callers never
//! change.

use std::collections::HashMap;
use std::sync::Arc;

use rusqlite::Connection;

use crate::embed::Embedder;
use crate::scorer::score_with_lexical;
use crate::{Memory, Query, ScoredMemory, Weights};

/// Build an in-memory SQLite FTS5 index over `memories`, MATCH `query`, and
/// return a per-id keyword relevance in [0,1] derived from SQLite's `bm25` rank.
///
/// SQLite's `bm25()` is negative and more-negative = better; we flip it to a
/// non-negative `x` and squash with `x / (1 + x)` — monotonic in match quality,
/// deterministic, and set-independent (so a single-memory `explain` gets the
/// same number a bulk `retrieve` would). Memories with no FTS match are absent
/// from the map → the scorer reads that as similarity 0.
///
/// Best-effort: if FTS5 is somehow unavailable, returns an empty map and the
/// scorer falls back to term overlap. rusqlite is `bundled` (FTS5 compiled in),
/// so that path is not expected in practice.
fn bm25_similarities(memories: &[Memory], query: &str) -> HashMap<i64, f32> {
    let mut out = HashMap::new();
    let match_expr = fts_match_expr(query);
    if match_expr.is_empty() {
        return out;
    }
    let conn = match Connection::open_in_memory() {
        Ok(c) => c,
        Err(_) => return out,
    };
    if conn
        .execute("CREATE VIRTUAL TABLE mem_fts USING fts5(text)", [])
        .is_err()
    {
        return out;
    }
    {
        let mut ins = match conn.prepare("INSERT INTO mem_fts(rowid, text) VALUES (?1, ?2)") {
            Ok(s) => s,
            Err(_) => return out,
        };
        // Insert in corpus order → deterministic bm25.
        for m in memories {
            let _ = ins.execute(rusqlite::params![m.id, m.text]);
        }
    }
    let mut stmt = match conn
        .prepare("SELECT rowid, bm25(mem_fts) FROM mem_fts WHERE mem_fts MATCH ?1")
    {
        Ok(s) => s,
        Err(_) => return out,
    };
    let rows = stmt.query_map([&match_expr], |r| {
        Ok((r.get::<_, i64>(0)?, r.get::<_, f64>(1)?))
    });
    if let Ok(rows) = rows {
        for (id, bm25) in rows.flatten() {
            let x = (-bm25).max(0.0) as f32; // flip: negative bm25 → non-negative
            out.insert(id, x / (1.0 + x));
        }
    }
    out
}

/// OR-of-quoted-terms MATCH expression: lowercase alphanumeric tokens (len ≥ 2)
/// each quoted so FTS5 special characters can't break the query syntax. OR
/// semantics so partial matches still earn a similarity (the blend then reorders
/// them). Mirrors the benchmark's lexical tokenizer for an apples-to-apples read.
fn fts_match_expr(query: &str) -> String {
    let mut seen = std::collections::BTreeSet::new();
    query
        .split(|c: char| !c.is_alphanumeric())
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() >= 2 && seen.insert(w.clone()))
        .map(|t| format!("\"{t}\""))
        .collect::<Vec<_>>()
        .join(" OR ")
}

pub trait MemoryEngine {
    fn name(&self) -> &str;
    /// Return the top-`k` memories for the query, each with its score + reasons.
    fn retrieve(&self, query: &Query, k: usize) -> Vec<ScoredMemory>;
    /// Full explanation for a single memory id (the `memory explain <id>` view).
    fn explain(&self, id: i64, query: &Query) -> Option<ScoredMemory>;
}

/// The default, zero-setup engine: scores an in-memory set with the explainable
/// scorer. (A SQLite-backed variant just loads `memories` from the DB first.)
///
/// With an [`Embedder`] attached, similarity becomes semantic (cosine over
/// embeddings); without one, it falls back to lexical term overlap.
pub struct BuiltinEngine {
    pub memories: Vec<Memory>,
    pub weights: Weights,
    embedder: Option<Arc<dyn Embedder>>,
}

impl BuiltinEngine {
    pub fn new(memories: Vec<Memory>) -> Self {
        Self {
            memories,
            weights: Weights::default(),
            embedder: None,
        }
    }

    pub fn with_weights(mut self, weights: Weights) -> Self {
        self.weights = weights;
        self
    }

    /// Attach an embedder and precompute embeddings for every memory that lacks
    /// one. Returns an error if embedding fails (e.g. Ollama not running).
    pub fn with_embedder(mut self, embedder: Arc<dyn Embedder>) -> anyhow::Result<Self> {
        for m in &mut self.memories {
            if m.embedding.is_none() {
                m.embedding = Some(embedder.embed(&m.text)?);
            }
        }
        self.embedder = Some(embedder);
        Ok(self)
    }

    /// Embed the query text if an embedder is attached (best-effort).
    fn query_embedding(&self, query: &Query) -> Option<Vec<f32>> {
        self.embedder.as_ref().and_then(|e| e.embed(&query.text).ok())
    }
}

impl MemoryEngine for BuiltinEngine {
    fn name(&self) -> &str {
        if self.embedder.is_some() {
            "builtin+embeddings"
        } else {
            "builtin"
        }
    }

    fn retrieve(&self, query: &Query, k: usize) -> Vec<ScoredMemory> {
        let qe = self.query_embedding(query);
        // Keyword relevance from FTS5 BM25 — only when we're not on the semantic
        // (embedding) path, which supersedes it.
        let sims = if qe.is_none() {
            bm25_similarities(&self.memories, &query.text)
        } else {
            HashMap::new()
        };
        let mut scored: Vec<ScoredMemory> = self
            .memories
            .iter()
            .map(|m| {
                score_with_lexical(m, query, &self.weights, qe.as_deref(), sims.get(&m.id).copied())
            })
            .collect();
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(k);
        scored
    }

    fn explain(&self, id: i64, query: &Query) -> Option<ScoredMemory> {
        let qe = self.query_embedding(query);
        let sims = if qe.is_none() {
            bm25_similarities(&self.memories, &query.text)
        } else {
            HashMap::new()
        };
        self.memories
            .iter()
            .find(|m| m.id == id)
            .map(|m| {
                score_with_lexical(m, query, &self.weights, qe.as_deref(), sims.get(&m.id).copied())
            })
    }
}

/// MemPalace as a retrieval backend, spoken to as an **MCP client** over stdio.
///
/// Behind the off-by-default `mempalace` feature: the default build (and
/// `cargo install` of the CLI) pulls in nothing extra.
///
/// Each call spawns the configured command (default `mempalace-mcp`), does the
/// MCP handshake, calls the search tool, and maps its hits into [`ScoredMemory`].
/// MemPalace does its own ranking, so we carry its relevance across as a single
/// `similarity` [`Signal`] rather than re-scoring — the reason string names the
/// source ("mempalace semantic score 0.87").
///
/// Expected search-tool result: JSON text content holding an array (or
/// `{"results": [...]}`) of objects with `text` and `score`, optionally `id`,
/// `tags`, `created_at`, `last_used`, `mentions`, `importance`.
#[cfg(feature = "mempalace")]
pub struct MemPalaceEngine {
    /// The MCP server command (default: `mempalace-mcp`).
    pub command: String,
    /// Extra argv for that command.
    pub args: Vec<String>,
    /// The search tool to call (default: `search`).
    pub tool: String,
}

#[cfg(feature = "mempalace")]
impl Default for MemPalaceEngine {
    fn default() -> Self {
        Self {
            command: "mempalace-mcp".into(),
            args: Vec::new(),
            tool: "search".into(),
        }
    }
}

#[cfg(feature = "mempalace")]
impl MemPalaceEngine {
    pub fn new(command: impl Into<String>, args: Vec<String>) -> Self {
        Self {
            command: command.into(),
            args,
            tool: "search".into(),
        }
    }

    pub fn with_tool(mut self, tool: impl Into<String>) -> Self {
        self.tool = tool.into();
        self
    }

    /// The fallible retrieval. [`MemoryEngine::retrieve`] can't return an error
    /// (the trait yields a `Vec`), so callers who need to *handle* a missing
    /// server or a failed handshake should call this instead.
    pub fn try_retrieve(&self, query: &Query, k: usize) -> anyhow::Result<Vec<ScoredMemory>> {
        let mut client = crate::mcp::McpClient::spawn(&self.command, &self.args)?;
        let tools = client.list_tools()?;
        if let Some(names) = tools.get("tools").and_then(serde_json::Value::as_array) {
            if !names
                .iter()
                .any(|t| t.get("name").and_then(serde_json::Value::as_str) == Some(&self.tool))
            {
                anyhow::bail!(
                    "`{}` does not expose a `{}` tool",
                    self.command,
                    self.tool
                );
            }
        }
        let text = client.call_tool(
            &self.tool,
            serde_json::json!({"query": query.text, "limit": k}),
        )?;
        let mut out = map_hits(&text, query)?;
        out.truncate(k);
        Ok(out)
    }
}

/// Map a MemPalace search payload into scored memories. Split out from the
/// process plumbing so it is testable against captured JSON.
#[cfg(feature = "mempalace")]
fn map_hits(payload: &str, query: &Query) -> anyhow::Result<Vec<ScoredMemory>> {
    use anyhow::Context;
    use serde_json::Value;

    let parsed: Value = serde_json::from_str(payload)
        .context("mempalace search result was not JSON")?;
    let hits = match &parsed {
        Value::Array(a) => a.clone(),
        Value::Object(o) => o
            .get("results")
            .and_then(Value::as_array)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("mempalace search result has no `results` array"))?,
        _ => anyhow::bail!("unexpected mempalace search result shape"),
    };

    let ts = |h: &Value, key: &str| {
        h.get(key)
            .and_then(Value::as_str)
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|d| d.with_timezone(&chrono::Utc))
            .unwrap_or(query.now)
    };

    Ok(hits
        .iter()
        .enumerate()
        .map(|(i, h)| {
            let score = h.get("score").and_then(Value::as_f64).unwrap_or(0.0) as f32;
            let score = score.clamp(0.0, 1.0);
            let memory = Memory {
                id: h.get("id").and_then(Value::as_i64).unwrap_or(i as i64),
                text: h
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                created_at: ts(h, "created_at"),
                last_used: ts(h, "last_used"),
                mentions: h.get("mentions").and_then(Value::as_u64).unwrap_or(0) as u32,
                importance: h.get("importance").and_then(Value::as_f64).unwrap_or(0.0) as f32,
                tags: h
                    .get("tags")
                    .and_then(Value::as_array)
                    .map(|t| {
                        t.iter()
                            .filter_map(Value::as_str)
                            .map(str::to_string)
                            .collect()
                    })
                    .unwrap_or_default(),
                embedding: None,
            };
            ScoredMemory {
                memory,
                score,
                signals: vec![crate::Signal {
                    name: "similarity".into(),
                    weight: 1.0,
                    score,
                    applicable: true,
                    detail: format!("mempalace semantic score {score:.2}"),
                }],
            }
        })
        .collect())
}

#[cfg(feature = "mempalace")]
impl MemoryEngine for MemPalaceEngine {
    fn name(&self) -> &str {
        "mempalace"
    }

    fn retrieve(&self, query: &Query, k: usize) -> Vec<ScoredMemory> {
        match self.try_retrieve(query, k) {
            Ok(hits) => hits,
            Err(e) => {
                eprintln!("[mw-memory] mempalace retrieval failed: {e:#}");
                Vec::new()
            }
        }
    }

    fn explain(&self, id: i64, query: &Query) -> Option<ScoredMemory> {
        // MemPalace ranks server-side; re-run the query and pick the hit out.
        self.try_retrieve(query, 50)
            .ok()?
            .into_iter()
            .find(|s| s.memory.id == id)
    }
}

/// One item to file into MemPalace: a verbatim `content` string placed under a
/// `wing` (project) and `room` (aspect). Mirrors `mempalace_add_drawer` inputs.
#[cfg(feature = "mempalace")]
pub struct Drawer {
    pub wing: String,
    pub room: String,
    pub content: String,
}

/// Outcome of a checkpoint push, parsed from the tool's summary.
#[cfg(feature = "mempalace")]
#[derive(Debug, Default, PartialEq, Eq)]
pub struct CheckpointOutcome {
    pub added: usize,
    pub duplicates: usize,
    pub errors: usize,
}

/// Push memories into a running MemPalace server in one batch via its
/// `mempalace_checkpoint` tool (which semantic-dedups, then files the
/// non-duplicates). Spawns `command args…`, does the MCP handshake, and calls
/// `tool` once with all items. Returns the server's added/duplicate/error tally.
#[cfg(feature = "mempalace")]
pub fn checkpoint(
    command: &str,
    args: &[String],
    tool: &str,
    items: &[Drawer],
) -> anyhow::Result<CheckpointOutcome> {
    use serde_json::{json, Value};
    let mut client = crate::mcp::McpClient::spawn(command, args)?;
    let payload = json!({
        "items": items
            .iter()
            .map(|d| json!({"wing": d.wing, "room": d.room, "content": d.content}))
            .collect::<Vec<_>>(),
        "added_by": "memorywhale",
    });
    let text = client.call_tool(tool, payload)?;
    // The summary is JSON with added/duplicates/errors arrays; tolerate shape
    // drift by counting whatever arrays are present rather than requiring them.
    let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
    let count = |k: &str| v.get(k).and_then(Value::as_array).map(Vec::len).unwrap_or(0);
    Ok(CheckpointOutcome {
        added: count("added"),
        duplicates: count("duplicates"),
        errors: count("errors"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{Duration, TimeZone, Utc};

    fn now() -> chrono::DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 6, 27, 12, 0, 0).unwrap()
    }

    fn sample() -> Vec<Memory> {
        let n = now();
        vec![
            Memory { id: 143, text: "I use Rust for systems software.".into(), created_at: n - Duration::days(20), last_used: n, mentions: 27, importance: 0.98, tags: vec!["rust".into()], embedding: None },
            Memory { id: 7, text: "I ate pizza.".into(), created_at: n - Duration::days(40), last_used: n - Duration::days(40), mentions: 1, importance: 0.01, tags: vec![], embedding: None },
            Memory { id: 22, text: "Use Tokio for async runtime.".into(), created_at: n - Duration::days(5), last_used: n - Duration::days(3), mentions: 6, importance: 0.6, tags: vec!["rust".into(), "tokio".into()], embedding: None },
        ]
    }

    #[test]
    fn retrieve_ranks_and_truncates() {
        let eng = BuiltinEngine::new(sample());
        let q = Query::new("which language for systems programming?", now());
        let top = eng.retrieve(&q, 2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].memory.id, 143); // Rust wins
    }

    #[test]
    fn bm25_similarity_signal_fires() {
        // The similarity signal is now FTS5 BM25: a memory whose text matches the
        // query scores > 0 on similarity alone, a non-matching one scores 0, and
        // the detail names BM25 (not term overlap).
        let eng = BuiltinEngine::new(sample());
        let q = Query::new("pizza", now());

        let matching = eng.explain(7, &q).unwrap(); // "I ate pizza."
        let sim_m = matching.signals.iter().find(|s| s.name == "similarity").unwrap();
        assert!(sim_m.detail.contains("BM25"), "should use BM25: {}", sim_m.detail);
        assert!(sim_m.score > 0.0, "matching memory sim should fire: {}", sim_m.score);

        let non = eng.explain(143, &q).unwrap(); // "I use Rust for systems software."
        let sim_n = non.signals.iter().find(|s| s.name == "similarity").unwrap();
        assert_eq!(sim_n.score, 0.0, "non-matching sim should be 0: {}", sim_n.score);
        assert!(sim_m.score > sim_n.score);
    }

    #[test]
    fn explain_returns_breakdown() {
        let eng = BuiltinEngine::new(sample());
        let q = Query::new("rust", now());
        let e = eng.explain(143, &q).unwrap();
        assert!(e.explain().contains("memory explain 143"));
        assert!(eng.explain(9999, &q).is_none());
    }

    #[cfg(feature = "mempalace")]
    #[test]
    fn mempalace_maps_hits_to_signals() {
        let payload = include_str!("../tests/fixtures/mempalace_search.json");
        let hits = map_hits(payload, &Query::new("rust", now())).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].memory.id, 143);
        assert_eq!(hits[0].memory.tags, vec!["rust".to_string()]);
        assert_eq!(hits[0].percent(), 87);
        let sig = &hits[0].signals[0];
        assert_eq!(sig.name, "similarity");
        assert_eq!(sig.detail, "mempalace semantic score 0.87");
        // No timestamps in the second hit -> falls back to the query's "now".
        assert_eq!(hits[1].memory.created_at, now());
    }

    #[cfg(feature = "mempalace")]
    #[test]
    fn mempalace_rejects_garbage() {
        assert!(map_hits("not json", &Query::new("x", now())).is_err());
        assert!(map_hits("{\"oops\": 1}", &Query::new("x", now())).is_err());
    }

    /// End-to-end over a fake MCP server (a shell script replaying fixture
    /// JSON) — no network, no real mempalace binary.
    #[cfg(all(feature = "mempalace", unix))]
    #[test]
    fn mempalace_talks_to_a_fake_server() {
        let script = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/fake-mempalace-mcp.sh");
        let eng = MemPalaceEngine::new("sh", vec![script.to_string()]);
        let hits = eng.try_retrieve(&Query::new("rust", now()), 10).unwrap();
        assert_eq!(hits.len(), 2);
        assert_eq!(hits[0].memory.id, 143);
        assert!(hits[0].reasons()[0].starts_with("mempalace semantic score"));
    }

    #[cfg(all(feature = "mempalace", unix))]
    #[test]
    fn checkpoint_pushes_and_parses_the_summary() {
        let script =
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/fake-mempalace-checkpoint.sh");
        let items = vec![
            Drawer { wing: "memorywhale".into(), room: "command".into(), content: "cargo build failed".into() },
            Drawer { wing: "memorywhale".into(), room: "note".into(), content: "the fix was X".into() },
        ];
        let out = checkpoint("sh", &[script.to_string()], "mempalace_checkpoint", &items).unwrap();
        // Fixture reports two added, one duplicate, no errors.
        assert_eq!(out, CheckpointOutcome { added: 2, duplicates: 1, errors: 0 });
    }

    #[cfg(feature = "mempalace")]
    #[test]
    fn missing_server_is_a_clear_error() {
        let eng = MemPalaceEngine::new("mw-definitely-not-installed", vec![]);
        let err = eng
            .try_retrieve(&Query::new("x", now()), 3)
            .unwrap_err()
            .to_string();
        assert!(err.contains("failed to start MCP server"), "{err}");
    }
}
