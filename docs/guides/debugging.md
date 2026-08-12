# Debug with previous evidence

Start with the exact current failure, then search for evidence that matches it:

```bash
mw context --last-error
mw search "distinctive error text"
mw git-fix
```

When a fix works, preserve the conclusion without replacing the original
evidence:

```bash
mw remember "the build needed PKG_CONFIG_PATH set to /opt/example/lib/pkgconfig"
```

Treat old lessons as leads rather than universal truth. Environments and
dependencies change; check the command, timestamp, project scope, and observed
output before reusing a fix.
