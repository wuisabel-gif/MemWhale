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

## Dashboard HTTP parser decision

`mw-serve` intentionally keeps a small dependency-free HTTP/1.1 parser for the
local dashboard and headless machines. The security contract is covered by
adversarial tests for bounded request lines, headers, bodies and connections;
duplicate or ambiguous lengths; Host rebinding; cookie and form handling;
percent-decoded paths; HTML escaping; response framing; and timezone parsing.
It rejects unauthenticated non-loopback binds, rejects chunked transfer
encoding, and applies security headers to normal, authentication, redirect,
and error responses. Each accepted connection has a 10-second read and write
timeout, in addition to the request/header/body byte limits.

The current decision is to retain this parser rather than replace it with a
larger HTTP dependency. A future replacement should preserve the localhost
default, authenticated LAN requirement, size/time limits, and these tests as
the compatibility contract.
