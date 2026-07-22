# Giving coding agents a memory of what already failed

*Draft — not published. For the maintainer to edit and post.*

Coding agents are strangely amnesiac. An agent will debug a gnarly build failure
with you, find the fix, and then — next session — hit the same failure and start
from scratch, proposing the same wrong turns you already ruled out. The knowledge
existed. It just wasn't anywhere the agent could reach.

MemoryWhale is an attempt to fix that: **persistent, local memory of what actually
happened in your terminal — commands, output, errors, and the fixes that worked —
served back to you and to your coding agent over MCP.**

## Where it came from

I was running the same robotics repo on two machines — an NVIDIA Jetson and my
laptop — for USC's Autonomous Underwater Vehicle team. The *code* was shared over
git. The *terminal history* was not. Whichever machine happened to run a build
held its errors, its logs, its half-finished debugging attempts. When a teammate
asked why something had failed, or when a machine rebooted, the exact context —
what command, what flags, what error, what we'd already tried — was just gone.
And the AI agent helping debug had none of it.

Shell-history tools (atuin and friends) sync your command *lines*. That's useful,
but it's the wrong unit. The command line isn't what you need six weeks later —
you need the *outcome*: the error text, the cause, the fix. So MemoryWhale records
outcomes.

## How it captures

Two tiers, so you can trade fidelity for zero-effort:

- **Shell hooks** (`mw hooks install`, zsh/bash/fish/PowerShell) — always-on,
  Atuin-style: every command, its cwd, exit code, and duration, with sub-
  millisecond overhead and a hard rule that a busy or missing database can never
  break your prompt.
- **`mw --live`** — the full-fidelity recorder: complete stdout/stderr transcripts
  of a session, autosaved so an SSH drop or a crash doesn't lose it.

Everything lands in local SQLite. Nothing is uploaded. Output is scrubbed for
common secret shapes (tokens, keys, `password=`) *before* it's written, and you
can gate capture per directory (`.mwignore`, `commands-only`, `off`) so
`~/finances` never gets recorded.

## How it ranks — and why the ranking is legible

Retrieval uses an explainable scorer. A memory's relevance is a weighted blend of
five signals — keyword similarity (SQLite FTS5 BM25), recency, importance,
reinforcement (how often you hit it), and task-relevance — and crucially, **it can
tell you why**. Every result carries per-signal reasons:

```
40%  [note] the E0308 in camera-driver was the fps field being a string; fix: parse as i32
       similarity    w0.40 × 0.10 = +0.040   10% keyword match (BM25)
       recency       w0.20 × 1.00 = +0.200   last used today
       importance    w0.15 × 0.55 = +0.083   importance 0.55
       reinforcement w0.10 × 0.20 = +0.020   mentioned 1×
```

Explainability isn't decoration. It's what lets you trust — and debug — the
ranking, and it's a genuinely distinctive design choice for a memory system.

## How the agent uses it

Register the MCP server once:

```bash
claude mcp add memorywhale -- mw-mcp
```

The agent gets five tools; the important one is **`similar_failures`**. When the
agent hits an error, it passes the text and learns — from *observed exit codes*,
not guesses — how many times that exact failure occurred before and how often a
later run resolved it, plus a pointer to the past occurrence. "You've hit this 3
times; a later run succeeded 2 of 3 times" is a very different starting point than
a cold prompt.

## Does it actually help? The honest answer.

We ran a controlled eval: a model as the agent-under-test, solving failures with
vs. without the relevant memory injected, judged against the known fix. Two task
sets, and the *contrast* is the result:

| task type | solves it cold | with memory |
|---|---|---|
| common/textbook errors | 94% | 100% |
| **project-specific fixes** | **25%** | **96%** |

On common errors — `E0308` → cast, a crate in dev-deps instead of deps, a git
non-fast-forward — a strong model already knows the fix. Memory adds almost
nothing. But on the **project-specific** failures that actually eat your
afternoons — the sensor field that arrives as a quoted JSON string, the servo
that overshoots because a specific chip's clock runs 4% fast, the SHA mismatch
that's really a release-ordering race — the cold agent confidently proposes the
*generic* fix, which is wrong, while the agent with the memory names the real
cause.

That's the whole thesis in one line: **memory is negligible on what a model
already knows, and decisive on the project-specific gotchas that dominate real
debugging.**

### The limitations, stated plainly

- Those numbers are a **controlled demonstration**, not a field measurement: the
  tasks are synthetic and hand-authored, the run is LLM-driven (so it varies), and
  the "with memory" condition *injects* the memory rather than making the agent
  retrieve it. Whether the memory is *retrievable* is measured separately (it
  surfaces the right fix for ~95% of recurring failures in a deterministic eval).
- The scorer's keyword component still loses to plain FTS5 on pure text-match
  queries — by design; it trades that for a decisive edge whenever *intent*, not
  wording, decides the answer.
- Semantic (embedding) search exists but needs a local Ollama server; the default
  build is deliberately dependency-light so it installs on a bare machine and a
  Jetson.

## Try it

```bash
curl -fsSL https://raw.githubusercontent.com/wuisabel-gif/MemWhale/main/install.sh | sh
```

Local-first, MIT-licensed, runs on Linux (incl. Jetson) and macOS. The code, the
benchmarks behind the numbers above, and the MCP setup are at
[github.com/wuisabel-gif/MemWhale](https://github.com/wuisabel-gif/MemWhale).
