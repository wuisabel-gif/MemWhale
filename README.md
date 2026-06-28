<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="MemoryWhale logo" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="release"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="license MIT"/>
  <img src="https://img.shields.io/badge/Rust-000000?logo=rust&logoColor=white" alt="Rust"/>
  <img src="https://img.shields.io/badge/Tauri-24C8DB?logo=tauri&logoColor=white" alt="Tauri"/>
  <img src="https://img.shields.io/badge/SQLite-003B57?logo=sqlite&logoColor=white" alt="SQLite"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="local-first, nothing uploaded"/>
</p>

<p align="center">
  🐬 <b>Sibling project:</b> <a href="https://github.com/wuisabel-gif/Delphin">Delphin</a> — the duplex communication layer (talk to your agent while it thinks). MemoryWhale is the memory layer; they refer to each other — see <a href="ECOSYSTEM.md">ECOSYSTEM.md</a>.
</p>

<p align="center"><b>Use it as</b></p>

<p align="center">
  <img src="https://img.shields.io/badge/CLI-mw%20%C2%B7%20mw--remember-2b43dd" alt="CLI"/>
  <img src="https://img.shields.io/badge/Web%20dashboard-mw--serve-10b6c6" alt="Web dashboard"/>
  <img src="https://img.shields.io/badge/Desktop%20app-Tauri-24C8DB?logo=tauri&logoColor=white" alt="Desktop app"/>
  <img src="https://img.shields.io/badge/Agent%20skill-Claude%20%2B%20Codex-e9663a" alt="Agent skill"/>
  <img src="https://img.shields.io/badge/Runs%20on-Jetson-76B900?logo=nvidia&logoColor=white" alt="Runs on Jetson"/>
</p>

MemoryWhale is a Rust/Tauri local-first terminal memory system that saves
commands, arguments, working directories, exit codes, stdout, stderr, and notes
into a searchable SQLite database.

It is built around local technical memory:

- Rust terminal-memory commands
- Tauri desktop shell
- SQLite local database
- React + TypeScript frontend
- Command and error-log recovery
- Local-first debugging history

## What It Can Do

- Save terminal commands with their full argument list.
- Store the working directory where each command ran.
- Preserve exit codes, stdout, stderr, timestamps, and debugging notes.
- Split command-line arguments into searchable SQLite rows.
- Record a whole interactive terminal session with `mw`.
- List and replay saved shell sessions from SQLite.
- Record important command attempts manually with `mw-remember`.
- Run single commands through `mw-run` to capture output automatically.
- Capture opt-in screenshots with `mw-screenshot` when a visual error matters.
- Query saved terminal memory from SQLite after shutdowns, SSH disconnects, or
  machine switches.
- Keep terminal memory local by default so project history stays on the
  machine unless the user chooses otherwise.

## Why You Should Use It

Use MemoryWhale when terminal history is too fragile for the work you are
doing. Normal shell history remembers commands loosely, but it does not preserve
the full debugging situation: the machine, the working directory, the exact
arguments, the output, the error log, and the note about what the attempt meant.

That missing context matters most in serious engineering work:

- Robotics and embedded development where the same repo runs on a Jetson,
  laptop, simulator, and deployment machine.
- AI-assisted debugging where the agent needs the actual history of failed
  attempts, not a vague memory that "something broke."
- Long-running projects where build errors, environment problems, and one-off
  fixes return weeks later.
- Work that can be interrupted by terminal shutdowns, restarts, SSH drops,
  hardware changes, or lost scrollback.
- Local-first workflows where project memory should stay on your machine.

MemoryWhale turns terminal history into a durable technical memory. It is not
just a prettier shell log. It is a place to preserve the reasoning trail of a
project so a human or AI agent can continue from what already happened.

## Run

On Ubuntu or Jetson, install the Node/npm and Tauri system dependencies first.
This fixes errors like `bash: npm: command not found`.

```bash
sudo apt update
sudo apt install -y nodejs npm build-essential pkg-config libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev
```

If Ubuntu installs an old Node version and `npm install` fails, install Node 20:

```bash
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
```

Then run MemoryWhale:

```bash
npm install
npm run tauri:dev
```

For a browser-only frontend pass while iterating:

```bash
npm run dev
```

The browser build uses an in-memory demo store when Tauri commands are not
available. The desktop app uses SQLite via the Rust backend.

## Linux Integration

To run MemoryWhale as a first-class Linux tool — install the `mw*` binaries,
auto-record every command with a shell hook, keep the dashboard alive as a
`systemd --user` service (survives SSH logout), add tab-completions and man
pages, or build a `.deb` — see [linux/README.md](linux/README.md):

```bash
linux/install.sh --all   # binaries + hook + dashboard service + completions + man
```

## Debug Notes

For Jetson/Ubuntu install issues, Tauri desktop errors, browser-mode notes, and
SQLite inspection commands, see [DEBUG.md](DEBUG.md). It records the exact
problems hit while setting up MemoryWhale on a Jetson and the fixes that worked.

## Terminal Memory

MemoryWhale now stores command runs as durable local memory:

- command name
- full argv as JSON
- each argument in a searchable table
- cwd
- exit code
- stdout
- stderr/error log
- notes

The desktop UI has a Terminal Memory panel for pasting a command and its
output. The Rust backend also ships helper binaries for terminal-first use.

Use `mw` when you want to record a whole interactive shell session:

```bash
cd src-tauri
cargo run --bin mw -- --notes "Jetson build debugging"
```

MemoryWhale starts a recorded subshell. Run commands normally inside it, then
type `exit` or press Ctrl-D when you want to stop recording. The raw transcript
is saved under the local MemoryWhale data folder, and searchable session
metadata is saved in SQLite.

Use `mw --live` when you want the session to appear in SQLite while it is still
running. This is useful for SSH sessions, demos, robotics logs, and sudden
shutdown risk: the dashboard gets refreshed every few seconds, so if the
terminal closes before you can type `exit`, the last autosaved transcript is
still available.

```bash
cd src-tauri
cargo run --bin mw -- --live --notes "project:demo live autosave"
```

After recording, inspect sessions from the terminal:

```bash
cd src-tauri
cargo run --bin mw -- list
cargo run --bin mw -- show 1
```

Use `mw-run` when you want MemoryWhale to run one command and automatically
capture its stdout, stderr, exit code, cwd, and arguments:

```bash
cd src-tauri
cargo run --bin mw-run -- --notes "Check the Rust backend" -- cargo check
```

The command output still appears in the terminal while MemoryWhale saves a copy
to SQLite. The `mw-run` process exits with the same exit code as the command it
ran.

By default, MemoryWhale stores its SQLite database in the local app data
directory. Set `MEMORYWHALE_DATA_DIR` when you want an explicit location:

```bash
MEMORYWHALE_DATA_DIR=/tmp/memorywhale-data cargo run --bin mw-run -- -- echo "saved here"
```

Use `mw-remember` when you already have output text and want to save it
manually:

```bash
cd src-tauri
cargo run --bin mw-remember -- \
  --cwd ../.. \
  --exit-code 127 \
  --stderr "zsh:1: command not found: cargo" \
  --notes "Rust verification failed because cargo was missing" \
  -- cargo check --manifest-path MemoryWhale/src-tauri/Cargo.toml
```

Use `mw-screenshot` only when you intentionally want to save the current screen
as part of a debugging trail:

```bash
cd src-tauri
cargo run --bin mw-screenshot -- --notes "VS Code showed the TypeScript warning"
```

Screenshots are local-only and opt-in. On headless machines, such as a Jetson
without an active desktop display, screenshot capture may fail; terminal memory
recording still works.

Command memories appear as graph nodes and connect to extracted concepts from
the command, arguments, and error text.

## Web Dashboard

`mw-serve` serves your memory as a local web page — useful on headless machines
(e.g. a Jetson) where the desktop app cannot open. Run it on the machine that has
the data, then open it from any browser on your network:

```bash
mw-serve                       # binds 0.0.0.0:7071
#   this machine:  http://localhost:7071/
#   over the LAN:   http://<machine-ip>:7071/   (find the IP with: hostname -I)
```

The dashboard lists your command runs and recorded sessions, opens a readable
detail page for each (with suggested next steps mined from your history), and
includes a `/graph` view of commands linked to the arguments they used. It also
auto-recovers any interrupted session transcripts on startup. Everything is
served locally; nothing is uploaded.

Open a single memory directly with `mw-view <id>`, or recover an interrupted
recording with `mw-recover`.

### Knowledge graph

The dashboard's `/graph` view turns your command history into a map. Each command
is one node, **sized by how often you ran it**; arguments are smaller nodes; and
arguments shared by two or more commands become **bridges** (orange) that reveal
which tools share a workflow. Click a command to see all of its runs.

<p align="center">
  <img src="assets/knowledge-graph.svg" alt="MemoryWhale knowledge graph" width="560" />
</p>

## Recording Across Terminals (Projects)

Each `mw` records only its own terminal, so work spread across several terminals
is captured as separate sessions. Tag them with the same `project:<name>` in the
notes and MemoryWhale groups them automatically:

```bash
# terminal 1
mw --notes "project:pop_playlist git history"
# terminal 2
mw --notes "project:pop_playlist server testing"
```

The dashboard shows a **Projects** section; opening a project merges every
command run and session for it into one time-ordered timeline, across all
terminals. To auto-record every new terminal without typing `mw` each time, use
`mw global on` (and `mw global off` to stop).

See [SOP.md](SOP.md) for the full operating procedure.

## Why I Built It

I was running the same robotics repo on two different machines: a Jetson and my
local computer for USC Autonomous Underwater Vehicle work. The codebase was
shared, but the terminal history was not. Commands, errors, build logs, and
debugging attempts lived on whichever machine happened to run them.

That became a real problem for AI-assisted debugging and team collaboration. If
a teammate asked why something failed, I could not always retrieve the last
terminal section that explained it. If the terminal shut down, the machine
changed, or the scrollback disappeared, the exact context disappeared with it:
what command was run, what flags were used, what error came back, and what had
already been tried.

MemoryWhale is built to remember what I put into it:

- Commands and arguments I tried.
- Error logs that explain what went wrong.
- Notes about why a fix worked or failed.
- Project context that should survive between sessions and machines.
- Debugging history from Jetson and local development workflows.
- Related ideas that are easier to see as a graph than as terminal scrollback.

The goal is simple: make the terminal feel like it has long-term memory, so I
can search old attempts, recover exact errors after shutdowns, and give an AI
agent enough history to continue debugging instead of starting over.

## Attribution and Learning Sources

MemoryWhale was built after studying two original projects:

- **CodeWhale** by **Hmbown**: <https://github.com/Hmbown/CodeWhale>
- **MemPalace** by the **MemPalace project**: <https://github.com/MemPalace/mempalace>

CodeWhale helped me understand how a Rust-first developer tool can organize
terminal workflows, command execution, safety boundaries, and agent-facing
runtime ideas. MemPalace helped me understand local-first memory: keeping
technical context on your own machine, where you can still search it months later.

MemoryWhale is my own project built from those lessons. It focuses specifically
on terminal command memory, error logs, notes, and debugging continuity for
work that moves between machines, such as Jetson and local robotics
development. The clean MemoryWhale branch does not vendor those repositories;
it credits them as the projects that taught me the core ideas behind this one.

## How I Use It

I import project notes, paste important terminal output, and save command runs
through the Terminal Memory panel or the `mw-remember` helper. MemoryWhale
stores everything locally in SQLite, so the memory stays on my machine and can
be backed up like any other project data.

## Project Governance

MemoryWhale is guided by a small set of project documents:

- [Philosophy](PHILOSOPHY.md) explains the communication and memory ideas behind
  the project.
- [Contributing](CONTRIBUTING.md) explains how to make useful changes.
- [Code of Conduct](CODE_OF_CONDUCT.md) defines the standard for collaboration.
- [Constitution](CONSTITUTION.md) governs everyone who works on MemoryWhale —
  users, contributors, maintainers, and AI agents alike, under the same
  principles, duties, and protections.
