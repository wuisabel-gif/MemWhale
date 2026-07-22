# MemoryWhale benchmarks

Evidence that the memory is worth having: it finds the right past memory, and
that memory changes whether a coding agent solves the failure. Three evals, from
most rigorous (deterministic, reproducible) to most illustrative (LLM-driven):

| Eval | Question | Deterministic? | Headline |
|---|---|---|---|
| [Retrieval quality](BENCHMARKS.md) | Does the scorer rank the right memory? | ✅ yes | Wins the intent set 0.83 vs 0.44 (keyword) |
| [Memory shortcut](SHORTCUT_EVAL.md) | Is a recurring failure's fix retrievable? | ✅ yes | Fix surfaced for ~95% of recurring failures |
| [Agent eval](agent_eval/AGENT_EVAL.md) | Does having the memory help an agent solve it? | ❌ LLM-driven | Project-specific fixes: 25% cold → 96% with memory |

## 1. Retrieval quality — [`BENCHMARKS.md`](BENCHMARKS.md)

Offline, deterministic, byte-reproducible. Ranks the explainable `BuiltinEngine`
against a plain keyword baseline and stock SQLite FTS5 over two blind-labeled gold
sets — one pure *term-overlap*, one *intent* (recency/importance/tags decide the
answer, not wording).

```bash
cargo run -p memorywhale-core --example benchmark -- benchmarks/
```

Each system wins the set it's built for: lexical baselines win pure text-match;
the blended engine wins intent (**recall@1 0.83 vs 0.44 keyword / 0.42 fts5**).
Full tables and the honest misses are in the file.

## 2. Memory shortcut — [`SHORTCUT_EVAL.md`](SHORTCUT_EVAL.md)

Offline, deterministic. Over a set of recurring failures whose fix is already in
memory, measures whether the real `mw-mcp` tool paths (`similar_failures`,
`search_memory`) surface the fix. Headline: **the known fix is surfaced for ~95%
of recurring failures** (combined@5); the cold-agent baseline is 0 by
construction. It's a retrieval *ceiling*, not an agent solve-rate — that's eval 3.

## 3. Agent eval — [`agent_eval/AGENT_EVAL.md`](agent_eval/AGENT_EVAL.md)

**LLM-driven, non-deterministic, synthetic tasks** — a controlled demonstration,
not a field measurement (read the caveats in the file). Runs a model as the
agent-under-test, with vs. without the relevant memory injected, and judges the
proposed fix against the gold resolution.

| task set | cold | with memory | delta |
|---|---|---|---|
| common/textbook errors | 94% | 100% | +6 pts |
| **project-specific fixes** | **25%** | **96%** | **+71 pts** |

The contrast is the point: memory is negligible on errors a model already knows,
and decisive on the project-specific gotchas that dominate real debugging.

## Reading the three together

Evals 2 and 3 compose into the end-to-end story: the fix is *retrievable* ~95% of
the time, and once retrieved the agent *applies* it (96% vs 25% cold on
project-specific failures). Eval 1 is the regression guard underneath — it keeps
the ranking honest whenever the scorer changes.
