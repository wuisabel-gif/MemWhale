//! The pluggable retrieval backend.
//!
//! MemoryWhale owns the [`MemoryEngine`] interface. Today it's backed by the
//! [`BuiltinEngine`] (the explainable scorer over a local memory set). Tomorrow a
//! [`MemPalaceEngine`] can sit behind the same interface, calling `mempalace-mcp`
//! as a *client* — so MemPalace stays an optional, swappable backend rather than
//! a hard dependency. Callers never change.

use std::sync::Arc;

use crate::embed::Embedder;
use crate::scorer::score;
use crate::{Memory, Query, ScoredMemory, Weights};

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
        let mut scored: Vec<ScoredMemory> = self
            .memories
            .iter()
            .map(|m| score(m, query, &self.weights, qe.as_deref()))
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
        self.memories
            .iter()
            .find(|m| m.id == id)
            .map(|m| score(m, query, &self.weights, qe.as_deref()))
    }
}

/// Pluggability seam for MemPalace. This is a **stub**: it documents the
/// integration shape without copying or depending on MemPalace's code. A real
/// implementation would spawn/connect to `mempalace-mcp`, call its `search` and
/// `kg_query` tools, and map the results into [`ScoredMemory`] (carrying
/// MemPalace's own scores/weights into our `Signal`s for explainability).
pub struct MemPalaceEngine {
    /// The MCP command MemoryWhale would talk to (default: `mempalace-mcp`).
    pub mcp_command: String,
}

impl Default for MemPalaceEngine {
    fn default() -> Self {
        Self {
            mcp_command: "mempalace-mcp".into(),
        }
    }
}

impl MemoryEngine for MemPalaceEngine {
    fn name(&self) -> &str {
        "mempalace (stub)"
    }

    fn retrieve(&self, _query: &Query, _k: usize) -> Vec<ScoredMemory> {
        eprintln!(
            "[mw-memory] MemPalaceEngine is a stub: a real build would call `{}` \
             over MCP (search + kg_query) and map drawers/triples into ScoredMemory.",
            self.mcp_command
        );
        Vec::new()
    }

    fn explain(&self, _id: i64, _query: &Query) -> Option<ScoredMemory> {
        None
    }
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
    fn explain_returns_breakdown() {
        let eng = BuiltinEngine::new(sample());
        let q = Query::new("rust", now());
        let e = eng.explain(143, &q).unwrap();
        assert!(e.explain().contains("memory explain 143"));
        assert!(eng.explain(9999, &q).is_none());
    }

    #[test]
    fn mempalace_stub_is_pluggable() {
        let eng: Box<dyn MemoryEngine> = Box::new(MemPalaceEngine::default());
        assert_eq!(eng.name(), "mempalace (stub)");
        assert!(eng.retrieve(&Query::new("x", now()), 5).is_empty());
    }
}
