//! `memorywhale-core` — explainable retrieval for MemoryWhale.
//!
//! The differentiator: retrieval that returns not just *which* memories, but
//! *why* — a blended score over interpretable signals (similarity, recency,
//! importance, reinforcement, task-relevance), each with a human-readable reason.
//!
//! # The core flow
//!
//! 1. Build [`Memory`] items — in code, or via [`sqlite::load_memories`] from a
//!    MemoryWhale database.
//! 2. Wrap them in an [`engine::BuiltinEngine`] (the default, zero-setup engine).
//! 3. Run a [`Query`] through [`engine::MemoryEngine::retrieve`].
//! 4. Read back [`ScoredMemory`] values: a blended `score`, plus the per-signal
//!    [`Signal`] breakdown. [`ScoredMemory::reasons`] gives the ranked
//!    "retrieved because…" strings; [`ScoredMemory::explain`] the full audit.
//!
//! ```
//! use chrono::Utc;
//! use memorywhale_core::engine::{BuiltinEngine, MemoryEngine};
//! use memorywhale_core::{Memory, Query};
//!
//! let now = Utc::now();
//! let mems = vec![Memory {
//!     id: 1,
//!     text: "Use Tokio for the async runtime.".into(),
//!     created_at: now,
//!     last_used: now,
//!     mentions: 6,
//!     importance: 0.6,
//!     tags: vec!["rust".into(), "tokio".into()],
//!     embedding: None,
//! }];
//! let engine = BuiltinEngine::new(mems);
//! let hits = engine.retrieve(&Query::new("async runtime", now), 5);
//! assert_eq!(hits[0].memory.id, 1);
//! // hits[0].reasons() explains *why* it was retrieved.
//! ```
//!
//! See `examples/embed.rs` for a fuller, runnable walkthrough.
//!
//! # Signal model
//!
//! The distinctive design lives in [`scorer`]: relevance is a **weighted mean
//! over the applicable signals** (a missing context, like no task tags, drops
//! out of the average rather than scoring zero), and each signal carries a
//! human-readable reason. See that module for the per-signal definitions.
//!
//! # Pluggable backends
//!
//! Everything is behind an [`engine::MemoryEngine`] interface that MemoryWhale
//! owns, so the storage/retrieval backend is pluggable: the built-in scorer by
//! default, or — with the off-by-default `mempalace` feature — an
//! `engine::MemPalaceEngine` that talks to a local `mempalace-mcp` server over
//! MCP. Callers never change.
//!
//! # Features
//!
//! - `embeddings` (off by default) — semantic similarity via a network embedder
//!   (`embed::OllamaEmbedder` + `BuiltinEngine::with_embedder`). Off, retrieval
//!   is purely lexical/BM25 and pulls in no network dependency.
//! - `mempalace` (off by default) — the MemPalace MCP backend above.

pub mod embed;
pub mod engine;
#[cfg(feature = "mempalace")]
mod mcp;
pub mod policy;
pub mod privacy;
pub mod scorer;
pub mod sqlite;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single memory item. (The built-in engine scores over these; the MemPalace
/// adapter maps its hits into the same shape.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: i64,
    pub text: String,
    pub created_at: DateTime<Utc>,
    /// When this memory was last retrieved/used — drives recency.
    pub last_used: DateTime<Utc>,
    /// How many times it has been reinforced (mentioned/used).
    pub mentions: u32,
    /// Stored importance weight in `[0,1]`.
    pub importance: f32,
    /// Links / entities / repo associations (used for task relevance and graph).
    pub tags: Vec<String>,
    /// Optional precomputed embedding. When present (and the query is embedded),
    /// similarity is semantic (cosine); otherwise it falls back to lexical.
    #[serde(default)]
    pub embedding: Option<Vec<f32>>,
}

/// A retrieval request plus the context that makes scoring explainable.
#[derive(Debug, Clone)]
pub struct Query {
    pub text: String,
    /// "Now" is passed in (not read from the clock) so scoring is deterministic
    /// and unit-testable.
    pub now: DateTime<Utc>,
    /// Current task context (e.g. repo/file/topic) for task-relevance.
    pub task_tags: Vec<String>,
    /// Recency half-life in days (score halves every `half_life_days`).
    pub half_life_days: f32,
}

impl Query {
    pub fn new(text: impl Into<String>, now: DateTime<Utc>) -> Self {
        Self {
            text: text.into(),
            now,
            task_tags: Vec::new(),
            half_life_days: 14.0,
        }
    }

    pub fn with_task(mut self, tags: Vec<String>) -> Self {
        self.task_tags = tags;
        self
    }
}

/// How much each signal counts toward the blended score. These are the mixing
/// weights of the weighted mean in [`scorer`]; [`Weights::default`] is the tuned
/// production blend. Pass a custom set via [`engine::BuiltinEngine::with_weights`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Weights {
    pub similarity: f32,
    pub recency: f32,
    pub importance: f32,
    pub reinforcement: f32,
    pub task: f32,
}

impl Default for Weights {
    fn default() -> Self {
        Self {
            similarity: 0.40,
            recency: 0.20,
            importance: 0.15,
            reinforcement: 0.10,
            task: 0.15,
        }
    }
}

/// One interpretable scoring signal and its contribution. A [`ScoredMemory`]
/// carries one per signal (similarity, recency, importance, reinforcement,
/// task). [`Signal::contribution`] is its `weight × score` share of the blend,
/// or `0` when the signal doesn't apply.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub name: String,
    pub weight: f32,
    /// Raw signal strength in `[0,1]`.
    pub score: f32,
    /// Whether this signal applies (e.g. task-relevance is inert with no task).
    pub applicable: bool,
    /// Human-readable explanation, e.g. "mentioned 27×" or "last used today".
    pub detail: String,
}

impl Signal {
    pub fn contribution(&self) -> f32 {
        if self.applicable {
            self.weight * self.score
        } else {
            0.0
        }
    }
}

/// A memory with its blended score and the signals that produced it — the unit
/// of explainable retrieval. Use [`ScoredMemory::reasons`] for the ranked
/// "retrieved because…" strings, or [`ScoredMemory::explain`] for the full
/// per-signal audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredMemory {
    pub memory: Memory,
    /// Blended relevance in `[0,1]`.
    pub score: f32,
    pub signals: Vec<Signal>,
}

impl ScoredMemory {
    pub fn percent(&self) -> u32 {
        (self.score * 100.0).round().clamp(0.0, 100.0) as u32
    }

    /// Top human-readable reasons, strongest contribution first.
    pub fn reasons(&self) -> Vec<String> {
        let mut active: Vec<&Signal> = self
            .signals
            .iter()
            .filter(|s| s.applicable && s.score >= 0.15)
            .collect();
        active.sort_by(|a, b| {
            b.contribution()
                .partial_cmp(&a.contribution())
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        active.into_iter().map(|s| s.detail.clone()).collect()
    }

    /// A full, inspectable breakdown — the `memory explain <id>` view.
    pub fn explain(&self) -> String {
        let m = &self.memory;
        let mut out = String::new();
        out.push_str(&format!(
            "memory explain {}\n  \"{}\"\n",
            m.id,
            truncate(&m.text, 72)
        ));
        out.push_str(&format!(
            "  created {} · last used {} · mentioned {}× · importance {:.2}\n",
            m.created_at.date_naive(),
            m.last_used.date_naive(),
            m.mentions,
            m.importance
        ));
        if !m.tags.is_empty() {
            out.push_str(&format!("  links: {}\n", m.tags.join(", ")));
        }
        out.push_str(&format!("  score {}%  =\n", self.percent()));
        for s in &self.signals {
            let mark = if s.applicable { " " } else { "·" };
            out.push_str(&format!(
                "   {} {:<13} w{:.2} × {:.2} = {:+.3}   {}\n",
                mark,
                s.name,
                s.weight,
                s.score,
                s.contribution(),
                s.detail
            ));
        }
        out
    }
}

pub(crate) fn truncate(s: &str, max: usize) -> String {
    let s = s.trim();
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let cut: String = s.chars().take(max).collect();
        format!("{cut}…")
    }
}
