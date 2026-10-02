# Launch text drafts

*Drafts, not published. For the maintainer to edit and post. Every number here
links back to how it was measured; keep the caveats when you shorten.*

## Show HN: MemoryWhale (~180 words)

**Show HN: MemoryWhale, local memory of what already failed, for you and your coding agent**

Coding agents forget everything between sessions. An agent debugs a build
failure with you, finds the fix, and next session hits the same error and works
it out from scratch.

MemoryWhale records what actually happened in your terminal (commands, output,
errors, and the fixes that worked) into local SQLite and serves it to your agent
over MCP. New in 0.16: with Claude Code or Cursor, it no longer waits to be
asked. When an error comes back, the agent sees one line naming what fixed it
last time; when a fix works, the agent is asked to save the lesson.

Everything stays local, and output is secret-scrubbed before it is written.
Rust, MIT, Linux (including Jetson) and macOS. README and site in 10 languages.

How well does recall work? On LongMemEval, a public long-term memory benchmark
we did not write, search puts the right past session in the top 5 for 97% of
470 questions (retrieval only, no model).

```
brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
mw integrate claude
```

github.com/wuisabel-gif/MemWhale

## Show HN: Joule (~150 words)

**Show HN: Joule, an OpenAI-compatible proxy that measures and cuts the energy of each LLM request**

Point any OpenAI-compatible client at Joule by changing the base URL. Every
response comes back with estimated joules, CO2, and cost in its headers, and
Joule applies prompt optimizations it can explain: caching, routing to smaller
or greener models, and, new in 0.7, sending only the chat history that matters.

That last one ranks older turns by relevance to the latest question instead of
dropping the oldest. On LongMemEval (470 questions, about 104,000-token
histories), it cut 91% of the prompt and kept all of the answer's evidence for
81% of questions; plain truncation to the same size kept it for 5%. That
measures what reaches the model, not answer accuracy; the repo shows how to
check that on your own traffic.

Rust, Apache-2.0. `brew install wuisabel-gif/joule/joule`, `cargo install joule-proxy`,
or `docker run ghcr.io/wuisabel-gif/joule`.

github.com/wuisabel-gif/Joule

## r/rust (~120 words)

**MemoryWhale: a local-first debugging memory for coding agents, in Rust**

I built a Rust CLI that records terminal commands, exit codes, and output into
SQLite (FTS5 for search) and serves them to coding agents over MCP. Agents in
Claude Code and Cursor now get a one-line note when an error comes back that a
later command fixed before, matched on normalized error text.

The retrieval ranker blends BM25 with recency, importance, and reinforcement.
LongMemEval showed that a fixed recency weight hurt, so recency now counts only
when the query asks about time. Write-up and numbers are in `benchmarks/`.

Happy to talk about the scoring, the hook design, or the PTY wrapper (Delphin)
that shows these hints live.

github.com/wuisabel-gif/MemWhale

## r/ClaudeAI or r/cursor (~90 words)

**Claude Code / Cursor: get "the fix was: ..." when an error you've hit before comes back**

MemoryWhale's capture hook records every shell command your agent runs. As of
0.16 it also talks back: when a command fails with an error a later command
fixed before, the agent sees "MemoryWhale: this error was seen once, the fix
was: xcode-select --install". When a fix works, the agent is asked to save the
lesson. It stays quiet otherwise, and `MEMORYWHALE_HOOK_FEEDBACK=0` turns it
off. Local, open source, MIT: `mw integrate claude` or
`mw integrate cursor --capture`.

## One line

> Coding agents forget what already failed. MemoryWhale keeps it locally and,
> when the same error comes back, tells the agent what fixed it last time.
> 97% top-5 recall on LongMemEval. Rust, MIT.

## Notes on the numbers

- **LongMemEval 97%** is retrieval only (an answer session in the top 5), on
  chat memory, not debugging. Source: `benchmarks/longmemeval/README.md`.
- **Joule 91% / 81% vs 5%** is evidence kept after trimming, not answer
  accuracy. Source: Joule's `bench/longmemeval/README.md`.
- **25% to 96%** (the older headline) is a controlled demonstration on
  synthetic project-specific tasks with the memory injected; on common textbook
  errors there was no difference. Use it only with that framing, from
  `agent-memory-of-what-failed.md`.
- **Terminal-Bench** has no score yet (`benchmarks/terminal_bench/PILOT.md`).
  Don't imply one.
