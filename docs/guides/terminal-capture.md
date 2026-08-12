# Capture terminal work

Choose the narrowest capture mode that fits the work.

```bash
mw-run -- cargo test      # one command
mw --notes "build debug"  # one interactive session
mw --live                 # crash-resistant session
mw global on              # future interactive shell commands
```

Run `mw doctor` after installing a global hook. Use `mw global off` to remove
it.

Terminal output can contain credentials, private paths, hostnames, and source
text. MemoryWhale scrubs common secret shapes, but regex redaction is not a
security boundary. Use capture gates and scoped deletion, and review the
[security guide](../SECURITY.md) before recording sensitive work.
