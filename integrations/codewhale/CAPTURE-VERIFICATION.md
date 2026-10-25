# Codewhale capture: native verification

## Stock Codewhale 0.10.1: PASS in Full Access mode, October 24, 2026

The capture bundle works with an unpatched, released Codewhale. Codewhale 0.10.1
ships the post-admission execution receipt (Hmbown/Codewhale#6713), and its
`tool_call_after` stdin document carries the fields `mw-remember --from-hook
codewhale` parses. This is an integration test with a deterministic model, not
an LLM usefulness benchmark.

**The TUI mode decides whether anything is captured.** Codewhale builds a
receipt only for an unsandboxed, pipe-backed, local foreground run
(`!spawned.sandboxed` in `crates/tui/src/tools/shell.rs`). On macOS the Ask and
Auto modes run the shell inside Seatbelt, so the hook fires with empty stdin and
no `DEEPSEEK_TOOL_EXECUTION_RECEIPT`, and MemoryWhale correctly stores nothing.

| TUI mode | Shell sandboxed | Receipt | Captured |
| --- | --- | --- | --- |
| Ask (default) | yes | none | no |
| Auto | yes | none | no |
| Full Access | no | stdin document and environment variable | yes |

Setting `sandbox_mode = "danger-full-access"` in the config does not change Ask
or Auto; the session itself has to be in Full Access.

### Tested

- Codewhale **0.10.1** (`fead51eeea0f`), the official `codewhale-macos-arm64.tar.gz`
  release asset. The tarball matched `codewhale-artifacts-sha256.txt`; the binary
  SHA-256 is `e50fdb7dfbb39d7c01eb63175da01bc40f16fb958e01cbcbef03220460302654`,
  the published hash of the standalone `codewhale-macos-arm64` asset.
- MemoryWhale helpers built from main at `8ab0512`; this branch changes no Rust.
- macOS arm64, a real TUI in a PTY with a sealed profile, an empty synthetic Git
  workspace, a new synthetic store, and the loopback model fixture with a dummy
  key. `sandbox_mode = "danger-full-access"`, and the TUI switched to Full Access
  in both sessions with Shift+Tab. Full Access runs commands without approval
  prompts; the only commands were the fixture's fixed synthetic ones.
- Both bundles reviewed natively and trusted with the displayed hashes: capture
  `d5c49dfe5665e70db58020e1eae410dd7cd277d3a4c94337285839343fccb580.184e76c07510ad41740f2105dc2fc46cbaea8f8cf8407da63849f06ec4bb66d1`
  (one `tool_call_after` hook, no MCP, no filesystem roots or network hosts) and
  memory `150c40d4210a426d223a10e62878b7e6d4c5f1b1671043ff774f453a0740f2c4.6d0f62add294575eb4dce138c212540a2d124c91b25ffab8e86d9cbfde5fa516`.

### Observed (run `stock-0.10.1-run-6`)

| Check | Observed result |
| --- | --- |
| Failed execution | `printf MW_NATIVE_FAIL; printf MW_NATIVE_STDERR >&2; exit 7` stored with exit 7, the actual workspace cwd, agent `codewhale`, and the session/call identity. |
| Stream meaning | Both markers in a labeled combined stdout/stderr preview with `output_mode: combined`; stored stderr empty. |
| Successful execution | `printf MW_NATIVE_SUCCESS; pwd` stored with exit 0; the printed directory matched the stored cwd. |
| Fresh client | Session one exited normally; a new process started with `--fresh` in the same profile and store. |
| MCP retrieval | The fresh session's plugin search returned `[command #1]` and the failure marker. |
| Capture disablement | After `/plugin disable memorywhale-capture`, `printf MW_NATIVE_DISABLED` ran and its output reached the model; the store still held two records. |
| Retrieval after disablement | Another search still returned the earlier record. `/plugin list` showed `memorywhale` active, `memorywhale-capture` disabled, and the built-in `computer-use` disabled. |
| Checker | `check_evidence.py`: PASS. For a Full Access run it requires Full Access in both terminal recordings in place of approval stamps. |

### Attempts not counted

- Run 1 stopped when the driving session ended, before plugin review.
- Runs 2 and 3, in Ask mode with `workspace-write` and then `danger-full-access`:
  the commands ran, no receipt arrived, and no record was stored.
- Run 4, diagnostic: a logging `mw-remember` wrapper and a config-level probe hook
  showed the hook firing with empty stdin in Ask and Auto, and both receiving the
  820-byte receipt in Full Access.
- Run 5 passed the same steps before a documentation edit changed the capture
  bundle's README and content hash (`39038d1d…`); run 6 repeats it with the
  final bundle.

### Key SHA-256 receipts (run 6)

| Artifact | SHA-256 |
| --- | --- |
| Codewhale 0.10.1 binary | `e50fdb7dfbb39d7c01eb63175da01bc40f16fb958e01cbcbef03220460302654` |
| `mw-remember` | `4c4fcb6070797a9c623dbdd901b04b08acd808135fb2aad10982a1a692a4ae13` |
| `mw-mcp` | `effec2588828ca0569a9da4a6e53728b8a8461d3652d20a29c4a084a2f5c25e4` |
| Native driver | `d5f62e02a1bba60df5d87bf305b89473e96a4de54fb892d21bfafb3d1176b13a` |
| Model request evidence | `b6e4b6604c71dacbf523c9b1834604f0a53e4b9f11224c377bedaf4ac6307d62` |
| Model response evidence | `c684faf94945876b5a989736e98ad76993ae8f822aa65753a1b0baf83e2623ab` |

### Remaining boundaries

- Capture needs Full Access, which turns off the OS sandbox and approval
  prompts. Users who keep Ask or Auto get MCP retrieval but no capture.
- Only macOS was run. Linux with bubblewrap is expected to behave like macOS;
  Linux without an OS sandbox was not tested.
- The model was deterministic, only the model-facing `bash` tool was exercised,
  and headless, ACP, and Windows paths were not run.
- Background capture is bounded and can lose events on saturation or shutdown.

## Earlier: receipt-patched Codewhale 0.9.13, September 16, 2026

This was a local receipt-patched host test, before the receipt shipped upstream.
No Codewhale push, issue, PR, merge, or release was made.

### Tested versions and isolation

- Codewhale **0.9.13 (dev)**, base `6ae17de09a894f870835208d552f4ef946fe129a`,
  with the existing local execution-receipt patch plus the explicit output-mode
  correction. The original checkout remained untouched and clean.
- Host binary SHA-256:
  `21969a634bf2421746cae294d628717a651e18a08ae7d99832bd3ba89415d084`.
- MemoryWhale base `bcf3e320c3644383504de516c60e64891806758e`, with the
  exact-identity deduplication fix and regression tests described below, then
  local and since merged in #335.
- Native plugin bundles: `memorywhale` and `memorywhale-capture`, both 0.1.0.
- macOS arm64; a real Codewhale terminal process in a PTY, a fresh private
  profile, an empty synthetic Git workspace, and an explicitly selected new
  MemoryWhale store. The only model endpoint was the task-owned loopback
  Chat Completions fixture, using a clearly synthetic dummy key.
- The client kept normal **Ask / on-request** approval and **workspace-write**
  sandbox settings. Known synthetic tools were approved **once**, through the
  native prompt. No Full Access mode or permission bypass was used. The built-in
  computer-use plugin remained disabled; no desktop access was used.

### Actual native observations

| Check | Observed result |
| --- | --- |
| Plugin review and activation | Native review displayed content/capability hashes; the exact reviewed tokens were submitted through `/plugin trust`, followed by separate enablement. |
| Capture-only component | One `tool_call_after` observer, background execution, five-second configured timeout; no before-hook verdict and no other test capture hook. |
| Failed execution | Native `bash` ran `printf MW_NATIVE_FAIL; printf MW_NATIVE_STDERR >&2; exit 7`; the database recorded the exact command, actual launch workspace, and exit 7. |
| Stream meaning | Both markers appeared in an explicitly labeled **combined stdout/stderr preview** with `output_mode: combined`; empty stored stderr was not presented as proof of no stderr. |
| Successful execution | Native `bash` ran `printf MW_NATIVE_SUCCESS; pwd`; exit 0 and the printed directory agreed with the captured launch cwd. |
| Fresh client | Session one exited normally. A different native process started with `--fresh`, the same isolated store, and no inherited failure conversation. |
| Actual MCP retrieval | The fresh client first loaded the deferred tool without executing it, retried with the advertised schema, received native approval, and obtained `[command #1]` and the failure marker from the plugin-owned MemoryWhale server. Deferred loading alone was not counted as retrieval. |
| Capture disablement | `/plugin disable memorywhale-capture` succeeded. A newly approved `printf MW_NATIVE_DISABLED` actually ran, but the database still contained only the original two records. |
| Retrieval after disablement | Another approved MCP search still returned the earlier record. The final plugin list showed `memorywhale` active and `memorywhale-capture` disabled. |
| Shutdown | Both test sessions were stopped through normal `/exit`; the fixture server was stopped afterward. |

The passing run was `native-run-3`. Earlier attempts are not counted: one
exposed terminal paste/submission timing and the removed model-facing
`exec_shell` name; another used an unnecessary MCP argument. The final harness
separates text from Return, uses the advertised `bash` interface, and submits
only supported `query`/`agent` search arguments.

### Defects corrected during verification

1. **Host/adapter mismatch:** the adapter correctly required `output_mode`, but
   the unpublished host patch omitted it. The local host now derives
   `separate`, `combined`, or `unavailable` from actual job capture buffers and
   ownership, not from requested command arguments. Its typed receipt requires
   this discriminator, including the synthetic observer fixture.
2. **Identity-prefix loss:** the earlier `LIKE` lookup could suppress `call_1`
   after `call_10`, or match identity-shaped text inside metadata. The
   MemoryWhale fix (#335) compares the complete anchored, hex-encoded identity with a
   trailing delimiter under the existing immediate transaction. Regressions
   cover prefix IDs, whitespace/wildcards, metadata-shaped IDs, and concurrent
   replay. Same-identity replay remains suppressed; distinct executions survive.

### Evidence and reproduction

The reusable local driver and evidence checker are in
[`scripts/verification/codewhale-capture/`](../../scripts/verification/codewhale-capture/README.md).
They do not generate the database records or substitute an independent MCP
client for Codewhale. The native host performs command execution, invokes the
reviewed capture hook, and dispatches the approved MCP search itself.

The local, untracked `.verification-codewhale-capture/native-run-3/` artifact
set contains `launch.json`, terminal recordings, native model-protocol requests
and responses, action records, the synthetic database, `source-state.json`, and
`verification-result.json`. Source-state hashes include untracked receipt
modules as well as the tracked patch; the base commit alone is not the tested
host. These synthetic artifacts are not automatically committed or uploaded.

Key SHA-256 receipts:

| Artifact | SHA-256 |
| --- | --- |
| `mw-remember` | `bf18d4a427988a8b73f9a5104e6f54fe317146993a0c92edb6f20ebdd3c3e722` |
| `mw-mcp` | `116e678e6b327ca8305f86f9b369a995a696860b7d65d1e5aac9a04d49bef8e7` |
| Native driver | `4171522cebdc9baaaf3e00c3f9f1013b8d8e21f051ed44950affb4a573d4cd95` |
| Model request evidence | `fe136e9b0476809db571700e9b95e16666afde0f313133efc0499325bbccaff3` |
| Model response evidence | `4873c1ac657a56058a5d60ae365d83bf0df6318701fd028c647cb6b95ed0777a` |

Hashes identify this local run; they are **not reusable plugin trust tokens**.
Any later bundle change requires a fresh native review.

### Remaining boundaries

- The model was deterministic. No claim is made that a live LLM selects useful
  memories or follows the guidance well.
- Only the current model-facing `bash` path was exercised through the TUI.
  Legacy producer aliases and separate/unavailable stream modes require their
  focused host tests; they are not additional live-TUI passes.
- This run confirms the launch cwd, not cwd changes inside arbitrary shell
  scripts. Rewritten inputs, cancellation, timeouts, truncation, and unsupported
  backends are covered separately where a focused test exists.
- Headless/ACP/workflow internals, native Windows, real provider accounts, and
  remote sandboxes were not exercised. Unsupported or unconfirmed execution
  paths still omit receipts.
- Background capture is bounded and can lose events on saturation or shutdown;
  this is not a guarantee that every execution will be captured.
- The host changes remain local. No upstream acceptance, release compatibility,
  marketplace installation, or security-policy enforcement follows from this
  result.
