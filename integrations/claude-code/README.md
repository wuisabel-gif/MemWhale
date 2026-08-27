# Claude Code + MemoryWhale

Claude Code can use MemoryWhale in three independent ways: `mw-mcp` provides
native memory tools, `PostToolUse` and `PostToolUseFailure` hooks record the
Bash commands Claude runs (successes and failures alike), and
a skill teaches Claude when to search or save debugging memory.

## Status

Verified against Claude Code's official [MCP](https://code.claude.com/docs/en/mcp),
[hooks](https://code.claude.com/docs/en/hooks), and
[skills](https://code.claude.com/docs/en/skills) documentation on 2026-08-21.
The hook and skill are optional repository-provided components.

## Requirements

- MemoryWhale installed with `mw-mcp` and `mw-remember` on `PATH`.
- Claude Code installed on Linux, macOS, or Windows through WSL.
- A local checkout of this repository to copy the skill (only for manual
  setup; `mw integrate claude` needs no checkout).

## Capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes |
| Automatic execution capture | Yes, optional `PostToolUse` and `PostToolUseFailure` hooks for Bash calls |
| Memory-use guidance | Yes, optional skill |

## Setup

From any machine with MemoryWhale installed:

```bash
mw integrate claude
```

That installs the skill into `~/.claude/`, points the `PostToolUse` and
`PostToolUseFailure` Bash hooks at `mw-remember --from-hook claude` in
`~/.claude/settings.json`, and registers `mw-mcp` when the Claude Code CLI is
on your PATH. Restart Claude Code afterward. To undo: `mw integrate claude --revert`.

The skill lives in `crates/mw-cli/integrate/` so it ships inside the published
package. Capture uses the `mw-remember` binary, not a copied script.

### Manual setup

If you prefer to install by hand from a repository checkout, run the file-copy
commands below from the MemoryWhale repository root.

#### Connect the MCP server

Register `mw-mcp` at user scope so it is available in every local project:

```bash
claude mcp add --scope user --transport stdio memorywhale -- mw-mcp
```

This gives Claude Code the six MemoryWhale tools: `recent_errors`,
`search_memory`, `get_context`, `remember`, `similar_failures`, and `stats`.
The command follows Claude Code's documented
[local stdio server syntax](https://code.claude.com/docs/en/mcp#option-3-add-a-local-stdio-server).

#### Install the capture hook

Add the following entry to `~/.claude/settings.json`. Use the absolute path
to `mw-remember` (`command -v mw-remember`):

```json
{
  "hooks": {
    "PostToolUse": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "\"$HOME/.cargo/bin/mw-remember\" --from-hook claude"
          }
        ]
      }
    ],
    "PostToolUseFailure": [
      {
        "matcher": "Bash",
        "hooks": [
          {
            "type": "command",
            "command": "\"$HOME/.cargo/bin/mw-remember\" --from-hook claude"
          }
        ]
      }
    ]
  }
}
```

If the file already contains settings or hooks, merge these Bash hook groups
with them instead of replacing the file. Claude Code also supports project
hooks in `.claude/settings.json`; see the official
[hook locations](https://code.claude.com/docs/en/hooks#hook-locations) before
committing a hook that teammates will run.

#### Install the skill

Copy the skill into Claude Code's personal skills directory:

```bash
mkdir -p ~/.claude/skills/memorywhale
cp crates/mw-cli/integrate/SKILL.md ~/.claude/skills/memorywhale/SKILL.md
```

Claude Code can load personal skills from
`~/.claude/skills/<skill-name>/SKILL.md`. To share the skill with one project
instead, copy it to `.claude/skills/memorywhale/SKILL.md` in that project.

## Verify

Check the binaries, hook, and MCP registration:

```bash
command -v mw-mcp
command -v mw-remember
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

`mw-remember --from-hook claude` receives Claude Code's hook JSON on standard input.
For each `Bash` call, it records the command, working directory, output, and
an exit status with the note `agent:claude-code`. Successful calls arrive on
`PostToolUse`; failed calls arrive on `PostToolUseFailure` with the error in a
top-level `error` field. Standard output and standard error are each capped at
20,000 characters before secret redaction.

The hook is registered for both events so successful and failed Bash commands
are captured. Commands run in an ordinary terminal are captured only through
MemoryWhale's normal terminal capture paths.

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

```bash
mw integrate claude --revert
```

That removes the hook, skill, and MemoryWhale entry from `~/.claude/settings.json`,
and unregisters the user-scoped MCP server when the Claude Code CLI is available.
It does not delete MemoryWhale's database or any captured records.

Manual removal (if you installed by hand):

Remove the user-scoped MCP server:

```bash
claude mcp remove --scope user memorywhale
```

Delete the MemoryWhale `PostToolUse` and `PostToolUseFailure` Bash groups from
`~/.claude/settings.json`, preserving any other hooks and settings. Then remove
the skill:

```bash
rm -rf ~/.claude/skills/memorywhale
rm -f ~/.claude/hooks/mw-record.py
```

If you installed project-scoped copies, remove the corresponding entries under
`.claude/` instead. Removing the integration does not delete MemoryWhale's
database or any captured records.
