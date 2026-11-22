# ADR 0001: Opt-in, user-reviewed knowledge memory

- Status: **proposed**. Nothing described here is implemented.
- Date: 2026-11-22
- Baseline: main at `9a67165`, product 0.29.0, `memorywhale-core` 0.13.0,
  SQLite schema 17.

This is the first ADR in the repository, so it also sets the convention:
`docs/adr/NNNN-short-title.md`, one decision per file, with a status line.

## Context

MemoryWhale today is a local debugging memory. The README says it is not "a
general-purpose personal memory", and `CONTRIBUTING.md` lists "unrelated
personal-memory features" as outside the project's responsibility.

The proposal is to add a second kind of memory: durable facts a person wants
several assistants to share, such as a preference, a project decision, or a
workflow. The sources are ChatGPT, Codex, Claude chat, and Claude Code. Each
memory is proposed, reviewed by the person, and only then usable.

That is a change of product scope, not a feature inside the current one. It
needs a decision before code.

### What exists already (audit)

| Area | Where | What it does |
| --- | --- | --- |
| Migrations | `crates/mw-cli/src/lib.rs` (`migrate`, `LATEST_SCHEMA_VERSION = 17`) and `crates/mw-cli/src/storage.rs` (`initialize`) | Numbered, additive `PRAGMA user_version` steps, run on every open. No pre-migration backup is taken. No binary refuses a store with a higher version. |
| Core crate | `crates/memorywhale-core/src` | Read side only: `sqlite.rs` loads memories, `engine.rs` and `scorer.rs` rank them, `privacy.rs` masks secrets, `contradiction.rs` is the lexical conflict heuristic, `policy.rs` is compaction. It owns no schema. |
| Lessons | `bookmarks` table | `label`, `cwd`, `author_kind`, `author_name`, `source_session_id`, `approved`, `status` (`active`, `stale`, superseded), `superseded_by_id`, `expires_at`. |
| Review queue | `pending_agent_notes`, `approve_note`, `reject_note` in `crates/mw-cli/src/lib.rs`; `mw memory pending`, `approve`, `reject` | Agent-written lessons wait with `approved = 0`. Readers filter `approved = 1 AND status = 'active'` (`memorywhale-core/src/sqlite.rs`, near line 500). |
| Conflicts | `contradiction_flags` table, `mw contradictions` | A pending, confirmed, or rejected flag between two memory ids. Nothing is overwritten. |
| Sharing | `mw memory publish` and `adopt` (`.memorywhale/lessons.md`), `mw export`, `mw push`, `mw import` | Lessons and evidence can leave the machine only on an explicit command. |
| MCP | `crates/mw-cli/src/mcp/tools.rs` | Six tools. `remember` is the only writer and calls `remember_as(.., "agent", ..)`, which stores the lesson unapproved while review is on. |
| HTTP | `crates/mw-cli/src/bin/mw-serve.rs`, `crates/mw-cli/src/serve_auth.rs` | Read-only `/api/v1/*` behind `--api`, and the six tools at `POST /mcp`. Loopback is open; a non-loopback bind needs a bearer token. |
| Interface | `src/App.tsx` (`PAGES`), `src/pages/*.tsx`, `src-tauri/src/store.rs` | Five pages: Home, Search, Recall, Commands, Lessons. The desktop shell calls the CLI library and has no database of its own. |
| Integrations | `integrations/TEMPLATE.md` | Eight fixed headings per client guide. |
| Policy | `docs/information-flow-policy.md` | A four-level lattice (`public < internal < sensitive < restricted`) and ten prerequisites. It is design policy; no enforcement exists. |

Every document the brief names exists. One correction to the brief: it places
"schema and migrations" in the core crate, but in this repository they live in
`mw-cli`, and the core crate has never owned a table.

### What the vendors support (retrieved 2026-11-22)

These were read as search summaries of the official help pages, not the full
pages. Treat each as "official source, to be re-read before it is relied on".

| Claim | Source | Standing |
| --- | --- | --- |
| ChatGPT exports a ZIP from Settings, Data controls. It holds conversations as JSON (`conversations.json`, or numbered files for large exports) and an HTML viewer. Not self-service on Business, Enterprise, or Healthcare workspaces. | [OpenAI help: exporting your ChatGPT history and data](https://help.openai.com/en/articles/7260999-how-do-i-export-my-chatgpt-history-and-data) | Official page. The JSON's internal fields are not documented there. |
| ChatGPT can connect a custom MCP server. Full support is for Business, Enterprise, and Edu; Pro is limited to read and fetch. The server needs a public HTTPS endpoint, or OpenAI's tunnel for a private one. | [OpenAI help: connecting custom MCP servers in ChatGPT](https://help.openai.com/en/articles/12584461-developer-mode-and-mcp-apps-in-chatgpt) | Official page. Plan limits are described as current and may change; sources disagree on whether developer mode is needed. |
| Claude exports conversations, account data, and memory entries from Settings, Privacy, on the web or desktop app, for individual plans. Uploaded files are not included. | [Anthropic support: export your Claude data](https://support.claude.com/en/articles/9450526-export-your-claude-data) | Official page. The file format is not documented there. |
| Claude accepts custom connectors over remote MCP on every plan, one on Free. The connection is made from Anthropic's cloud, so the server must be reachable over HTTPS on port 443. | [Anthropic support: custom connectors using remote MCP](https://support.claude.com/en/articles/11175166-get-started-with-custom-connectors-using-remote-mcp) | Official page. |
| Claude Desktop can start a local MCP server from `claude_desktop_config.json`. | [MCP docs: connect to local servers](https://modelcontextprotocol.io/docs/2026-07-28/develop/connect-local-servers) | Official page. MemoryWhale already has a Claude Desktop guide for this. |

Three consequences:

1. Neither vendor documents the fields inside its export. A parser can only
   be written against real export files a person supplies. None are in the
   repository.
2. Both chat products reach a custom MCP server from their own cloud. Serving
   them means exposing MemoryWhale on a public HTTPS endpoint, which the
   local-first rule forbids by default. The only local route to a chat client
   today is Claude Desktop's local MCP server.
3. Neither vendor offers a way to write into its own built-in memory.
   MemoryWhale will not claim to sync with it.

## Decision

### 1. The scope changes, and says so

MemoryWhale gains an opt-in knowledge memory beside its debugging memory. The
README, `docs/architecture.md`, and the scope paragraph in `CONTRIBUTING.md`
are updated in the same release that ships the first usable command, not
before. Until then this ADR is the only place the feature is described.

A store that never runs a `mw knowledge` command behaves exactly as it does
today.

### 2. Knowledge lives in its own tables, not in `bookmarks`

This is the main open question, and the recommendation is **new
`knowledge_*` tables**.

`bookmarks` already has approval, lifecycle, authorship, and a review queue,
so extending it looks cheaper. It is the wrong place because of what reads it:

- **A lesson can stop a command.** `lesson_before` denies an agent's command
  when a lesson opens with "when" and backticked text. An imported chat
  sentence shaped like that would become a command filter.
- **A lesson attaches to failures.** `matching_lesson` shows a lesson when its
  words match an error line or a program name. A preference such as "I use
  cargo build with the release profile" would be shown on unrelated failures.
- **Every MCP client reads every lesson.** `search_memory` and `get_context`
  return all approved lessons to any connected client. Knowledge needs
  per-client limits, and the six tools must keep their semantics.
- **Lessons travel.** `mw memory publish`, `mw export`, and `mw push` send
  lessons to a repository or another machine. Personal knowledge must not
  ride along.
- **The loader is strict.** `memorywhale-core` rejects a `bookmarks` table
  with an unexpected column mix. New columns on it risk older binaries.

Separate tables make "knowledge never changes debugging behavior" true by
construction, since no existing query can see them.

What is reused instead of copied: the approval wording and review flow of
`mw memory`, `provenance_label` for display, `sanitize_capture` for masking,
and the `memorywhale-core::contradiction` heuristic for conflicts.

A lesson stays a lesson. The brief lists `lesson` as a knowledge kind; this
ADR drops it, so there is one home for "what fixed this".

### 3. Schema, in the smallest steps that work

Migration 18 adds three tables. Later tables arrive with the milestone that
first needs them, each in its own migration.

`knowledge_memories`

| Column | Notes |
| --- | --- |
| `id` | Integer key. |
| `uid` | Random stable id, unique. Survives export and re-import; row ids do not. |
| `canonical_text` | One atomic statement, masked on write. |
| `kind` | `preference`, `profile`, `project`, `decision`, `workflow`. |
| `scope` | `global` or `project`. |
| `project` | Text, matching `sessions.project`; null for global. |
| `sensitivity` | A label from the information-flow lattice. |
| `status` | `proposed`, `approved`, `rejected`, `archived`. |
| `version` | Starts at 1; an edit adds one. |
| `created_at`, `updated_at`, `valid_until` | RFC 3339; `valid_until` may be null. |

`knowledge_sources`: `memory_id`, `platform` (`chatgpt`, `codex`,
`claude_chat`, `claude_code`, `manual`), `ingestion_method`, `captured_at`,
`source_hash`, a masked `source_excerpt`, and optional `conversation_ref`. A
unique index on `(memory_id, source_hash)` makes re-import idempotent.

`knowledge_audit`: `memory_id`, `action` (`proposed`, `approved`, `edited`,
`rejected`, `archived`, `deleted`, `exported`), `actor`, `at`. It holds no
memory text, so deleting a memory leaves a record that something was deleted
and nothing of what it said.

Deferred, with the milestone that needs each:

| Table | Arrives with | Why not now |
| --- | --- | --- |
| `knowledge_relations` | M2 | Conflicts and duplicates only appear once imports exist. |
| `knowledge_imports` | M2 | Job state matters for large export files. |
| `knowledge_client_rules` | M3 | Rules need a client to apply to; no MCP tool reads knowledge before M3. |

### 4. Migration, backup, and rollback

- Migration 18 only runs `CREATE TABLE IF NOT EXISTS` and `CREATE INDEX IF
  NOT EXISTS`. It touches no existing row.
- Before the first write to a knowledge table, MemoryWhale copies the store
  once with `VACUUM INTO` to `memorywhale.pre-knowledge.sqlite3` beside it.
  The repository takes no backup before migrations today; this is new and
  limited to this feature.
- Rollback is installing the previous version. No binary checks for a newer
  schema, so a 0.29 binary opens a schema 18 store and ignores the new
  tables. This is tested by opening a migrated store with queries the 0.29
  code uses.
- `mw export`, `mw push`, `mw import`, and `mw memory publish` do not read the
  knowledge tables. Knowledge leaves only through `mw knowledge export`.

### 5. Approval

Nothing is usable until a person approves it. That holds for every writer:
import, an agent's proposal, and manual entry through MCP. Manual entry at the
person's own terminal (`mw knowledge add`) is approved on entry, as
`mw remember` is today.

`review_agent_memories = false` does not apply to knowledge. There is no
setting that approves knowledge automatically.

An edit to an approved memory raises its version and writes an audit row. M1
does not keep the previous text; say so if version history should. A conflict never rewrites an
approved claim; both stay until the person chooses.

### 6. Access rules, and their honest limit

Every read of knowledge goes through one function that takes the caller's
client name, project, and allowed sensitivity, and returns only approved,
unexpired memories inside those limits. CLI, MCP, HTTP, and export all call
it. A denied read looks the same as an empty one.

The limit must be stated wherever the feature is documented: **this is a
filter for honest clients, not a security boundary.** Any process running as
the same operating-system user can read the SQLite file directly. A client
name sent over stdio MCP is not an identity. The information-flow policy
lists ten prerequisites for real enforcement, and this ADR meets none of
them. The documentation and interface will say "limits what a configured
client is shown" and never "protects" or "enforces".

The `sensitivity` column uses the lattice's four labels so that later
enforcement has something to work with. The policy forbids choosing a default
label implicitly, so the default is an open decision below.

### 7. Imported text is data

- Imported text is stored and shown as quoted source, never placed where a
  client would read it as an instruction. A context pack wraps each memory as
  a labeled fact with its source and date.
- No knowledge text is ever tested against commands or errors. The stop and
  the failure note read `bookmarks` only.
- Secrets are masked on write with the existing rules. Text that still looks
  like a credential after masking is skipped and counted, not stored.
- Extraction is rule-based and deterministic. A model-assisted extractor is a
  later, opt-in addition and no test depends on one.
- A test stores a prompt-injection payload and checks it comes back inert.

### 8. Where the code goes

| Piece | Crate | Reason |
| --- | --- | --- |
| Schema, migration 18, repository functions, the access function, audit | `mw-cli` (new `knowledge.rs`) | Schema and writes already live here. `lib.rs` is over 5,000 lines, so this does not go in it. |
| Conflict heuristic | `memorywhale-core`, unchanged | Reused as is. |
| Ranking of knowledge search | `mw-cli` at first, plain SQLite `LIKE` | A ranked engine is not needed for hundreds of rows. Move to core if it earns it. |
| Commands | `mw-cli` (`mw knowledge ...`) | New namespace; no existing command changes. |
| MCP tools | `mw-cli/src/mcp`, M3 | Added beside the six, never replacing them. |
| Pages | `src/pages`, after the CLI works | One Knowledge page with a review tab first. |
| Desktop | `src-tauri/src/store.rs` | Thin commands over the same functions. |

`memorywhale-core` stays at 0.13.0 through M1.

### 9. M1, cut into pull requests

M1 is offline, local, and reviewed. One pull request per line, each with its
own tests and each leaving every existing test green.

1. Migration 18 and the three tables, with migration and downgrade tests.
2. Repository functions: add, propose, approve, reject, edit, archive,
   delete, each writing an audit row.
3. The single access function and its tests, including project A and
   project B isolation and the empty-looking denial.
4. `mw knowledge add`, `list`, `show`.
5. `mw knowledge review`: pending, approve, reject, edit.
6. `mw knowledge search`.
7. `mw knowledge import` for a documented MemoryWhale JSON and Markdown
   format, idempotent, with masking and the injection test.
8. `mw knowledge export`: a bounded Markdown context pack with generated
   markers, for one project and one client.
9. `mw knowledge rm` and `mw knowledge doctor`, plus the one-time backup.
10. Docs: CLI reference, privacy write-path table, architecture, the README
    and `CONTRIBUTING.md` scope lines.
11. Knowledge page in the interface with the review flow.

Not in M1: any vendor export parser, any MCP tool, client rules, relations,
model-assisted extraction, sync, and a browser extension.

## Open decisions for the maintainer

1. **Scope.** Does MemoryWhale become a debugging memory with an opt-in
   knowledge memory, or should this be a separate project? Everything else
   depends on this.
2. **Tables.** Accept new `knowledge_*` tables over extending `bookmarks`?
3. **Default sensitivity.** Recommended: new memories start `sensitive`, and
   a context pack includes only `internal` and below unless asked. That fails
   closed, at the cost of a labeling step before anything is exported.
4. **Export fixtures.** M2 needs a real ChatGPT export and a real Claude
   export to write parsers against. Will you supply them, and may redacted
   fixtures derived from them be committed?
5. **Chat clients.** Both vendors reach a custom MCP server from their cloud.
   Recommended: do not expose MemoryWhale publicly; support chat clients
   through exports in, context packs out, and Claude Desktop's local MCP.
6. **Naming.** `mw knowledge` as the namespace, and "knowledge" as the word
   in the interface.
7. **Release.** Ship M1 behind no flag as soon as the first command works, or
   hold it until the review flow and export exist? Recommended: hold until
   items 1 to 9 are merged, then release once.

## Consequences

- The README's "not a general-purpose personal memory" line has to change.
  That is the visible cost of this decision.
- Two memory domains mean two review queues. The interface has to make the
  difference plain.
- The access filter will be read as security by some people however it is
  worded. The limit has to be repeated in the CLI output of
  `mw knowledge doctor`, not only in docs.
- Chat-client support stays manual until a vendor offers a local route.
