# MemoryWhale documentation

The repository documentation is organized by the question a reader is asking.

## Concepts: what does MemoryWhale mean?

- [Developer memory](concepts/README.md)
- [Capture](concepts/capture.md)
- [Memory](concepts/memory.md)
- [Use cases](concepts/use-cases.md)
- [Retrieval](concepts/retrieval.md)

## Guides: how do I accomplish a task?

- [Getting started](guides/getting-started.md)
- [Capture terminal work](guides/terminal-capture.md)
- [Debug with previous evidence](guides/debugging.md)
- [Connect a coding agent](guides/agent-memory.md)
- [Move memory between machines](guides/multi-machine.md)

## Reference: what is the exact interface?

- [CLI and helper binaries](reference/cli.md)
- [Memory pet](reference/pet.md)
- [MCP tools](reference/mcp.md)
- [Configuration](reference/configuration.md)
- [Storage](reference/storage.md)
- [Environment variables](reference/environment-variables.md)

## Design and policy

- [Architecture](architecture.md)
- [Security and local threat model](SECURITY.md)
- [Limitations](limitations.md)
- [Integration matrix and client guides](../integrations/README.md)

The root README is the product landing page. `docs/` explains how MemoryWhale
works, `integrations/` explains how external tools connect, `crates/` contains
the Rust implementation, and `benchmarks/` records how retrieval is measured.
