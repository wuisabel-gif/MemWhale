# Ecosystem

MemoryWhale is one of a small set of related, local-first projects. They are
separate repositories that **refer to each other** and compose cleanly — the
memory belongs to *you*, not to any one model.

| Project | Role | Repo |
|---|---|---|
| **Delphin** 🐬 | **Communication** — a duplex wrapper for AI agent CLIs: keep talking while the agent thinks; an arbiter decides interrupt-vs-wait. | https://github.com/wuisabel-gif/Delphin |
| **ContextGC** 🧠 | **Context management** — predicts context pressure and selectively keeps, compresses, externalizes, or evicts the active model working set. | https://github.com/wuisabel-gif/ContextGC |
| **MemoryWhale** 🐋 | **Memory** — an inspectable memory OS: capture, retrieve with explanations, forget. (this repo) | https://github.com/wuisabel-gif/MemWhale |

## How they fit together

```
        you ⇄ AI agent
            │ (Delphin makes the conversation duplex)
            ▼
        Delphin  ──writes conversation turns──▶  MemoryWhale
        (communication)                          (memory: recall + explain)
                                                     │
                                                     ▼
        MemoryWhale ◄── durable candidates + recall ──► ContextGC
        (long-term memory)       (context: keep / compress / evict)
                                      │
                                      ▼
                              active model working set
```

- **Delphin** smooths the live conversation and records every turn.
- **ContextGC** manages what the model should keep in its active context right now.
- **MemoryWhale** stores, ranks, and **explains** what's worth remembering.

The boundary is deliberate: ContextGC manages the temporary working set for a
long-running agent, while MemoryWhale preserves useful development experience
after it leaves that working set. The intended future composition is for
ContextGC to promote a durable fix or decision through `mw-mcp`; noisy output
can simply be evicted. ContextGC's current integration documentation marks
that adapter as future work, so this is not a shipped end-to-end integration
yet.

## Wiring them together (optional)

Delphin keeps its own local memory by default, but you can point it at
MemoryWhale's database so your conversations flow into MemoryWhale's recall:

```bash
delphin --db ~/Library/Application\ Support/MemoryWhale/memorywhale.sqlite3 -- claude
```

Then MemoryWhale's **Recall** panel searches those conversation turns alongside
your notes and terminal commands — each result with a "retrieved because…"
explanation.

## Naming

These projects are related infrastructure, not one combined agent: **Delphin**
for communication, **ContextGC** for active context, and **MemoryWhale** for
durable memory.
