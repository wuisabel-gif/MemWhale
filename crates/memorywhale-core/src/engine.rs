//! The pluggable retrieval backend.
//!
//! MemoryWhale owns the [`MemoryEngine`] interface. By default it's backed by the
//! [`BuiltinEngine`] (the explainable scorer over a local memory set). With the
//! off-by-default `mempalace` feature, `MemPalaceEngine` sits behind the same
//! interface and talks to `mempalace-mcp` as an MCP *client* — so MemPalace stays
//! an optional, swappable backend rather than a hard dependency. Callers never
//! change.

use std::collections::HashMap;
#[cfg(feature = "embeddings")]
use std::sync::Arc;

use rusqlite::Connection;

#[cfg(feature = "embeddings")]
use crate::embed::Embedder;
use crate::scorer::score_with_lexical;
use crate::{Memory, Query, ScoredMemory, Weights};

/// Build an in-memory SQLite FTS5 index over `memories`, MATCH `query`, and
/// return a per-id keyword relevance in `[0,1]` derived from SQLite's `bm25` rank.
///
/// The index has two columns — `text` (the memory body) and `tags` (its tags,
/// space-joined). Tags are explicit user/agent signal, so a query term that only
/// lives in a tag (e.g. `compiler-error`, `flaky`) still earns similarity even
/// when the body never says the word. The MATCH is unfiltered, so a term hits on
/// either column.
///
/// **Per-column BM25 weight `bm25(mem_fts, 1.0, 1.0)` — tags weighted equal to
/// body.** A tag-weight sweep (1.0–3.0) showed the *intent* gains come entirely
/// from tags being *indexed at all*, not from up-weighting them: intent recall
/// was identical across the whole range. A tag>body boost (1.5–2.5) meanwhile
/// *cost* term-overlap recall@1 (0.522 → 0.489) by letting a tag-sharing neighbor
/// jump a stronger body match on ties, for zero intent benefit. So tags are
/// indexed but not boosted — the win is coverage, not weight.
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
        .execute("CREATE VIRTUAL TABLE mem_fts USING fts5(text, tags)", [])
        .is_err()
    {
        return out;
    }
    {
        let mut ins =
            match conn.prepare("INSERT INTO mem_fts(rowid, text, tags) VALUES (?1, ?2, ?3)") {
                Ok(s) => s,
                Err(_) => return out,
            };
        // Insert in corpus order → deterministic bm25.
        for m in memories {
            let _ = ins.execute(rusqlite::params![m.id, m.text, m.tags.join(" ")]);
        }
    }
    let mut stmt = match conn
        .prepare("SELECT rowid, bm25(mem_fts, 1.0, 1.0) FROM mem_fts WHERE mem_fts MATCH ?1")
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

/// The tokens the FTS index sees: lowercase alphanumeric tokens (len ≥ 2).
/// Shared so the query, the body, and the tags are all split the same way — the
/// scorer reuses it to classify which column a match came from.
pub(crate) fn fts_tokens(s: &str) -> impl Iterator<Item = String> + '_ {
    s.split(|c: char| !c.is_alphanumeric())
        .map(|w| w.to_lowercase())
        .filter(|w| w.len() >= 2)
}

/// OR-of-quoted-terms MATCH expression: [`fts_tokens`] each quoted so FTS5
/// special characters can't break the query syntax. OR semantics so partial
/// matches still earn a similarity (the blend then reorders them). Mirrors the
/// benchmark's lexical tokenizer for an apples-to-apples read.
fn fts_match_expr(query: &str) -> String {
    let mut seen = std::collections::BTreeSet::new();
    fts_tokens(query)
        .filter(|w| seen.insert(w.clone()))
        .map(|t| format!("\"{t}\""))
        .collect::<Vec<_>>()
        .join(" OR ")
}

/// The retrieval backend interface MemoryWhale owns. Implemented by
/// [`BuiltinEngine`] (the default explainable scorer) and, behind the
/// `mempalace` feature, `MemPalaceEngine`. Callers depend on this trait, so the
/// backend is swappable without touching call sites.
pub trait MemoryEngine {
    /// A short backend label (e.g. `"builtin"`, `"builtin+embeddings"`).
    fn name(&self) -> &str;
    /// Return the top-`k` memories for the query, each with its score + reasons.
    fn retrieve(&self, query: &Query, k: usize) -> Vec<ScoredMemory>;
    /// Full explanation for a single memory id (the `memory explain <id>` view).
    fn explain(&self, id: i64, query: &Query) -> Option<ScoredMemory>;
}

/// The default, zero-setup engine: scores an in-memory set with the explainable
/// scorer. (A SQLite-backed variant just loads `memories` from the DB first.)
///
/// Without the `embeddings` feature this is the lexical/BM25 engine, end of
/// story. With the feature, an `Embedder` can be attached via `with_embedder`
/// and similarity becomes semantic (cosine over embeddings), falling back to
/// lexical when a memory or query isn't embedded.
pub struct BuiltinEngine {
    pub memories: Vec<Memory>,
    pub weights: Weights,
    #[cfg(feature = "embeddings")]
    embedder: Option<Arc<dyn Embedder>>,
}

impl BuiltinEngine {
    pub fn new(memories: Vec<Memory>) -> Self {
        Self {
            memories,
            weights: Weights::default(),
            #[cfg(feature = "embeddings")]
            embedder: None,
        }
    }

    pub fn with_weights(mut self, weights: Weights) -> Self {
        self.weights = weights;
        self
    }

    /// Attach an embedder and precompute embeddings for every memory that lacks
    /// one. Returns an error if embedding fails (e.g. Ollama not running).
    ///
    /// Only present with the `embeddings` feature; without it the engine is
    /// purely lexical/BM25.
    #[cfg(feature = "embeddings")]
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
    #[cfg(feature = "embeddings")]
    fn query_embedding(&self, query: &Query) -> Option<Vec<f32>> {
        self.embedder.as_ref().and_then(|e| e.embed(&query.text).ok())
    }

    /// No embedder without the feature — retrieval is always lexical/BM25.
    #[cfg(not(feature = "embeddings"))]
    fn query_embedding(&self, _query: &Query) -> Option<Vec<f32>> {
        None
    }
}

impl MemoryEngine for BuiltinEngine {
    fn name(&self) -> &str {
        #[cfg(feature = "embeddings")]
        if self.embedder.is_some() {
            return "builtin+embeddings";
        }
        "builtin"
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
                eprintln!("[memorywhale-core] mempalace retrieval failed: {e:#}");
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

/// One reconcile primitive for [`sync_ops`]: either file a new drawer or delete
/// an existing one by its server-assigned id.
#[cfg(feature = "mempalace")]
pub enum SyncOp {
    Add {
        wing: String,
        room: String,
        content: String,
        added_by: String,
    },
    Delete {
        drawer_id: String,
    },
}

/// The outcome of one [`SyncOp`], positionally matched to the input `ops`.
#[cfg(feature = "mempalace")]
pub enum SyncResult {
    /// An `Add` succeeded; carries the server-assigned `drawer_id`.
    Added { drawer_id: String },
    /// A `Delete` succeeded.
    Deleted,
}

/// Run a batch of add/delete reconcile ops over ONE MCP session — the id-based
/// counterpart to [`checkpoint`]. Spawns `command args…`, does the handshake,
/// then calls `add_tool` (`mempalace_add_drawer`) / `delete_tool`
/// (`mempalace_delete_drawer`) for each op in order. Adds parse the new
/// `drawer_id` out of the tool's JSON result and return it. Results line up 1:1
/// with `ops`, so the caller can zip them back to the memories they came from.
#[cfg(feature = "mempalace")]
pub fn sync_ops(
    command: &str,
    args: &[String],
    add_tool: &str,
    delete_tool: &str,
    ops: &[SyncOp],
) -> anyhow::Result<Vec<SyncResult>> {
    use anyhow::Context;
    use serde_json::{json, Value};
    let mut client = crate::mcp::McpClient::spawn(command, args)?;
    let mut out = Vec::with_capacity(ops.len());
    for op in ops {
        match op {
            SyncOp::Add { wing, room, content, added_by } => {
                let text = client.call_tool(
                    add_tool,
                    json!({"wing": wing, "room": room, "content": content, "added_by": added_by}),
                )?;
                let v: Value =
                    serde_json::from_str(&text).context("add_drawer result was not JSON")?;
                let drawer_id = v
                    .get("drawer_id")
                    .and_then(Value::as_str)
                    .ok_or_else(|| anyhow::anyhow!("add_drawer returned no drawer_id"))?
                    .to_string();
                out.push(SyncResult::Added { drawer_id });
            }
            SyncOp::Delete { drawer_id } => {
                client.call_tool(delete_tool, json!({"drawer_id": drawer_id}))?;
                out.push(SyncResult::Deleted);
            }
        }
    }
    Ok(out)
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
    fn tag_match_outranks_weaker_body_match() {
        // "flaky" lives only in memory 1's tags and only in memory 2's (longer)
        // body. With tags weighted 2.0 above body 1.0, the tag hit must earn the
        // higher BM25 similarity — and the reason must name the tag source.
        let n = now();
        let mems = vec![
            Memory { id: 1, text: "the recall test failed sometimes on CI".into(), created_at: n, last_used: n, mentions: 0, importance: 0.0, tags: vec!["flaky".into()], embedding: None },
            Memory { id: 2, text: "flaky pastry notes: butter, layers, oven temperature, resting time, and folds".into(), created_at: n, last_used: n, mentions: 0, importance: 0.0, tags: vec![], embedding: None },
        ];
        let eng = BuiltinEngine::new(mems);
        let q = Query::new("flaky", n);
        let a = eng.explain(1, &q).unwrap();
        let b = eng.explain(2, &q).unwrap();
        let sim = |s: &ScoredMemory| s.signals.iter().find(|x| x.name == "similarity").unwrap().clone();
        let (sa, sb) = (sim(&a), sim(&b));
        assert!(sa.score > 0.0 && sb.score > 0.0, "both should match: {} {}", sa.score, sb.score);
        assert!(sa.score > sb.score, "tag hit ({}) should outrank weaker body hit ({})", sa.score, sb.score);
        assert!(sa.detail.contains("tag"), "reason should name the tag source: {}", sa.detail);
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

    /// One session, mixed delete+add ops against a fake server: deletes succeed
    /// (no result), adds return distinct drawer ids in order.
    #[cfg(all(feature = "mempalace", unix))]
    #[test]
    fn sync_ops_adds_and_deletes_over_one_session() {
        let script =
            concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/fake-mempalace-sync.sh");
        let ops = vec![
            SyncOp::Add {
                wing: "memorywhale".into(),
                room: "note".into(),
                content: "first".into(),
                added_by: "you".into(),
            },
            SyncOp::Delete { drawer_id: "old-1".into() },
            SyncOp::Add {
                wing: "memorywhale".into(),
                room: "note".into(),
                content: "second".into(),
                added_by: "memorywhale".into(),
            },
        ];
        let out = sync_ops(
            "sh",
            &[script.to_string()],
            "mempalace_add_drawer",
            "mempalace_delete_drawer",
            &ops,
        )
        .unwrap();
        assert_eq!(out.len(), 3);
        let ids: Vec<&str> = out
            .iter()
            .filter_map(|r| match r {
                SyncResult::Added { drawer_id } => Some(drawer_id.as_str()),
                SyncResult::Deleted => None,
            })
            .collect();
        assert_eq!(ids, vec!["drawer-1", "drawer-2"]);
        assert!(matches!(out[1], SyncResult::Deleted));
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
