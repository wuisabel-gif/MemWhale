# AI agent integrations

Give an AI coding agent both sides of memory: it can *read* what already
failed, and it can *write down* what it figured out.

## 1. MCP server (recommended) — the agent reads and writes

`mw-mcp` is a [Model Context Protocol](https://modelcontextprotocol.io) server
over stdio. Register it once and the agent gets four native tools — no
copy-paste:

- `recent_errors` — recent failed commands with their error output
- `search_memory` — search commands, output, notes, and remembered lessons
- `get_context` — a compact digest of recent failures, optionally per project
- `remember` — save a conclusion ("the fix was X") for its future self to find

Claude Code:

```bash
claude mcp add memorywhale -- mw-mcp
```

The same `mw-mcp` server plugs into every MCP-speaking client — only the config
file differs. Per-client setup (config + a "when to use it" rule):

- **Cursor** → [`cursor/`](cursor/README.md)
- **VS Code / GitHub Copilot** (agent mode) → [`vscode/`](vscode/README.md)
- **Windsurf** → [`windsurf/`](windsurf/README.md)
- **Zed** → [`zed/`](zed/README.md)
- **Codex CLI** → [`codex/`](codex/README.md)
- **OpenClaw** → [`openclaw/`](openclaw/README.md)
- **CrowClaw** → [`crowclaw/`](crowclaw/README.md)
- **Goose** (Block) → [`goose/`](goose/README.md)
- **Claude Desktop** → [`claude-desktop/`](claude-desktop/README.md)
- **Cline** (VS Code) → [`cline/`](cline/README.md)
- **Continue** (VS Code / JetBrains) → [`continue/`](continue/README.md)
- **Gemini CLI** (Google) → [`gemini-cli/`](gemini-cli/README.md)
- **Any other MCP client** — add a stdio server whose
  command is `mw-mcp` (no arguments). It honours `MEMORYWHALE_DATA_DIR` like the
  rest of the CLI.

Quick check that it responds:

```bash
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}' | mw-mcp
```

## 2. Claude Code hook — auto-capture what the agent runs

The MCP server above is how the agent *reads and writes lessons*. This hook is
how its *commands* get captured automatically, the same way `mw global on`
auto-records your own terminals — without it, memory only grows when you
manually run `mw`/`mw-run`.

[`claude-code/hooks/mw-record.py`](claude-code/hooks/mw-record.py) is a
`PostToolUse` hook: after Claude Code runs a Bash command, it saves the
command, its output, and its exit status into MemoryWhale via `mw-remember`
(so it's redacted like everything else). Non-Bash tool calls are ignored, and
any failure here is swallowed — it can never block the agent's tool call.

Add to your project's `.claude/settings.json` (or `~/.claude/settings.json`
for every project):

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "python3 /absolute/path/to/MemWhale/integrations/claude-code/hooks/mw-record.py"
          }
        ]
      }
    ]
  }
}
```

Requires `mw-remember` on `PATH` (it's part of the standard install). Combine
with the skill below and the `remember` MCP tool, and the agent's failures and
conclusions both accumulate automatically — next week's session inherits this
week's debugging.

## 3. Claude Code skill

[`claude-code/memorywhale/SKILL.md`](claude-code/memorywhale/SKILL.md) teaches
Claude Code *when* to reach for the memory (recurring failures, "how did we fix
this last time"). Install it by copying (or symlinking) the folder into your
skills directory:

```bash
cp -r integrations/claude-code/memorywhale ~/.claude/skills/
```

The skill uses the MCP tools when connected and falls back to the `mw context`
CLI otherwise, so it's useful with or without the MCP server.

## Without any of these

`mw context` prints a compact, paste-ready digest for any agent or chat:

```bash
mw context --last-error
```
