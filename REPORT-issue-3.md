# Issue #3 — Reproducible offline retrieval benchmark

Branch: `issue/3-benchmark`

## What changed

| File | Change |
|------|--------|
| `benchmarks/corpus.json` | New. 37 synthetic-but-realistic captured terminal sessions (Rust compiler errors incl. E0308 camera driver, git failures, flaky tests, cross-compile/Jetson OOM, remembered fixes). All secrets obviously fake (`AKIAFAKEFAKEFAKE1234`, `crates-io-TOKEN-NOT-REAL-000`). |
| `benchmarks/questions.json` | New. 30 queries with hand-labeled `relevant_ids`; some carry `task_tags` to exercise the task-relevance signal. |
| `crates/mw-memory/examples/benchmark.rs` | New. Deterministic, offline harness. Reads `benchmarks/`, fixed `now`, compares 3 systems, computes recall@1/recall@5/MRR, writes per-question `results/qNN.json` + `results/summary.json`, prints a table. |
| `benchmarks/results/*.json` | New. 30 per-question files + `summary.json`, committed. |
| `benchmarks/BENCHMARKS.md` | New. Exact reproduce command, determinism check, methodology, results table. |
| `README.md` | One-line headline (recall@5 = 0.74) added to the Developing section. |

No schema changes. No new dependencies — `rusqlite` (bundled, FTS5-enabled) was already a dev-dependency of `mw-memory`. No network / no API keys / no embedding downloads.

## Systems & why only three
1. `builtin` — `BuiltinEngine`, default weights, lexical similarity (no embedder).
2. `keyword` — substring/term-overlap baseline.
3. `fts5` — in-memory SQLite FTS5 (bm25) over the same text.

No fourth "local embedding" row: `embed.rs` ships only an Ollama-backed embedder (needs a running local server — not offline/deterministic), so it is intentionally excluded.

## Results (37 memories, 30 queries, now=2026-07-19)

| system  | recall@1 | recall@5 | MRR   |
|---------|----------|----------|-------|
| builtin | 0.389    | 0.744    | 0.615 |
| keyword | 0.889    | 0.983    | 0.983 |
| fts5    | 0.889    | 0.989    | 0.983 |

Headline: built-in scorer recall@5 = 0.744. Lexical baselines score higher because this is a pure text-match gold set — `builtin` deliberately blends recency/importance/reinforcement, which reorders near-ties (e.g. ranks the recent fix above the original error for the E0308 query). Discussed honestly in BENCHMARKS.md.

## Reproduce
```
cargo run -p mw-memory --example benchmark -- benchmarks/
git diff --exit-code benchmarks/results   # no output = byte-identical
```
Verified: two runs produce byte-identical `benchmarks/results/`.

## Build / test
- `cargo build --workspace` — passes.
- `cargo test --workspace` — passes (0 failures).

## Decisions
- New `benchmark.rs` instead of editing `eval.rs`: `eval.rs` is an Ollama semantic weight sweep (network, non-deterministic by design); kept it and its README untouched.
- Ranking determinism: stable sorts + explicit id tie-breaks (keyword), `ORDER BY bm25, rowid` (fts5), fixed corpus/insert order.
- Blind labeling per the anti-self-flattery rules in `examples/README.md`.
