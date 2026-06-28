# Ecosystem

MemoryWhale is one of a small set of related, local-first projects. They are
separate repositories that **refer to each other** and compose cleanly — the
memory belongs to *you*, not to any one model.

| Project | Role | Repo |
|---|---|---|
| **Delphin** 🐬 | **Communication** — a duplex wrapper for AI agent CLIs: keep talking while the agent thinks; an arbiter decides interrupt-vs-wait. | https://github.com/wuisabel-gif/Delphin |
| **MemoryWhale** 🐋 | **Memory** — an inspectable memory OS: capture, retrieve with explanations, forget. (this repo) | https://github.com/wuisabel-gif/MemWhale |

## How they fit together

```
        you ⇄ AI agent
            │ (Delphin makes the conversation duplex)
            ▼
        Delphin  ──writes conversation turns──▶  MemoryWhale
        (communication)                          (memory: recall + explain)
```

- **Delphin** smooths the live conversation and records every turn.
- **MemoryWhale** stores, ranks, and **explains** what's worth remembering.

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

Both are cetaceans on purpose: **Delphin** (the dolphin) for communication,
**MemoryWhale** for memory — related pieces of infrastructure, not isolated tools.
