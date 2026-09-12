# Codewhale native plugin verification

Checked September 12, 2026 for issue #281.

## Versions and isolation

- Host: Codewhale **0.9.12 (dev)**, source `d5beab040`, built with the locked Cargo dependency graph on macOS arm64. The source checkout and its branch were not changed.
- Helpers: the MemoryWhale development checkout based on `aef1682`, using matching local `mw` and `mw-mcp` binaries.
- Bundle: native declarative `memorywhale` 0.1.0.
- The terminal test used a fresh HOME, Codewhale configuration, workspace, cache, and MemoryWhale store. Plugin trust-state directories were private (0700).
- A test-owned **loopback HTTP model-protocol fixture** returned deterministic requests/responses. No real model credentials, provider account, normal client configuration, normal memory store, or desktop access was used.

## Native-client results

| Check | Observed result |
| --- | --- |
| Discovery/validation | Native `/plugin validate memorywhale` reported a valid bundle: skill, stdio MCP, commands, and no hooks/native-code extension. |
| Inspection boundary | Validation/inspection did not create plugin trust state. |
| Review/trust | The TUI displayed full content/capability hashes and local process authority. The exact reviewed token was submitted through the normal trust command. |
| Enablement | Explicit `/plugin enable memorywhale` activated the reviewed bundle. |
| Skill loading | `$memorywhale:memorywhale-evidence` produced the native skill-activation confirmation. |
| Tool delivery | The host first reported a deferred tool load without execution. The fixture then retried with a distinct call ID; this was not counted as a successful retrieval. |
| MCP retrieval | After the normal MCP approval interaction, the actual plugin-owned server returned `MW_CODEWHALE_NATIVE_RECALL_MARKER` from the synthetic store into the host's tool-result conversation. |
| Disablement | Native `/plugin disable memorywhale` succeeded and durable plugin state recorded `enabled: false`. |
| Shutdown | The successful run exited normally through `/exit`. |

The reviewed bundle content hash in the final base-bundle run was
`150c40d4210a426d223a10e62878b7e6d4c5f1b1671043ff774f453a0740f2c4`.
This is a verification receipt, **not a reusable trust token**. Any later bundle
change needs the host's fresh content/capability review.

## What this does and does not prove

This exercises the real Codewhale binary, plugin authority/activation path,
skill loader, and MCP dispatch—not merely an independent MCP client. The model
side was deterministic, so it does not prove a natural-language agent will use
the guidance well or that another Codewhale release has the same behavior.

Initial checks exposed two real host requirements: the standard manifest needs
its `$schema` field, and user plugin-state directories must be owner-only.
The bundle/guide and isolated fixture were corrected accordingly. No host
security check was disabled. Failed terminal-harness attempts are not counted
as successful lifecycle verification.

Capture, automatic recall, lesson workflows, native Windows, and headless/ACP
parity are separate work. This base bundle does not enable those capabilities.

## Repeatable offline checks

```bash
cargo build --locked -p memorywhale-cli --bins
python3 scripts/test-codewhale-plugin.py --bin-dir target/debug
```

CI runs these asset/wrapper checks after CLI tests. They verify strict JSON,
explicit store selection, shell syntax, rejection of missing/relative store
settings, and a fresh isolated MemoryWhale MCP round-trip. Native client trust
and activation must still be exercised separately with a pinned host and a
reviewed synthetic fixture; do not copy credentials or weaken permissions to
turn an unavailable native check into an apparent pass.
