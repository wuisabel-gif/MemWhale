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
