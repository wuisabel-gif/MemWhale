# Retrieval feedback

Feedback is attached to the namespaced memory IDs printed by `mw search`.
It is a displayed signal only: recording feedback does not delete memories,
change retrieval ranking, or rewrite provenance.

```text
mw feedback add <memory-id> helpful|irrelevant|outdated|contradicted [--actor NAME]
mw feedback list [<memory-id>]
mw feedback show <feedback-id>
mw feedback undo <feedback-id>
```

`undo` preserves the feedback row and records an undo timestamp. The database
migration creates `retrieval_feedback` with the memory ID, kind, timestamp,
optional actor, optional source session, and undo timestamp.

`add` rejects IDs that do not resolve to a current memory, and `undo` fails if
the feedback does not exist or was already undone.

Feedback is machine-local, like `mw link` edges: it references this store's row
IDs, which `mw import`, `mw pull`, and `mw push` do not preserve. Those commands
report how many feedback records they skipped instead of copying them. `mw
export` still includes the table in its raw SQLite snapshot.
