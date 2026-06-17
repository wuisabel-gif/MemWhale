# Contributing to MemoryWhale

MemoryWhale is a local-first terminal memory system. Contributions should make
technical memory more durable, searchable, and useful across sessions and
machines.

## What This Project Values

- Preserve exact command and error history.
- Keep data local by default.
- Make debugging context easier to recover.
- Prefer structured storage over loose text dumps.
- Keep the app understandable for both humans and AI agents.

## Good Contributions

Useful changes include:

- Better command capture.
- Better stdout/stderr storage and display.
- Search improvements for arguments, paths, and errors.
- Graph improvements that reveal relationships between commands and concepts.
- Import/export tools for moving memory between machines.
- Documentation that explains real workflows clearly.
- Tests or checks that protect persistence and search behavior.

## Development Setup

Install frontend dependencies:

```bash
npm install
```

Run the desktop app:

```bash
npm run tauri:dev
```

Run frontend checks:

```bash
npm run build
```

Run Rust checks:

```bash
cd src-tauri
cargo fmt
cargo check
```

Try the terminal memory helper:

```bash
cd src-tauri
cargo run --bin mw-remember -- --help
```

## Contribution Rules

1. Keep original evidence.
   If a feature stores terminal history, preserve the raw command, arguments,
   stdout, stderr, exit code, cwd, and timestamp whenever possible.

2. Do not make cloud sync implicit.
   Sync or remote storage must be explicit and documented.

3. Keep the database understandable.
   Prefer clear SQLite tables and migrations over clever opaque blobs.

4. Do not break local use.
   MemoryWhale should remain useful without accounts, servers, or network
   access.

5. Explain why the change matters.
   A good change should connect back to the core problem: terminal history and
   debugging context disappear too easily.

## Pull Request Checklist

- The change preserves or improves local-first behavior.
- Command/log memory remains searchable.
- `npm run build` passes when frontend code changes.
- `cargo fmt` and `cargo check` pass when Rust code changes.
- Documentation is updated for user-facing behavior.

