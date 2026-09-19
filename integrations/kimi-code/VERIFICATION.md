# Kimi Code integration verification

Checked September 16, 2026 against Kimi Code revision
`bd06178913d2cc4ebd229ecee6714081105e5ae1`. Result: **MCP contract and
MemoryWhale transport PASS; native Kimi Code client call pending.**

## Verified in this MemoryWhale PR

- The configuration uses Kimi Code's documented `mcpServers` object, local
  stdio `command`/`args`/`env`, `enabledTools`, and timeout fields.
- The six names match MemoryWhale's current MCP tool contract.
- The local `mw-mcp` discovery request returned server `memorywhale`, version
  `0.10.0`, a tools capability, and supported protocol revisions including
  `2026-07-28`.
- The guide and optional skill contain no user-home paths, provider keys, or
  real store paths. The asset test passed five cases, documentation-reference
  checks passed, and the working tree diff was clean.

The transport check used a task-owned store path and did not read the normal
MemoryWhale store. It verified MemoryWhale directly, not Kimi Code's client
registry.

## Not yet verified

A Kimi Code binary was not installed or run in this PR. Therefore the following
remain pending and are intentionally not represented as passing capabilities:

- Native `/mcp` connection and all six tools appearing in a Kimi Code session.
- A real Kimi Code model turn dispatching `search_memory` and `remember`.
- Fresh-session persistence, pending-note review behavior, and disconnect while
  Kimi Code remains usable.
- Skill discovery through the selected Kimi Code release.
- Headless and ACP surfaces.
- Any lifecycle hook or automatic command capture.

A live provider-backed test would also require user credentials and is separate
from this local transport verification. Do not promote this report to a native
client integration claim until a pinned Kimi Code build passes the native checks
in the guide.

## Source boundary

The Kimi Code checkout used for source inspection was pinned to the revision
above and remained unmodified. No Kimi Code issue, pull request, push, source
change, installation in the user's profile, or credential access was performed.
Only the MemoryWhale branch and PR are in scope.
