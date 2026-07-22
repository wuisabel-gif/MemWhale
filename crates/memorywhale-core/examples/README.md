# Retrieval eval

Two examples that measure `memorywhale-core`'s ranking quality honestly — semantic
(Ollama embeddings) vs lexical — over a gold set built from your real memory.

- **`dump_eval.rs`** — reads a `memorywhale.sqlite3` into an `eval_data.json`
  *skeleton*: every memory (remembered lessons + command runs), plus empty
  query stubs for you to fill.
- **`eval.rs`** — scores each weight config against the gold set and reports
  P@5 / R@5 / MRR / nDCG@5, isolating embedding latency from scoring latency.

```bash
cargo run -p memorywhale-core --example dump_eval > eval_data.json
#   ... label the queries by hand (see below) ...
cargo run -p memorywhale-core --example eval -- eval_data.json
```

## Two rules that keep the numbers real

These are not optional. Break either and the benchmark grades the scorer
against itself, and every number it prints is a lie that *looks* like evidence.

### 1. Don't measure until the corpus is big enough

`dump_eval` runs on any database, but **the metrics are meaningless on a small
corpus.** P@5 needs at least 5 memories to even be defined in spirit, and
nDCG/MRR don't stabilize until you have dozens of realistic memories and a
spread of queries. Rule of thumb: **~50+ memories before you trust a number.**

Until then the honest status is *"pipeline verified, retrieval not yet
measured."* Those are different claims. Do not let the first slide into the
second. Running the eval on a near-empty DB and reporting the result is the
start of self-flattery, not evidence — build the corpus by using MemoryWhale
for real work first.

### 2. Label `relevant_ids` blind — before you run the eval

`dump_eval` prints the memory `id → text` map to stderr on purpose: you label
from the memory list, **not** from the scorer's output.

The trap: if you write queries and mark relevant IDs while the eval's ranking
is already on screen, you unconsciously rationalize the scorer's ordering into
your "ground truth," and the labels drift toward the model. Then you're grading
the model against itself.

The fix is procedural:

1. Run `dump_eval`, open `eval_data.json`.
2. For each query, write the `text` and mark `relevant_ids` **from the memory
   list alone** — decide what *should* rank before you've seen what does.
3. Save.
4. *Only then* run `eval`.

If you want to revise labels after seeing results, that's fine — but revise
because a label was genuinely wrong, never because "the scorer ranked it low so
maybe it isn't relevant." That reasoning is the leak.

## Reporting

State the embedding model and the cold-embed latency with any result:
"semantic nDCG@5 = 0.81 vs lexical 0.68, at ~40 ms/query on nomic-embed-text."
"Semantic is better" without that denominator is exactly the missing-cost claim
that makes a benchmark untrustworthy.

The embedding cache (`embed_cache.json`) is keyed by text and namespaced by
model — it auto-invalidates if you switch models, so you can't accidentally
compare vectors from two different embedders. It exists to keep re-runs fast; the
*cold* embed latency is the number to report, not the cached ~0 ms.
