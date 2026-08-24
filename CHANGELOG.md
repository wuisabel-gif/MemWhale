# Changelog

All notable changes to MemoryWhale are documented here. This project follows
[Semantic Versioning](https://semver.org/).

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
