---
name: Integration request
about: Add an MCP client / editor to integrations/
title: "Add a <tool> integration"
labels: documentation, good first issue
---

## Summary
<!-- Which tool, and confirm it speaks MCP (can run a local stdio server). -->
`<tool>` supports MCP servers, so it can use MemoryWhale's four tools
(`recent_errors`, `search_memory`, `get_context`, `remember`). Add
`integrations/<tool>/`.

## Why it's a good first issue
Docs + config only, mirroring an existing folder (e.g. `integrations/codex/`).
No Rust.

## What to do
1. Create `integrations/<tool>/` with a config snippet registering a stdio
   server whose command is `mw-mcp`.
2. **Verify the exact config format/location against the tool's current docs** —
   don't assume; formats change between versions.
3. Add a `README.md` mirroring an existing one: register the server, the
   `mw-mcp`-on-PATH note, `MEMORYWHALE_DATA_DIR` for a non-default DB, and a
   "when to use it" instruction for the tool's rules/system-prompt file.
4. List the tool in `integrations/README.md`.
