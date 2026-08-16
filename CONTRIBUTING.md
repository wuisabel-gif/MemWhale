# Contributing to MemoryWhale

MemoryWhale is a local-first terminal memory system. Contributions should make
technical memory more durable, searchable, and useful across sessions and
machines.

## Project Scope

A feature belongs in MemoryWhale if it improves **capturing, preserving,
retrieving, or sharing development experience** while respecting the
local-first model.

Use the four architectural areas in [docs/architecture.md](docs/architecture.md)
when placing a change:

- **Capture:** shell, command, session, and verified agent-hook ingestion.
- **Memory:** durable evidence, lessons, provenance, retention, and SQLite.
- **Retrieval:** search, context, recent errors, and failure similarity.
- **Interfaces:** CLI, MCP, TUI, web, desktop, and thin client integrations.

Client-specific configuration belongs under `integrations/` and should not add
agent-specific behavior to core. Autonomous coding-agent behavior, model
provider routing, and unrelated personal-memory features are outside the
project's responsibility. Proposals that add remote storage or cross-cut the
local privacy model need architectural discussion before implementation.

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
npm test
npm run build
```

Run Rust checks:

```bash
cargo fmt --all -- --check
cargo clippy -p memorywhale-core -p memorywhale-cli --all-targets -- -D warnings
cargo test -p memorywhale-core -p memorywhale-cli
cargo build --workspace
```

Run repository consistency checks:

```bash
bash scripts/check-release-version.sh
bash scripts/check-doc-references.sh
```

The release-version check keeps the CLI, npm, lockfile, and Tauri versions in
sync. The documentation-reference check verifies the MCP tool list and CLI
binary names against the runtime and reference docs.

Try the terminal memory helper:

```bash
cargo run -p memorywhale-cli --bin mw-remember -- --help
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

6. Keep integrations thin.
   Integrations configure an external client at the `mw-mcp` or CLI seam. Use
   [integrations/TEMPLATE.md](integrations/TEMPLATE.md), declare only verified
   capabilities, and do not describe MCP access as automatic execution capture.

## Pull Request Checklist

- The change preserves or improves local-first behavior.
- Command/log memory remains searchable.
- `npm test` passes when frontend code changes.
- `npm run build` passes when frontend code changes.
- `cargo fmt --all -- --check` passes when Rust code changes.
- `cargo clippy -p memorywhale-core -p memorywhale-cli --all-targets -- -D warnings` passes when Rust code changes.
- `cargo test -p memorywhale-core -p memorywhale-cli` passes when Rust code changes.
- `cargo build --workspace` passes for workspace changes.
- Documentation is updated for user-facing behavior.
