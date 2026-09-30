# Ecosystem

MemoryWhale is one of a small set of related, local-first projects. They are
separate repositories that **refer to each other** and compose cleanly — the
memory belongs to *you*, not to any one model.

| Project | Role | Repo |
|---|---|---|
| **Delphin** 🐬 | **Communication** — a duplex wrapper for AI agent CLIs: keep talking while the agent thinks; an arbiter decides interrupt-vs-wait. | https://github.com/wuisabel-gif/Delphin |
| **ContextGC** 🧠 | **Context management** — predicts context pressure and selectively keeps, compresses, externalizes, or evicts the active model working set. | https://github.com/wuisabel-gif/ContextGC |
| **Joule** ⚡ | **Energy** — an OpenAI-compatible proxy that measures and cuts the energy of each LLM request. | https://github.com/wuisabel-gif/Joule |
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
- **Joule** uses MemoryWhale's ranking (`memorywhale-core`) to send a model only
  the parts of a long chat that matter to the current question.

The boundary is deliberate: ContextGC manages the temporary working set for a
long-running agent, while MemoryWhale preserves useful development experience
after it leaves that working set. The intended future composition is for
ContextGC to promote a durable fix or decision through `mw-mcp`; noisy output
can simply be evicted. ContextGC's current integration documentation marks
that adapter as future work, so this is not a shipped end-to-end integration
yet.

## Wiring them together (optional)

**Delphin** (0.4.0 or newer). `mw integrate delphin` checks the install and
prints the command:

```bash
delphin --memorywhale -- claude
```

Delphin streams each turn to `mw turns`, which redacts it and stores it, so
MemoryWhale's **Recall** searches those turns alongside your notes and terminal
commands. When an error scrolls past, Delphin asks `mw hint` and shows a line
such as `🐬 MemoryWhale: seen 2 times, the fix was: xcode-select --install`.

**Joule** (0.7.0 or newer). Run the proxy with `--optimize ultra`. Its
`context-recall` pass keeps recent turns and the older exchanges most relevant
to the latest question, ranked by `memorywhale-core`, and reports the tokens
and joules saved in response headers.

## Naming

These projects are related infrastructure, not one combined agent: **Delphin**
for communication, **ContextGC** for active context, **Joule** for energy, and
**MemoryWhale** for durable memory.
