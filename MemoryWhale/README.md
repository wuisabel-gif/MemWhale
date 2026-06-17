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
- Plain-text and Markdown import MVP

## MVP Features

- Import `.txt` and `.md` files from disk.
- Paste text, transcript, or web article notes directly into the app.
- Store documents, concepts, quotes, tags, and links in SQLite.
- Auto-extract keywords with a deterministic local algorithm.
- Build graph edges between documents and concepts.
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

## Resume Description

MemoryWhale - Built a Rust/Tauri desktop knowledge graph app that imports
notes and transcripts into a local SQLite database, extracts key concepts,
and visualizes relationships as an interactive zoomable graph.

## Resume Keywords

Rust, Tauri, SQLite, Knowledge Graph, Graph Visualization, Desktop App,
Information Retrieval, NLP, Local-First Software
