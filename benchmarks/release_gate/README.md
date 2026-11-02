# Next-release evidence gate

`python3 benchmarks/release_gate/evaluate.py trials.jsonl` scores paired trials.
It makes no model calls, runs no suggested commands, and opens no memory store.
A nonzero exit means the gate has **not passed**. No model trial has been run
for this change; passing scorer unit tests is not measured debugging benefit.

Collect both `cold` and `memory` for every `(task, trial)`, using the same
starting repository/commit/test command, model/settings and attempt budget.
Use temporary synthetic repositories/stores. Include irrelevant and outdated
memories, not just helpful ones. Alternate condition order, reset the workspace
between runs, and preserve raw test receipts for independent review. A memory
claim of “verified” never substitutes for the harness executing the test.

Each JSONL row requires:

```json
{"task":"calc-add","trial":1,"condition":"memory","kind":"model","attempts":1,"fresh_test_exit":0,"misleading_hint":false,"repository":"synthetic-calc","commit":"starting-commit-sha","test_command":"python -m unittest"}
```

- `kind`: `fixture` for scripted/fake agents; `model` only for actual model runs.
- `attempts`: attempted fixes, including the final successful attempt.
- `fresh_test_exit`: the actual final verification exit code.
- `misleading_hint`: independent review found that a surfaced memory misled the
  attempt. Record this even if the agent eventually recovered.
- Provenance describes the **starting** task snapshot, so pairs are comparable.
  Keep resulting commit, stdout/stderr, model/version, budget and timing in the
  accompanying raw receipts.

At least 20 complete task/trial pairs are required by the command-line gate.
This is a minimum smoke-test budget, not a statistical power calculation.

The scorer reports solves, wasted attempts (all attempts except a verified
success), and misleading hints separately. Its mechanical gate requires model
runs only, more solves with memory, no increase in wasted attempts, and zero
misleading hints. Fixture-only and mixed-kind reports cannot pass. This is a
screening rule, not statistical significance or an authenticated attestation;
reviewers must check sample size, task coverage and raw receipts before release.
Self-labelled JSON cannot prove a real model or fresh test was used. No universal
benefit claim follows from this synthetic benchmark.

Run scorer regressions without inference:

```sh
python3 -m unittest discover -s benchmarks/release_gate -v
```
