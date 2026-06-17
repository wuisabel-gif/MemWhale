# MemoryWhale

MemoryWhale is a Rust/Tauri desktop knowledge graph app that turns notes,
transcripts, and imported text into a calm, zoomable knowledge galaxy.

It combines the local-first memory philosophy of MemPalace with the Rust
desktop/runtime direction of CodeWhale:

- Rust backend commands
- Tauri desktop shell
- SQLite local database
- React + TypeScript frontend
- Interactive concept graph
- Plain-text and Markdown import workflow

## Project Origins

MemoryWhale started by combining two local projects in this workspace:

- **MemPalace** provides the local-first memory direction: private storage,
  searchable knowledge, durable context, and the idea that important work
  should stay available between sessions.
- **CodeWhale** provides the Rust-first desktop/runtime direction: command
  tooling, terminal awareness, and a stronger foundation for developer-facing
  workflows.

The result is a local desktop memory tool focused on notes, transcripts,
command history, and terminal error memory in one searchable graph.

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

Terminal work is full of useful context, but most of it disappears. A command
fails, the error scrolls away, the exact flags are forgotten, and the next
session starts without the history that would have made debugging faster.

MemoryWhale is built to remember what I put into it:

- Commands and arguments I tried.
- Error logs that explain what went wrong.
- Notes about why a fix worked or failed.
- Project context that should survive between sessions.
- Related ideas that are easier to see as a graph than as terminal scrollback.

The goal is simple: make the terminal feel like it has long-term memory, so I
can search old attempts, recover exact errors, and build on previous work
instead of rediscovering it.

## How I Use It

I import project notes, paste important terminal output, and save command runs
through the Terminal Memory panel or the `mw-remember` helper. MemoryWhale
stores everything locally in SQLite, so the memory stays on my machine and can
be backed up like any other project data.
