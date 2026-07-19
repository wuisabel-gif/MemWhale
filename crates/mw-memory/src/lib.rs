//! `mw-memory` — explainable retrieval for MemoryWhale.
//!
//! The differentiator: retrieval that returns not just *which* memories, but
//! *why* — a blended score over interpretable signals (similarity, recency,
//! importance, reinforcement, task-relevance), each with a human-readable reason.
//!
//! Everything is behind a [`engine::MemoryEngine`] interface that MemoryWhale
//! owns, so the storage/retrieval backend is pluggable: a built-in scorer today,
//! a `MemPalace` adapter (over MCP) tomorrow — without changing callers.

pub mod embed;
pub mod engine;
pub mod scorer;
pub mod sqlite;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A single memory item. (The built-in engine scores over these; a MemPalace
/// adapter would map its drawers/triples into the same shape.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: i64,
    pub text: String,
    pub created_at: DateTime<Utc>,
    /// When this memory was last retrieved/used — drives recency.
    pub last_used: DateTime<Utc>,
    /// How many times it has been reinforced (mentioned/used).
    pub mentions: u32,
    /// Stored importance weight in [0,1].
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

/// How much each signal counts toward the blended score.
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

/// One interpretable scoring signal and its contribution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Signal {
    pub name: String,
    pub weight: f32,
    /// Raw signal strength in [0,1].
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

/// A memory with its blended score and the signals that produced it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoredMemory {
    pub memory: Memory,
    /// Blended relevance in [0,1].
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
