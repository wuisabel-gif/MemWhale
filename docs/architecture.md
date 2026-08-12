# Architecture

MemoryWhale is a local debugging-memory layer. It sits below terminals and
coding agents: those tools do the work, while MemoryWhale captures, preserves,
and retrieves what happened.

```text
                    MEMORYWHALE
     ┌────────────────────────────────┐
     │ 1. CAPTURE                     │
     │ shell hooks · mw-run · sessions│
     │ verified agent hooks           │
     └───────────────┬────────────────┘
                     ▼
     ┌────────────────────────────────┐
     │ 2. MEMORY                      │
     │ executions · failures · output │
     │ lessons · local SQLite         │
     └───────────────┬────────────────┘
                     ▼
     ┌────────────────────────────────┐
     │ 3. RETRIEVAL                   │
     │ search · context · similarity  │
     │ recent errors                  │
     └───────────────┬────────────────┘
                     ▼
     ┌────────────────────────────────┐
     │ 4. INTERFACES                  │
     │ CLI · MCP · TUI · Web · Desktop│
     │ thin client integrations       │
     └────────────────────────────────┘
```

## 1. Capture

Capture adapters observe terminal or supported agent execution and submit
structured evidence. Capture owns evidence fidelity and redaction; it does not
own retrieval policy.

## 2. Memory

The memory module owns durable local representation: executions, failures,
output, transcripts, lessons, provenance, and lifecycle. SQLite is the source
of truth.

## 3. Retrieval

Retrieval interprets queries against memory and returns evidence with enough
context to understand why it matched. Search, context generation, recent errors,
and similar failures belong here.

## 4. Interfaces

Interfaces make the three core capabilities usable. The CLI, MCP server, TUI,
web dashboard, desktop shell, and client integrations should remain thin. An
integration configures an external client; it does not move client-specific
behavior into core.

## Capture and retrieval are independent

```text
CAPTURE                              RETRIEVAL
Terminal / agent execution          Coding agent
          │                              │
          ▼                              ▼
shell, command, or agent hook        mw-mcp
          │                              │
          ▼                              ▼
      MemoryWhale ◄──────────────── local memory
          │                              │
          ▼                              ▼
     local SQLite               failures and lessons
```

MCP retrieves and explicitly writes memory. It does not automatically capture a
normal terminal. Automatic agent execution capture exists only where a verified
client hook is installed.

## Feature-placement rule

A feature belongs in MemoryWhale if it improves **capturing, preserving,
retrieving, or sharing development experience** while respecting the local-first
model.

- A new capture hook belongs in Capture or a thin integration adapter.
- Failure similarity belongs in Retrieval.
- Retention and provenance belong in Memory.
- A client configuration belongs in Interfaces under `integrations/`.
- An autonomous coding agent, model-provider router, or unrelated web-search
  system is outside MemoryWhale's responsibility.

Cross-cutting storage, privacy, or remote-service proposals require explicit
architectural discussion before implementation.
