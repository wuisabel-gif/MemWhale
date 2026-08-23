# Use cases

MemoryWhale preserves debugging *evidence* — the command, its output, the
error, and eventually the fix — in a local, searchable store that both people
and coding agents can query. This page walks through the three situations it
was built for, end to end.

Each scenario follows the same arc: something fails, the context around the
failure would normally evaporate, and MemoryWhale instead turns it into
retrievable memory.

## 1. The shell-centric debugger

You spend your day in `cargo check`, `pip install`, `docker build`,
`git bisect`. Every so often you hit an error you *know* you have solved
before — and shell history is useless, because it remembers the command but
not the output, the error tail, or what actually fixed it.

**Before MemoryWhale:** you re-derive the fix from scratch, or dig through
dead scrollback hoping the terminal is still open.

**With MemoryWhale:**

```bash
# when it first happened (months ago):
mw-run -- cargo check                      # captured with output + exit code
mw remember "the E0308 was the fps field being a string; fix: parse it as i32"

# today, same error:
mw search "E0308"                          # ranked hits across commands + lessons
mw explain <id>                            # why this memory ranks
```

The search returns the old failing run *and* the lesson linked to it, so the
fix arrives attached to its evidence rather than as a vague memory.

**Components involved:** `mw-run` / `mw` capture, `command_runs` storage,
`mw search`, `remember` lessons, optional `mw link` between evidence and fix.

## 2. The multi-machine or SSH worker

Your work lives on a Jetson, a lab server, and a laptop. Sessions drop,
terminals close mid-debug, machines reboot at the worst moment — and shell
history on machine A tells you nothing about what happened on machine B.

**Before MemoryWhale:** an SSH dropout erases the session you were halfway
through explaining to a teammate; each machine keeps a private, incomplete
history.

**With MemoryWhale:**

```bash
mw --live --notes "project:auv jetson bring-up"   # autosaves every few seconds
# … SSH dies mid-session …

# back on the Jetson later: the transcript survived; recovery imports it
mw list                                           # session shows up as interrupted/recovered

# share or move memory between machines:
mw push robot@jetson.local                        # scp + remote mw import
mw pull robot@jetson.local                        # or copy another machine's memory here
```

Because capture is crash-resistant and memory moves as an explicit bundle,
no single dropped connection or dead machine takes the debugging context with
it.

**Components involved:** `--live` autosave, startup recovery (`mw-recover`),
`mw push` / `mw pull` / `mw export` / `mw import`, `project:` tags.

## 3. The coding-agent user

You work with Claude Code, Codex CLI, or a similar agent. Without shared
memory, every session starts with re-explaining your environment, and the
agent's "past experience" is hallucination-prone guesswork.

**Before MemoryWhale:** the agent proposes a plausible-but-wrong fix for an
error your machine has seen twice already, because it cannot ask "have we hit
this before?"

**With MemoryWhale:**

```bash
claude mcp add memorywhale -- mw-mcp          # one-time setup
mw doctor                                     # confirms the six tools respond
```

Inside the agent session, the model can now call:

- `recent_errors` / `search_memory` — grounded past failures, with exit codes
  and output, not vibes;
- `get_context` — a ranked digest of relevant memory for the current task;
- `remember` — save the verified root cause once confirmed;
- `similar_failures` / `stats` — pattern and volume views.

Memories written by the agent are attributed to it and start pending, so you
review what it learned in the TUI or dashboard before it enters retrieval.

**Components involved:** `mw-mcp` (stdio MCP server), agent-side MCP config,
the review queue, plus the optional Claude Code hook/skill for capture and
guidance.

## What all three have in common

- **Evidence over recall.** The stored unit is command + output + exit code +
  notes, not a summary someone has to trust.
- **Local-first.** Nothing uploads; sync between machines is an explicit
  `push`/`pull`/`import` decision.
- **Two audiences.** The same store serves humans (`mw search`, `mw tui`,
  `mw-serve`) and agents (`mw-mcp`) without duplication.
- **Lessons attach to evidence.** A saved fix stays linked to the failure it
  resolved, so future-you can audit the reasoning.

## Try it

```bash
mw demo                 # seed a small dataset, no setup required
mw search "error"
mw tui                  # interactive browser
mw pet                  # check how well-fed your store is
```

Then see [Getting started](../guides/getting-started.md),
[Terminal capture](../guides/terminal-capture.md),
[Moving memory between machines](../guides/multi-machine.md), and
[Connecting a coding agent](../guides/agent-memory.md).
