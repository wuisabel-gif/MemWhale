# OpenHands SDK integration verification

Checked September 26, 2026 against the OpenHands `software-agent-sdk` revision
`bd88f050259276978dc31541d8099d98e4994428`
(example [`examples/01_standalone_sdk/07_mcp_integration.py`](https://github.com/OpenHands/software-agent-sdk/blob/bd88f050259276978dc31541d8099d98e4994428/examples/01_standalone_sdk/07_mcp_integration.py)).
Result: **MemoryWhale transport PASS; native OpenHands SDK agent run pending.**

## Verified in this MemoryWhale PR

- The configuration matches the SDK's documented shape: `MCPServer` from
  `openhands.sdk.mcp`, a `mcp_config` **dict** of `name -> MCPServer` passed to
  `Agent(mcp_config=...)`, and a normal `Conversation`. The `MCPServer` model
  (`openhands.sdk.mcp.config`) has `command`, `args`, and
  `env: dict[str, SecretStr] | None` fields, so the store path is wrapped in
  `SecretStr` in the example.
- The six tool names match MemoryWhale's current MCP contract.
- The local `mw-mcp` transport was exercised directly against the freshly built
  `target/release/mw-mcp` (version **0.12.1**) with an isolated
  `MEMORYWHALE_DATA_DIR` (a `mktemp` directory), using a standard MCP handshake:
  `initialize` -> `notifications/initialized` -> `tools/list`.

  `initialize` response:

  ```json
  {"id":1,"jsonrpc":"2.0","result":{"capabilities":{"tools":{"listChanged":false}},"protocolVersion":"2025-11-25","serverInfo":{"name":"memorywhale","version":"0.12.1"}}}
  ```

  `tools/list` returned exactly the six tools: `recent_errors`, `search_memory`,
  `get_context`, `remember`, `similar_failures`, `stats`.

The transport check used a task-owned store path and did not read the normal
MemoryWhale store. It verified MemoryWhale directly, not the OpenHands SDK's MCP
client registry.

## Not yet verified

The OpenHands SDK was not installed or run in this PR. The following remain
pending and are intentionally not represented as passing capabilities:

- The SDK actually discovering the six tools through its MCP registry and exposing
  them under the `memorywhale` namespace.
- A scripted or local-model `Conversation` dispatching `search_memory` and an
  explicit `remember` whose proposed note preserves MemoryWhale's pending-note
  review flow.
- Fresh-session persistence and disconnect-while-usable behavior.
- Anything requiring container, VM, or remote Agent Server runtimes: whether
  `mw-mcp` and its store are reachable inside those runtimes is not tested here.
- Any automatic command/execution capture (the SDK exposes MCP tools only).

A live provider-backed test also requires user model credentials and is separate
from this local transport verification. Do not promote this report to a native
SDK integration claim until a pinned OpenHands build passes the native checks in
the guide.

## Source boundary

The OpenHands SDK example used for source inspection was pinned to the revision
above and was only read, not installed or modified. No OpenHands issue, pull
request, push, or credential access was performed. Only the MemoryWhale branch
and PR are in scope.
