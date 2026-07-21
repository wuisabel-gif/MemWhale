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
| 1 | `builtin` | `mw-memory`'s `BuiltinEngine`, default weights (similarity 0.40, recency 0.20, importance 0.15, reinforcement 0.10, task 0.15). **Similarity = SQLite FTS5 BM25** keyword relevance, *blended* with the four context signals. No embedder attached. |
| 2 | `keyword` | Plain baseline: rank by how many distinct query terms substring-match the memory text. |
| 3 | `fts5`    | In-memory **SQLite FTS5** (bm25) index over the same text — the same keyword signal `builtin` starts from, with **none** of the context blending. |

`builtin` is a strict superset of `fts5`'s signal: same BM25 relevance, then
recency/importance/reinforcement/task reorder it. So the two sets isolate exactly
what that blending buys — and costs.

There is **no local-embedding row**: `embed.rs` only ships an Ollama-backed
embedder, which needs a running local server and so is neither offline nor
deterministic. It is intentionally excluded from the committed numbers.

## Results — term-overlap set (`questions.json`, 30 queries)

| system  | recall@1 | recall@5 | MRR   |
|---------|----------|----------|-------|
| builtin | 0.556    | 0.950    | 0.781 |
| keyword | 0.889    | 0.983    | 0.983 |
| fts5    | 0.889    | 0.989    | 0.983 |

**Honest read.** On a pure *text-match* gold set the lexical baselines still win,
and that is expected, not a bug: recall here rewards nothing but term overlap,
while `builtin` deliberately mixes in recency/importance/reinforcement/task. Those
signals reorder near-ties — e.g. for the E0308 query it ranks the more-recent,
more-reinforced *fix* (item 2) above the original *error* (item 1), costing
recall@1 while still catching both inside the top 5. Switching similarity to BM25
did lift `builtin` a lot on this set (recall@1 0.389 → 0.556, recall@5 0.744 →
0.950), but it does **not** beat plain keyword/FTS5 here, and it isn't supposed
to: this set is designed to measure exactly the thing the context signals dilute.

## Results — intent set (`questions_intent.json`, 18 queries)

| system  | recall@1 | recall@5 | MRR   |
|---------|----------|----------|-------|
| builtin | **0.667**| **0.917**| **0.812** |
| keyword | 0.444    | 0.806    | 0.630 |
| fts5    | 0.417    | 0.694    | 0.580 |

**Honest read.** When the query's answer is decided by *context* rather than
wording, the blend wins clearly — it leads on all three metrics. `builtin` gets
*"the git fix I rely on most"* (reinforcement), *"the most recent E0308 fix"*
(recency), and *"what was I doing on the jetson last week"* (task tags) that the
lexical baselines miss.

It is **not** perfect, and the misses are reported here rather than hidden.
`builtin` still trips (recall@1 miss) on 5 of 18:

- **`i14` "the most recent sqlite problem"** — the target text says
  `sqlite3_open_v2`, which FTS5 tokenizes as `sqlite3`, so the exact token
  `sqlite` never matches it and BM25 scores it 0. Keyword's *substring* match
  catches it. (We keep exact-token MATCH to stay apples-to-apples with the `fts5`
  baseline — not tuned away.)
- **`i11` "the highest priority lesson"** and **`i07` "the flaky test I hit the
  most times"** — the intended item loses to a neighbour that is more recent /
  more reinforced; with importance weighted only 0.15, a pure-importance or
  pure-reinforcement intent isn't always decisive.
- **`i03`, `i18`** — the distinguishing concept lives in the *tags* (e.g.
  `flaky`, `compiler-error`), not the memory text, so BM25 can't see it and only
  recency is left to lean on.

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
