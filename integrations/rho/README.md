# Rho + MemoryWhale

[Rho](https://github.com/matthewyjiang/rho) can use MemoryWhale in three
independent ways: `mw-mcp` provides native memory tools, an `after_tool_use`
hook records bash and powershell calls when the payload includes them, and a
skill teaches Rho when to search or save debugging memory.

Rho is a separate open-source coding-agent project that MemoryWhale's maintainer
actively uses and contributes to. That hands-on experience led to
MemoryWhale's dedicated Rho integration, which supports MCP memory access,
optional command-capture hooks, and memory-use guidance through a Rho skill.

## Status

Verified against Rho's [hooks](https://matthewyjiang.github.io/rho/hooks),
[skills](https://matthewyjiang.github.io/rho/skills), and
[MCP](https://matthewyjiang.github.io/rho/integrations/mcp) documentation.
The hook and skill are optional repository-provided components.

Native **Rho 2.10.0** was checked on **September 12, 2026** with fresh/resumed
retrieval, skill loading, execution/denial capture, and an unsaved interactive
session. See the [versioned verification report](../../docs/research/rho-2.10-compatibility.md)
and live-derived regression fixtures. The run-timeout case exposed a delivery
gap; this is not a claim of complete cancellation capture.

- `rho mcp list` lists configured MCP servers
- `rho mcp show memorywhale` shows the MemoryWhale entry
- `/hooks` in the TUI reloads hooks and prints the spawn contract

## Requirements

- MemoryWhale installed with `mw-mcp` and `mw-remember` on `PATH`.
- Rho installed on Linux, macOS, or Windows.
- A local checkout of this repository to copy the skill (only for manual
  setup; `mw integrate rho` needs no checkout).

## Setup

For the normal user profile (`~/.rho`), with MemoryWhale installed:

```bash
mw integrate rho
```

That installs the skill into `~/.rho/`, points the
MemoryWhale hook in `hooks.toml` at `mw-remember --from-hook rho`, and registers
`mw-mcp` in `config.toml`. Restart Rho afterward. To undo:
`mw integrate rho --revert`.

For a custom `RHO_HOME` on Rho 2.10.0, use the manual setup below and its
project-local skill path. The legacy installer writes a skill inside its selected
profile directory, which Rho may not discover; its custom-profile warning is not
a claim of successful runtime skill loading. It does not silently redirect
that write into your global HOME or a project.

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

With Rho 2.10.0, do not assume a custom `RHO_HOME` relocates loose user skills.
Project `.agents/skills/memorywhale/SKILL.md` was verified separately. Check
actual skill discovery after installing into a custom profile.

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
env = ["MEMORYWHALE_DATA_DIR"]
```

Rho runs hook programs as argv, not as a shell string. If `hooks.toml` already
has other `[[hook]]` entries, append this one. Do not use `before_tool_use`
for capture: that event is fail-closed, so a broken hook would deny the tool
call.

For a custom store, start Rho with `MEMORYWHALE_DATA_DIR` set and explicitly
allow it through the hook environment as above; separately set the MCP server's
`env` to the same location. A variable in an unrelated terminal or in only the
MCP configuration does not configure the capture subprocess.

#### Install the skill

```bash
mkdir -p .agents/skills/memorywhale
cp -n crates/mw-cli/integrate/SKILL.md .agents/skills/memorywhale/SKILL.md
```

This project-local path was verified with a custom Rho profile. If a skill file
already exists, `cp -n` leaves it unchanged: inspect it rather than assuming it
was updated. The directory name must match the skill `name`.

Personal alternatives are `~/.rho/skills/memorywhale/` and the shared
`~/.agents/skills/memorywhale/`; these are HOME-based and affect more than one
profile/project. Existing personal or built-in skills may take precedence over
a project copy. Verify the selected source with the client.

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

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes |
| Automatic execution capture | Yes, optional `after_tool_use` hook for bash and powershell |
| Memory-use guidance | Yes, optional skill |

### How to use

The MCP server lets Rho read prior failures and explicitly save lessons. The
skill prompts Rho to consult those tools when a failure may have happened
before and to save the reason a fix worked. The hook independently records
bash and powershell tool calls when the event payload includes command text.

The skill falls back to `mw context`, `mw search`, and `mw remember` when the
MCP server is not connected, so each component can be installed separately.

### Automatic capture

`mw-remember --from-hook rho` receives Rho's hook JSON on standard input. It
matches `after_tool_use` for `bash` and `powershell`, then records command
text and working directory when the payload includes them. Tool-status notes
are not numeric process exit status.

The observed Rho 2.10.0 schema-2 payload includes
`payload.capability.shell_command` and `working_directory`, plus tool status,
failure information, and duration. It does not provide a numeric process exit
code or a stdout field. The hook records failed or unavailable calls even without
command text, using a sentinel command (`[rho:after_tool_use]`) plus status and
failure metadata in notes and stderr so the event is kept without inventing a
shell command. The existing parser already reads the capability-based fields.
Successful calls with no command text are skipped so the store is not filled
with bare tool-name rows. Upstream truncation reported in `bounds` omits
affected fields and adds a marker instead of presenting shortened text as
complete evidence.

Use `mw search ... agent:rho` to inspect these records. Numeric-exit-based
`recent_errors` and error counts are not a complete inventory of failed Rho
tool calls when the stored exit code is unknown.

Commands run in an ordinary terminal are captured only through MemoryWhale's
normal terminal capture paths. MCP access alone is not automatic capture.

### How Rho sessions and MemoryWhale differ

Rho keeps its own saved session transcripts in `~/.rho/sessions/` and may
compact or summarize them. MemoryWhale stores durable evidence in its own
SQLite database at `<data_local>/MemoryWhale/memorywhale.sqlite3`. The two are
independent: compaction in Rho does not touch MemoryWhale data, and
MemoryWhale data survives Rho restarts and machine transfers. Use `mw agent`
or `mw context` to bridge a Rho session into MemoryWhale when you want the
evidence to outlive the current session.

### Limitations

`rho --no-save` only disables Rho conversation/prompt-history persistence.
The live test confirmed that an explicitly enabled MemoryWhale capture hook
still records commands. It is not a privacy or capture-off mode. To keep MCP
access without capture, remove only the MemoryWhale `after_tool_use` hook from
the selected hooks file and reload `/hooks` to verify removal. Existing memory
is not deleted. Full `mw integrate rho --revert` removes the broader integration.

- The bundled hook captures only bash and powershell.
- `before_tool_use` is not used, because a crash or timeout there denies the
  tool call.
- Rho 2.10.0 command text is available, but stdout and numeric exit-code fields
  are not. Do not infer an exit code from a human-readable failure message.
- A tested run timeout delivered `session_failed` without `after_tool_use` for
  the interrupted command. Do not assume all cancellation paths produce a row.
- User-level hooks and skills are local to the machine where they are installed.
- Guidance helps Rho choose when to use memory, but does not force a tool call
  on every failure.
- Secret redaction reduces accidental retention but is not a security boundary.

## Example prompt

> Use MemoryWhale to check whether I encountered a similar failure before.
> Explain the relevant saved evidence before suggesting a fix, then remember
> the root cause after it is verified.

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
- Run `mw doctor` to verify the MemoryWhale database and data directory, and
  to see Rho MCP, hook, and skill status separately from MemoryWhale's own
  health. For a custom store, set `MEMORYWHALE_DATA_DIR` in the MCP server's
  `env` block **and** the capture hook's `env` allowlist, and export its matching
  value in the environment that starts Rho. Otherwise MCP and capture can use
  different stores. An unrelated terminal's environment is not sufficient.
- `RHO_HOME` selects configuration/state locations, but Rho 2.10.0 loose-skill
  discovery remains HOME/project-based. Inspect `/skills` rather than infer
  discovery from files existing under a custom home.

## Uninstall

```bash
mw integrate rho --revert
```

That removes the hook, skill, MemoryWhale entry from `hooks.toml`, and the
`[mcp.servers.memorywhale]` table from `config.toml`. It does not delete
MemoryWhale's database or any captured records.

Manual removal (if you installed by hand):

Delete `[mcp.servers.memorywhale]` from `$RHO_DIR/config.toml`. Delete the
`[[hook]]` block whose `id` is `memorywhale-record` from `$RHO_DIR/hooks.toml`,
preserving any other hooks. Remove only the skill copy you installed. For the
project-local example above, after reviewing that file:

```bash
rm -- .agents/skills/memorywhale/SKILL.md
rmdir -- .agents/skills/memorywhale
```

Restart Rho. Removing the integration does not delete MemoryWhale's database
or any captured records.
