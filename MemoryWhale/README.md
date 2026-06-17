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

The result is a client-ready desktop knowledge tool focused on notes,
transcripts, command history, and terminal error memory in one local graph.

## What Clients Can Do

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

## Why Teams Use It

MemoryWhale gives teams a private, local workspace for technical memory. It
keeps project notes, debugging history, command attempts, and terminal errors
in one searchable place, so important context does not disappear between
sessions.

It is designed for client work where privacy and continuity matter:

- Local-first storage: project knowledge stays on the machine.
- Faster handoffs: command history and error logs remain attached to the work.
- Better debugging continuity: failed attempts become searchable references.
- Visual exploration: documents, concepts, and commands can be inspected as a graph.
- Simple onboarding: import notes, paste terminal output, and search what happened.

## Client Setup Notes

For a client install, configure a workspace folder, import the project notes,
then start saving important terminal runs through the Terminal Memory panel or
the `mw-remember` helper. The database is local SQLite, so backups can be
handled with normal file backup tools.
