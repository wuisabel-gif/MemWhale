---
name: Integration request
about: Add an MCP client / editor to integrations/
title: "Add a <tool> integration"
labels: documentation, good first issue
---

## Summary
<!-- Which tool, and confirm it speaks MCP (can run a local stdio server). -->
`<tool>` supports MCP servers, so it can use MemoryWhale's six tools
(`recent_errors`, `search_memory`, `get_context`, `remember`,
`similar_failures`, `stats`). Add
`integrations/<tool>/`.

## Why it's a good first issue
Docs + config only, mirroring an existing folder (e.g. `integrations/codex/`).
No Rust.

## What to do
1. Create `integrations/<tool>/` with a config snippet registering a stdio
   server whose command is `mw-mcp`.
2. **Verify the exact config format/location against the tool's current docs** —
   don't assume; formats change between versions.
3. Add a `README.md` using `integrations/TEMPLATE.md`: declare verified
   capabilities, register the server, include the `mw-mcp`-on-PATH note,
   document `MEMORYWHALE_DATA_DIR` for a non-default DB, and distinguish MCP
   access from automatic execution capture.
4. List the tool in `integrations/README.md`.
