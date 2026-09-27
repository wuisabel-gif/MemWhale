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

Feedback references memories by this store's row IDs. `mw import`, `mw pull`,
and `mw push` transfer it anyway (along with `mw link` edges): as they merge the
memory tables they build an old-to-new row-id map, then remap each feedback
record and link to the destination IDs before inserting. A record whose target
memory did not import (for example a desktop-only document or a duplicate that
was filtered) is skipped and counted; re-import is idempotent. Those commands
report how many records and links they imported and skipped. `mw export` still
includes the table in its raw SQLite snapshot.
