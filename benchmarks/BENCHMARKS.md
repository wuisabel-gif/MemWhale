# mw-memory retrieval benchmark

A reproducible, **offline** recall benchmark for the built-in explainable scorer.
No API keys, no network, no embedding downloads, no local database — a stranger
with a clean clone runs one command and gets the numbers below, byte-for-byte.

## Reproduce

```bash
cargo run -p mw-memory --example benchmark -- benchmarks/
```

That regenerates every file under `benchmarks/results/` (30 per-question
`qNN.json` + `summary.json`) **and** `benchmarks/results_intent/` (18
per-question `iNN.json` + `summary.json`). It is deterministic: run it twice and
every file is byte-identical to the committed ones.

```bash
# determinism check
cargo run -q -p mw-memory --example benchmark -- benchmarks/
git diff --exit-code benchmarks/results benchmarks/results_intent  # no output = reproduced exactly
```

## What's measured

- **Corpus** (`corpus.json`): 37 synthetic-but-realistic captured terminal
  sessions — Rust compiler errors (`E0308` in a camera driver, borrow-checker,
  trait bounds), git failures (non-fast-forward pushes, detached HEAD, merge
  conflicts, LFS), flaky tests (races, port-in-use, timeouts), cross-compile /
  Jetson OOM builds, and the remembered fixes for each. Any tokens are
  obviously fake (e.g. `AKIAFAKEFAKEFAKE1234`).
- **Two gold sets**, each labeled **blind** — `relevant_ids` written from the
  corpus alone, before running any ranker, so the labels can't drift toward the
  scorer (the integrity rules live in `crates/mw-memory/examples/README.md`):
  - **`questions.json`** — 30 **term-overlap** queries (e.g. *"what was the exact
    E0308 error in the camera driver build"* → item 1). Answerable by pure text
    match. This is the set the lexical baselines are built to win.
  - **`questions_intent.json`** — 18 **intent** queries (e.g. *"the most recent
    E0308 fix"*, *"the git fix I rely on most"*, *"what was I doing on the jetson
    last week"*). The distinguishing signal is **context** — recency, importance,
    reinforcement, or task-relevance — not term overlap. Plain keyword/FTS5 is
    near-blind to it; this is the set the blend is built to win.
- **Determinism**: fixed `now = 2026-07-19T12:00:00Z` for recency, fixed corpus
  order (drives FTS5 insert order and lexical tie-breaks).

Metrics are binary-relevance **recall@1**, **recall@5**, and **MRR**, averaged
over each set's queries and computed over each system's full ranking of the corpus.

## Systems compared

| # | System    | What it is |
|---|-----------|------------|
| 1 | `builtin` | `mw-memory`'s `BuiltinEngine`, default weights (similarity 0.40, recency 0.20, importance 0.15, reinforcement 0.10, task 0.15). **Similarity = SQLite FTS5 BM25** keyword relevance over `(text, tags)` — tags weighted 2.0 above body 1.0 — *blended* with the four context signals. No embedder attached. |
| 2 | `keyword` | Plain baseline: rank by how many distinct query terms substring-match the memory text. |
| 3 | `fts5`    | In-memory **SQLite FTS5** (bm25) index over the memory text **only** — the plain keyword signal, with **none** of the context blending. |

**By design, `builtin`'s internal FTS index also indexes each memory's tags
(weighted above the body), while the standalone `fts5` baseline stays text-only —
so the baseline remains a clean apples-to-apples keyword reference.**

`builtin` no longer shares an identical BM25 signal with `fts5` (it adds the tags
column); it starts from the same text relevance, adds tag relevance, then
recency/importance/reinforcement/task reorder it. So the two sets isolate what
that blending — now including tags — buys and costs.

There is **no local-embedding row**: `embed.rs` only ships an Ollama-backed
embedder, which needs a running local server and so is neither offline nor
deterministic. It is intentionally excluded from the committed numbers.

## Results — term-overlap set (`questions.json`, 30 queries)

| system  | recall@1 | recall@5 | MRR   |
|---------|----------|----------|-------|
| builtin | 0.489    | 0.961    | 0.731 |
| keyword | 0.889    | 0.983    | 0.983 |
| fts5    | 0.889    | 0.989    | 0.983 |

**Honest read.** On a pure *text-match* gold set the lexical baselines still win,
and that is expected, not a bug: recall here rewards nothing but term overlap,
while `builtin` deliberately mixes in recency/importance/reinforcement/task *and*
now tag relevance. Those signals reorder near-ties — e.g. for the E0308 query it
ranks the more-recent, more-reinforced *fix* (item 2) above the original *error*
(item 1), costing recall@1 while still catching both inside the top 5. Adding
tags to the BM25 index nudged this set slightly (recall@1 0.556 → 0.489,
recall@5 0.950 → 0.961, MRR 0.781 → 0.731): two queries whose target shares tags
with a neighbour lose rank-1 to it (`q17`, `q23`), while `q29` gains a relevant
item into its top 5. That is the same intended trade — a tag hit reorders
near-ties — and it is exactly what the intent set below rewards. `builtin` does
**not** beat plain keyword/FTS5 here, and it isn't supposed to.

## Results — intent set (`questions_intent.json`, 18 queries)

| system  | recall@1 | recall@5 | MRR   |
|---------|----------|----------|-------|
| builtin | **0.833**| **1.000**| **0.935** |
| keyword | 0.444    | 0.806    | 0.630 |
| fts5    | 0.417    | 0.694    | 0.580 |

**Honest read.** When the query's answer is decided by *context* rather than
wording, the blend wins clearly — it leads on all three metrics. `builtin` gets
*"the git fix I rely on most"* (reinforcement), *"the most recent E0308 fix"*
(recency), and *"what was I doing on the jetson last week"* (task tags) that the
lexical baselines miss. **Indexing tags into the BM25 signal lifted this set
markedly (recall@1 0.667 → 0.833, recall@5 0.917 → 1.000, MRR 0.812 → 0.935):**
queries whose distinguishing concept lives in a tag rather than the body now
match — `i03` *"the most recent compiler error I hit"* (tag `compiler-error`),
`i14` *"the most recent sqlite problem"*, and `i07` *"the flaky test I hit the
most times"* (tag `flaky`) all resolve to rank 1.

It is **not** perfect, and the misses are reported here rather than hidden.
`builtin` still trips (recall@1 miss) on 2 of 18:

- **`i11` "the highest priority lesson"** — the intended item loses to a
  neighbour that is more recent / more reinforced; with importance weighted only
  0.15, a pure-importance intent isn't always decisive.
- **`i18` "the most recent flaky test fix"** — the tag `flaky` now lifts the
  target (item 36) into the top 5, but items 14 and 15 are *equally recent*
  (all four days old) and share the same `test`/`flaky` tags — and item 15 is
  itself a flaky-test fix — so BM25+recency can't single out item 36 at rank 1
  without special-casing. It lands at rank 3 (recall@5 = 1.0).

The two multi-relevant intent queries (`i12`, `i17`) score recall@1 = 0.5 for
`builtin` because only one of the two relevant items lands at rank 1 — both are
inside the top 5 (recall@5 = 1.0 for each).

## Takeaway

Each system wins the benchmark it is designed for: the lexical baselines win pure
term-overlap; the blended `builtin` wins context/intent. The honest one-liner is
not "builtin beats FTS5" — it's **"builtin = FTS5's keyword relevance plus context,
so it trades a little pure-text recall for a decisive edge whenever intent, not
wording, decides the answer."**

To see similarity-only behavior, the scorer supports custom `Weights`
(`similarity: 1.0`, everything else `0.0`) — that path is exercised in
`examples/eval.rs`.
