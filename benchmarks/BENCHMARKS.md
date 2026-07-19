# mw-memory retrieval benchmark

A reproducible, **offline** recall benchmark for the built-in explainable scorer.
No API keys, no network, no embedding downloads, no local database — a stranger
with a clean clone runs one command and gets the numbers below, byte-for-byte.

## Reproduce

```bash
cargo run -p mw-memory --example benchmark -- benchmarks/
```

That regenerates every file under `benchmarks/results/` (30 per-question
`qNN.json` files + `summary.json`). It is deterministic: run it twice and the
results are byte-identical to the committed files.

```bash
# determinism check
cargo run -q -p mw-memory --example benchmark -- benchmarks/
git diff --exit-code benchmarks/results   # no output = reproduced exactly
```

## What's measured

- **Corpus** (`corpus.json`): 37 synthetic-but-realistic captured terminal
  sessions — Rust compiler errors (`E0308` in a camera driver, borrow-checker,
  trait bounds), git failures (non-fast-forward pushes, detached HEAD, merge
  conflicts, LFS), flaky tests (races, port-in-use, timeouts), cross-compile /
  Jetson OOM builds, and the remembered fixes for each. Any tokens are
  obviously fake (e.g. `AKIAFAKEFAKEFAKE1234`).
- **Questions** (`questions.json`): 30 queries, each with hand-labeled
  `relevant_ids` (e.g. *"what was the exact E0308 error in the camera driver
  build"* → item 1). Labels were written from the corpus text alone, before
  looking at any ranker's output, so the gold set doesn't drift toward the
  scorer (see `crates/mw-memory/examples/README.md`).
- **Determinism**: fixed `now = 2026-07-19T12:00:00Z` for recency, fixed corpus
  order (drives FTS5 insert order and lexical tie-breaks).

Metrics are binary-relevance **recall@1**, **recall@5**, and **MRR**, averaged
over the 30 queries and computed over each system's full ranking of the corpus.

## Systems compared

| # | System    | What it is |
|---|-----------|------------|
| 1 | `builtin` | `mw-memory`'s `BuiltinEngine`, default weights (similarity 0.40, recency 0.20, importance 0.15, reinforcement 0.10, task 0.15). Lexical similarity — no embedder attached. |
| 2 | `keyword` | Plain baseline: rank by how many distinct query terms substring-match the memory text. |
| 3 | `fts5`    | In-memory **SQLite FTS5** (bm25) index over the same text. |

There is **no local-embedding row**: `embed.rs` only ships an Ollama-backed
embedder, which needs a running local server and so is neither offline nor
deterministic. It is intentionally excluded from the committed numbers.

## Results

| system  | recall@1 | recall@5 | MRR   |
|---------|----------|----------|-------|
| builtin | 0.389    | 0.744    | 0.615 |
| keyword | 0.889    | 0.983    | 0.983 |
| fts5    | 0.889    | 0.989    | 0.983 |

**Headline:** the built-in scorer reaches **recall@5 = 0.744** on this set.

**Honest read of the gap.** On a pure *text-match* gold set the two lexical
baselines beat the blended scorer, and that is expected, not a bug: recall here
rewards nothing but term overlap, while `builtin` deliberately mixes in recency,
importance, reinforcement, and task-relevance. Those signals reorder near-ties —
e.g. for the E0308 query it ranks the more-recent, more-reinforced *fix* (item 2)
above the original *error* (item 1), costing recall@1 while still catching both
inside the top 5. The built-in engine's value is explainable, context-aware
ranking (the *why*), which a recall@k-over-text-match benchmark does not credit;
these numbers are the floor it holds on the part that *is* measurable this way.

To see similarity-only behavior, the scorer supports custom `Weights`
(`similarity: 1.0`, everything else `0.0`) — that path is exercised in
`examples/eval.rs`.
