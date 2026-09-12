# MemoryWhale native Codewhale bundle

Development bundle version 0.1.0 for Codewhale's declarative Agent Plugins
format. It contributes one stdio MCP server, the `memorywhale-evidence` skill,
and explicit check/recall guidance commands. It contains **no capture hook,
automatic recall hook, native-code extension, or provider credentials**.

Use the [setup and removal guide](../README.md). This directory is a complete
source bundle; it is not an already-published marketplace package or something
installed automatically by `cargo install`.

## Store and process boundary

Set an absolute `MEMORYWHALE_DATA_DIR` in the environment that starts Codewhale.
The plugin declares the exact `${MEMORYWHALE_DATA_DIR}` source mapping. Its
reviewed shell entrypoint refuses a missing, empty, or relative selection
instead of silently opening the default store. `mw-mcp` must already be on
Codewhale's trusted PATH; nothing is downloaded by this bundle.

The MCP child has the host user's OS authority; plugin trust is not a sandbox.
The mapped store may contain sensitive records, and selected results can reach
the model provider. Test first with a disposable synthetic store and no other
private data sources in the client profile. Keep Codewhale preference memory
separate from MemoryWhale debugging evidence.

## Trust and scope

Review the full bundle and the exact content/capability receipt shown by the
host. Trust stages the reviewed bytes; enablement is a separate action.
Disable or revoke through the native plugin manager. Preserve existing bundles
and configuration; new user plugin-state directories must be owner-only
(0700 on Unix), as required by Codewhale's trust-state guard.

The commands and skill are guidance, not an approval bypass or a hard policy
engine. MCP tool approval remains owned by Codewhale. Updating bundle contents
requires another host review; do not reuse an old receipt.

## Verification

The local Codewhale 0.9.12 host at `d5beab040` exercised this bundle through a
real terminal PTY: validation, hash-bound trust, enablement, skill activation,
actual MCP retrieval of a synthetic marker, disablement, and normal exit.
A deterministic loopback model-protocol fixture drove the tool call. This is
**native-client integration evidence, not a live-LLM usefulness benchmark**.
No real model key, normal memory store, or desktop access was used.

See [the verification record](../VERIFICATION.md). Offline checks run with:

```bash
cargo build --locked -p memorywhale-cli --bins
python3 scripts/test-codewhale-plugin.py --bin-dir target/debug
```

These offline checks validate bundle assets, shell parsing, explicit store
selection, and a real local MCP round-trip. They do not substitute for the
native host's plugin review/activation checks.
