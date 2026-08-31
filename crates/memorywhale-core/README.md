# memorywhale-core

Explainable retrieval for [MemoryWhale](https://github.com/wuisabel-gif/MemWhale):
a `MemoryEngine` interface, a built-in scorer that ranks memories with a
per-signal reason for every hit (similarity, recency, importance, reinforcement,
task), and an optional MemPalace backend over MCP. No embedder is required —
`BuiltinEngine` runs a lexical FTS5/BM25 similarity signal out of the box, with
semantic embeddings as an opt-in.

## Embed

```rust
use chrono::Utc;
use memorywhale_core::engine::{BuiltinEngine, MemoryEngine};
use memorywhale_core::{Memory, Query};

let now = Utc::now();
let memories = vec![
    Memory {
        id: 31,
        text: "Decided to use Postgres instead of SQLite for the API.".into(),
        created_at: now,
        last_used: now,
        mentions: 3,
        importance: 0.55,
        tags: vec!["db".into()],
        embedding: None,
        agent: None,
    },
    // ...more memories
];

let engine = BuiltinEngine::new(memories);
let query = Query::new("what database did we choose?", now);

for sm in engine.retrieve(&query, 3) {
    println!("#{} — {}%", sm.memory.id, sm.percent());
    for reason in sm.reasons() {
        println!("  · {reason}");   // explainable per-signal scores
    }
    // sm.explain() prints the full per-signal breakdown.
}
```

Run it end to end — a few memories, a query, and the full per-signal
`explain()` breakdown of the winner:

```
cargo run -p memorywhale-core --example embed
```

## Features

Both off by default:

- `embeddings` — semantic similarity via a local Ollama embedder
  (`BuiltinEngine::with_embedder`). Off, retrieval is purely lexical/BM25 and
  pulls in no network dependency (`ureq`).
- `mempalace` — route retrieval through a local MemPalace `mempalace-mcp`
  server. Adds no dependencies.

See `examples/` for the `embed` walkthrough above, semantic (Ollama) recall
(`--features embeddings`), and the deterministic ranking benchmark.
