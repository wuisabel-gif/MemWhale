# Issue 1 — Wire `mw-memory` into the CLI + MCP, one shared loader

Branch: `issue/1-wire-mw-memory`

## What this does

`mw search` and the MCP server now rank results through the explainable
`mw-memory` engine (five signals + per-signal reasons) instead of raw `LIKE`
scans. The DB->`Memory` loading logic that previously lived only in `src-tauri`
is extracted into `mw-memory` so the CLI and the desktop app share **one**
loader and provably rank identically for a fixed DB + fixed `now`.

## The shared-loader API decision (where it lives)

`crates/mw-memory/src/sqlite.rs` — a new `mw_memory::sqlite` module.

- `pub fn load_memories(conn: &rusqlite::Connection) -> Vec<Memory>` — the one
  loader. Queries `documents`, `command_runs`, `agent_turns`, `bookmarks`,
  `sessions`, **each independently and tolerant of a missing table** (missing
  table = zero rows, not an error). That is what lets a single function serve
  two different schemas: the desktop DB (documents + command_runs +
  agent_turns) and the CLI DB (sessions + command_runs + bookmarks). Each side
  simply gets whatever tables it has.
- Ids are namespaced per source so `explain(id)` stays unique/stable:
  documents `id`, commands `+1e9`, conversation `+2e9`, notes/bookmarks `+3e9`,
  sessions `+4e9`.
- `pub fn decode_id(id) -> (Source, i64)` + `pub enum Source` recover the source
  and original row id for display (`mw replay <id>` / `mw show <id>`, MCP tags).

Why here and not a CLI-only module: the desktop app already had this logic;
putting it in `mw-memory` (which both crates depend on) is the only place that
removes the duplication instead of creating a second copy. `rusqlite` moved
from `mw-memory`'s dev-deps to deps — it was already a transitive dep of both
consumers, so `cargo install memorywhale-cli` gains no new dependency.

## Changed files

- **`crates/mw-memory/src/sqlite.rs`** (new): the shared loader + `decode_id` +
  `Source`, with unit tests (table-tolerance, id round-trip, CLI/desktop
  agreement).
- **`crates/mw-memory/src/lib.rs`**: `pub mod sqlite;`.
- **`crates/mw-memory/Cargo.toml`**: `rusqlite` dev-dep -> dep.
- **`crates/mw-cli/Cargo.toml`**: added `mw-memory` path dependency.
- **`crates/mw-cli/src/bin/mw.rs`**: `mw search` rewritten to load -> engine ->
  ranked output. Terse by default (score, source tag, snippet, action hint);
  `--explain` appends the full per-signal breakdown + reasons per hit. `now` is
  supplied by the caller (`Utc::now()`), not the scorer. Help text updated.
- **`crates/mw-cli/src/bin/mw-mcp.rs`**: `search_memory` and `get_context` now
  return engine-ranked results, each with a `reasons:` line (additive — input
  schemas unchanged; tool descriptions updated to mention ranking + reasons).
- **`src-tauri/src/lib.rs`**: deleted the private `load_memories` (~125 lines);
  both call sites now use `mw_memory::sqlite::load_memories(&conn)`. The
  embedding cache + `build_recall_engine` stay desktop-side (they use the
  desktop-only `memory_embeddings` table). Desktop behavior is unchanged: it
  still loads exactly documents + command_runs + agent_turns (the CLI-only
  tables are absent -> skipped).

## Note on the "provenance/review/migrate" preservation instruction

The task said a provenance string, an `approved=1` review filter, and a
`migrate()` runner at `user_version=1` had "just landed on main" and must be
preserved. **None of these exist anywhere in this repo** (`grep -rn` across all
crates + `src-tauri` for `provenance`, `approved`, `migrate`, `user_version`
returns nothing; no such column in either schema). There was nothing to
preserve or regress. I deliberately did **not** fabricate an `approved=1`
filter, because filtering on a non-existent column would break every query. If
that feature lands later, the filter belongs in `mw_memory::sqlite`'s
per-source queries (one place) and the provenance string in the render helpers.

## `mw search linker --explain` (against a scratch DB)

```
# matches for "linker"  (ranked)

 38%  [note] the linker error was a missing -lstdc++ on the jetson cross build
      memory explain 3000000001
        "the linker error was a missing -lstdc++ on the jetson cross build"
        created 2026-07-15 - last used 2026-07-15 - mentioned 1x - importance 0.55
        links: note
        score 38%  =
           similarity    w0.40 x 0.14 = +0.057   14% term overlap (lexical)
           recency       w0.20 x 0.80 = +0.160   last used 4 days ago
           importance    w0.15 x 0.55 = +0.083   importance 0.55
           reinforcement w0.10 x 0.20 = +0.020   mentioned 1x
         . task          w0.15 x 0.00 = +0.000   no task context

 34%  [session] jetson build session  — `mw show 1`
 33%  [command] cargo ["cargo","build"]  — `mw replay 2`
 30%  [command] cargo ["cargo","build"] ... error: linking with cc failed ...  — `mw replay 1`
 25%  [command] git ["git","push"]  ! [rejected] main -> main (non-fast-forward)  — `mw replay 3`
```

The bookmark that literally names the linker error tops the ranking; the
unrelated `git push` rejection sinks to the bottom despite being an error,
because its lexical similarity to "linker" is 0. Each result carries the "why".

MCP `search_memory` over the same DB returns the same ranking with a `reasons:`
line per result (input schema unchanged).

## Verification

- `cargo build --workspace` — clean, no warnings.
- `cargo test --workspace` — all pass (14 tests in `mw-memory`, including the
  new `cli_and_desktop_rank_identically_for_fixed_now`,
  `tolerates_missing_tables_and_namespaces_ids`,
  `decode_roundtrips_each_source`). No pre-existing failures observed.
- Manual `mw search --explain` and an `mw-mcp` `search_memory` JSON-RPC call run
  against a seeded scratch DB (output above).

## Deliberate simplification

The CLI runs the engine in **lexical** mode (term overlap) — no Ollama, no
embedding cache — so it stays dependency-light and fully offline, matching the
`mw-cli` "no heavy deps" constraint. The desktop app keeps its semantic
embedding path over the same loader + engine. Upgrade path: if the CLI ever
wants semantic recall, attach `OllamaEmbedder` in `mw search` the way
`build_recall_engine` does desktop-side.
