# CLIProxyAPI + MemoryWhale

[CLIProxyAPI](https://github.com/router-for-me/CLIProxyAPI) is a self-hosted
proxy server that exposes OpenAI-, Anthropic-, Gemini-, Codex-, and
Grok-compatible HTTP APIs on a local port. It routes requests across multiple
provider accounts with round-robin, failover, and cooldown handling.

CLIProxyAPI is a **model-provider proxy**, not an MCP client. It does not speak
the Model Context Protocol and does not call `mw-mcp` itself. Instead, it
composes with MemoryWhale through the coding agent that sits in the middle:

```text
coding agent (Claude Code, Codex CLI, etc.)
├── model traffic ──────────────► CLIProxyAPI ──► OpenAI / Anthropic / Gemini / …
└── memory tools (MCP) ─────────► mw-mcp ───────► local memorywhale.sqlite3
```

## Status

Verified against CLIProxyAPI's public `main` branch documentation (README,
`config.example.yaml`, and `docs/`) in August 2026. The verified facts:

- CLIProxyAPI exposes OpenAI-, Anthropic-, and Gemini-compatible HTTP endpoints
  on a configurable port (default `8317`);
- it authenticates requests with keys from its `api-keys` config, accepted via
  `Authorization: Bearer`, `X-Api-Key`, or `X-Goog-Api-Key` headers;
- it has no MCP server, MCP client, or `mcpServers` configuration of any kind.

The compositional setup below was reviewed for coherence against those facts;
verify the exact endpoint paths for your CLIProxyAPI version before relying on
them.

## Requirements

- MemoryWhale installed (`mw` on `PATH`);
- a coding agent that supports both MCP servers and a configurable model base
  URL — Claude Code, Codex CLI, OpenCode, and similar tools all qualify;
- CLIProxyAPI installed and running with at least one `api-keys` entry.

## Capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | No — CLIProxyAPI is not an MCP client; compose through an agent |
| Automatic execution capture | No — comes from the agent's hooks, not the proxy |
| Memory-use guidance | No — configure guidance in the agent, not the proxy |
| Model-provider consolidation | Yes — one local endpoint, multiple upstream accounts |

## Setup

### 1. Start CLIProxyAPI

Follow the CLIProxyAPI README to install it and authenticate at least one
upstream account. Its config sets the port and the keys clients may use:

```yaml
# CLIProxyAPI config.yaml (excerpt)
port: 8317
api-keys:
  - "sk-your-local-key"
```

### 2. Point the coding agent at CLIProxyAPI

Configure the agent's model base URL to the local proxy and use a key from
CLIProxyAPI's `api-keys`. For OpenAI-compatible clients:

```bash
export OPENAI_BASE_URL="http://127.0.0.1:8317/v1"
export OPENAI_API_KEY="sk-your-local-key"
```

For Anthropic-compatible clients:

```bash
export ANTHROPIC_BASE_URL="http://127.0.0.1:8317"
export ANTHROPIC_API_KEY="sk-your-local-key"
```

CLIProxyAPI accepts `Authorization: Bearer`, `X-Api-Key`, and `X-Goog-Api-Key`
headers, so standard client auth works unchanged. See the CLIProxyAPI docs for
the exact endpoint paths each protocol exposes.

### 3. Keep `mw-mcp` configured in the same agent

This step is unchanged from the agent's normal MemoryWhale setup — the proxy
does not replace it. For example, in Claude Code:

```bash
claude mcp add memorywhale -- mw-mcp
```

or in an OpenCode config:

```json
{
  "mcp": {
    "memorywhale": {
      "type": "local",
      "command": ["mw-mcp"],
      "enabled": true
    }
  }
}
```

The agent now sends model requests through CLIProxyAPI and memory requests to
the local MemoryWhale store.

## Verify

Check each side independently:

```bash
# CLIProxyAPI is reachable and accepts your key (adjust the path per protocol):
curl -s http://127.0.0.1:8317/v1/models \
  -H "Authorization: Bearer sk-your-local-key" | head

# MemoryWhale MCP is healthy:
mw doctor
```

`mw doctor` reports whether `mw-mcp` starts and advertises the six memory
tools. If model calls fail but `mw doctor` passes, the problem is on the proxy
side; if model calls work but memory tools are missing, re-check the agent's
MCP configuration.

## How to use

Use this stack when you want:

- one local endpoint that consolidates several model-provider accounts;
- automatic failover when one account hits a quota;
- your existing agent's MemoryWhale memory to keep working unchanged.

Use plain provider keys instead when you have only one account and no routing
needs — a proxy in the middle adds a hop without benefit.

## Example prompt

No prompt changes are needed; memory behavior lives in the agent's
configuration, not the proxy. A normal memory-aware prompt still works:

> Use MemoryWhale to check whether I encountered a similar failure before.

## Automatic capture

Automatic capture is unchanged and comes from the agent's hooks (for example,
Claude Code's `PostToolUse` hook), not from CLIProxyAPI. The proxy only sees
model requests and cannot record terminal commands or sessions.

## Limitations

- CLIProxyAPI does not provide MCP memory access. Any guide or matrix claim of
  direct `mw-mcp` ↔ CLIProxyAPI connectivity would be false.
- Requests sent through the proxy contain your prompts and code context, which
  the proxy forwards to the configured upstream providers. MemoryWhale's
  local-first guarantees do not extend to model traffic; review CLIProxyAPI's
  and each provider's data policies separately.
- The local proxy must be running for model requests to succeed; memory
  retrieval through `mw-mcp` keeps working even when the proxy is down.
- If you bind CLIProxyAPI beyond loopback, its endpoints are only as safe as
  the `api-keys` you configure. Keep `host` on loopback unless you need LAN
  access, and treat keys in `config.yaml` as secrets.

## Troubleshooting

- **Model calls fail with 401:** the key the agent sends is not in
  CLIProxyAPI's `api-keys` list, or the header name is unsupported for that
  endpoint. Confirm the key and try `Authorization: Bearer` explicitly.
- **Model calls fail with connection refused:** CLIProxyAPI is not running or
  is on a different port; check `port` in its `config.yaml`.
- **Memory tools missing:** this is an agent MCP configuration issue, not a
  proxy issue. Run `mw doctor` and re-check the agent's `mw-mcp` registration.
- **Wrong model served:** CLIProxyAPI routes by model name across your
  authenticated accounts; see its routing docs (`round-robin`,
  `fill-first`, session affinity) rather than MemoryWhale's.

## Remove integration

Stop CLIProxyAPI and unset the agent's base-URL override
(`OPENAI_BASE_URL` / `ANTHROPIC_BASE_URL`). The agent falls back to direct
provider access, and `mw-mcp` memory continues to work unchanged. Removing
CLIProxyAPI never deletes MemoryWhale data.
