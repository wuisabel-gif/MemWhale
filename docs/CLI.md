# CLI reference

All commands ship as prebuilt binaries (see the README's Install section). If
you're working from a source checkout instead, prefix any command with
`cargo run -p memorywhale-cli --bin <name> --` from the repo root, e.g.
`cargo run -p memorywhale-cli --bin mw -- --notes "…"`.

## mw — record sessions

```bash
mw --notes "Jetson build debugging"   # record a whole shell session until exit
mw --live --notes "project:demo"      # autosave to SQLite every few seconds
mw list                               # list recorded sessions
mw show 1                             # print the full transcript of a session
mw search "linker error"              # search commands, output, notes, transcripts
mw tui                                # interactive terminal browser (type to search, Enter to act, F1 help, Esc quit)
mw git-fix                            # diagnose the last failed git command: what, why, the fix
mw mark "before the risky flash"      # bookmark the current debugging moment
mw remember "the fix was passing --features vendored-ssl"  # save a lesson/conclusion
mw replay 12                          # rerun a saved command run
mw demo                               # seed a small demo dataset to explore
mw rm 5                               # delete a session (+ its transcript); mw rm command <id> for a run
mw prune [--min-bytes N] [--dry-run]  # delete empty auto-recorded sessions (noise cleanup)
mw share 5 [-o file.html]             # write a self-contained HTML page of one item to send someone
mw discard                            # inside a recording: throw the current session away
mw context [project:name] [--last-error] [--limit N]   # compact failures digest for agents
mw agent [session-id]                 # export a full session as text to paste into an agent
mw ask [question] [--chat gemini]     # package the last failure for your chat AI → clipboard
mw doctor                             # check the install
mw export [project:name]              # export a bundle (Markdown + JSON + SQLite)
mw import <bundle|sqlite>             # merge another machine's export
mw push <ssh-host>                    # send this machine's memory to another (scp + remote import)
mw pull <ssh-host> [path]             # the reverse: copy another machine's memory here and merge it
mw global on|off|status               # auto-record every new terminal
```

`mw` starts a recorded subshell; run commands normally, then `exit` or Ctrl-D
to stop. The raw transcript lands in the data folder, searchable metadata in
SQLite. `--live` matters for SSH sessions and sudden shutdown risk: if the
terminal dies before `exit`, the last autosaved transcript is still there.
Recorded a garbage terminal? Type `mw discard` inside it before exiting, or
`mw rm <id>` after the fact.

`mw context` gives an agent a short digest of recent *failures*; `mw agent`
dumps a whole *session transcript* as Markdown to hand over later — e.g.
`mw agent > session.md` or `mw agent | pbcopy`. `mw remember`/`mw mark` share
one store (a "note") — once you've figured out *why* something failed or *how*
a fix worked, `mw remember` saves that conclusion so `mw search`/`mw context`
surface it later instead of re-deriving it. For live agent access, register the
MCP server instead (see `mw-mcp` below — it also exposes a `remember` tool, so
the agent can write lessons itself).

`mw ask` is the bring-your-own-AI bridge: it packages the most recent failure
(exact error, cwd, exit code) plus similar past failures and saved lessons into
one debugging prompt, copies it to the clipboard, and opens chatgpt.com — you
paste (Cmd-V) and the chat has full context. No API key, and no per-token
billing: it rides the flat-rate chat subscription you already pay for
(ChatGPT/Claude/Gemini — effectively unlimited for daily debugging, where
API-key tools meter every call). The payload is plain Markdown. Pick the chat
with `--chat chatgpt|claude|gemini` (or any URL), or set a permanent default
with `MEMORYWHALE_CHAT=gemini` in your shell rc. Add a question
(`mw ask "why does this keep breaking"`), include the tail of the current
session with `--session`, or skip the browser with `--no-open`. Everything in
the payload was secret-redacted at capture time.

`mw git-fix` recognizes a handful of common git failure shapes — push rejected
(non-fast-forward), merge conflicts, a dirty working tree blocking an
operation, diverged branches, unrelated histories, and SSH auth failures — from
the stderr already captured in `command_runs`. It explains what happened,
prints the fix, and checks whether this exact class of failure has come up
before (a past command run, or a lesson you already `mw remember`'d). With no
argument it uses the most recent failed `git` command; `mw git-fix <id>`
targets a specific `command_run` id (from `mw list`/`mw search`). Patterns it
doesn't recognize get an honest "didn't recognize this" instead of a wrong
guess.

## mw-run — capture one command

```bash
mw-run --notes "Check the Rust backend" -- cargo check
```

Output still streams to your terminal while a copy (stdout, stderr, exit code,
cwd, argv) is saved. `mw-run` exits with the same exit code as the command.

## mw-remember — save output you already have

```bash
mw-remember \
  --cwd . \
  --exit-code 127 \
  --stderr "zsh:1: command not found: cargo" \
  --notes "Rust verification failed because cargo was missing" \
  -- cargo check --manifest-path src-tauri/Cargo.toml
```

## mw-screenshot — opt-in visual evidence

```bash
mw-screenshot --notes "VS Code showed the TypeScript warning"
```

Local-only and opt-in. On headless machines (e.g. a Jetson without a display)
screenshot capture may fail; terminal memory recording still works.

## mw-serve / mw-view / mw-recover / mw-mcp

```bash
mw-serve [--host addr] [--port n] [--token secret]  # web dashboard
mw-view <id>                                        # open one memory directly
mw-recover                                          # recover interrupted recordings
mw-mcp                                              # MCP server for AI agents (stdio)
```

## Data location

By default the SQLite database lives in the local app data directory. Set
`MEMORYWHALE_DATA_DIR` for an explicit location:

```bash
MEMORYWHALE_DATA_DIR=/tmp/memorywhale-data mw-run -- echo "saved here"
```

## Secret redaction

Captured stdout/stderr/notes/transcripts are scrubbed for common secret shapes
(API keys, tokens, `password=`, PEM blocks) before they reach SQLite. Set
`MEMORYWHALE_NO_REDACT=1` to store raw text.
