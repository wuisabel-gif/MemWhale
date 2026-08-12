# Capture

Capture is how development evidence enters MemoryWhale. Supported paths include
interactive shell hooks, `mw-run` for one command, `mw` or `mw --live` for a
session, `mw-remember` for structured ingestion, and verified client-specific
agent hooks.

Capture should preserve the command, arguments, working directory, exit status,
timestamp, stdout, stderr, and relevant notes whenever that evidence exists.
Secret scrubbing and capture gates reduce exposure, but they are mitigations—not
a reason to capture unrelated data.

MCP is not the default capture path for terminal activity. An MCP client can
explicitly call `remember`, but ordinary commands run outside a supported hook
are captured only through terminal capture mechanisms.

See [Capture terminal work](../guides/terminal-capture.md) for setup.
