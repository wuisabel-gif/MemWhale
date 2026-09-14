# Optional Codewhale execution capture

This is a **separate capture-only bundle**. The base `memorywhale` plugin remains
MCP/guidance-only; enabling it does not enable this bundle or change native
Codewhale preferences. No new `mw integrate codewhale` command is introduced.

## Required host contract

Requires the separately reviewed Codewhale **post-admission execution receipt**
change. Stock hosts that emit only legacy `DEEPSEEK_*` environment fields do not
supply sufficient evidence and are not supported by this adapter. Installation
or a plugin-enabled flag is not proof that a host produces receipts.

The adapter consumes version-1 JSON on stdin for `tool_call_after`, restricted
to native `exec_shell`, `Bash`, and `bash` aliases. It never joins before-hook
arguments with results: another hook may have changed those arguments before
execution. Receipt command/cwd must come from the actual native execution.

Supported final receipt states are `completed`, `failed`, `killed`, and
`timed_out`, with `execution: started`. A genuinely reported signed 64-bit exit
is stored as supplied; otherwise SQL NULL remains unknown. A running job is not
a completion and is skipped. Absent receipts do not prove that nothing ran.

The host may omit receipts for denial, spawn failure, cancellation before a
receipt can be returned, external sandboxes, untracked execution paths, or
read-only lanes that internally rewrite command arguments. These cases are
**not** claimed as captured. Headless/ACP/workflow internals are not covered by
this TUI integration.

## Explicit enablement

Build all MemoryWhale helpers/readers together from the implementation checkout:

```bash
cargo build --release --locked -p memorywhale-cli --bins
```

Use those matching binaries on the PATH seen by Codewhale. Older readers may
reject the new structured `codewhale` provenance value. First test with an
isolated client profile and a new synthetic `MEMORYWHALE_DATA_DIR`, not a normal
store. The selected directory must be absolute; missing/relative selection
causes capture to skip without opening a default store.

Follow the base plugin's native installation/review/trust procedure, using this
directory as the source and the distinct name **`memorywhale-capture`**. Review
the actual content/capability hashes shown by the host; never reuse a published
example hash. This bundle starts `mw-remember --from-hook codewhale` from the
trusted host PATH, with the host user's process authority.

**The bundle does not change Codewhale's global hooks switch.** If hooks are
disabled, capture remains inactive even when the plugin is enabled. Inspect
`/hooks` and review existing global/project/plugin hooks before choosing to
enable that global switch yourself; doing so can enable more than MemoryWhale.
Do not change an unrelated hook or approval policy to make a smoke test pass.
The component's own `enabled = true` only permits its entry to load; it does not
overwrite the host's global configuration.

Only `tool_call_after` is subscribed. It is a background observer and cannot
steer the tool call. There is no before-hook approval verdict, `on_error`
duplicate subscription, prompt rewriting, automatic recall, or restart/continue
action. The observer has a five-second configured timeout;
Codewhale's global timeout override can replace it. Check the effective host
configuration rather than assuming the file alone defines runtime behavior.

## Storage, bounds, and privacy

- Known effective cwd must be absolute and available locally. Missing or
  truncated command/cwd skips the event; workspace/process cwd is never a
  substitute. Shared per-directory exclusions and commands-only capture apply.
- Read only the versioned stdin receipt and the required store selection.
  Legacy hook environment fields are not treated as execution evidence. The
  helper does not dump inherited environment variables or provider credentials.
- Input is bounded to 128 KiB. Command/cwd identities are accepted up to 4096
  UTF-8 bytes; correlation IDs up to 256 bytes. Truncated identities are skipped.
- Separate streams remain separate. Combined terminal output is explicitly
  labeled as a combined preview in the stored stdout field; empty stderr then
  does **not** mean the process emitted no stderr. Unavailable output stays
  unavailable. Host/local truncation markers are retained.
- Normal hook processing is silent and nonfatal. Explicit
  `MEMORYWHALE_HOOK_DIAGNOSTICS=1` enables fixed diagnostics, not payload dumps.
- Different executions of the same command remain separate. There is no global
  exactly-once delivery promise; manually replaying the same receipt can create
  another record. Session/call IDs remain available for evidence correlation.
- Captured content can contain secrets despite redaction. The store remains
  local, but retrieved excerpts may be sent to a model provider by a client.
  Session transcript retention is not a capture-consent signal.

## Disable and remove

Use Codewhale's native `/plugin disable memorywhale-capture` or revoke the
bundle's trust. This leaves the base memory-access plugin and stored records
alone. Native authority checks prevent later queued launches after revocation;
an already-running capture process may finish. Inspect `/hooks` and verify a
new synthetic command does not add a record while MCP retrieval still works.

Remove only this bundle through the host's normal plugin lifecycle. Do not
remove unrelated hooks, native memory, or the MemoryWhale database. No global
installation or marketplace publication is performed by this source bundle.

## Verification boundary

The implementation must be validated against a receipt-capable pinned host.
Offline parser/subprocess tests are not a substitute for native plugin review,
activation, actual command execution, and fresh-client retrieval. The recorded
host/adapter verification and prerequisite PR links belong in the main
Codewhale integration verification report before release.
