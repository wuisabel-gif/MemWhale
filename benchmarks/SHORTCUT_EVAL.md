# Memory-shortcut eval

**Question:** when an agent hits a failure it has hit before, how often is the
fix *already retrievable* from MemoryWhale via the real MCP tools?

**Headline:** **95% of recurring failures have their fix surfaced by MemoryWhale**
(combined `shortcut@5`, 22 scenarios). Of the 21 scenarios whose fix is actually
in memory, at least one tool surfaces it for all 21 at k=5; the 22nd is a
deliberate no-fix control, correctly missed by both tools.

## What this is — and what it is NOT

This measures **retrievability: a ceiling**, not agent solve-rate. It answers
"is the fix in reach?", not "did the agent use it to solve the task faster?" —
that is the follow-up real-agent eval. Everything here is offline and
deterministic: **no network, no LLM, no API keys.** A fixed `now`
(`2026-07-19T12:00:00Z`), the corpus in file order, the task set in file order.

**With / without framing.** The number above is the *with-MCP* shortcut rate.
The *without-MCP* (cold agent) baseline is **0 by construction**: with no memory
there is nothing to retrieve, so every recurring failure is re-derived from
scratch. The eval does not print a "without" row because it is definitionally
zero.

## Reproduce

```bash
cargo run -p memorywhale-cli --example shortcut_eval -- benchmarks/
```

Regenerates `benchmarks/shortcut_results/*.json` **byte-identically** on every
run (verify with a `diff` of two runs). The harness seeds a temporary in-memory
SQLite DB through the production schema + `migrate()`, so retrieval runs the
exact code paths `mw-mcp` uses.

## The two paths (same code as `mw-mcp`)

- **`similar_failures`** — `error_fingerprint(command, error_text)` →
  `error_insight`. The re-hit stderr is fingerprinted exactly as stored failures
  are; a shortcut counts when the fingerprint is **recognized** (occurrences > 0)
  **and resolved** (a later successful run of the same command is on record —
  "you've hit *and fixed* this before"). This is a single fingerprint lookup, so
  it is rank-free: `@1 == @5`.
- **`search_memory`** — the natural-language `query` through the
  `BuiltinEngine`-ranked `load_memories` path (identical to `mw search`). A
  shortcut counts when a labelled `fix_id` note lands in the top-k.
- **combined** — the fix is surfaced by *at least one* tool.

## Results

| path              | shortcut@1 | shortcut@5 |
|-------------------|-----------:|-----------:|
| similar_failures  |      0.909 |      0.909 |
| search_memory     |      0.727 |      0.909 |
| combined          |      0.909 |      0.955 |

22 tasks (21 with a fix in corpus). Per-task detail — fingerprint, occurrence and
resolution counts, and the top-5 retrieved note ids — is in
`benchmarks/shortcut_results/<task-id>.json`.

### Honest misses (not curated away)

- `t08-git-file-too-big` — **similar_failures miss.** The re-hit reports a
  different file size (`214.00 MB` → `231.40 MB`); the fingerprint normalizer
  masks integers but not the decimal fraction, so `.00` vs `.40` fingerprints
  differently. A real limitation, left in.
- `t14-tokio-version-select` — **search_memory miss @5.** The fix note (#21) is
  out-ranked by sibling tokio/cargo notes for that query.
- `t22-no-fix-in-memory` — **control**, no fix in the corpus. Missed by both
  tools (no resolution recorded; no fix note to retrieve). It exists so the eval
  cannot be trivially all-hits and drags the all-task denominator honestly.

## Integrity guardrails

1. **`fix_ids` labelled blind.** Every task's `fix_id` was chosen from
   `corpus.json` alone — the id(s) whose text *states the resolution* — **before
   any retrieval was run**, per the rule in
   `crates/memorywhale-core/examples/README.md`. The task set was committed in a
   separate commit *before* the commit that generated these results, so the
   ordering is auditable in git history.
2. **No task authored to a ranker quirk.** `error_text` is a realistic re-hit of
   the stored failure (drifted line numbers / paths / ids), not a string tuned to
   collide; `query` is a plain question, not reverse-engineered from the scorer's
   output.
3. **Unimpressive numbers reported as-is.** The misses above are kept, and the
   `search_memory@1` rate (0.727) is reported alongside the flattering combined
   number.
4. **Ceiling, not solve-rate.** Because the corpus contains each fix by
   construction, a high combined rate is *expected* — that is what a retrieval
   ceiling looks like. It says the fix is in reach, nothing more.

## Caveats

- Small corpus (37 memories). The recall benchmark's rule of thumb is ~50+
  memories before ranking numbers stabilize; treat `search_memory@k` here as
  indicative, and note the ceiling framing does not depend on ranking precision.
- The seeded resolution run models the real fact that a fixed failure *has* a
  later green run; a never-fixed failure gets none. That coupling is faithful,
  not a thumb on the scale — flip it and the control still fails honestly.
