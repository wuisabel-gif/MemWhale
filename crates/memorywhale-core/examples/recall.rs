//! Demo: explainable retrieval over a small memory set.
//!
//!   cargo run -p memorywhale-core --example recall
//!
//! Shows ranked results with "retrieved because…" reasons, then a full
//! `memory explain <id>` breakdown — the differentiator in action.

use std::sync::Arc;

use chrono::{Duration, Utc};
use memorywhale_core::embed::OllamaEmbedder;
use memorywhale_core::engine::{BuiltinEngine, MemoryEngine};
use memorywhale_core::{Memory, Query};

fn main() {
    let now = Utc::now();
    let m = |id, text: &str, days: i64, mentions, importance, tags: &[&str]| Memory {
        id,
        text: text.into(),
        created_at: now - Duration::days(days + 10),
        last_used: now - Duration::days(days),
        mentions,
        importance,
        tags: tags.iter().map(|s| s.to_string()).collect(),
        embedding: None,
    };

    let memories = vec![
        m(
            143,
            "I use Rust for systems software.",
            0,
            27,
            0.98,
            &["rust"],
        ),
        m(7, "I ate pizza.", 40, 1, 0.01, &[]),
        m(
            22,
            "Use Tokio for the async runtime.",
            3,
            6,
            0.60,
            &["rust", "tokio"],
        ),
        m(
            31,
            "Decided to use Postgres instead of SQLite for the API.",
            9,
            3,
            0.55,
            &["delphin", "db"],
        ),
        m(
            58,
            "Prefer Axum for HTTP services.",
            25,
            2,
            0.40,
            &["rust", "axum"],
        ),
    ];

    // Try semantic mode (local Ollama embeddings); fall back to lexical offline.
    let engine = match BuiltinEngine::new(memories.clone())
        .with_embedder(Arc::new(OllamaEmbedder::default()))
    {
        Ok(e) => {
            eprintln!("(semantic mode: Ollama `nomic-embed-text`)\n");
            e
        }
        Err(_) => {
            eprintln!(
                "(lexical mode — start Ollama + `ollama pull nomic-embed-text` for semantic recall)\n"
            );
            BuiltinEngine::new(memories)
        }
    };

    // 1) A query with task context (current work is on "delphin").
    let q = Query::new("what database did we choose for the service?", now)
        .with_task(vec!["delphin".into()]);

    println!(
        "Query: \"{}\"   (engine: {}, task: delphin)\n",
        q.text,
        engine.name()
    );
    for sm in engine.retrieve(&q, 3) {
        println!(
            "#{:<4} score {:>3}%   \"{}\"",
            sm.memory.id,
            sm.percent(),
            trunc(&sm.memory.text, 56)
        );
        println!("       retrieved because:");
        for r in sm.reasons() {
            println!("         · {r}");
        }
        println!();
    }

    // 2) Full inspectable breakdown for one memory.
    println!("{}", "-".repeat(64));
    if let Some(sm) = engine.explain(31, &q) {
        print!("{}", sm.explain());
    }
}

fn trunc(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        format!("{}…", s.chars().take(n).collect::<String>())
    }
}
