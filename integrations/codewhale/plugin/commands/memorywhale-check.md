---
name: memorywhale-check
description: Explicitly check the reviewed MemoryWhale MCP connection; this does not enable capture or write memory.
---

Inspect the discovered MemoryWhale plugin server identity and call its `stats`
tool only after confirming the user intends to access the configured store.
Report connectivity, the selected store, and the returned counters without
claiming that configuration alone proves retrieval or capture. Numeric-exit
error counts do not include every tool failure whose exit is unknown.

Do not save a note, execute shell commands, connect an unrelated memory server,
or change plugin trust/enablement. Keep Codewhale native memory distinct.
