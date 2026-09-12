# Rho 2.10.0 compatibility verification

Issue #276; checked **September 12, 2026**, on **macOS arm64**.

- Client: installed native `rho 2.10.0`.
- MemoryWhale: `mw 0.10.0`, built from source commit `6d1de54` (all release helpers from the same checkout).
- Model and normal permission classifier: `openai-codex/gpt-6-astra`, low reasoning, existing Codex OAuth connection.
- Configuration: explicit temporary Rho profiles, disposable Git project, project-local memory skill, reviewed project hooks, and a dedicated MemoryWhale SQLite store.

This is a version-specific verification, not a promise about every Rho release
or configuration. No production capture/parser changes were needed.
The installer now warns about custom-profile skill discovery rather than
silently implying that its copied file will load; it does not redirect writes
into another scope. The manual guide uses the verified project-local path.

## Observed results

| Scenario | Evidence and outcome |
| --- | --- |
| Successful command | A native Bash call printed `MW_RHO210_SUCCESS`. The delivered schema-2 `after_tool_use` event included command, working directory, `status: succeeded`, and duration. MemoryWhale recorded the command with `agent: rho`, an unknown numeric exit, and no stdout. |
| Nonzero command | Bash reported exit 7 and `MW_RHO210_FAILURE`. The hook supplied `status: failed` plus an execution failure message, but no numeric exit-code field. The stored exit correctly remained NULL rather than being inferred from the message. |
| Policy denial | A reviewed test-only blocking hook denied `MW_RHO210_DENIED`. MemoryWhale retained `policy_denied` evidence and did not invent an execution exit code. The agent did not retry or bypass the denial. |
| Fresh retrieval | A fresh native session loaded the project memory skill, called real MCP search, and retrieved the three synthetic records. An interactive fresh session repeated this successfully. |
| Resumed session | The saved interactive session was reopened by its exact session ID. Skill loading and MCP retrieval succeeded; the resumed client exited normally. |
| Unsaved interactive session | `rho --no-save --prompt ...` loaded the memory skill and ran one harmless Bash command. After normal exit, the temporary Rho profile contained configuration and a usage ledger, but no session transcript or prompt-history files. **The separately enabled MemoryWhale hook still recorded the command.** |
| Cancellation/run timeout | A start-marker file proved the synthetic sleep command began. A 20-second automation timeout stopped the run with exit 124. The collector received a pre-tool event and `session_failed`, but no corresponding `after_tool_use`. No cancelled-command row appeared in MemoryWhale. **Do not promise complete cancellation capture.** |
| Missing/truncated fields | Regression tests mutate copies of the live fixtures. Missing/truncated command evidence is omitted or represented by the existing sentinel; truncated cwd is not presented as complete evidence. These mutations are fixture tests, not live oversized-command tests. |
| Custom profile and skills | Explicit configuration/state directories and project `.agents/skills/memorywhale` worked. Pinned Rho source still discovers loose user skills from HOME-based directories; a custom `RHO_HOME` must not be assumed to relocate them. |
| CLI restrictions | The installed client rejected `--no-save run ...` and `--no-save --resume ...`, both with exit 2 before a model turn. Unsaved mode and resume are interactive-only concerns, not flags to attach to an automation run. |

## What this means for capture and privacy

Rho 2.10 does provide capability-based command text and working directory. The
older statement that its hook payload never includes a command is outdated.
However, **tool status is not process exit status**, and stdout is still not a
field in the observed hook envelope. The existing adapter preserves what it
receives without fabricating numeric exits.

In the initial three-record test, MCP `stats` reported three memories and zero
numeric-exit errors despite the two failed tool-status records. That counter
and `recent_errors` are not a complete inventory of Rho tool failures. Search
by the command/error marker or `agent:rho` when inspecting this evidence.

**`--no-save` is not a MemoryWhale capture-off switch.** Rho explicitly documents
that hooks and other components can retain their own data. The live unsaved
session confirmed this distinction. A user who does not want command capture
must remove/disable the separate MemoryWhale hook or use an appropriate capture
exclusion; do not infer privacy consent from session persistence mode.

To retain MCP while disabling capture, remove only the `memorywhale-record`
`after_tool_use` entry from the selected hooks file, preserving other hooks,
and reload `/hooks` to verify it is absent. `mw integrate rho --revert` instead
removes the broader integration. Removing either does not erase stored memory.

## Test setup and boundaries

The collector was a small reviewed program in the disposable project. It saved
bounded hook envelopes and passed after-tool events to the built `mw-remember
--from-hook rho`, with an explicitly isolated capture HOME and data directory.
A separate test-only pre-tool hook denied one marker command; it could only
make policy stricter. Native Rho used Auto mode with a configured permission
classifier, not a bypass mode. MCP exposed only read-only memory tools.

Normal HOME was retained for the client's existing OS-keychain authentication.
No credentials were extracted or copied. No global hook file or shared plugin
directory existed at the preflight check; no normal client configuration or
MemoryWhale store was edited. Normal user-level instructions could still load:
this was not an operating-system sandbox. Interactive checks used a terminal
PTY, not desktop/computer access.

Two harness details are recorded rather than hidden:

- An initial temporary Auto profile lacked a permission-classifier model and
  was rejected before a turn. The ordinary classifier was then configured in
  that temporary profile; permissions were not bypassed.
- The first terminal harness sent `/exit` and Enter in one burst. The model run
  completed, but the harness failed to submit the exit command and stopped its
  own process after timeout. That persisted session resumed successfully. The
  corrected harness sent text and Enter separately; resumed and unsaved sessions
  exited normally. A nonfatal attach-output collision warning also occurred in
  automation; direct result files and hook evidence were used instead.

No live power-shell or other-platform claim is made. No session-start context
injection or pre-compaction automation is inferred from observational hooks.

## Reusable regression evidence

Normalized live envelopes are in
[`tests/fixtures/rho-2.10`](../../crates/mw-cli/tests/fixtures/rho-2.10/README.md).
Commands, status, failure text, and bounds are retained; temporary paths and
identifiers are normalized. Fixtures never execute their embedded commands.

```bash
test_data="$(mktemp -d)"
MEMORYWHALE_DATA_DIR="$test_data" cargo test --locked -p memorywhale-cli --test rho_210_compatibility --test rho_integration
```

A future live recheck should use the same versioned scenarios, a reviewed
collector, explicit project-hook trust, isolated stores/profiles, and normal
authentication/approval. Report any unsupported case instead of buying access,
copying credentials, or weakening policy to make a test pass.

## Pinned primary references

- [Rho 2.10.0 release](https://github.com/matthewyjiang/rho/releases/tag/rho-coding-agent-v2.10.0)
- [2.10.0 session documentation](https://github.com/matthewyjiang/rho/blob/rho-coding-agent-v2.10.0/docs/sessions.md)
- [2.10.0 CLI validation](https://github.com/matthewyjiang/rho/blob/rho-coding-agent-v2.10.0/crates/rho/src/app/cli_config.rs)
- [2.10.0 path/discovery definitions](https://github.com/matthewyjiang/rho/blob/rho-coding-agent-v2.10.0/crates/rho/src/paths.rs)
- [Hook protocol](https://matthewyjiang.github.io/rho/hooks/protocol)
