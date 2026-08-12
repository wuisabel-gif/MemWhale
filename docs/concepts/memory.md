# Memory

MemoryWhale stores two related forms of developer memory:

- **Evidence:** observed commands, arguments, output, exit status, working
  directory, timestamps, and transcripts.
- **Lessons:** conclusions or fixes saved by a person or an agent, with
  provenance and review state where applicable.

Local SQLite is the durable source of truth. Search indexes, views, graphs, and
agent responses are interfaces over that stored evidence; they should not erase
the distinction between an observed event and a later conclusion.

See the [storage reference](../reference/storage.md) for locations and data
ownership.
