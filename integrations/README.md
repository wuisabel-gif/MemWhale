# MemoryWhale integrations

Integrations are thin adapters between external tools and MemoryWhale's public
interfaces. They do not define the core product: capture, memory, and retrieval
remain client-neutral.

## Capability definitions

- **Context export:** a person can paste output from `mw context` into any tool.
- **MCP memory access:** the client can read and explicitly write local memory
  through the `mw-mcp` stdio server.
- **Automatic execution capture:** a verified client mechanism records commands
  or tool results without wrapping each command manually.
- **Memory-use guidance:** a rule, skill, or documented instruction tells the
  client when to consult or update MemoryWhale.

MCP memory access is not automatic execution capture. Unless the matrix says
otherwise, commands run by a client are recorded only through normal terminal
capture or an explicitly installed hook.

## Capability matrix

The values below describe files in this repository, not assumed client
features.

| Client | MCP memory access | Auto-capture | Guidance | Setup |
| --- | --- | --- | --- | --- |
| Any stdio MCP client | Yes | No | Client-specific | [Generic MCP](generic-mcp/README.md) |
| Claude Code | Yes | Yes, optional `PostToolUse` hook | Yes, optional skill | [Hook and skill](claude-code/) |
| Claude Desktop | Yes | No | No | [Guide](claude-desktop/README.md) |
| Cline | Yes | No | Yes | [Guide](cline/README.md) |
| Codex CLI | Yes | No | Yes | [Guide](codex/README.md) |
| Continue | Yes | No | Yes | [Guide](continue/README.md) |
| CrowClaw | Yes | No | Yes | [Guide](crowclaw/README.md) |
| Cursor | Yes | No | Yes | [Guide](cursor/README.md) |
| Gemini CLI | Yes | No | Yes | [Guide](gemini-cli/README.md) |
| Goose | Yes | No | Yes | [Guide](goose/README.md) |
| Hermes Agent | Yes | No | Example prompt | [Guide](hermes/README.md) |
| Jan Desktop | Yes | No | No | [Guide](jan/README.md) |
| OpenClaw | Yes | No | Yes | [Guide](openclaw/README.md) |
| OpenCode | Yes | No | Example prompt | [Guide](opencode/README.md) |
| Pi coding agent | Unverified | No | No | [Guide](pi/README.md) |
| Rho | Yes | No | Yes, via `AGENTS.md` | [Guide](rho/README.md) |
| VS Code / GitHub Copilot | Yes | No | Yes | [Guide](vscode/README.md) |
| Windsurf | Yes | No | Yes | [Guide](windsurf/README.md) |
| Zed | Yes | No | Yes | [Guide](zed/README.md) |
| Neovim plugin | No; uses the CLI directly | No | Commands only | [Guide](neovim/README.md) |

Every tool can still use context export:

```bash
mw context --last-error
```

## MCP interface

`mw-mcp` exposes six tools over newline-delimited JSON-RPC 2.0 on stdio:

- `recent_errors`
- `search_memory`
- `get_context`
- `remember`
- `similar_failures`
- `stats`

- **Cursor** → [`cursor/`](cursor/README.md)
- **VS Code / GitHub Copilot** (agent mode) → [`vscode/`](vscode/README.md)
- **Windsurf** → [`windsurf/`](windsurf/README.md)
- **Zed** → [`zed/`](zed/README.md)
- **Codex CLI** → [`codex/`](codex/README.md)
- **OpenClaw** → [`openclaw/`](openclaw/README.md)
- **Any other MCP client** (Cline, Continue, …) — add a stdio server whose
  command is `mw-mcp` (no arguments). It honours `MEMORYWHALE_DATA_DIR` like the
  rest of the CLI.

The [MCP reference](../docs/reference/mcp.md) is authoritative for parameters,
responses, and the [local stdio trust model](../docs/reference/mcp.md#trust-model).
A transport-level check is:

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | mw-mcp
```

## Claude Code automatic capture

The optional
[`claude-code/hooks/mw-record.py`](claude-code/hooks/mw-record.py)
`PostToolUse` hook captures Claude Code Bash calls through `mw-remember`. It is
separate from MCP: MCP lets the client retrieve and explicitly save memory,
while the hook records observed execution.

The optional [`claude-code/memorywhale/SKILL.md`](claude-code/memorywhale/SKILL.md)
teaches Claude Code when to search and save memory. Both require explicit
installation and neither is needed for ordinary terminal capture.

## Adding or updating an integration

Use [`TEMPLATE.md`](TEMPLATE.md). Verify the current client configuration from
an authoritative source, declare capabilities from repository evidence, and
keep client-specific behavior out of MemoryWhale core.
