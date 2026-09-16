# Codewhale capture: local native verification

**Native integration result: PASS — September 16, 2026.** This is a local
receipt-patched host test, not stock/released Codewhale support or an LLM
usefulness benchmark. No Codewhale push, issue, PR, merge, or release was made.

## Tested versions and isolation

- Codewhale **0.9.13 (dev)**, base `6ae17de09a894f870835208d552f4ef946fe129a`,
  with the existing local execution-receipt patch plus the explicit output-mode
  correction. The original checkout remained untouched and clean.
- Host binary SHA-256:
  `21969a634bf2421746cae294d628717a651e18a08ae7d99832bd3ba89415d084`.
- MemoryWhale base `bcf3e320c3644383504de516c60e64891806758e`, with the local
  exact-identity deduplication fix and regression tests described below.
- Native plugin bundles: `memorywhale` and `memorywhale-capture`, both 0.1.0.
- macOS arm64; a real Codewhale terminal process in a PTY, a fresh private
  profile, an empty synthetic Git workspace, and an explicitly selected new
  MemoryWhale store. The only model endpoint was the task-owned loopback
  Chat Completions fixture, using a clearly synthetic dummy key.
- The client kept normal **Ask / on-request** approval and **workspace-write**
  sandbox settings. Known synthetic tools were approved **once**, through the
  native prompt. No Full Access mode or permission bypass was used. The built-in
  computer-use plugin remained disabled; no desktop access was used.

## Actual native observations

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

## Defects corrected during verification

1. **Host/adapter mismatch:** the adapter correctly required `output_mode`, but
   the unpublished host patch omitted it. The local host now derives
   `separate`, `combined`, or `unavailable` from actual job capture buffers and
   ownership, not from requested command arguments. Its typed receipt requires
   this discriminator, including the synthetic observer fixture.
2. **Identity-prefix loss:** the earlier `LIKE` lookup could suppress `call_1`
   after `call_10`, or match identity-shaped text inside metadata. The local
   MemoryWhale fix compares the complete anchored, hex-encoded identity with a
   trailing delimiter under the existing immediate transaction. Regressions
   cover prefix IDs, whitespace/wildcards, metadata-shaped IDs, and concurrent
   replay. Same-identity replay remains suppressed; distinct executions survive.

## Evidence and reproduction

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

## Remaining boundaries

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
