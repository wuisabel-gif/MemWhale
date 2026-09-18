# Retrieval

Retrieval turns stored development evidence into useful context. MemoryWhale
supports text search, recent failures, compact context, similar-failure lookup,
and remembered lessons.

Retrieval should remain explainable: a result should lead back to the command,
output, or lesson that caused it to rank. Finding evidence is not the same as
solving a new problem; agents and people remain responsible for checking that a
past fix still applies.

The same retrieval capabilities appear through the CLI, TUI, web and desktop
views, and the MCP interface.

## Explain mode

`mw search <query> --explain` keeps the normal ranked result lines and adds the
scorer's full signal breakdown. The MCP `search_memory` tool accepts the
backward-compatible boolean `explain: true` for the same audit details. Signal
details identify the available evidence (for example keyword fields and
matched terms, recency, importance, reinforcement, or task tags), their
weights, and their numeric contributions. An inapplicable signal is explicitly
marked rather than treated as evidence. Explain output also reports whether a
display snippet was truncated and whether note provenance was available;
unknown status is not inferred as a positive match or causal explanation.
