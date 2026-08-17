# Capture privacy policy

MemoryWhale applies the shared `memorywhale-core::privacy::sanitize_capture`
policy before captured free text is written to SQLite. The policy redacts known
secret shapes and bounds each stored text field to
`MEMORYWHALE_MAX_CAPTURE_BYTES` (1 MiB by default). Set
`MEMORYWHALE_NO_REDACT=1` only when raw capture is an intentional, local
decision; it is an explicit opt-out, not a guarantee that other tools will not
expose the data.

## Write-path inventory

| Adapter | Sanitized before write | Preserved as operational metadata |
| --- | --- | --- |
| `mw-remember` / `mw-run` | stdout, stderr, notes, command/argv representations, argument index values | cwd |
| `mw` session capture | transcript and session notes | shell, cwd, transcript path, timestamps |
| `mw-recover` / `mw-serve` recovery | recovered transcript | transcript path and timestamps |
| `mw import` / `mw pull` | imported command runs, sessions, and lessons | cwd, filesystem paths, timestamps |
| `mw-mcp remember` / `mw mark` | lesson text through `remember_as` | provenance fields |
| Tauri command capture | stdout, stderr, notes, command/argv representations, derived concepts | cwd and exit status |
| Tauri text import | title, content, summary, quotes, derived concepts | source type and timestamps |
| `mw-screenshot` | notes | screenshot path, cwd, command-run ID |

Command arguments and filesystem paths remain operational metadata with a
documented limitation: a secret embedded in an executable path, cwd, or a
non-recognized argument format may not be detected. Do not treat redaction as a
security boundary.

All derived summaries, quotes, concepts, fingerprints, and indexes in the
covered adapters are created from the sanitized representation. Existing
databases are not rewritten automatically; use an explicit backup and review
plan before attempting historical cleanup.

## Storage and retrieval

Redaction occurs before SQLite writes, not only when rendering a dashboard or
retrieval result. This prevents the raw value from entering the durable table,
FTS indexes, or embedding input through the covered paths. SQLite file
permissions remain part of the local trust boundary.

## Review checklist for new adapters

Before adding a write path:

1. Apply `sanitize_capture` to every user- or process-provided text field.
2. Derive summaries, quotes, concepts, fingerprints, and indexes from the
   sanitized value.
3. Decide explicitly whether paths and command metadata are operational values
   that must remain replayable.
4. Add a SQLite-facing test containing a fake secret and assert the raw value
   is absent.
5. Preserve the documented `MEMORYWHALE_NO_REDACT=1` opt-out unless a separate
   architecture decision changes it.
