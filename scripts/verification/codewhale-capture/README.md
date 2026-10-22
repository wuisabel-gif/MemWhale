# Local native capture verification

This driver runs a **real Codewhale TUI** in its own PTY with a sealed test
profile and a deterministic loopback model-protocol server. It is not a desktop
controller, a live-provider test, or a replacement for the host's approvals.

Prerequisites: Python 3 on macOS/Linux, matching built MemoryWhale helpers, and
an explicitly selected **receipt-patched** Codewhale CLI binary. Stock legacy
hosts do not emit the required receipt. Build the pinned host with its repository
instructions and keep those local changes separate; these scripts do not fetch,
install, publish, or patch Codewhale.

## Start

Run from the MemoryWhale root, selecting an unused task-specific directory:

```bash
python3 scripts/verification/codewhale-capture/native_fixture.py serve \
  --scratch "$PWD/.verification-codewhale-capture/my-run" \
  --host /absolute/path/to/receipt-patched/codewhale \
  --bin-dir "$PWD/target/debug"
```

Leave this process running. It creates a new isolated Git workspace, HOME,
configuration, cache, and MemoryWhale store, and copies the two source bundles
into that **test profile only**. It does not grant trust or enable either bundle.
The normal user profile/store is not selected. The server binds only loopback;
its control endpoint has no authentication and must never be exposed or reused
with a real profile or sensitive data. Stop it when the test finishes.

Use another terminal for control. Every command below needs the same explicit
`--scratch` path. `send` separates typed text from Return to avoid the native
paste detector consuming submission. `keys` writes only to this test child's
PTY, never to the system desktop.

```bash
python3 scripts/verification/codewhale-capture/native_fixture.py send \
  --scratch "$PWD/.verification-codewhale-capture/my-run" \
  --text '/plugin enable memorywhale-capture'
```

Review the native content/capability hashes and process authority. Close the
review pager with `keys --text q`; submit the exact displayed
`/plugin trust memorywhale-capture <content>.<capability>` token, then enable the
bundle. Do not copy a token from a prior run or bypass native trust storage.

## Exercise the boundary

1. Send `CAPTURE_FAIL`. The fixture requests only the advertised native `bash`
   call `printf MW_NATIVE_FAIL; printf MW_NATIVE_STDERR >&2; exit 7`.
2. Read the normal approval prompt; choose **Allow once** using `keys --text y`
   only if the displayed command is exactly the intended synthetic command.
3. Send `CAPTURE_SUCCESS`, inspect/approve its exact `printf MW_NATIVE_SUCCESS;
   pwd` call, and confirm both completed executions reached the synthetic store.
4. Review, trust, and enable the **separate** `memorywhale` MCP bundle through
   the same native UI flow.
5. Use `stop` to submit `/exit`. After normal exit, `restart` starts a different
   process with `--fresh` in the same isolated profile/store.
6. Send `RECALL_CAPTURE`. Deferred tool loading is **not** execution. Allow the
   subsequent native MCP search once after inspecting its synthetic query.
7. Send `/plugin disable memorywhale-capture`, then `CAPTURE_DISABLED`. Inspect
   and approve `printf MW_NATIVE_DISABLED` once. It must actually execute,
   without creating a third memory record.
8. Send `RECALL_CAPTURE` again and inspect/approve the search. Retrieval must
   still work while capture remains disabled.
9. Send `/plugin list`; confirm memory access is active, capture is disabled,
   and the built-in computer-use plugin remains disabled. Use `stop` for normal
   `/exit`, then terminate only the task-owned fixture server.

`screen` returns accumulated terminal output; raw and stripped recordings are
also saved in the chosen scratch. This is not a terminal emulator screenshot.
If a step fails, retain that attempt as failure evidence rather than silently
editing stored records or treating a later partial result as a full pass.

## Check the result

```bash
python3 scripts/verification/codewhale-capture/check_evidence.py \
  --scratch "$PWD/.verification-codewhale-capture/my-run"
```

The checker opens the database read-only and checks native tool-result messages,
not just marker text echoed in a request. It requires distinct native processes,
actual exit/cwd evidence, a fresh-session MCP result, execution of the disabled
command without a corresponding row, and post-disablement retrieval. It writes
`verification-result.json` only after all checks pass.

The fixture records binary/plugin hashes, actions, and local protocol messages.
Keep those synthetic artifacts local unless deliberately reviewed for sharing.
A passing run demonstrates the integration of the pinned local binaries; it
says nothing about natural-language model judgment, other host versions,
headless/ACP support, or capture of every possible execution path.
