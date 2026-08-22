# Claude Code + MemoryWhale

Claude Code can use MemoryWhale in three independent ways: `mw-mcp` provides
native memory tools, a `PostToolUse` hook records the Bash commands Claude
runs (successes and failures alike), and
a skill teaches Claude when to search or save debugging memory.

## Status

Verified against Claude Code's official [MCP](https://code.claude.com/docs/en/mcp),
[hooks](https://code.claude.com/docs/en/hooks), and
[skills](https://code.claude.com/docs/en/skills) documentation on 2026-08-21.
The hook and skill are optional repository-provided components.

## Requirements

- MemoryWhale installed with `mw-mcp` and `mw-remember` on `PATH`.
- Claude Code installed on Linux, macOS, or Windows through WSL.
- Python 3 for the capture hook.
- A local checkout of this repository to copy the bundled hook and skill.

## Capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes |
| Automatic execution capture | Yes, optional `PostToolUse` hook for Bash calls |
| Memory-use guidance | Yes, optional skill |

## Setup

Run the file-copy commands below from the MemoryWhale repository root.

### Connect the MCP server

Register `mw-mcp` at user scope so it is available in every local project:

```bash
claude mcp add --scope user --transport stdio memorywhale -- mw-mcp
```

This gives Claude Code the six MemoryWhale tools: `recent_errors`,
`search_memory`, `get_context`, `remember`, `similar_failures`, and `stats`.
The command follows Claude Code's documented
[local stdio server syntax](https://code.claude.com/docs/en/mcp#option-3-add-a-local-stdio-server).

### Install the capture hook

Copy the bundled hook into your personal Claude Code configuration:

```bash
mkdir -p ~/.claude/hooks
cp integrations/claude-code/hooks/mw-record.py ~/.claude/hooks/mw-record.py
chmod +x ~/.claude/hooks/mw-record.py
```

Add the following entry to `~/.claude/settings.json`:

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "python3 \"$HOME/.claude/hooks/mw-record.py\""
          }
        ]
      }
    ]
  }
}
```

If the file already contains settings or hooks, merge this `PostToolUse` group
with them instead of replacing the file. Claude Code also supports project
hooks in `.claude/settings.json`; see the official
[hook locations](https://code.claude.com/docs/en/hooks#hook-locations) before
committing a hook that teammates will run.

### Install the skill

Copy the skill into Claude Code's personal skills directory:

```bash
mkdir -p ~/.claude/skills/memorywhale
cp integrations/claude-code/memorywhale/SKILL.md ~/.claude/skills/memorywhale/SKILL.md
```

Claude Code can load personal skills from
`~/.claude/skills/<skill-name>/SKILL.md`. To share the skill with one project
instead, copy it to `.claude/skills/memorywhale/SKILL.md` in that project.

## Verify

Check the binaries, hook, and MCP registration:

```bash
command -v mw-mcp
command -v mw-remember
python3 ~/.claude/hooks/mw-record.py --selftest
claude mcp get memorywhale
```

In Claude Code, run `/skills` and confirm that `memorywhale` is listed. To test
capture, ask Claude Code to run a unique successful command such as
`printf 'memorywhale-hook-check\n'`, then check the local store:

```bash
mw search "memorywhale-hook-check"
```

## How to use

The MCP server lets Claude read prior failures and explicitly save lessons. The
skill prompts Claude to consult those tools when a failure may have happened
before and to save the reason a fix worked. The hook independently records the
successful Bash commands Claude runs, even when no MCP tool is called.

The skill falls back to `mw context`, `mw search`, and `mw remember` when the
MCP server is not connected, so each component can be installed separately.

## Example prompt

> Use MemoryWhale to check whether I encountered a similar failure before.
> Explain the relevant saved evidence before suggesting a fix, then remember
> the root cause after it is verified.

## Automatic capture

The bundled [`hooks/mw-record.py`](hooks/mw-record.py) hook receives Claude
Code's documented `PostToolUse` JSON on standard input. For each `Bash` call,
it passes the command, working directory, output, and an exit status to
`mw-remember` with the note `agent:claude-code`; failed commands are recorded
with a nonzero exit status derived from the tool response's error flags.
Standard output and standard error are each capped at 20,000 characters before
being passed to `mw-remember`, which applies MemoryWhale's normal secret
redaction.

This hook is registered for `PostToolUse`, which Claude Code fires after every
Bash tool call, so both successful and failed commands are captured. Commands
run in an ordinary terminal are captured only through MemoryWhale's normal
terminal capture paths.

## Limitations

- The bundled hook captures only the Bash tool.
- Each output stream is truncated to 20,000 characters by the hook.
- User-level hooks and skills are local to the machine where they are installed.
- Guidance helps Claude choose when to use memory, but does not force a tool
  call on every failure.
- Secret redaction reduces accidental retention but is not a security boundary.

## Troubleshooting

- Run `command -v mw-mcp` and `command -v mw-remember` in the environment that
  launches Claude Code. Use absolute binary paths if its `PATH` differs from
  your shell.
- Run `python3 ~/.claude/hooks/mw-record.py --selftest` to check the copied hook.
- Validate `~/.claude/settings.json` as JSON and restart Claude Code after
  changing hook configuration.
- Run `claude mcp list` or use `/mcp` inside Claude Code to inspect the
  `memorywhale` server connection.
- Run `/skills` to check skill discovery. If you created the top-level skills
  directory during an active session, restart Claude Code.
- Run `mw doctor` to verify the MemoryWhale database and data directory. If
  `MEMORYWHALE_DATA_DIR` is set, ensure Claude Code and your shell use the same
  value.

## Remove integration

Remove the user-scoped MCP server:

```bash
claude mcp remove --scope user memorywhale
```

Delete only the MemoryWhale `PostToolUse` group from
`~/.claude/settings.json`, preserving any other hooks and settings. Then remove
the copied files:

```bash
rm -f ~/.claude/hooks/mw-record.py
rm -rf ~/.claude/skills/memorywhale
```

If you installed project-scoped copies, remove the corresponding entries under
`.claude/` instead. Removing the integration does not delete MemoryWhale's
database or any captured records.
