# Cursor + MemoryWhale

Give Cursor's agent the same local MemoryWhale tools: it can read what already
failed and write down what it figured out. Memory access, guidance, and optional
execution capture are separate components.

## Status

Verified against Cursor's [MCP](https://cursor.com/docs/context/mcp) and
[CLI MCP](https://cursor.com/docs/cli/mcp) documentation on 2026-08-29.
Project config is `.cursor/mcp.json`. User config is `~/.cursor/mcp.json`.
Both files are merged; a duplicate name in the project file wins.

The development branch adds opt-in local Agent/Shell capture using the published
[Cursor hooks contract](https://cursor.com/docs/hooks). Its fixtures are
documentation-derived, not recordings from a pinned live Cursor version.
**Live Cursor verification is pending:** Cursor was not installed in the test
environment. This does not claim cloud-agent or native Windows capture support.

## Requirements

- MemoryWhale installed with `mw-mcp` on `PATH`.
- Cursor (editor or CLI `agent`).
- Optional: the repository Rule file [`memorywhale.mdc`](memorywhale.mdc).
- For capture: a build containing the new `--capture` and `--from-hook cursor`
  options, including `mw-remember`, on local macOS or Linux/WSL. These options
  are not in the published 0.10.0 binaries.

## Setup

### Connect the MCP server

`mw-mcp` is a local stdio MCP server. Point Cursor at it via `mcp.json`:

- This project only: copy [`mcp.json`](mcp.json) to `.cursor/mcp.json` in
  your project root.
- Every project: put the same JSON in `~/.cursor/mcp.json`.

```json
{
  "mcpServers": {
    "memorywhale": {
      "command": "mw-mcp"
    }
  }
}
```

`mw-mcp` must be on `PATH` (the standard MemoryWhale install). If it isn't, use
its absolute path as `command`. To point at a non-default database, add
`"env": { "MEMORYWHALE_DATA_DIR": "/path/to/dir" }`.

### Add the Rule

The MCP server gives Cursor the tools; this Rule teaches it when to reach
for them (recurring failures, "how did we fix this last time", and saving a
conclusion once a fix is found).

- This project only: copy [`memorywhale.mdc`](memorywhale.mdc) to
  `.cursor/rules/memorywhale.mdc`.
- Every project: add it via Cursor Settings → Rules → User Rules.

It is an "Agent Requested" rule (`alwaysApply: false`), so Cursor pulls it in
only when the description matches what you're doing.

### Opt into command capture (development branch)

Build all helpers together from the repository root. This also provides a reader
that recognizes the new structured `cursor` provenance value:

```bash
cargo build --release --locked -p memorywhale-cli --bins
export MW="$PWD/target/release/mw"
"$MW" integrate cursor --capture --dry-run
"$MW" integrate cursor --capture
"$MW" integrate cursor --capture --check
```

Before enabling capture against an existing store, point the Cursor MCP
configuration at this same build's absolute `target/release/mw-mcp` path and
reload the connection. Building new capture helpers does not replace an older
`mw-mcp` on PATH. Prefer a disposable data directory for the first test.

This capture-only mode adds Shell-matched `postToolUse` and
`postToolUseFailure` command hooks to `~/.cursor/hooks.json`. It does not install
MCP, a Rule, or a skill, change permissions, or approve workspace trust. Follow
Cursor's normal hook review/trust behavior before using them.

Use `--hooks-file .cursor/hooks.json` on every command for an explicit project
file. Arbitrary custom files are not automatically registered with Cursor.
Existing hooks/settings are preserved. Unowned MemoryWhale capture conflicts,
duplicate owned entries, and modified ownership snapshots fail without silently
overwriting configuration. A private sidecar stores only the owned entries;
interrupted writes require inspection rather than forced recovery.

If `MEMORYWHALE_DATA_DIR` is set during installation, its absolute location is
quoted into the hook command. Keep it aligned with the MCP server's store.
Changing that setting or moving the helper requires explicit revert/reinstall
with the same hooks-file selection. `--check` checks local configuration and
executable paths; it does not run Cursor or prove hook delivery.

## Verify

```bash
command -v mw-mcp
```

In the editor, open Settings → MCP and confirm `memorywhale` is listed. From
the Cursor CLI, which uses the same config:

```bash
agent mcp list
agent mcp list-tools memorywhale
```

You should see the six tools: `recent_errors`, `search_memory`, `get_context`,
`remember`, `similar_failures`, and `stats`.

For capture, ask an approved local Cursor session to run a harmless unique
command, then use the same MemoryWhale store to search its marker with
`agent:cursor`. Inspect the recorded command, exit field, and `cursor_status`
notes. Repeat with a nonzero exit and a cancelled/denied call where available.
Record the actual Cursor version and event payload shape before calling that
client configuration verified.

The repository's credential-free tests exercise the complete helper-to-SQLite
path and retrieve fixtures in fresh CLI and MCP processes:

```bash
test_data="$(mktemp -d)"
MEMORYWHALE_DATA_DIR="$test_data" cargo test --locked -p memorywhale-cli --test cursor_capture --test cursor_capture_install
```

These tests do not execute the commands written inside synthetic hook payloads
or substitute for a live Cursor session.

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes |
| Automatic execution capture | Opt-in development adapter; documented local Shell contract, live client verification pending |
| Memory-use guidance | Yes, optional Rule |

MCP access is not automatic execution capture. Without explicitly installing
the capture hooks, commands are recorded only through normal terminal capture,
`mw-run`, or explicit `mw-remember` calls.

### Capture fidelity and limits

- Only the generic Shell outcome pair is consumed. `afterShellExecution`, other
  tools, and blocking/pre-execution events are ignored to avoid overlapping
  surfaces. Repeated legitimate commands remain separate records; replaying an
  identical hook payload is not a globally deduplicated delivery protocol.
- `postToolUse.tool_output` must be a JSON-stringified object. Numeric
  `exitCode` values from 0 through 255 are recorded when present. Tool completion,
  malformed output, or an error message never implies exit zero or exit one.
- Failure, timeout, denial, and cancellation have separate `cursor_status`
  notes. Their numeric exit remains NULL; permission denial is explicitly
  marked not executed, and other failures do not confirm execution.
- The actual absolute `cwd` must exist locally so directory capture policy can
  be applied. Missing, invalid, or unavailable cwd skips
  the event; neither workspace roots nor the hook process cwd are substituted.
- Available conversation/generation/tool IDs, duration, and client-version
  metadata are retained with bounds. User email, arbitrary input notes, and
  claimed agent identity are not imported.
- Hook JSON is bounded to 4 MiB. Commands over 8 KiB are skipped rather than
  truncated. Output previews are UTF-8-safe and capped at 20,000 bytes before
  shared redaction, with explicit truncation markers. IDs are capped at 256 bytes.
- Shared exclusions, `off` and `commands-only` capture policy, and redaction
  still apply. Hook processing emits no decisions or stdout, returns normally
  on malformed input/storage failure, and uses a five-second configured timeout.
- Build/use matching helpers and readers. Older releases that reject unknown
  producing-agent values may not read stores containing `cursor` records.

The Rule falls back to the `mw` CLI, so even with only the binaries installed,
Cursor can run `mw context --last-error` / `mw search "…"` / `mw remember "…"`.
With no editor integration at all, `mw ask` packages the last failure onto
your clipboard for any chat.

## Example prompt

> Use MemoryWhale to check whether I encountered a similar failure before.

## Troubleshooting

- Run `command -v mw-mcp` in the environment that launches Cursor. Use an
  absolute `command` if the GUI `PATH` differs from your shell.
- Confirm you edited `.cursor/mcp.json` or `~/.cursor/mcp.json`, not some other
  client's file.
- Restart Cursor or reload MCP if the server does not appear.
- If the wrong database opens, set `MEMORYWHALE_DATA_DIR` in the server `env`.
- Run `mw doctor` to check the MemoryWhale install.
- For capture, check the selected hooks file, executable permissions, and the
  same data-directory setting with `mw integrate cursor --capture --check`.
  Hook failures are silent by default. For a controlled diagnostic invocation,
  set `MEMORYWHALE_HOOK_DIAGNOSTICS=1` in the environment of `mw-remember
  --from-hook cursor`; fixed, nonfatal stderr messages can explain skipped
  payloads or storage failures. Test with synthetic input and a disposable
  `MEMORYWHALE_DATA_DIR` to avoid recording an event twice in the normal store.
  The diagnostic messages never include the payload or underlying storage error.

## Uninstall

Remove only the capture hooks with the same build/file selection used to install:

```bash
"$MW" integrate cursor --capture --revert
# For an explicit project file:
"$MW" integrate cursor --capture --hooks-file .cursor/hooks.json --revert
```

This removes only unchanged owned capture entries, not memory, MCP configuration,
Rules, or optional skills. Remove those independently if desired.

Delete the `memorywhale` entry from `.cursor/mcp.json` and/or
`~/.cursor/mcp.json`. Remove `.cursor/rules/memorywhale.mdc` or the matching
user Rule. Restart Cursor. This does not delete the MemoryWhale database.
