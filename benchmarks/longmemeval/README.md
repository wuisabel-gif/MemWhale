# LongMemEval retrieval study

[LongMemEval](https://github.com/xiaowu0162/LongMemEval) (Wu et al., ICLR 2025,
MIT license) is a public benchmark for long-term memory. Each question comes
with its own haystack of about 50 timestamped chat sessions and labels the
session(s) that hold the answer. We did not write the questions or the labels.

This study measures only the **retrieval step**: does a ranker put an answer
session near the top? No model is called, so there is no answer accuracy here.

## Protocol

- Dataset: `longmemeval_s`, Hugging Face revision
  `2ec2a557f339b6c0369619b1ed5793734cc87533`, file SHA-256
  `08d8dad4be43ee2049a22ff5674eb86725d0ce5ff434cde2627e5e8e7e117894`.
- 470 questions; the 30 abstention questions (`*_abs`) are excluded, as in the paper.
- Per question: one memory per haystack session (every user and assistant turn),
  created at the session's date. The query is the question, asked at the
  question's date.
- Metrics: `any@k` (an answer session is in the top k), `all@k` (every answer
  session is), and NDCG@10.
- Systems over the same corpus: MemoryWhale's default and Bayesian rankers, two
  ablations of the default blend, plain SQLite FTS5 BM25, and term overlap.

```bash
curl -L -o longmemeval_s.json \
  https://huggingface.co/datasets/xiaowu0162/longmemeval/resolve/2ec2a557f339b6c0369619b1ed5793734cc87533/longmemeval_s
cargo run --release -p memorywhale-core --example longmemeval -- longmemeval_s.json benchmarks/longmemeval/
```

Deterministic: no network, no model, fixed dates. Full numbers, including per
question type, are in [`results.json`](results.json).

## Results

| System | any@5 | all@5 | any@10 | all@10 | NDCG@10 |
|---|---|---|---|---|---|
| MemoryWhale default, 0.13 | 0.945 | 0.740 | 0.985 | 0.879 | 0.802 |
| MemoryWhale Bayesian, 0.13 | 0.962 | 0.751 | 0.983 | 0.864 | 0.817 |
| MemoryWhale default, query-aware recency | 0.966 | 0.817 | 0.983 | 0.896 | 0.877 |
| MemoryWhale Bayesian, query-aware recency | 0.964 | 0.813 | 0.981 | 0.894 | 0.882 |
| Default without recency | **0.970** | **0.834** | **0.987** | **0.904** | **0.901** |
| Similarity signal only | **0.970** | **0.834** | **0.987** | **0.904** | **0.901** |
| Plain BM25 (FTS5) | **0.970** | **0.834** | **0.987** | **0.904** | **0.901** |
| Term overlap | 0.891 | 0.719 | 0.934 | 0.815 | 0.781 |

**Plain BM25 ranks better than MemoryWhale's default here, and the recency
signal is the whole difference.** With recency off, the default blend ranks
exactly like BM25: in this setup importance and reinforcement are the same for
every session and there is no task context, so similarity and recency are the
only signals that can reorder results.

Recency helps one category, temporal reasoning (any@5 0.976 vs 0.961), and costs
more in the others, most in single-session-assistant (0.893 vs 1.000).

## The trade-off

Sweeping the recency weight (all other weights at their defaults) against both
this study and our own [intent gold set](../BENCHMARKS.md):

| Recency weight | LongMemEval NDCG@10 | LongMemEval all@5 | Intent set recall@1 |
|---|---|---|---|
| 0 | 0.901 | 0.834 | 0.583 |
| 0.05 | 0.895 | 0.830 | 0.583 |
| 0.10 | 0.864 | 0.817 | 0.639 |
| 0.20 (default) | 0.802 | 0.740 | 0.833 |

Neither benchmark settles the default on its own. The intent set is small and
hand-written, and was built to reward recency. LongMemEval is chat memory, not
debugging, and its answer sessions are placed at random dates in the haystack.
In real debugging, "the fix from last week" often is the right answer.

What this study showed is that a fixed recency weight treats every query as
time-sensitive. The follow-up, **query-aware recency**, gives recency full weight
only when the query asks about time ("most recent", "latest", "last week") and a
tie-breaker share otherwise:

| | LongMemEval NDCG@10 | LongMemEval all@5 | Term-overlap recall@1 | Intent recall@1 |
|---|---|---|---|---|
| 0.13 default | 0.802 | 0.740 | 0.522 | **0.833** |
| Query-aware recency | **0.877** | **0.817** | **0.822** | 0.778 |

It recovers most of the LongMemEval gap and most of the term-overlap set, and
costs one intent question (`i09`, see [BENCHMARKS.md](../BENCHMARKS.md)). The
[Terminal-Bench harness](../terminal_bench/README.md) is the place to check it
on real debugging work.

## What this does not measure

- Answer accuracy: no reader model is run.
- Embedding retrieval: the committed numbers are lexical only.
- `longmemeval_m` (about 500 sessions per question), where ranking is much harder.
- Debugging data: see the [retrieval benchmarks](../BENCHMARKS.md) and
  [Terminal-Bench harness](../terminal_bench/README.md).
