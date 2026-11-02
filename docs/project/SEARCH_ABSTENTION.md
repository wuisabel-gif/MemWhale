# Search abstention

The built-in engine exposes `retrieve_matches`: for nonempty text queries a row
must have a positive content-similarity signal. Recency, importance, reinforcement
or project tags in the task context cannot by themselves make a result relevant.
Filtering happens before the requested result limit. Empty-text browsing remains
unchanged. CLI and MCP explain an empty result as insufficient evidence; HTTP,
TUI and desktop search use the same selection method and return empty results.

This removes zero-content matches; it is not a calibrated confidence threshold.
A generic shared word can still match. Optional external MemPalace retrieval is
unchanged, and semantic similarity requires separate threshold evaluation.

