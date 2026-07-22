<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="MemoryWhale logo" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="release"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <a href="https://deepwiki.com/wuisabel-gif/MemWhale"><img src="https://deepwiki.com/badge.svg" alt="Ask DeepWiki"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="license MIT"/>
  <img src="https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white" alt="Rust"/>
  <img src="https://img.shields.io/badge/SQLite-003B57?logo=sqlite&logoColor=white" alt="SQLite"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="local-first, nothing uploaded"/>
  <img src="https://img.shields.io/badge/Runs%20on-Jetson-76B900?logo=nvidia&logoColor=white" alt="Runs on Jetson"/>
</p>

<p align="center">
  🐬 <b>Sibling project:</b> <a href="https://github.com/wuisabel-gif/Delphin">Delphin</a> — the duplex communication layer (talk to your agent while it thinks). MemoryWhale is the memory layer — see <a href="ECOSYSTEM.md">ECOSYSTEM.md</a>.
</p>

Shell history remembers commands loosely. It does not preserve the debugging
situation: the machine, the working directory, the exact flags, the error
output, and the note about what the attempt meant. When the terminal crashes,
the SSH session drops, or the scrollback scrolls away, that context is gone —
and you (or your AI agent) re-debug what was already solved.

**MemoryWhale records commands, arguments, output, errors, and whole sessions
into local SQLite, so what already failed stays searchable** — across crashes,
SSH drops, and machine switches. Everything stays on your machine; nothing is
uploaded.

<p align="center">
  <img src="assets/demo.gif" alt="Capturing a failed build with mw-run, then recalling its exact error later with mw context" width="820" />
</p>

## Install

Prebuilt binaries for Linux x86_64/aarch64 (incl. Jetson) and macOS — no Rust
toolchain needed:

```bash
curl -fsSL https://raw.githubusercontent.com/wuisabel-gif/MemWhale/main/install.sh | sh
```

Installs into `~/.local/bin` (override with `PREFIX=/usr/local`). Other routes:

```bash
# cargo (builds the CLI only — no Tauri/GTK)
cargo install memorywhale-cli

# Homebrew (macOS/Linux)
brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

Debian/Ubuntu/Jetson: grab the `.deb` from the
[releases page](https://github.com/wuisabel-gif/MemWhale/releases) and
`sudo apt install ./memorywhale_*.deb`.

All three install paths stay in sync via the release workflow — one tag
produces the prebuilt binaries, bumps the Homebrew formula, and publishes to
crates.io, so `install.sh`, `brew install`, and `cargo install memorywhale-cli`
all land on the same version.

**Windows:** run it inside [WSL](https://learn.microsoft.com/windows/wsl/) —
MemoryWhale is a Linux binary there, so the one-line install above works as-is
from your WSL shell. (A native Windows build isn't available yet; the session
recorder relies on Unix `script`.)

First run: type `mw` — it explains itself and offers to auto-record every new
terminal. That's the whole setup.

## Sixty-second tour

```bash
mw --notes "Jetson build debugging"   # record a whole shell session (exit to stop)
mw --live                             # same, but autosaved — survives SSH drops
mw-run -- cargo check                 # run one command, capture its output + exit code
mw list && mw show 1                  # inspect recorded sessions
mw search "linker error"              # find it across commands, output, and transcripts
mw tui                                # interactive terminal browser — type to search, Enter to act
mw remember "the fix was X"           # save a conclusion so it doesn't get re-derived
mw git-fix                            # diagnose the last failed git command: what, why, the fix
mw context --last-error               # the most recent failure, with its exact error
mw-serve                              # web dashboard (works headless, e.g. on a Jetson)
```

`mw tui` opens an interactive terminal browser — no web page, no server. Type
to search (ranked live), arrow keys to move, **Enter** to reveal the
`mw replay`/`mw show` command for the selected memory, **F1** for help, **Esc**
to quit. The detail pane shows the full memory plus *why* it ranked where it
did.

Every command run stores the command, each argument as a searchable row, the
cwd, exit code, stdout, stderr, and your notes. Captured output is scrubbed for
common secret shapes (tokens, keys, `password=`) before it reaches SQLite.

`mw git-fix` recognizes common git failures — push rejected, merge conflicts,
a dirty working tree, diverged branches, SSH auth — from output MemoryWhale
already captured, and tells you whether you've hit this exact one before (or
already solved it via `mw remember`). There's a **Neovim plugin** too —
`:MwAsk`, `:MwGitFix`, `:MwSearch`, and `:MwRemember` (which can save a visual
selection as a lesson) — in [integrations/neovim/](integrations/neovim/README.md).

📖 **Full command reference:** [docs/CLI.md](docs/CLI.md) — every `mw` subcommand
and all the helper binaries (`mw-run`, `mw-serve`, `mw-mcp`, …), with flags and
examples.

## AI agents

Coding agents forget everything between sessions. `mw-mcp` is a Model Context
Protocol server over your local memory — register it once and Claude Code /
Codex / Cursor can query past failures directly instead of re-deriving them,
and can write down what they figure out:

```bash
claude mcp add memorywhale -- mw-mcp
```

That gives the agent five tools: `recent_errors`, `search_memory`,
`get_context`, `remember`, and `similar_failures` — the last one lets it paste
an error it just hit and learn, from observed exit codes, how many times that
exact failure was seen and how often a later run resolved it. And once it works
out *why* something failed, it can save that conclusion for its future self (or
a teammate) to find later, the same way you'd type:

```bash
mw remember "the E0308 in camera-driver was the fps field being a string; fix: parse it as i32"
```

Or bring your own AI — no API key, no per-token billing. Every "AI in your
terminal" tool wants an API key that meters you by the token; `mw ask` instead
uses the flat-rate chat subscription you already pay for (ChatGPT Plus, Claude
Pro — effectively unlimited for daily debugging). It packages the last failure
(exact error + similar past failures + saved lessons) into one prompt on your
clipboard and opens chatgpt.com; you press paste:

```bash
mw ask                          # or: mw ask "why does this keep breaking"
mw context --last-error         # smaller digest, same idea, no browser
```

See [integrations/](integrations/README.md) for the MCP tools, a Claude Code
hook that auto-records every command the agent runs, and a skill that knows
when to read from (and write to) the memory.

## Web dashboard (solo or team)

`mw-serve` serves your memory as a local web page — useful on headless machines
where the desktop app cannot open, and for teams: run it on the machine that has
the data, and everyone on the network sees the same errors instead of asking for
copy-pastes.

```bash
mw-serve                       # binds 0.0.0.0:7071
#   this machine:  http://localhost:7071/
#   over the LAN:   http://<machine-ip>:7071/   (find the IP with: hostname -I)
```

On a shared or untrusted network, require a token — the first `?token=` visit
sets a cookie so links keep working:

```bash
MEMORYWHALE_TOKEN=some-shared-secret mw-serve
#   open once:  http://<machine-ip>:7071/?token=some-shared-secret
```

The dashboard lists command runs and sessions, opens a readable detail page for
each (with suggested next steps mined from your history), auto-recovers
interrupted recordings on startup, and includes a `/graph` view: each command is
a node **sized by how often you ran it**, and arguments shared by two or more
commands become **bridges** (orange) that reveal which tools share a workflow.

<p align="center">
  <img src="assets/knowledge-graph.svg" alt="MemoryWhale knowledge graph" width="560" />
</p>

## Projects across terminals

Each `mw` records one terminal. Tag related sessions with the same
`project:<name>` in the notes and the dashboard merges them into one
time-ordered timeline:

```bash
mw --notes "project:pop_playlist git history"     # terminal 1
mw --notes "project:pop_playlist server testing"  # terminal 2
```

To auto-record every new terminal without typing `mw`, use `mw global on`
(`mw global off` to stop). Full operating procedure: [SOP.md](SOP.md).

## Two capture tiers: shell hooks vs `mw --live`

MemoryWhale captures at two levels. They layer — you can run both.

```bash
mw hooks install       # lightweight, always on, every shell
mw hooks install pwsh  # PowerShell (not in $SHELL, so name it explicitly)
mw hooks uninstall     # removes exactly the managed block it added
```

| | Shell hooks (`mw hooks install`) | Full capture (`mw --live`, `mw-run`) |
| --- | --- | --- |
| Always on | Yes, every interactive shell | Per terminal / per command |
| Records | Command line, working directory, exit code, duration | All of that **plus the full output transcript** |
| Does **not** record | stdout, stderr, screen content, TUI redraws | — |
| Cost | One detached write per command, no `script` pty | A pty wrapper for the session |
| Row marker | `command_runs.capture_kind = 'hook'` | `capture_kind = 'full'` |

Everything lands in the same SQLite database and the same `command_runs`
table, so `mw search`, `mw context` and the dashboard see hook rows alongside
full-capture rows. `capture_kind` is how you tell them apart.

How they layer:

- **Hooks are the index.** Always-on, cheap, so you can always answer *what did
  I run, where, and did it work?* — even months later.
- **`mw --live` is the recording.** Start it when you're about to do something
  you'll want to re-read: a gnarly debug session, a deploy, a pairing session.
- **No double capture.** While a full `mw` session is running it exports
  `MW_RECORDING=1`; the hook sees that and skips logging entirely, so a command
  is never recorded twice.
- **Capture gates apply to both.** A directory gated `off` (via `.mwignore` or
  `[capture.paths]`) produces zero hook rows — the gate is checked before the
  database is even opened.

Supported shells: zsh (`preexec`/`precmd`), bash (`DEBUG` trap +
`PROMPT_COMMAND`), fish (`fish_postexec`), and PowerShell (a wrapped `prompt`
function; install with `mw hooks install pwsh`).
The hook fails silently by design: a locked or missing database, a missing
binary, a bad path — none of it blocks or breaks your prompt, and your `$?` is
preserved. Turn it off for one shell with `export MW_HOOK_OFF=1`.

## Capture gates: never record certain directories

Some directories should never end up in your memory. Capture gates decide, per
directory, how much may be stored — and they are enforced *before* anything is
written to SQLite, not cleaned up afterwards.

| Mode | What is stored |
| --- | --- |
| `full` | Everything (the default, unchanged). |
| `commands-only` | What ran, where, exit code, and timing — no stdout/stderr or transcript. |
| `off` | Nothing at all. |

Drop a `.mwignore` file at the root of a directory tree:

```toml
# ~/finances/.mwignore
capture = "off"
```

…or gate paths globally in `<data dir>/config.toml`:

```toml
machine = "laptop"

[capture.paths]
"~/finances" = "off"
"~/work/client-repo" = "commands-only"
```

Precedence: the nearest `.mwignore` walking **up** from your working directory,
then the longest matching prefix under `[capture.paths]`, then `full`. Paths are
resolved through symlinks, so a shortcut into a gated tree stays gated.

Check what applies where you are:

```bash
mw status     # capture mode for this directory, and which rule produced it
```

Already-recorded memory can be aged out with `mw prune --older-than 90d`
(add `--dry-run` to see what would go first).

## Sharing memory across machines

Memory is local per machine, but you can move it:

```bash
mw export project:demo   # bundle: Markdown + JSON + a SQLite copy
mw import path/to/bundle # merge into this machine (skips duplicates)
mw push jetson           # snapshot → scp → remote mw import, any ssh host
mw pull jetson           # the reverse: copy a machine's memory here and merge it
```

Nothing goes through a third party; it's your own `scp`/`ssh`.

## Why I built it

I was running the same robotics repo on two machines — a Jetson and my laptop —
for USC Autonomous Underwater Vehicle work. The codebase was shared, but the
terminal history was not. Commands, errors, build logs, and debugging attempts
lived on whichever machine happened to run them.

That became a real problem for AI-assisted debugging and team collaboration. If
a teammate asked why something failed, I could not always retrieve the terminal
scrollback that explained it. If the terminal shut down or the machine changed,
the exact context disappeared: what command ran, what flags, what error came
back, what had already been tried.

The goal is simple: make the terminal feel like it has long-term memory, so I
can search old attempts, recover exact errors after shutdowns, and give an AI
agent enough history to continue debugging instead of starting over.

## Developing

The repo is a Cargo workspace: `crates/mw-cli` (the CLI, no GUI dependencies),
`crates/memorywhale-core` (retrieval), and `src-tauri` (the desktop app).

```bash
cargo build -p memorywhale-cli            # CLI only — fast, no GTK/WebKit needed
npm install && npm run tauri:dev # full desktop app (needs Tauri system deps)
```

- Linux/Jetson system deps, shell hook, systemd dashboard service, completions,
  man pages: [linux/README.md](linux/README.md) (`linux/install.sh --all`)
- Real problems hit while setting up on a Jetson, and the fixes:
  [DEBUG.md](DEBUG.md)
- Reproducible offline retrieval benchmark (no API keys), 37-item
  terminal-session corpus, two blind-labeled gold sets: the built-in scorer uses
  **SQLite FTS5 BM25** for keyword relevance and blends in recency/importance/
  reinforcement/task. On the 18-query **intent** set (where context, not wording,
  decides the answer) it leads every baseline — **recall@1 = 0.67, recall@5 =
  0.92** vs FTS5's 0.42 / 0.69; on the 30-query pure term-overlap set the lexical
  baselines still win, as designed. See
  [benchmarks/BENCHMARKS.md](benchmarks/BENCHMARKS.md) for both tables and the
  one-line reproduce command.

## Documentation

- [docs/CLI.md](docs/CLI.md) — full command reference for every `mw*` binary.
- [VISION.md](VISION.md) — where the project is headed and why.
- [PHILOSOPHY.md](PHILOSOPHY.md) — the communication and memory ideas behind it.
- [ECOSYSTEM.md](ECOSYSTEM.md) — how the pieces (CLI, dashboard, integrations) fit together.
- [CONSTITUTION.md](CONSTITUTION.md) — governance for users, contributors, maintainers, and AI agents.
- [SOP.md](SOP.md) — standard operating procedure for building and releasing.
- [HANDOFF.md](HANDOFF.md) — context handoff notes for picking up the work.
- [benchmarks/BENCHMARKS.md](benchmarks/BENCHMARKS.md) — the offline retrieval benchmark and how to reproduce it.
- [integrations/README.md](integrations/README.md) — editor/tool integrations.
- [linux/README.md](linux/README.md) — Linux/Jetson system deps, shell hook, systemd service, completions, man pages.

## Attribution and learning sources

MemoryWhale was built after studying two original projects:

- **CodeWhale** by **Hmbown**: <https://github.com/Hmbown/CodeWhale>
- **MemPalace** by the **MemPalace project**: <https://github.com/MemPalace/mempalace>

CodeWhale taught me how a Rust-first developer tool can organize terminal
workflows, command execution, safety boundaries, and agent-facing runtime
ideas. MemPalace taught me local-first memory: keeping technical context on
your own machine, where you can still search it months later. MemoryWhale is my
own project built from those lessons; it does not vendor those repositories.

## Project governance

- [Philosophy](PHILOSOPHY.md) — the communication and memory ideas behind the project.
- [Contributing](CONTRIBUTING.md) — how to make useful changes.
- [Code of Conduct](CODE_OF_CONDUCT.md) — the standard for collaboration.
- [Constitution](CONSTITUTION.md) — governs users, contributors, maintainers,
  and AI agents alike, under the same principles, duties, and protections.
