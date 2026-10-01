# Terminal-Bench pilot report

**Date: September 30, 2026. Status: plumbing verified, no score yet.**

Before a full run we ran small pilots of the [harness](README.md) to check that
both agents work, that MemoryWhale is really wired into run B, and what a task
costs. No with/without-memory comparison came out of it, so **this report
claims no effect of MemoryWhale on Terminal-Bench**, in either direction.

Model names below are the ones Claude Code reported in its own logs. Costs are
Claude Code's estimate at list prices (`total_cost_usd`), not an invoice.

## What ran

Dataset `terminal-bench/terminal-bench@4.0.0`, Harbor 0.23.0, local Docker,
`-n 1 -k 1`, Harbor's dataset order.

### Pilot 1: Sonnet 5.5, baseline only

| Task | Result | Steps | Time | Cost |
|---|---|---|---|---|
| layout-config-recreation2 | stopped: API credit ran out | 73 | 24 min | $4.83 |

### Pilot 2: Haiku 4.5, one task each

| | Baseline | With MemoryWhale |
|---|---|---|
| Task | layout-config-recreation2 | layout-config-recreation2 |
| Verifier | 8 of 9 checks, failed | 8 of 9 checks, failed |
| Pixel match (98% required) | 49.2% | 1.9% |
| Steps | 127 | 181 |
| Time | 17 min | 25 min |
| Cost | $1.35 | $1.87 |

With one task the store is empty when the task starts (by design), so memory
had nothing to offer here; the difference between the two columns is
run-to-run variance.

### Pilot 3: Haiku 4.5, five tasks (baseline only)

| Task | Result | Steps | Time | Cost |
|---|---|---|---|---|
| layout-config-recreation2 | agent setup timed out (360 s) | | | |
| photonic-waveguide-routing | agent setup timed out (360 s) | | | |
| mp-checkpoint-consolidation | failed | 90 | 12 min | $1.35 |
| biped-contact-dynamics | failed | 67 | 10 min | $1.11 |
| ks-solver-cpp | stopped: API credit ran out | 17 | 4 min | $0.30 |

The MemoryWhale run of pilot 3 did not get past its first request (credit).

## What we learned

**Run B is wired correctly.** In pilot 2, Claude Code reported the
`memorywhale` MCP server as connected, the Bash hooks recorded all 91 commands
the agent ran (5 of them failures) into the store, and the store was saved back
to the host for the next task.

**The agent saved no lessons.** It recorded commands through the hooks but
never wrote a note (`remember`), and with an empty store it never searched.
Whether it searches on later tasks is the main open question for the full run.

**Agent setup can exceed Harbor's default.** Installing Claude Code inside a
task container took longer than 360 seconds on two tasks. The run commands now
pass `--agent-setup-timeout-multiplier 3`; it only extends setup, not the
agent's working time, and applies to both runs equally.

**Budget per task.** Observed costs ranged from $0.30 to $4.83 per task (Haiku
$0.30 to $1.87, Sonnet $4.83 for one long task). A 5-task pilot of both runs is
roughly $15 to $20 on Haiku; a full Terminal-Bench comparison on Sonnet is
likely several hundred dollars. Check the balance before starting, since a run
that runs out of credit mid-task produces failures that say nothing about the
agent.

**Haiku is for plumbing.** It solved none of the attempted tasks. Use it to
check the setup, and a stronger model for numbers worth publishing.

## Next

1. A 5-task pilot of both runs with enough credit, to see whether later tasks
   search the memory from earlier ones.
2. Then the full comparison on one pinned model, published with per-task
   results for both runs, as the [rules](README.md#rules-that-keep-it-honest)
   require.
