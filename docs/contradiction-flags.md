# Contradiction flags

`mw contradictions <id> <id>` performs a conservative, local lexical check on two selected memory IDs. It reports both records and records a `pending` flag in SQLite. Matching is explicitly a heuristic (shared terms plus differing negation cues), not a semantic truth claim. No memory is overwritten or deleted. Review status can be `pending`, `confirmed`, or `rejected` in `contradiction_flags`; the original evidence remains available.
