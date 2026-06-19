# MemoryWhale

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
- Record important command attempts manually with `mw-remember`.
- Run commands through the `mw` wrapper to capture output automatically.
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
output. The Rust backend also ships helper binaries.

Use `mw` when you want MemoryWhale to run a command and automatically capture
its stdout, stderr, exit code, cwd, and arguments:

```bash
cd src-tauri
cargo run --bin mw -- --notes "Check the Rust backend" -- cargo check
```

The command output still appears in the terminal while MemoryWhale saves a copy
to SQLite. The `mw` process exits with the same exit code as the command it ran.

By default, MemoryWhale stores its SQLite database in the local app data
directory. Set `MEMORYWHALE_DATA_DIR` when you want an explicit location:

```bash
MEMORYWHALE_DATA_DIR=/tmp/memorywhale-data cargo run --bin mw -- -- echo "saved here"
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

Those command memories appear as graph nodes and connect to extracted concepts
from the command, arguments, and error text.

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
technical context private, searchable, durable, and useful across sessions.

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
- [AI Constitution](CONSTITUTION.md) defines how AI agents should reason and act
  when working on this project.
