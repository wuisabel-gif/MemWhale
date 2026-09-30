# Terminal-Bench: does MemoryWhale help a real agent?

The [agent eval](../agent_eval/AGENT_EVAL.md) uses synthetic tasks we wrote.
This eval uses [Terminal-Bench](https://www.tbench.ai), a public benchmark of
real terminal tasks (builds, installs, debugging) with its own verifiers, run
through its official harness, [Harbor](https://github.com/laude-institute/harbor).
We did not write the tasks or the grading.

**Status: harness ready, not yet run.** No numbers are published until both
runs below finish.

## The comparison

Same tasks, same model, same Claude Code version, same settings. One difference:

| Run | Agent | Memory |
|---|---|---|
| A — baseline | Harbor's stock `claude-code` | none |
| B — MemoryWhale | [`memorywhale_agent.py`](memorywhale_agent.py) | carried across tasks |

Run B adds exactly what a real user gets from `mw integrate claude`:

- `mw-mcp` registered as an MCP server (the six memory tools);
- Claude Code's Bash hooks recording every command, error, and exit code;
- a four-sentence system-prompt note that the memory exists
  (`MEMORY_PROMPT` in the agent file).

The store starts **empty**. Tasks run one at a time in a fixed order and the
store is carried from each task to the next, so a task can only recall what
the agent itself did on *earlier, different* tasks. Nothing from the task's
tests, solutions, or other runs is ever loaded.

This is the "continual" setting, not the standard leaderboard setting (where
every task starts cold). We report it as such.

## Rules that keep it honest

- `-n 1`: one trial at a time; the carried store is a single host directory.
- `-k 1`: one attempt per task, so no task can recall its own earlier attempt.
- Same task order for A and B (Harbor's dataset order), and the same `-l` limit.
- Pin the model (`-m`) and Claude Code version (`--ak version=...`) for both runs.
- Delete `store/` before starting run B. Keep the raw store private: it holds
  every captured command and output from the run, which can include tokens or
  paths. To show what was remembered, publish only a redacted export
  (`MEMORYWHALE_DATA_DIR=benchmarks/terminal_bench/store mw export`, which
  applies capture redaction) after reading it through yourself.
- Publish both scores, per-task pass/fail for both, cost, and every failure,
  including tasks where memory made things worse.

## Requirements

Only the maintainer can set these up:

- Docker (local runs) or a [Modal](https://modal.com) account (full runs).
- `ANTHROPIC_API_KEY` in your environment. Costs are real: every task is a
  full Claude Code session, twice (A and B).
- Harbor 0.23.0, pinned because the agent builds on its Claude Code agent:
  `uv tool install 'harbor[modal]==0.23.0'`. The agent refuses to load if a
  newer Harbor stops calling the hook it relies on.

## Run it

Start with a small pilot (10 tasks, local Docker) to check the plumbing and
estimate cost before a full run. From the repository root:

```bash
export PYTHONPATH="$PWD"
MODEL=anthropic/claude-sonnet-5
DATASET=terminal-bench/terminal-bench@4.0.0
# Pin one Claude Code version for both runs (your local one here) and record it.
CLAUDE_CODE="$(claude --version | cut -d' ' -f1)"

# A: baseline
harbor run -d "$DATASET" -a claude-code -m "$MODEL" --ak version="$CLAUDE_CODE" -e docker -n 1 -k 1 -l 10 \
  --job-name tb-pilot-baseline

# B: with MemoryWhale (fresh store)
rm -rf benchmarks/terminal_bench/store
harbor run -d "$DATASET" -a benchmarks.terminal_bench.memorywhale_agent:MemoryWhaleClaudeCode \
  -m "$MODEL" --ak version="$CLAUDE_CODE" -e docker -n 1 -k 1 -l 10 \
  --job-name tb-pilot-memorywhale

harbor view jobs
```

For the full run, drop `-l 10`, use `-e modal`, and give both jobs new names.
Record the Harbor version, dataset version, model, and Claude Code version.

## What to expect

Memory can only help when a later task hits something an earlier one already
solved: the same missing library, toolchain quirk, or service setup. Terminal-Bench
tasks are varied, so the gain may be small, and small is a real result too.
If B is not better than A, we publish that.
