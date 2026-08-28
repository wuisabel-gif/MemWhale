# Rho + MemoryWhale

[Rho](https://github.com/matthewyjiang/rho) can use MemoryWhale in three
independent ways: `mw-mcp` provides native memory tools, an `after_tool_use`
hook records bash and powershell calls when the payload includes them, and a
skill teaches Rho when to search or save debugging memory.

## Status

Verified against Rho's [hooks](https://matthewyjiang.github.io/rho/hooks),
[skills](https://matthewyjiang.github.io/rho/skills), and
[MCP](https://matthewyjiang.github.io/rho/integrations/mcp) documentation.
The hook and skill are optional repository-provided components.

- `rho mcp list` lists configured MCP servers
- `rho mcp show memorywhale` shows the MemoryWhale entry
- `/hooks` in the TUI reloads hooks and prints the spawn contract

## Requirements

- MemoryWhale installed with `mw-mcp` and `mw-remember` on `PATH`.
- Rho installed on Linux, macOS, or Windows.
- A local checkout of this repository to copy the skill (only for manual
  setup; `mw integrate rho` needs no checkout).

## Capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes |
| Automatic execution capture | Yes, optional `after_tool_use` hook for bash and powershell |
| Memory-use guidance | Yes, optional skill |

## Setup

From any machine with MemoryWhale installed:

```bash
mw integrate rho
```

That installs the skill into `~/.rho/` (or `$RHO_HOME`), points the
MemoryWhale hook in `hooks.toml` at `mw-remember --from-hook rho`, and registers
`mw-mcp` in `config.toml`. Restart Rho afterward. To undo:
`mw integrate rho --revert`.

Stdio is the default. To point Rho at `mw-serve`'s `POST /mcp` endpoint
instead (Rho 2.2.0+ `streamable_http` transport; one JSON-RPC object per
POST, not SSE):

```bash
mw integrate rho --http
# remote Jetson, token from the server:
mw integrate rho --http http://192.168.1.42:7071/mcp --token "$(ssh jetson mw-serve --lan --print-token)"
```

Loopback HTTP needs no token. Pass `--token` if the loopback server itself
was started with a token. A LAN `http://` URL sets `allow_insecure_http =
true` and `headers_from_env = { Authorization = "MEMORYWHALE_AUTHORIZATION" }`.
The raw token is stored as `Bearer …` in `mcp-authorization` under the
MemoryWhale data directory, not in `config.toml`. That directory is
`$MEMORYWHALE_DATA_DIR` when set, otherwise the platform default
(`~/.local/share/MemoryWhale/` on Linux, `~/Library/Application
Support/MemoryWhale/` on macOS). The installer prints the export line with
the resolved path. For a manual export when the variable is set:

```bash
export MEMORYWHALE_AUTHORIZATION="$(tr -d '\n' < "$MEMORYWHALE_DATA_DIR/mcp-authorization")"
```

The skill lives in `crates/mw-cli/integrate/` so it ships inside the published
package. Capture uses the `mw-remember` binary, not a copied script.

### Manual setup

If you prefer to install by hand from a repository checkout, run the file-copy
commands below from the MemoryWhale repository root.

#### Connect the MCP server

Add the server to `$RHO_DIR/config.toml`. Rho requires `transport`:

```bash
RHO_DIR="${RHO_HOME:-$HOME/.rho}"
```

```toml
[mcp.servers.memorywhale]
transport = "stdio"
command = "mw-mcp"
```

For a non-default store, add the environment:

```toml
[mcp.servers.memorywhale]
transport = "stdio"
command = "mw-mcp"
env = { MEMORYWHALE_DATA_DIR = "/path/to/store" }
```

If `mw-mcp` is not on the `PATH` Rho sees, use its absolute path as `command`.
This gives Rho the six MemoryWhale tools: `recent_errors`, `search_memory`,
`get_context`, `remember`, `similar_failures`, and `stats`.

An explicit `rho --config` file replaces `~/.rho/config.toml` for that run.
`mw integrate rho` only edits the default user file.

#### Install the capture hook

Add this block to `$RHO_DIR/hooks.toml`. Hook policy lives in that file, not in
`config.toml`. Use the absolute path to `mw-remember`
(`command -v mw-remember`):

```toml
version = 1

[[hook]]
id = "memorywhale-record"
on = "after_tool_use"
tools = ["bash", "powershell"]
command = ["/home/you/.cargo/bin/mw-remember", "--from-hook", "rho"]
timeout = "15s"
```

Rho runs hook programs as argv, not as a shell string. If `hooks.toml` already
has other `[[hook]]` entries, append this one. Do not use `before_tool_use`
for capture: that event is fail-closed, so a broken hook would deny the tool
call.

#### Install the skill

```bash
mkdir -p "$RHO_DIR/skills/memorywhale"
cp crates/mw-cli/integrate/SKILL.md "$RHO_DIR/skills/memorywhale/SKILL.md"
```

Rho loads personal skills from `$RHO_DIR/skills/<name>/SKILL.md`. The directory
name must match the skill `name`. To share the skill with other agents that
use the same layout, copy it to `~/.agents/skills/memorywhale/` instead.

## Verify

```bash
RHO_DIR="${RHO_HOME:-$HOME/.rho}"
command -v mw-mcp
command -v mw-remember
rho mcp list
rho mcp show memorywhale
```

In Rho, run `/mcp` and confirm `memorywhale` is listed, and `/skills` for the
skill. `/hooks` should show `user:memorywhale-record` as active.

## How to use

The MCP server lets Rho read prior failures and explicitly save lessons. The
skill prompts Rho to consult those tools when a failure may have happened
before and to save the reason a fix worked. The hook independently records
bash and powershell tool calls when the event payload includes command text.

The skill falls back to `mw context`, `mw search`, and `mw remember` when the
MCP server is not connected, so each component can be installed separately.

## Example prompt

> Use MemoryWhale to check whether I encountered a similar failure before.
> Explain the relevant saved evidence before suggesting a fix, then remember
> the root cause after it is verified.

## Automatic capture

`mw-remember --from-hook rho` receives Rho's hook JSON on standard input. It
matches `after_tool_use` for `bash` and `powershell`, then records command
text when the payload includes it, along with working directory and exit
status when available.

Rho's current `after_tool_use` payload reports tool name, status, failure
kind and message, and duration. It does not include the shell command, process
exit code, or stdout. The hook records failed or unavailable calls even without
command text, using a sentinel command (`[rho:after_tool_use]`) plus status and
failure metadata in notes and stderr so the event is kept without inventing a
shell command. It reads `capability.shell_command` if a later schema adds it.
Successful calls with no command text are skipped so the store is not filled
with bare tool-name rows. Upstream truncation reported in `bounds` omits
affected fields and adds a marker instead of presenting shortened text as
complete evidence.

Commands run in an ordinary terminal are captured only through MemoryWhale's
normal terminal capture paths. MCP access alone is not automatic capture.

## How Rho sessions and MemoryWhale differ

Rho keeps its own saved session transcripts in `~/.rho/sessions/` and may
compact or summarize them. MemoryWhale stores durable evidence in its own
SQLite database at `<data_local>/MemoryWhale/memorywhale.sqlite3`. The two are
independent: compaction in Rho does not touch MemoryWhale data, and
MemoryWhale data survives Rho restarts and machine transfers. Use `mw agent`
or `mw context` to bridge a Rho session into MemoryWhale when you want the
evidence to outlive the current session.

## Limitations

- The bundled hook captures only bash and powershell.
- `before_tool_use` is not used, because a crash or timeout there denies the
  tool call.
- Current Rho `after_tool_use` events do not include command text or stdout.
- User-level hooks and skills are local to the machine where they are installed.
- Guidance helps Rho choose when to use memory, but does not force a tool call
  on every failure.
- Secret redaction reduces accidental retention but is not a security boundary.

## Troubleshooting

- Run `command -v mw-mcp` and `command -v mw-remember` in the environment that
  launches Rho. Use absolute binary paths if its `PATH` differs from your
  shell.
- Validate `$RHO_DIR/hooks.toml` and `$RHO_DIR/config.toml`. Unknown keys are a
  load error in both files.
- Run `rho mcp list` or `/mcp` inside Rho to inspect the `memorywhale` server.
- Run `/hooks` to confirm the MemoryWhale hook is active. A session that
  started without observational hooks needs a restart to pick a new one up.
- Run `/skills` to check skill discovery.
- Run `mw doctor` to verify the MemoryWhale database and data directory. If
  `MEMORYWHALE_DATA_DIR` is set, put it in the server's `env` block, not only
  in an unrelated terminal.
- `RHO_HOME` moves the whole Rho directory, including `hooks.toml`,
  `config.toml`, and `skills/`.

## Remove integration

```bash
mw integrate rho --revert
```

That removes the hook, skill, MemoryWhale entry from `hooks.toml`, and the
`[mcp.servers.memorywhale]` table from `config.toml`. It does not delete
MemoryWhale's database or any captured records.

Manual removal (if you installed by hand):

Delete `[mcp.servers.memorywhale]` from `$RHO_DIR/config.toml`. Delete the
`[[hook]]` block whose `id` is `memorywhale-record` from `$RHO_DIR/hooks.toml`,
preserving any other hooks. Then remove the skill (and any leftover Python hook from an older install):

```bash
RHO_DIR="${RHO_HOME:-$HOME/.rho}"
rm -rf "$RHO_DIR/skills/memorywhale"
rm -f "$RHO_DIR/hooks/mw-record.py"
```

Restart Rho. Removing the integration does not delete MemoryWhale's database
or any captured records.
