# Knowledge memory: repository map for M1

Companion to [ADR 0001](../adr/0001-knowledge-memory.md). It lists the files
milestone M1 would touch and why. Nothing here is implemented; the ADR is
still proposed.

Baseline on main at `9a67165`: 530 core and CLI tests and 14 interface tests
pass.

## New files

| File | Purpose |
| --- | --- |
| `crates/mw-cli/src/knowledge.rs` | Tables, repository functions, the one access function, audit rows, import and export. Kept out of `lib.rs`, which is already over 5,000 lines. |
| `crates/mw-cli/tests/knowledge.rs` | End-to-end tests through the `mw` binary: add, review, search, import twice, export, delete, project isolation, injection payload. |
| `src/pages/Knowledge.tsx` | The Knowledge page with its review tab. |
| `src/knowledge.test.tsx` | Interface tests on the existing jsdom harness. |
| `docs/reference/knowledge.md` | The import format, the context-pack format, and the stated limit of the access filter. |

## Files that change

| File | Change | Risk to existing behavior |
| --- | --- | --- |
| `crates/mw-cli/src/lib.rs` | `LATEST_SCHEMA_VERSION` 17 to 18; one `if version < 18` block calling into `knowledge.rs`; `pub mod knowledge`. | Low. The block only creates tables. Two tests assert the latest version and move with it. |
| `crates/mw-cli/src/storage.rs` | Nothing, unless a fresh store should create the tables in `initialize`. Recommended: leave it to `migrate`, which already runs on open. | None. |
| `crates/mw-cli/src/bin/mw.rs` | A `Some("knowledge")` arm in the command match, help lines, and the handlers. | Low. A new namespace; no existing arm changes. |
| `crates/mw-cli/tests/cli_errors.rs` | A usage test for `mw knowledge` with no action. | None. |
| `src/App.tsx` | One entry in `PAGES`. | Low. `src/App.test.tsx` checks navigation and gains one case. |
| `src/api.ts`, `src/types.ts`, `src/mock.ts` | Calls, types, and mock data for the page. | None. |
| `src-tauri/src/store.rs`, `src-tauri/src/lib.rs` | Thin commands over `knowledge.rs`, registered with the app. | None. |
| `docs/reference/cli.md` | The `mw knowledge` commands. `scripts/check-doc-references.sh` checks binaries, not subcommands. | None. |
| `docs/reference/privacy.md` | A row in the write-path table. | None. |
| `docs/reference/storage.md` | The three tables and the one-time backup file. | None. |
| `docs/architecture.md`, `README.md`, `CONTRIBUTING.md` | The scope change, in the release that ships M1. The README change also restamps nine translations. | Visible wording change only. |
| `docs/README.md` | Links to the ADR and the new reference page. | None. |

## Files that must not change in M1

| File | Why |
| --- | --- |
| `crates/memorywhale-core/**` | The core crate stays at 0.13.0. Knowledge search starts as plain SQLite in the CLI crate. |
| `crates/mw-cli/src/mcp/tools.rs` | The six tools keep their semantics. Knowledge tools are M3, and adding one also means updating the tool table in `docs/reference/mcp.md` and `integrations/README.md`, which `scripts/check-doc-references.sh` counts. |
| `crates/mw-cli/src/bin/mw-serve.rs`, `docs/reference/api.openapi.json` | No HTTP surface for knowledge in M1. |
| `lesson_before`, `matching_lesson`, `recall_before`, `hook_feedback`, `draft_recurring_lessons` in `crates/mw-cli/src/lib.rs` | They read `bookmarks` only. Knowledge text must never reach a command stop or a failure note. |
| `mw export`, `mw push`, `mw import`, `mw memory publish` | They must not read the knowledge tables. |
| `integrations/*` | No client guide changes until a capability is tested. |

## Existing code M1 reuses

| Need | Existing piece |
| --- | --- |
| Masking secrets on write | `sanitize_capture` and `sanitize_arguments`, re-exported from `memorywhale-core::privacy`. |
| Showing who wrote a memory | `provenance_label` in `crates/mw-cli/src/lib.rs`. |
| Review wording and flow | `mw memory pending`, `approve`, `reject` in `crates/mw-cli/src/bin/mw.rs`. |
| Conflict heuristic (M2) | `memorywhale_core::contradiction`. |
| Adding a column or table safely | `add_column_if_missing` and `table_exists` in `crates/mw-cli/src/lib.rs`. |
| Capture gate | `capture_rule_for`; with capture off for a directory, nothing is stored. |
| Interface tests | `src/test/setup.ts`, `src/test/tauri.ts`, `src/test/helpers.ts`. |

## Checks every M1 pull request runs

```bash
cargo fmt --all -- --check
```

```bash
cargo clippy -p memorywhale-core -p memorywhale-cli --all-targets -- -D warnings
```

```bash
cargo test -p memorywhale-core -p memorywhale-cli
```

```bash
npm test
```

```bash
bash scripts/check-doc-references.sh
```
