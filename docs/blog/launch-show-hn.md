# Launch text drafts

*Drafts — not published. For the maintainer to edit and post.*

## Show HN / r/rust (~150 words)

**MemoryWhale — local memory of what already failed, for you and your coding agent**

Coding agents forget everything between sessions. An agent debugs a build failure
with you, finds the fix, then next session hits the same error and re-derives it
from scratch.

MemoryWhale records what actually happened in your terminal — commands, output,
errors, and the fixes that worked — into local SQLite, and serves it back to your
agent over MCP. Register it once (`claude mcp add memorywhale -- mw-mcp`) and the
agent can ask "have I hit this error before, and did the fix work?" — answered from
observed exit codes, not guesses.

Everything is local; nothing is uploaded; output is secret-scrubbed before it's
written. Rust, MIT, runs on Linux (incl. Jetson) and macOS.

In a controlled eval, agents solved *project-specific* failures 25% cold vs 96%
with the memory in hand. (Common textbook errors: no difference — a model already
knows those.)

```
curl -fsSL https://raw.githubusercontent.com/wuisabel-gif/MemWhale/main/install.sh | sh
```

github.com/wuisabel-gif/MemWhale

---

## One-line / social

> Coding agents are amnesiac. MemoryWhale gives them local memory of what already
> failed — commands, errors, and the fixes that worked — over MCP. Project-specific
> bugs: 25% solved cold → 96% with the memory. Local-first, Rust, MIT.

## Note on the 25% → 96% number

Use it *with* the framing from `agent-memory-of-what-failed.md`: it's a controlled
demonstration on synthetic project-specific tasks with the memory injected, not a
universal field number. The common-error row (no difference) is part of the honest
story — include it.
