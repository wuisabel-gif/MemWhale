# Qwen Code integration verification

Checked September 26, 2026 against Qwen Code's MCP documentation at commit
`4ddbf227e898ceb322116af5e2004c37a51184f6`, with the issue #295 pinned contract
(`b8def02aadfc384ecb860909155d196267c4fa0c`, September 16, 2026) re-verified
against it. Result: **MCP contract and MemoryWhale transport PASS; native Qwen
Code client call pending.**

## Verified in this MemoryWhale PR

- The configuration uses Qwen Code's documented `mcpServers` object, local stdio
  `command`/`args`/`cwd`/`env`, the `includeTools` allowlist, `timeout`, and
  `trust` fields.
- The six names match MemoryWhale's current MCP tool contract.
- The local `mw-mcp` discovery request against a freshly built
  `target/release/mw-mcp` returned server `memorywhale`, version `0.12.1`, a
  `tools` capability, and supported protocol revisions `2026-07-28`,
  `2025-11-25`, and `2024-11-05`. A `tools/list` call advertised exactly the six
  tools.
- The guide and example config contain no user-home paths, provider keys, or
  real store paths.

The transport check used a task-owned store path
(`MEMORYWHALE_DATA_DIR="$(mktemp -d)"`) and did not read the normal MemoryWhale
store. It verified MemoryWhale directly, not Qwen Code's client registry.

### Recorded transport output

Command:

```bash
STORE="$(mktemp -d)"
printf '%s\n%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"qwen-verify","version":"1"},"io.modelcontextprotocol/clientCapabilities":{}}}}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}' \
  | MEMORYWHALE_DATA_DIR="$STORE" target/release/mw-mcp
```

`server/discover` response (`id` 1):

```json
{"id":1,"jsonrpc":"2.0","result":{"_meta":{"io.modelcontextprotocol/serverInfo":{"name":"memorywhale","version":"0.12.1"}},"cacheScope":"public","capabilities":{"tools":{"listChanged":false}},"instructions":"MemoryWhale provides local development memory retrieval and explicit note storage.","resultType":"complete","supportedVersions":["2026-07-28","2025-11-25","2024-11-05"],"ttlMs":3600000}}
```

`tools/list` response (`id` 2) advertised exactly six tools:
`recent_errors`, `search_memory`, `get_context`, `remember`,
`similar_failures`, `stats`.

## Not yet verified

A Qwen Code binary was not installed or run in this PR. The following remain
pending and are intentionally not represented as passing capabilities:

- Native discovery and all six tools appearing in a Qwen Code session
  (`qwen mcp list`).
- A real Qwen Code model turn dispatching `search_memory` and `remember`.
- Fresh-session persistence, pending-note review behavior, and disconnect while
  Qwen Code remains usable.
- Safe removal via `qwen mcp remove memorywhale` leaving both stores intact.
- Progressive interactive vs non-interactive discovery timing on a live client.
- Any lifecycle hook or automatic command capture.

A live provider-backed test would also require user credentials and is separate
from this local transport verification. Do not promote this report to a native
client integration claim until a pinned Qwen Code build passes the native checks
in the guide.

## Contract notes

Qwen Code names the per-server tool allowlist `includeTools` (with an
`excludeTools` blocklist), not the `enabledTools` field used by some other
clients. It shares MCP-config ancestry with the Gemini CLI; the setup here was
written and verified against Qwen Code's own documentation rather than copied
from the Gemini guide.

## Source boundary

The Qwen Code documentation used for inspection was read at the commit above and
was not modified. No Qwen Code issue, pull request, push, source change,
installation in the user's profile, or credential access was performed. Only the
MemoryWhale branch and PR are in scope.
