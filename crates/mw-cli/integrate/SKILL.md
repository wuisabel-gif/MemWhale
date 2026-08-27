---
name: memorywhale
description: Query and write durable debugging memory recorded by MemoryWhale. Use when debugging a failure that may have happened before, when you need the exact error/flags/output from an earlier attempt, when the user asks "how did we fix this last time?", or once you've figured out why something failed / how a fix worked and it's worth remembering. Works across machines and past sessions.
---

# MemoryWhale memory

MemoryWhale records terminal commands, their arguments, exit codes, output, and
errors into a local SQLite database. That history survives crashes, SSH drops,
and switching machines — it's the record of what was already tried. It also
holds freeform lessons you or a past agent session chose to remember.

## When to read from it

- A build/test/deploy is failing and it might have failed before.
- You need the *exact* earlier error text, flags, or working directory, not a
  paraphrase.
- The user references past work ("last week", "on the Jetson", "how did I fix").

## How to pull the memory

Prefer the MCP tools if the `memorywhale` MCP server is connected:
`recent_errors`, `search_memory`, `get_context`. Otherwise shell out:

```bash
mw context                 # recent failed commands + sessions, compact
mw context --last-error    # just the most recent failure, with its error tail
mw context project:NAME    # scope to a project tag
mw search "linker error"   # full-text search across commands, output, notes, lessons
```

`mw context`/`search_memory` return failed commands with `cwd`, exit code, and
the tail of their error, plus any remembered lessons. Use it to avoid
re-deriving context: check whether the current failure already has a known
cause before proposing a fix.

## When to write to it

Once you've figured out *why* something failed or *how* a fix worked — not
just that it's fixed — save it. That's the part a raw command log doesn't
capture, and it's what saves the next session (yours or a teammate's) from
re-deriving the same conclusion.

Use the MCP `remember` tool if connected, otherwise:

```bash
mw remember "the E0308 in camera-driver was the fps field being a string; fix: parse it as i32"
```

Keep it a self-contained conclusion (what was wrong + what fixed it), not a
narration of the debugging process.

## Note

Captured output is secret-redacted on the way in, but treat it as real project
data. Everything is local — nothing is uploaded.
