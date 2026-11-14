# Debug with previous evidence

Start with the exact current failure, then search for evidence that matches it:

```bash
mw context --last-error
mw search "distinctive error text"
mw git-fix
```

When a fix works, preserve the conclusion without replacing the original
evidence:

```bash
mw remember "the build needed PKG_CONFIG_PATH set to /opt/example/lib/pkgconfig"
```

Treat old lessons as leads rather than universal truth. Environments and
dependencies change; check the command, timestamp, project scope, and observed
output before reusing a fix.

## Live autosave and interrupted-session recovery

`mw --live` writes the session row with status `recording` when the shell
starts, then a background thread refreshes it every 2 s — cleaned transcript,
byte count, and `ended_at` — so the dashboard stays current while you work.
A clean `exit` finalizes the row as `finished`. Without `--live`, nothing is
written until the shell exits normally.

If the parent process dies first — terminal closed, SSH drop, killed shell —
a watcher finalizes the row as `interrupted` immediately instead of waiting
for the dashboard's next startup. Linux uses `PR_SET_PDEATHSIG`; macOS and
BSD watch a pipe for EOF or fall back to polling `getppid()`. The tested
behavior lives in `crates/mw-cli/tests/parent_death.rs`. Session recording
requires a Unix shell: Linux, macOS, or WSL (see
[known limitations](../limitations.md)).

### If the recording never wrote a row

If `mw` itself is cut off before it writes its row, the raw transcript still
survives in the `sessions/` folder of the data directory. Import it:

```bash
mw-recover              # import orphaned transcripts; also removes empty 0-byte files
mw-recover --quiet      # print only the recovered count
```

`mw-recover` is idempotent: already-imported transcripts are skipped, empty
ones deleted, so it is safe to rerun. `mw-serve` runs the same scan when it
starts. Recovered sessions are marked `interrupted` and carry the note
"recovered from transcript (recording was interrupted before saving)".

### Commands-only capture

A per-directory gate (`[capture.paths]` or `.mwignore`) can narrow capture to
`commands-only`: what ran and how it went — command, argv, cwd, exit code,
timestamps — but no output. Under this gate the `script` transcript goes to a
temp file that is deleted on exit, so no output survives anywhere;
`--live` finalizes an interrupted row the same way, without stored transcript
text. See [capture terminal work](terminal-capture.md) for choosing between
`mw-run`, `mw`, `mw --live`, and global hooks.

### Discard instead of save

While inside a recorded shell, `mw discard` marks the session so that on
`exit` both the transcript and its row are deleted — nothing is saved. Use it
when secrets or garbage hit the screen; `mw rm <id>` handles a session that
was already saved. Redaction is regex-based and not a security boundary, so
before recording sensitive work review the
[local data threat model](../SECURITY.md) and prefer narrowing capture over
relying on later deletion.
