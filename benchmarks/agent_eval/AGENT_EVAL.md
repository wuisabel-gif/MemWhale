# Agent eval — does the memory actually help an agent?

The retrieval benchmarks ([`../BENCHMARKS.md`](../BENCHMARKS.md)) measure whether
MemoryWhale can *find* the right past memory. This eval asks the next question:
when an agent hits a failure, does having that memory **change whether it solves
it** — and by how much.

**Honest framing up front.** Unlike the retrieval benchmarks, this is **not**
deterministic and **not** a field measurement:

- It runs a real LLM (Claude Sonnet) as the agent-under-test, so numbers vary run
  to run. Treat them as a controlled demonstration of the *mechanism*, not a
  precise universal figure.
- The tasks here are **synthetic, hand-authored** scenarios (`tasks_common.json`,
  `tasks_hard.json`), not real captured failures.
- The "with-memory" condition **injects the relevant memory into the prompt** (as
  if `similar_failures`/`search_memory` had returned it). So it measures "once the
  right memory is in hand, does the agent apply it" — the *upper half* of the real
  pipeline. The lower half — "is the memory retrievable at all" — is measured
  separately by the memory-shortcut eval.

## Method

For each task the agent is given the failing command + error text and asked to
propose the concrete fix. Two conditions, 2 trials each:

- **cold** — error only.
- **with-memory** — error + the recorded memory (which contains the fix).

An independent judge (also Sonnet) scores each proposed fix against the gold
resolution. For the project-specific set the judge is **strict**: a plausible but
*generic* fix that doesn't match the specific root cause counts as wrong.

Re-run it with the workflow harness (`mw-mcp-agent-eval-pilot`) over either task
file; the per-task and aggregate numbers print at the end. Because an LLM drives
it, expect small run-to-run variation.

## Results

Two task sets, and the contrast **is** the finding:

| task set | tasks | cold solve-rate | with-memory | delta |
|---|---|---|---|---|
| **common** (textbook errors) | 8 | 94% | 100% | **+6 pts** |
| **project-specific** (non-derivable fixes) | 12 | 25% | 96% | **+71 pts** |

- On **common** errors (`E0308` → cast, `tokio` in dev-deps, git non-fast-forward
  → `pull --rebase`), a strong model already knows the fix cold, so memory adds
  almost nothing.
- On **project-specific** fixes — the fps field that arrives as a JSON string, the
  PCA9685 clock running 4% fast, the ring-buffer capacity that must be a power of
  two, the `SHA256 mismatch` that's really a tag-before-formula-bump race — the
  cold agent proposes the plausible *generic* fix (which the strict judge
  rejects), while the agent *with the memory* identifies the specific cause.

That's the honest thesis: **memory is negligible on errors a model already knows,
and decisive on the project-specific gotchas that dominate real debugging.**

## Composing with retrieval — the end-to-end estimate

This eval injects the memory. Whether the memory is actually *retrievable* is the
memory-shortcut eval's job (see [`../SHORTCUT_EVAL.md`](../SHORTCUT_EVAL.md)):
MemoryWhale surfaces the known fix for ~95% of recurring failures. Composed:

> retrieval (~95% the fix is surfaced) × application (96% vs 25% cold once it is)
> ≈ project-specific failures go from ~1-in-4 solved first-try to near-certain.

## Known limits (don't over-read these numbers)

- Synthetic tasks and injected memory (see framing above).
- `n=2` trials per task — enough to see a large delta, too few for tight error
  bars. A couple of common-set tasks the model happened to solve cold (`h09`
  publish-order, `h12` `include_str!` packaging); one project task hit only 50%
  *with* memory (`h03`, the `thread_local` fix) under the strict judge.
- Single model (Sonnet). A weaker or stronger model would shift the cold baseline.
