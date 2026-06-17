# MemoryWhale

MemoryWhale is a Rust/Tauri desktop memory system that turns terminal commands,
error logs, notes, transcripts, and imported text into a calm, searchable
knowledge graph.

It is built around local technical memory:

- Rust backend commands
- Tauri desktop shell
- SQLite local database
- React + TypeScript frontend
- Interactive concept graph
- Plain-text and Markdown import workflow

## What It Can Do

- Import `.txt` and `.md` files from disk.
- Paste text, transcript, or web article notes directly into the app.
- Store documents, concepts, quotes, tags, and links in SQLite.
- Auto-extract keywords with a deterministic local algorithm.
- Build graph edges between documents and concepts.
- Remember terminal commands, split command-line arguments, exit codes,
  stdout/stderr, and notes in SQLite.
- Search documents and concepts by keyword/source/tag-style text.
- Click graph nodes to inspect connected notes and summaries.

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
output. The Rust backend also ships a small helper binary:

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
local computer for USC AUV work. The codebase was shared, but the terminal
history was not. Commands, errors, build logs, and debugging attempts lived on
whichever machine happened to run them.

That became a real problem for AI-assisted debugging. If the terminal shut down,
the machine changed, or the scrollback disappeared, the agent lost the exact
context it needed: what command was run, what flags were used, what error came
back, and what had already been tried.

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

## How I Use It

I import project notes, paste important terminal output, and save command runs
through the Terminal Memory panel or the `mw-remember` helper. MemoryWhale
stores everything locally in SQLite, so the memory stays on my machine and can
be backed up like any other project data.

## Project Governance

MemoryWhale is guided by a small set of project documents:

- [Contributing](CONTRIBUTING.md) explains how to make useful changes.
- [Code of Conduct](CODE_OF_CONDUCT.md) defines the standard for collaboration.
- [AI Constitution](CONSTITUTION.md) defines how AI agents should reason and act
  when working on this project.
