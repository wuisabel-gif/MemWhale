# Local JSON API

`mw-serve --api` enables a small, versioned, read-only JSON API alongside the
human dashboard. The API is disabled unless the flag is present.

```bash
mw-serve --api                         # loopback by default
MEMORYWHALE_TOKEN=secret mw-serve --lan --api
```

The API uses the same listener, connection limits, request bounds, security
headers, host checks, and token gate as the dashboard. It is loopback-only by
default. A non-loopback bind still requires `MEMORYWHALE_TOKEN` or `--token`.

There are no write endpoints, arbitrary SQL endpoints, or arbitrary filesystem
reads. MCP remains the primary agent interface.

## Response envelope

Successful responses use:

```json
{
  "api_version": "v1",
  "data": {}
}
```

Errors use:

```json
{
  "api_version": "v1",
  "error": {
    "code": "invalid_limit",
    "message": "limit must be between 1 and 50"
  }
}
```

The API returns `application/json`, preserves the stored redaction policy, and
limits search/list results to at most 50 items per request.

The machine-readable contract is available at
[`api.openapi.json`](api.openapi.json). Requests to the API while `--api` is
disabled return `404` with `error.code` `api_disabled`; unsupported methods and
unauthenticated LAN requests return JSON errors with the same envelope.

## Endpoints

### `GET /api/v1/health`

Returns server version, status, and the number of loaded memories:

```json
{
  "api_version": "v1",
  "data": {
    "status": "ok",
    "version": "0.9.1",
    "memory_count": 42
  }
}
```

### `GET /api/v1/search?q=<text>&limit=<n>&agent=<agent>`

Searches the same explainable retrieval engine as the CLI. `q` is required;
`limit` defaults to 20 and accepts 1–50. The optional `agent` filter accepts
`claude`, `rho`, or `terminal`; `terminal` matches records whose nullable storage value is `NULL`. The same filter can also be written inline in `q`,
for example `q=linker+error+agent%3Aclaude`.

Each result includes the namespaced memory ID, score, full stored memory,
signals, and human-readable ranking reasons:

```json
{
  "api_version": "v1",
  "data": {
    "query": "linker error",
    "results": [
      {
        "id": 1000000001,
        "source": "command",
        "source_id": 1,
        "command_id": 1,
        "agent": "claude",
        "agent_label": "claude",
        "score": 0.91,
        "memory": {},
        "signals": [],
        "reasons": ["…"]
      }
    ]
  }
}
```

### `GET /api/v1/memories/:id`

Returns one namespaced memory by ID, or `404` if it is not present.

### `GET /api/v1/commands/:id`

Returns one captured command run by its raw `command_runs.id`, including command, argv, cwd, exit code,
stdout, stderr, notes, timestamp, and nullable `agent` metadata. `agent` is
`null` for terminal/manual or legacy rows; `agent_label` renders that value as
the canonical `terminal` label. Malformed historical `argv_json` is returned as
`null` rather than crashing the API.

Search results include `command_id` when the result comes from a command
record. Pass that value to this endpoint; do not use the namespaced search
result `id` as the command route ID.

### `GET /api/v1/sessions?limit=<n>`

Returns the newest captured sessions, bounded by the same 1–50 limit. Each
session includes its ID, timestamps, status, retained byte count, notes, and
cwd.

### `GET /api/v1/repositories`

Returns persisted canonical repository IDs, names, and worktree roots. Linked
worktrees remain distinguishable through `worktree_root` while sharing a
canonical `id`.

## Authentication

Loopback requests use the same local trust model as the dashboard. When the
server uses a token, API requests may authenticate with the dashboard's
`mw_token` cookie obtained from `POST /login` or with
`Authorization: Bearer <token>`. API keys or tokens in query strings are not
supported. A protected failure returns a JSON error plus a Bearer challenge.

## Example

```bash
curl -fsS 'http://127.0.0.1:7071/api/v1/health'
curl -fsS --get 'http://127.0.0.1:7071/api/v1/search' \
  --data-urlencode 'q=linker error' --data-urlencode 'limit=10'
```

For a coding agent, use [`mw-mcp`](mcp.md). For the human dashboard, omit
`--api` and open `/`.
