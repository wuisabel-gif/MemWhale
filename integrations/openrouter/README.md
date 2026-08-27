# OpenRouter + MemoryWhale

[OpenRouter](https://openrouter.ai) is a hosted API gateway that exposes
hundreds of models from many providers behind a single OpenAI-compatible API
and one key. `stealth/ox-alpha` is one such model slug. OpenAI-compatible
clients can select any catalog model directly; Claude Code is the exception —
OpenRouter guarantees it only with Anthropic's first-party providers, so other
models there are best-effort. (Some clients qualify slugs with a provider
prefix — for example OpenCode's `openrouter/stealth/ox-alpha` — but the API
model id itself is `stealth/ox-alpha`.)

OpenRouter is a **model gateway**, not an MCP client for MemoryWhale. It does
not call `mw-mcp` and provides no access to the local store. It composes with
MemoryWhale through the coding agent in the middle:

```text
coding agent (Claude Code, Codex CLI, Continue, …)
├── model traffic ──────────────► OpenRouter ──► upstream model providers
└── memory tools (MCP) ─────────► mw-mcp ───────► local memorywhale.sqlite3
```

## Status

Verified against OpenRouter's public documentation in August 2026:

- OpenAI-compatible base URL: `https://openrouter.ai/api/v1`;
- Anthropic-compatible endpoint: `https://openrouter.ai/api/v1/messages`
  (Claude Code connects via `ANTHROPIC_BASE_URL=https://openrouter.ai/api`);
- authentication is `Authorization: Bearer <OPENROUTER_API_KEY>`; keys start
  with `sk-or-` and are managed at openrouter.ai/keys;
- it is not an MCP client for `mw-mcp`: OpenRouter publishes an MCP server of
  its own (model catalog, documentation search, test inference), but it
  provides no access to MemoryWhale.

Model availability, pricing, and context limits for any slug — including
`stealth/ox-alpha` — belong to the model's page on openrouter.ai and change
independently of MemoryWhale.

## Requirements

- MemoryWhale installed (`mw` on `PATH`);
- an OpenRouter account with an API key; credits are needed for paid models,
  while free model variants (`stealth/ox-alpha` reports zero pricing at this
  writing) are rate-limited instead;
- a coding agent that supports MCP servers plus either an OpenAI-compatible
  base URL or the Anthropic environment variables shown below;
- `jq` for the verification step below (not installed by default on macOS;
  `brew install jq`).

## Capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | No — OpenRouter is not an MCP client for `mw-mcp`; compose through an agent |
| Automatic execution capture | No — comes from the agent's hooks, not the gateway |
| Memory-use guidance | No — configure guidance in the agent, not the gateway |
| Multi-provider model access | Yes — one key, many models |

## Setup

### 1. Get an API key

Create a key at openrouter.ai/keys and export it:

```bash
export OPENROUTER_API_KEY="sk-or-your-key"
```

### 2. Point the coding agent at OpenRouter

**Claude Code** (Anthropic-compatible endpoint):

```bash
export ANTHROPIC_BASE_URL="https://openrouter.ai/api"
export ANTHROPIC_AUTH_TOKEN="$OPENROUTER_API_KEY"
export ANTHROPIC_API_KEY=""   # must be explicitly empty, not unset
```
OpenRouter documents that Claude Code is guaranteed only with Anthropic
first-party providers; other models (including `stealth/ox-alpha`) work on a
best-effort basis through this endpoint.

**OpenAI-compatible agents** (Continue, OpenCode, Codex CLI, …): set the base
URL to `https://openrouter.ai/api/v1`, use the key as the API key, and select
the model by its OpenRouter slug — for example `stealth/ox-alpha`. In clients
whose provider syntax requires a prefix, qualify it (OpenCode:
`openrouter/stealth/ox-alpha`). The exact setting name depends on the agent
(Continue uses `apiBase` on the model entry; OpenCode uses
`provider.<id>.options.baseURL`; Codex CLI defines a custom provider in
`~/.codex/config.toml` — a `[model_providers.openrouter]` block with
`base_url = "https://openrouter.ai/api/v1"` and an `env_key` — selected via
`model_provider` and `model`. Omit `wire_api`: current Codex CLI defaults to
the Responses protocol, which is what OpenRouter's Codex endpoint speaks.)

### 3. Keep `mw-mcp` configured in the same agent

This step is unchanged from the agent's normal MemoryWhale setup — the gateway
does not replace it. For example, in Claude Code:

```bash
claude mcp add memorywhale -- mw-mcp
```

The agent now sends model requests through OpenRouter and memory requests to
the local MemoryWhale store.

## Verify

Check each side independently:

```bash
# The key is valid (this endpoint requires auth; /v1/models is public):
curl -fsS https://openrouter.ai/api/v1/auth/key \
  -H "Authorization: Bearer $OPENROUTER_API_KEY"

# The model slug exists (public catalog; also useful for discovery):
curl -fsS https://openrouter.ai/api/v1/models \
  | jq -e '.data[] | select(.id == "stealth/ox-alpha")'

# MemoryWhale CLI and database are healthy:
mw doctor

# The mw-mcp transport responds to current-protocol discovery:
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}' | mw-mcp
```

`curl -fsS` fails on HTTP and connection errors, so an invalid key surfaces as
a nonzero exit instead of looking like success.

If model calls fail but the MemoryWhale checks pass, the problem is on the
OpenRouter side (key, credits, or model slug). If model calls work but memory
tools are missing in the agent, re-check its MCP configuration.

## How to use

Use this stack when you want:

- one key across many model providers;
- to try a specific model such as `stealth/ox-alpha` without separate vendor
  accounts;
- your existing agent's MemoryWhale memory to keep working unchanged.

Use direct provider keys instead when you only need one provider — the gateway
adds a hop and a data processor without benefit in that case.

## Example prompt

No prompt changes are needed; memory behavior lives in the agent's
configuration, not the gateway. A normal memory-aware prompt still works:

> Use MemoryWhale to check whether I encountered a similar failure before.

## Automatic capture

Automatic capture comes from the agent's hooks (for example, Claude Code's
`PostToolUse` hook), not from OpenRouter. The gateway only sees model requests
and cannot record terminal commands or sessions.

## Limitations

- OpenRouter does not provide MCP memory access. Its own MCP server serves
  OpenRouter's catalog and documentation; claims of direct
  `mw-mcp` ↔ OpenRouter connectivity would be false.
- Prompts and code context sent through the gateway reach OpenRouter and then
  the upstream provider serving the model. MemoryWhale's local-first
  guarantees do not extend to model traffic; review OpenRouter's privacy terms
  and the upstream provider's policies for the specific model.
- Stealth/anonymous-publisher models may have undisclosed training or retention
  policies; check the model page before sending sensitive code.
- Model calls require internet access; paid models also require credits.
  Memory retrieval through `mw-mcp` keeps working offline.
- Routing quality (tool use, long context) varies by upstream provider for the
  same slug; OpenRouter's own docs recommend pinning providers where tool-use
  reliability matters.

## Troubleshooting

- **401 from OpenRouter:** the key is missing, revoked, or not exported in the
  shell the agent runs from. Re-check `echo ${OPENROUTER_API_KEY:+set}`.
- **402 / insufficient credits:** the selected model is a paid variant and the
  account is out of credits; top up at openrouter.ai/credits or pick a free
  variant.
- **Model not found:** the slug is wrong or qualified in the wrong place. The
  API id is `stealth/ox-alpha`; add the `openrouter/` prefix only inside
  clients whose provider syntax requires it. Copy slugs exactly from the model
  page.
- **Claude Code falls back to subscription login:** `ANTHROPIC_API_KEY` must be
  set to an empty string explicitly, per OpenRouter's Claude Code guide.
- **Memory tools missing:** an agent MCP configuration issue, not a gateway
  issue. Run `mw doctor` and re-check the agent's `mw-mcp` registration.

## Remove integration

Stop exporting the OpenRouter variables (and remove any persisted provider
entries you added in the agent's config, restoring the direct-provider
settings). `mw-mcp` memory continues to work unchanged, and removing
OpenRouter never deletes MemoryWhale data.
