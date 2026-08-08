# Changelog

All notable changes to MemoryWhale are documented here. This project follows
[Semantic Versioning](https://semver.org/).

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
