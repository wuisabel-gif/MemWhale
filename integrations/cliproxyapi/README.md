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
  URL. Claude Code works with the environment variables shown below; for other
  agents, use their documented base-URL configuration:
  - **Claude Code**: the `ANTHROPIC_BASE_URL` environment variable;
  - **other Anthropic-compatible clients**: their `ANTHROPIC_BASE_URL`-style
    setting;
  - **OpenAI-compatible clients** (generic): the `OPENAI_BASE_URL` environment
    variable;
  - **Codex CLI**: `openai_base_url` or a custom provider entry in
    `~/.codex/config.toml` (see Codex's configuration docs);
  - **OpenCode**: `provider.<id>.options.baseURL` plus model selection in
    `opencode.json` (see OpenCode's provider docs);
  - **Continue**: `apiBase` on the model entry in `~/.continue/config.yaml`;
  - other OpenAI- or Anthropic-compatible clients: their respective base-URL
    settings.
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
host: "127.0.0.1"   # bind loopback only; the default empty host binds all interfaces
port: 8317
api-keys:
  # Replace this placeholder with a unique local secret before starting.
  - "sk-your-local-key"
```

### 2. Point the coding agent at CLIProxyAPI

Configure the agent's model base URL to the local proxy and use a key from
CLIProxyAPI's `api-keys`. Export that key once as `CLIPROXY_API_KEY` — it is
what the verification step below authenticates with, independent of protocol.
If you already have provider base URLs or keys set, save their values (and
whether each was set at all) first so they can be restored on removal:

```bash
# save current values and whether each was set, for restoration on removal
OLD_OPENAI_BASE_URL="${OPENAI_BASE_URL:-}"
OLD_OPENAI_BASE_URL_WAS_SET="${OPENAI_BASE_URL+x}"
OLD_OPENAI_API_KEY="${OPENAI_API_KEY:-}"
OLD_OPENAI_API_KEY_WAS_SET="${OPENAI_API_KEY+x}"
OLD_ANTHROPIC_BASE_URL="${ANTHROPIC_BASE_URL:-}"
OLD_ANTHROPIC_BASE_URL_WAS_SET="${ANTHROPIC_BASE_URL+x}"
OLD_ANTHROPIC_API_KEY="${ANTHROPIC_API_KEY:-}"
OLD_ANTHROPIC_API_KEY_WAS_SET="${ANTHROPIC_API_KEY+x}"

export CLIPROXY_API_KEY="sk-your-local-key"
```

For OpenAI-compatible clients:

```bash
export OPENAI_BASE_URL="http://127.0.0.1:8317/v1"
export OPENAI_API_KEY="$CLIPROXY_API_KEY"
```

For Anthropic-compatible clients:

```bash
export ANTHROPIC_BASE_URL="http://127.0.0.1:8317"
export ANTHROPIC_API_KEY="$CLIPROXY_API_KEY"
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
# CLIProxyAPI is reachable and accepts your key (adjust the path per protocol).
# Uses the dedicated proxy-key variable, valid for either setup variant above:
curl -fsS "http://127.0.0.1:8317/v1/models" \
  -H "Authorization: Bearer ${CLIPROXY_API_KEY}"

# MemoryWhale MCP is healthy:
mw doctor
```

`curl -fsS` fails on HTTP and connection errors, so a 401 or a stopped proxy
surfaces as a nonzero exit instead of looking like success.

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

Stop CLIProxyAPI, then undo **both** the base-URL and key overrides you set in
step 2 — leaving `OPENAI_API_KEY`/`ANTHROPIC_API_KEY` pointing at the proxy key
breaks direct provider authentication. Remove the overrides first, then
restore only what was actually set before:

```bash
# remove the overrides unconditionally
unset OPENAI_BASE_URL OPENAI_API_KEY
unset ANTHROPIC_BASE_URL ANTHROPIC_API_KEY
unset CLIPROXY_API_KEY

# restore exactly the variables that existed before step 2, independently
if [ -n "${OLD_OPENAI_BASE_URL_WAS_SET:-}" ]; then
  export OPENAI_BASE_URL="$OLD_OPENAI_BASE_URL"
fi

if [ -n "${OLD_OPENAI_API_KEY_WAS_SET:-}" ]; then
  export OPENAI_API_KEY="$OLD_OPENAI_API_KEY"
fi

if [ -n "${OLD_ANTHROPIC_BASE_URL_WAS_SET:-}" ]; then
  export ANTHROPIC_BASE_URL="$OLD_ANTHROPIC_BASE_URL"
fi

if [ -n "${OLD_ANTHROPIC_API_KEY_WAS_SET:-}" ]; then
  export ANTHROPIC_API_KEY="$OLD_ANTHROPIC_API_KEY"
fi
```

Run setup and removal in the same shell (or persist the `OLD_*` values
securely) so the restore state is available. If you persisted the proxy URL in
the agent's own configuration (Codex's `config.toml`, OpenCode's
`opencode.json`, Continue's `config.yaml`), remove or restore that entry too —
otherwise model calls keep targeting `127.0.0.1:8317` after the proxy stops.
Direct-provider fallback then works when direct provider access is already
configured. `mw-mcp` memory continues to work unchanged, and removing
CLIProxyAPI never deletes MemoryWhale data.
