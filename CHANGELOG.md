# Changelog

## 0.7.0 (unreleased)

### Security and privacy

- Bind the dashboard to `127.0.0.1` by default and require authenticated,
  explicit LAN mode for non-loopback access.
- Move dashboard authentication out of URL query strings and into a sign-in
  form backed by an HTTP-only cookie.
- Add bounded output capture, retention controls, sensitive-source auditing,
  repository-scoped deletion, restrictive local file permissions, and a
  documented local threat model.

### Memory lifecycle

- Hold automatically authored conclusions for review by default.
- Add active, stale, and superseded states while preserving original evidence.
- Keep stale and superseded memories out of normal retrieval.

### Reliability and maintainability

- Centralize SQLite connection policy, schema creation, migrations, and writes.
- Split large dashboard and frontend modules by capability.
- Add formatting, linting, dependency auditing, frontend tests, clean-install
  validation, release version checks, and package smoke tests.
- Synchronize command and MCP documentation with the runtime surface.

### Upgrade notes

- Databases are migrated in place when first opened. Back up
  `memorywhale.sqlite3` before upgrading if it contains important history.
- LAN dashboard users must now opt in explicitly and configure authentication.
