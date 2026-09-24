# Contradiction flags

`mw contradictions <id> <id>` performs a conservative, local lexical check on two selected memory IDs. It reports both records and records a `pending` flag in SQLite. Matching is explicitly a heuristic (at least two shared distinct terms plus differing negation cues), not a semantic truth claim, and the result does not depend on argument order. No memory is overwritten or deleted.

Review stored flags with `mw contradictions list`, then `mw contradictions confirm <flag-id>` or `mw contradictions reject <flag-id>`. Review sets the flag's `status` (`pending`, `confirmed`, or `rejected`) and `reviewed_at` in `contradiction_flags`; the original evidence remains available.
