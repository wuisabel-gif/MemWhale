# Changelog

All notable changes to MemoryWhale are documented here. This project follows
[Semantic Versioning](https://semver.org/).

## [0.10.0] — Agent-Native Memory — September 6, 2026

Product version `0.10.0` across the CLI, web UI, and desktop app;
`memorywhale-core` `0.5.0`; SQLite schema `10`.

This release makes shared debugging memory easier to connect to coding agents
without confusing memory access with automatic capture. See the
[release notes](docs/releases/0.10.0.md) for installation, migration, and
integration limitations.

### Added

- **Claude Code and Rho setup** — `mw integrate claude` and `mw integrate rho`
  install MCP registration, a bundled memory-use skill, and Rust-backed capture
  hooks; `--revert` removes the integration. No checkout or Python hook is
  required. Rho also supports `--http [url] --token <secret>`. (#230, #241)
- **Independent integration diagnostics** — `mw doctor` reports each client's
  MCP registration, capture hook, and skill separately. Optional clients that
  are absent are `not detected`, not failed installs. The bounded MCP probe is
  read-only and does not call memory tools. (#245)
- **Canonical repositories and distinct worktrees** — schema 9 adds repository
  IDs/names and worktree roots to commands and sessions, using local Git
  metadata without contacting remotes. Linked worktrees share a repository ID
  while keeping their own paths; legacy project tags remain supported. (#234)
- **Structured agent provenance** — schema 10 adds nullable `command_runs.agent`:
  `claude` or `rho` for supported hooks, `NULL` for terminal/manual and legacy
  records. `terminal` is the display/filter label for `NULL`, not a stored agent
  string. Provenance is separate from source type and is not inferred from
  notes. CLI, TUI, dashboard, desktop, JSON API, and MCP expose it; search accepts
  `agent:claude|rho|terminal`. (#256, #257)
- **HTTP MCP** — `mw-serve` exposes the same six tools as `mw-mcp` at
  `POST /mcp`, one JSON-RPC object per request, without SSE sessions. Explicit
  loopback tokens and all non-loopback binds require Bearer authentication;
  `--lan` can mint a persistent token. (#241)
- **Opt-in read-only JSON API** — `mw-serve --api` enables bounded `/api/v1`
  health, search, memory, command, session, and repository endpoints, with an
  OpenAPI contract and the dashboard's access controls. Disabled by default;
  no write, arbitrary SQL, or arbitrary file-read endpoints. (#236)
- **Explicit GitHub context** — `mw github context <pr>` uses the existing
  `gh` login to print bounded, redacted PR metadata, CI check-runs, classic
  commit statuses, and reviews.
  Read-only: no checkout, review submission, automatic save, or background
  synchronization. (#260, with release-branch status and subprocess hardening)

### Improved

- **MCP compatibility** — supports revision `2026-07-28` with per-request
  metadata and `server/discover`, retaining legacy `2025-11-25` and
  `2024-11-05` initialization. (#233)
- **Integration guides** — standardized setup, verification, capabilities,
  limitations, and uninstall instructions. The cross-agent handoff demo uses
  Claude fixtures and a simulated Rho client against real `mw-mcp`; it does
  not run live agents or verify the fixture's Cargo fix. (#246, #258)
- **README locale synchronization** — source hashes and checks protect
  structure, commands, and link targets. Provider-backed translation requires
  explicit configuration; synchronization is not native-speaker review. (#242)
- **Release and CI maintenance** — core publication was corrected after
  v0.9.0, Homebrew PR creation may be skipped when unavailable, and memory-audit
  storage is scoped to consuming steps. The optional Second-Opinion reviewer
  skips review when `REVIEW_API_KEY` is missing; its presence is not evidence
  that a review ran. (#224, #225, #226, #240, #261)

### Compatibility and limits

- **Breaking Rust API:** `memorywhale_core::Memory` now requires
  `agent: Option<String>` in Rust struct literals. Add `agent: None` for existing
  un-attributed memories. `#[serde(default)]` keeps older serialized memories
  without the field readable. Core is independently versioned at `0.5.0`.
- Existing databases migrate additively to schema 10. Historical rows retain
  `NULL` agent provenance rather than guessing their producer.
- Rho hooks preserve failure metadata when command text is unavailable, using
  a sentinel rather than inventing a command or stdout. Successful calls with
  no command text are skipped.
- Task-start recall, automatic failure lookup, and pre-compaction saving are
  a documented future client-orchestration loop, **not implemented lifecycle
  automation in this release**. Hooks capture supported events; skills provide
  guidance; MCP calls remain client-driven. (#262)

## [0.9.0] — Memory stewardship

Product version `0.9.0`; `memorywhale-core` `0.3.0`.

This release makes memory maintenance explicit and safer: users can compact
redundant evidence without deleting rows, retrieval reuses FTS indexes, and
present-but-damaged SQLite sources report errors instead of appearing empty.
It also adds desktop CSP hardening, a public security policy, a website refresh,
and localized README entry points.

### Added

- **Rule-of-thumb memory compaction** — `mw memory compact` previews a
  conservative plan by default; `--apply` is explicit. Failed, fingerprinted,
  and bookmarked runs stay protected. Session raw files and original byte
  counts remain intact, while large successful output is bounded with
  head/tail markers. (#212)
- **Memory compaction reference** — documents thresholds, safety invariants,
  dry-run workflow, and the distinction between TTL expiry and compaction.
  (#212)
- **ContextGC ecosystem documentation** — explains how active context
  management complements MemoryWhale's durable development memory. (#220)
- **Localized README entry points** — Simplified Chinese, Korean, Traditional
  Chinese, and Japanese. (#214, #215, #216, #217)

### Improved

- **FTS5 retrieval performance** — caches a bounded set of corpus indexes across
  engine instances and requests, coalesces concurrent builds, preserves
  `BuiltinEngine`'s `Send + Sync` contract, and invalidates on corpus changes.
  (#221)
- **Website onboarding** — adds use cases, integration discovery, docs/security
  navigation, `mw demo`, real install commands, v0.8 release messaging, and
  responsive long-command handling. (#213)

### Security

- **Restrictive Tauri CSP** — same-origin scripts/assets, Tauri IPC only,
  explicit `base-uri 'none'` and `form-action 'none'`, no wildcard scripts or
  `unsafe-eval`. (#218)
- **Retrieval integrity** — absent optional source tables remain supported, but
  present schema/query/row failures now surface as structured errors. Bookmark
  compatibility is selected from schema metadata and unsupported shapes fail
  closed. (#219)
- **Security disclosure policy** — added `SECURITY.md` and enabled GitHub
  private vulnerability reporting. (#207)

### Notes

- The original tag declared no `memorywhale-core` version bump. Post-tag
  correction (#224) updated the v0.9.0 publication to depend on core `0.4.0`,
  which published the already-used public API. The `0.3.0` version above
  describes the original tag, not the corrected package dependency.
- Upgrade from v0.8.0 if you want compaction, safer retrieval failures, FTS5
  reuse, or the desktop CSP hardening.

## [0.8.0] — Security & hardening

Product version `0.8.0`; `memorywhale-core` `0.3.0`.

This release ships seven accumulated security fixes, two new commands
(`mw doctor`, `mw --version`), the dashboard integration grid, and guides for
eleven additional agents and model gateways.

### Added

- **`mw doctor`** — diagnoses the local install: data dir, database, shell
  hooks, and the `mw-mcp` MCP server (resolve, initialize, tool-list, timeout
  bounds), with actionable next steps per failure. (#187)
- **`mw --version` / `mw -V`** — prints the version without opening the
  database. (#127)
- **Dashboard integration grid** — the local dashboard now shows which coding
  agents, editors, and model-routing tools MemoryWhale composes with, linking
  each cell to its setup guide. (#182, #205)
- **Pullfrog workflow memory archive** — an opt-in GitHub Actions workflow
  records sanitized Pullfrog run metadata (never prompts, logs, or diffs) into
  a MemoryWhale bundle artifact. (#189)
- **Hermes Agent integration** — `mw integrate hermes` registers `mw-mcp` in
  Hermes' config. (#137)
- **Integration guides** for Rho, Pi, Jan, OpenCode, CodeWhale, Pullfrog,
  CLIProxyAPI, OpenRouter, and a full Claude Code guide (hook + skill), plus a
  use-cases page documenting the three target users end to end.
  (#175, #178, #179, #180, #188, #202, #203, #204, #208)
- **MCP trust model documentation** — the local stdio trust boundary is now
  written down. (#165)
- CI now audits memory capture on PRs and runs repository consistency checks.
  (#136, #184)

### Fixed

- **Installer release metadata** — `/releases/latest` is fetched once, parsed
  with `jq` (portable fallback), and the tag validated as semver before any
  URL is built; API failures get a dedicated message. (#190)
- **`mw-serve` supports HEAD requests** — same headers as GET, no body; health
  checks no longer get 405. (#199)
- **Import tolerance** — malformed `argv_json` in an imported database no
  longer aborts the merge; legacy `bookmarks` tables with missing columns
  import with defaults. (#196 follow-up on #185)
- Shell completions list every current `mw` / `mw-serve` command. (#183)
- Frontend postcss path-traversal vulnerability (GHSA-r28c-9q8g-f849). (#197)

### Security

- **DNS-rebinding protection on the dashboard** — loopback binds reject
  rebound Host headers; responses carry hardening headers
  (`X-Content-Type-Options`, `Referrer-Policy`, `Cache-Control`, strict CSP);
  the token comparison is constant-time. (#181)
- **Bounded HTTP parsing** — request lines, headers, body size, and concurrent
  connections are capped; malformed input gets bounded errors instead of
  unbounded reads. (#169, #186)
- **Cookie safety** — control characters in timezone cookie values are
  rejected, preventing header injection. (#170)
- **Desktop: unrestricted `import_file` removed** — the Tauri shell no longer
  exposes a command that could read arbitrary paths. (#174)
- **Shared capture sanitization before storage** — all write paths
  (`mw-remember`, `mw-run`, session transcripts, recovery, import, desktop)
  route through one `sanitize_capture` policy: secret-shape redaction and a
  1 MiB bound per field, before SQLite. (#185)
- **Split secret arguments redacted** — `--token SECRET` (two argv elements)
  is now redacted like `--token=SECRET`, across CLI and desktop capture.
  (#196)

### Notes

- No schema migrations this release; existing databases are unchanged.
- Users on 0.7.0 should upgrade for the dashboard security fixes; see
  `docs/SECURITY.md` for the disclosure policy.

## [0.7.0]

Product version `0.7.0`; `memorywhale-core` `0.3.0`.

### Added

- **`mw explain <id>`** — per-signal score breakdown for a single memory
  (similarity, recency, importance, reinforcement, task). `mw search` now prints
  each result's `#id` so it's discoverable. (#91)
- **Search filters** on `mw search`: `tag:`, `source:`, `before:`/`after:`
  (YYYY-MM-DD), and `limit:` — e.g. `mw search docker after:2026-01-01 tag:infra`. (#105)
- **Duplicate detection** on `mw remember` — a near-identical lesson is refused
  with a clear message; `--force` keeps both. Read-only (never merges/deletes). (#106)
- **TTL / auto-expiring memories** — `mw remember <text> ttl:7d` (m/h/d/w). Past
  their TTL, notes are swept out of retrieval on open; the row (evidence) is
  preserved. (#103)
- **Memory links** — `mw link <a> <b> [rel:<type>]`, `mw unlink`, and
  `mw links <id>` to build and inspect a typed graph between memories. (#104)
- **`mw pet`** — a whale whose mood reflects your store (well-fed / content /
  sleepy / hungry); `mw pet --watch` animates it. Read-only. (#118)
- **MCP client integrations** under `integrations/`: OpenClaw, CrowClaw, Goose,
  Claude Desktop, Cline, Continue, Gemini CLI, and Zed — each `mw-mcp` plugs in
  with no new code.
- GitHub issue and pull-request templates encoding the project's contribution
  rules.

### Fixed

- **`parse_ts` no longer masquerades corrupt timestamps as "now."** A malformed
  RFC3339 timestamp now falls back to the Unix epoch (and warns) instead of
  `Utc::now()`, so a corrupted row sorts as oldest rather than getting an
  unearned recency boost. (#101)

### Security

- **The installer verifies release integrity.** Releases now publish a
  `<asset>.sha256`, and `install.sh` checks the downloaded tarball against it,
  aborting on a mismatch. Older releases without a checksum are skipped with a
  warning. (#102)

### Notes

- Schema migrations to `user_version = 8` are additive (a nullable `expires_at`
  column and a new `memory_links` table); existing databases upgrade in place.
