# MemoryWhale — Vision

> An inspectable **memory operating system** for AI agents.

## The reframe

The narrow framing is "a local-first terminal memory system" — which sounds like
a note-taking app. The real idea is bigger:

- **Wrong question:** *How do I save chat history?*
- **Right question:** *How should an AI remember?*

Storage is not the innovation — anyone can write `INSERT INTO memories ...`. The
hard problem is **retrieval**: when an agent has 500,000 memories, *which 12*
should it surface right now? Human cognition is mostly retrieval, decay, and
consolidation — not perfect recording. MemoryWhale is built around that whole
**lifecycle**, made **visible** and **owned by the user**.

```
   Capture → Filter → Consolidate → Link → Retrieve → Explain → Forget
```

## Where MemoryWhale sits (and what it is *not*)

MemoryWhale is the **OS shell** — the part you see, trust, and control. It ships a
**built-in store so it works standalone**, and it doesn't try to reinvent a heavy
retrieval engine, because a good one already exists in **MemPalace** (knowledge
graph, hybrid BM25 + vector retrieval, temporal validity, importance weighting,
consolidation/compression). MemoryWhale's job is the layer nobody ships well:
**the lifecycle UI, explainability, and forgetting.**

**No fork required.** MemPalace is MIT-licensed and exposes an **MCP server**, so
MemoryWhale talks to it as a *client* — like calling an API — never by copying or
vendoring its code. MemPalace is therefore a **pluggable, optional backend**, not
a dependency: MemWhale defines its own `MemoryEngine` interface, with a
`MemPalaceEngine` adapter on one side and a built-in SQLite engine on the other.

```
        Claude ─┐
        Codex  ─┤
        Gemini ─┤   (any agent, via Delphin / MCP)
        tools  ─┘
                │  capture
        ┌───────▼─────────────────────────────────┐
        │  MemoryWhale  — the memory OS *shell*     │
        │  graph view · explain-why · forget UI ·   │
        │  importance/decay controls · timelines    │
        └───────┬─────────────────────────────────┘
                │  MemoryEngine interface (MemWhale owns this)
        ┌───────▼──────────────────┬──────────────────────────┐
        │  Built-in SQLite engine   │  MemPalace (optional)     │
        │  works out of the box     │  via MCP — KG + hybrid     │
        │                           │  retrieval, consolidation │
        └───────────────────────────┴──────────────────────────┘
```

**Ecosystem:** **Delphin → communication · MemoryWhale → memory · MemPalace →
optional engine.** The memory belongs to *you*, not to any single model.

## Principles

1. **Memory is retrieval, not storage.** The product is judged by *which* 12 of
   500k memories it returns — and by being able to explain why.
2. **Memories evolve.** They gain weight when reinforced and fade when unused.
   `"I use Rust"` (mentioned 27×, importance 0.98) is not stored equally to
   `"I ate pizza"` (mentioned once, importance 0.01).
3. **Consolidate the index, never the source.** We compress what we *show*, not
   what we *keep* — preserving MemPalace's verbatim guarantee while still letting
   the working memory shrink over time. (See "The verbatim tension" below.)
4. **Everything is explainable.** No opaque "retrieved memory." Always *why*.
5. **User-owned and local-first.** One memory across every agent, on your machine.

## The lifecycle

### 1. Capture — memory isn't just chat
The biggest opportunity is breadth. Remember **everything** worth remembering:
terminal commands, git commits, compiler errors, PDFs, code reviews, browser
pages, emails, agent conversations. (Today: commands + agent turns + notes.)
This is why "memory OS" beats "chat history."

### 2. Filter
Not everything deserves to persist. Score incoming items for salience before they
earn a place (importance, novelty, task-relevance). Cheap, fast, at write time.

### 3. Consolidate
Humans compress. Instead of keeping
`"I like Rust"` / `"I enjoy Rust"` / `"I'm building Rust tools"` as three rows,
the system derives a persistent fact:
> *User primarily develops systems software in Rust.*
The **working set shrinks over time instead of growing forever** — while the
originals remain in the verbatim store for inspection and trust.

### 4. Link — build a memory graph
Memories are not independent rows. They form a graph:
```
Rust
├── Cargo
├── Tokio
├── Axum
└── Delphin
```
Retrieval becomes **graph traversal**, not just SQL filtering — neighbors,
co-occurrence, and relationships, not only direct matches.

### 5. Retrieve — the hard problem
Ranking blends multiple signals into a single relevance score:
- **semantic similarity** (embeddings)
- **recency** (when last seen)
- **decay** (age-based fade)
- **importance** (learned weight)
- **reinforcement** (how often mentioned/used)
- **task relevance** (tied to current repo / file / goal)
- **forgetting** (low-score memories drop out of the working set)

### 6. Explain — the differentiator
Never just "Retrieved memory." Show the reasoning:
```
Retrieved because:
  92%  semantic similarity
       mentioned yesterday
       linked to the current repository
  0.84 importance score
```
This is what earns trust, and it's exactly what a visual app can do that a CLI
cannot.

### 7. Forget — make it visible and controllable
Decay is a feature, not data loss. The user can see what's fading and why, pin
what matters, and let the rest sink — closing the loop so memory stays small,
relevant, and honest.

## Inspectability

Everything is explainable on demand:
```
memory explain 123

Created:        2026-06-18
Referenced:     42 times
Last used:      today
Importance:     0.93
Retrieved via:  similarity + recency
Links:          Rust, Cargo, Delphin
Source:         <verbatim original, never summarized>
```

## The verbatim tension (a deliberate choice)

Consolidation ("the base gets smaller") appears to conflict with MemPalace's
founding rule — *verbatim always, never summarize.* MemoryWhale resolves it by
splitting the two layers:

- **Source layer (MemPalace):** verbatim, append-only, never compressed. The
  ground truth you can always inspect.
- **Working memory (MemoryWhale):** a consolidated, weighted, decaying *index*
  over that source — the small evolving view an agent actually retrieves from.

You get the smaller, evolving memory **and** keep every original word.

## Roadmap (phased)

1. **Reposition + vision** ✅ (this document).
2. **Explainable retrieval** — a scorer (similarity + recency + decay +
   importance + reinforcement) that returns ranked memories *with reasons*; wire
   `memory explain <id>` and "retrieved because…" into the UI.
3. **Memory graph view** — render links/neighbors; retrieval as traversal.
4. **Importance + decay** — reinforcement counting, age decay, pin/forget
   controls, surfaced in the UI.
5. **Consolidation** — derive persistent facts from repeated mentions (index
   only; sources preserved).
6. **Capture breadth** — beyond chat: commits, errors, PDFs, pages, emails.
7. **Engine integration** — *shipped*: `MemPalaceEngine` is a real MCP client
   behind the `MemoryEngine` interface, gated on the off-by-default `mempalace`
   cargo feature (`cargo build -p mw-memory --features mempalace`). The built-in
   SQLite engine stays the zero-setup default and the default build gains no
   dependencies.
8. **Multi-agent** — one user-owned memory shared across Claude / Codex / Gemini.

## Why the name fits

A whale has enormous memory, a long lifespan, migration (memory across time),
intelligence, and communication. It also fits the ecosystem: **Delphin** (the
dolphin) for communication, **MemoryWhale** for memory — related pieces of
infrastructure, not isolated tools.

## In one line

Not just storage — **infrastructure for persistent, transparent, user-owned
memory that any AI agent can use.**
