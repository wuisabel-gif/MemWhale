# Local data threat model

MemoryWhale records terminal commands and, in full-capture mode, terminal
output. That data can include credentials, private paths, hostnames, source
code, and proprietary output. The database and transcript directory therefore
need the same protection as shell history or an unencrypted developer backup.

## Trust boundaries

- MemoryWhale is local-first, but any process running as the same operating
  system user may be able to read its files.
- Secret redaction reduces accidental retention; it is not a security
  boundary. Unknown formats and transformed credentials can evade patterns.
- Synced, exported, shared, or backed-up files leave the local trust boundary.
  Review them before sending and protect the destination independently.
- The dashboard should only be exposed beyond loopback when authentication is
  enabled and the network is trusted.

## Controls

- Use `.mwignore` or `[capture.paths]` to set sensitive trees to `off` or
  `commands-only`.
- Captured text fields are limited to 1 MiB by default and contain an explicit
  truncation marker. Set `MEMORYWHALE_MAX_CAPTURE_BYTES` to a positive byte
  count to choose a different limit.
- On Unix, MemoryWhale restricts its data directory to mode `0700` and its
  SQLite database to `0600` when opened.
- Run `mw audit` to inspect the effective capture policy, retained volume, and
  highest-volume session sources.
- Use `mw rm` for individual deletion and `mw prune --older-than <window>` for
  retention cleanup. Preview bulk cleanup with `--dry-run`.

For a sensitive repository, prefer preventing capture over relying on
redaction or later deletion.
