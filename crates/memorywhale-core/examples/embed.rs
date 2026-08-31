//! Embedding `memorywhale-core` in your own code.
//!
//!   cargo run -p memorywhale-core --example embed
//!
//! Build a few `Memory` items, construct a `BuiltinEngine`, run a `Query`, and
//! print the top results *with their explainable per-signal scores*. Fully
//! deterministic: no network, no ML embeddings — the default lexical/BM25 path.

use chrono::{Duration, TimeZone, Utc};
use memorywhale_core::engine::{BuiltinEngine, MemoryEngine};
use memorywhale_core::{Memory, Query};

fn main() {
    // A fixed "now" so the recency signal (and the whole run) is reproducible.
    let now = Utc.with_ymd_and_hms(2026, 6, 27, 12, 0, 0).unwrap();

    let mem =
        |id: i64, text: &str, days_ago: i64, mentions: u32, importance: f32, tags: &[&str]| {
            Memory {
                id,
                text: text.into(),
                created_at: now - Duration::days(days_ago + 5),
                last_used: now - Duration::days(days_ago),
                mentions,
                importance,
                tags: tags.iter().map(|s| s.to_string()).collect(),
                embedding: None, // lexical/BM25 path — no embeddings needed
                agent: None,
            }
        };

    let memories = vec![
        mem(
            1,
            "Use Tokio for the async runtime.",
            3,
            6,
            0.60,
            &["rust", "tokio"],
        ),
        mem(
            2,
            "Decided to use Postgres instead of SQLite for the API.",
            9,
            3,
            0.55,
            &["db", "api"],
        ),
        mem(3, "I ate pizza for lunch.", 40, 1, 0.02, &[]),
        mem(
            4,
            "Prefer Axum for HTTP services in Rust.",
            25,
            2,
            0.40,
            &["rust", "axum"],
        ),
    ];

    // Build the engine and query it. `task_tags` activates the task-relevance
    // signal for memories tagged "rust".
    let engine = BuiltinEngine::new(memories);
    let query =
        Query::new("which async runtime should I use in rust?", now).with_task(vec!["rust".into()]);

    println!("query: {:?}   (engine: {})\n", query.text, engine.name());

    let top = engine.retrieve(&query, 3);
    println!("top {} results — retrieved because…\n", top.len());
    for (rank, hit) in top.iter().enumerate() {
        println!(
            "  {}. [{:>3}%] {}",
            rank + 1,
            hit.percent(),
            hit.memory.text
        );
        for reason in hit.reasons() {
            println!("        · {reason}");
        }
        println!();
    }

    // Full per-signal breakdown for the winner — the `memory explain <id>` view.
    if let Some(winner) = top.first() {
        println!("{}", winner.explain());
    }
}
