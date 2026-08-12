# Move memory between machines

MemoryWhale is local-first and does not silently synchronize its database.
Use the documented export and import commands when moving selected memory:

```bash
mw export --help
mw import --help
```

Keep independent backups before transferring or replacing a database. Do not
copy a live SQLite database while another process is writing to it. Review
exports for secrets and proprietary output before moving them between trust
zones.

See the [CLI reference](../reference/cli.md) for exact options and the
[storage reference](../reference/storage.md) for data locations.
