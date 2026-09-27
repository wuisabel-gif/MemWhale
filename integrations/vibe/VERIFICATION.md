# Mistral Vibe integration verification

Checked September 26, 2026 against the Mistral Vibe README at revision
`7c19608af06f6c61d63f8f7a5c3430da73fba2ab`. Result: **MCP contract and
MemoryWhale transport PASS; native Mistral Vibe client call pending.**

## Verified in this PR

- The configuration uses Vibe's documented `[[mcp_servers]]` block with
  `name`, `transport = "stdio"`, `command`, `args`, `env`,
  `startup_timeout_sec`/`tool_timeout_sec`, and per-tool
  `[tools.memorywhale_*]` permissions. Vibe names MCP tools
  `{server_name}_{tool_name}`, matching the `memorywhale_*` names in the guide.
- The six tool names match MemoryWhale's current MCP tool contract.
- A real transport handshake against the freshly built
  `target/release/mw-mcp` (version 0.12.1) with an isolated
  `MEMORYWHALE_DATA_DIR` returned server `memorywhale`, version `0.12.1`, a
  `tools` capability, and negotiated protocol `2025-11-25`. `tools/list`
  returned all six tools: `recent_errors`, `search_memory`, `get_context`,
  `remember`, `similar_failures`, `stats`. Observed replies:

  ```json
  {"id":1,"jsonrpc":"2.0","result":{"capabilities":{"tools":{"listChanged":false}},"protocolVersion":"2025-11-25","serverInfo":{"name":"memorywhale","version":"0.12.1"}}}
  {"id":2,"jsonrpc":"2.0","result":{"tools":[{"name":"recent_errors",...},{"name":"search_memory",...},{"name":"get_context",...},{"name":"remember",...},{"name":"similar_failures",...},{"name":"stats",...}]}}
  ```

- The guide, example config, and optional skill contain no user-home paths,
  provider keys, or real store paths.

The transport check used a task-owned store path and did not read the normal
MemoryWhale store. It verified MemoryWhale directly, not Vibe's client registry.

## Not yet verified

Mistral Vibe was not installed or run in this PR. The following remain pending
and are intentionally not represented as passing capabilities:

- Native `/mcp` connection and all six tools appearing in a Vibe session.
- A real Vibe model turn dispatching `memorywhale_search_memory` and
  `memorywhale_remember`.
- Per-tool `[tools.memorywhale_*]` permission enforcement in a live session.
- Fresh-session persistence, pending-note review behavior, and disconnect while
  Vibe remains usable.
- Skill discovery from `.vibe/skills` (and the Agent Skills `.agents/skills`
  paths) through the selected Vibe release.
- Any lifecycle hook or automatic command capture.

A live provider-backed test would also require user credentials and is separate
from this local transport verification. Do not promote this report to a native
client integration claim until a pinned Vibe build passes the native checks in
the guide.

## Source boundary

The Vibe README used for the contract was pinned to the revision above and only
read. No Vibe issue, pull request, push, source change, installation in the
user's profile, or credential access was performed. Only the MemoryWhale branch
and PR are in scope.
