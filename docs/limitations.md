# Limitations

- Secret scrubbing recognizes common patterns but cannot guarantee that all
  sensitive output is removed.
- MCP access does not automatically capture ordinary terminal or agent
  execution.
- Client integrations differ in automatic capture and memory-use guidance.
- A remembered lesson may become stale as environments and dependencies change.
- Native Windows session recording is not currently available; use WSL.
- MemoryWhale is local-first and does not silently synchronize between machines.

Use the [integration matrix](../integrations/README.md) for client-specific
capabilities and the [security guide](SECURITY.md) for the local threat model.
