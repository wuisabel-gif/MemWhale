# Codewhale + MemoryWhale

[CodeWhale](https://github.com/Hmbown/CodeWhale) is an open-source terminal
coding agent written in Rust: TUI, headless `codewhale exec`, and a local web
client, with per-role model fleets. It is an MCP host. Local stdio MCP servers
configured in its MCP file become tools the agent can call, so `mw-mcp` plugs
in directly.

## Status

Verified against CodeWhale's official MCP documentation
(`docs/MCP.md` in the CodeWhale repository, `main` branch). CodeWhale is under
active development; check that file if a field or command changes.

The development branch also contains a native **declarative plugin bundle**
in [`plugin/`](plugin/). It uses Codewhale's existing Skills, Commands, and MCP
engines—not a native-code extension or a replacement for its preference memory.
The inspected host checkout is **0.9.12 at `d5beab040`**. Bundle/runtime
verification is recorded separately from model-backed usefulness; see
[`plugin/README.md`](plugin/README.md). No capture or automatic recall hook is
enabled by this base bundle.

## Requirements

- MemoryWhale installed with `mw-mcp` on `PATH`.
- CodeWhale installed (`npm install -g codewhale`, or see its
  `docs/INSTALL.md`).
- A model that supports tool calling (CodeWhale supports 30+ providers and
  local servers; tool calling is a model capability).

## Setup

### Native plugin (development bundle, macOS/Linux)

Use trusted, matching MemoryWhale helpers on PATH and explicitly choose the
store in the environment **before starting Codewhale**:

```bash
export MEMORYWHALE_DATA_DIR=/absolute/path/to/your/chosen/store
```

For the first check, choose a new temporary directory and seed only synthetic
notes. Do not connect the normal store or other private data sources to a test
agent. The bundle maps the named environment source; it does not read values
from a workspace `.env` or choose the default database if the variable is missing.

Place a reviewed copy of `integrations/codewhale/plugin/` in an unused
`$CODEWHALE_HOME/plugins/memorywhale/` directory (normally
`~/.codewhale/plugins/memorywhale/`), or the corresponding workspace
`.codewhale/plugins/memorywhale/` directory. Do not overwrite an existing bundle
or use a symlink. Inspect any user-scope bundle with the same name, which takes
precedence over a workspace copy.
New user plugin-state directories must be private (0700 on Unix); Codewhale
rejects group/world-accessible trust-state directories. Use the host's install
flow or create new private directories rather than changing existing paths blindly.

In the Codewhale TUI:

```text
/plugin list
/plugin validate memorywhale
/plugin show memorywhale
/plugin enable memorywhale
```

The first enable requests review. Inspect the bundle, local process authority,
data-directory source, and full content/capability hashes. Only then use the
exact `/plugin trust memorywhale <content-hash>.<capability-hash>` confirmation
shown by Codewhale, followed by `/plugin enable memorywhale`. Trust alone does
not enable it. Do not copy an invented or old receipt from documentation.

The server identity is `plugin-11-memorywhale-memory` on the inspected host;
verify the actual discovered name before using it. The plugin supplies the
`memorywhale:memorywhale-evidence` skill and explicit `/memorywhale-check` and
`/memorywhale-recall` guidance commands. These are model-facing instructions,
not permission enforcement or autonomous shell execution.

### Manual MCP configuration

CodeWhale reads MCP servers from `~/.codewhale/mcp.json` (the path can be
overridden with the `mcp_config_path` setting or the `DEEPSEEK_MCP_CONFIG`
environment variable; `~/.deepseek/mcp.json` is still read as a legacy
fallback when the CodeWhale file is absent). The file accepts a `servers`
object (a `mcpServers` key also works).

Add MemoryWhale as a local stdio server:

```json
{
  "servers": {
    "memorywhale": {
      "command": "mw-mcp",
      "args": [],
      "env": {},
      "disabled": false
    }
  }
}
```

To use a non-default MemoryWhale store, set the environment in the entry:

```json
{
  "servers": {
    "memorywhale": {
      "command": "mw-mcp",
      "args": [],
      "env": {
        "MEMORYWHALE_DATA_DIR": "/path/to/store"
      },
      "disabled": false
    }
  }
}
```

If `mw-mcp` is not on the `PATH` CodeWhale sees, use its absolute path as the
`command`. If other servers are already configured, add the `"memorywhale"`
key alongside them.

Alternatively, register it with the CLI instead of editing the file:

```bash
codewhale mcp add memorywhale --command "mw-mcp"
```

CodeWhale also offers an in-TUI manager: `/mcp` lists configured servers with
transport, status, and discovered tools; `/mcp add stdio`, `/mcp enable`,
`/mcp disable`, and `/mcp remove` manage entries.

## Verify

Confirm the server binary, then check CodeWhale discovered it:

```bash
command -v mw-mcp
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"server/discover","params":{"_meta":{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientCapabilities":{}}}}' | mw-mcp
codewhale mcp list
codewhale mcp tools memorywhale
```

`mcp list` should show `memorywhale`; `mcp tools memorywhale` should discover
the six MemoryWhale tools. After editing the MCP file, run `/mcp reload` in the
TUI (no restart needed). Headless `codewhale exec` surfaces do **not** hot
reload. Restart the process after config changes.

CodeWhale exposes MCP tools to the model under a server-name prefix
(`mcp_<server>_<tool>`), so the tools appear as `mcp_memorywhale_search_memory`
and so on. Ask:

> Use MemoryWhale to check whether I have encountered a similar build failure
> before. Search for `openssl` and explain which saved evidence is relevant
> before suggesting a fix.

The six tools:

- `recent_errors`
- `search_memory`
- `get_context`
- `remember`
- `similar_failures`
- `stats`

An empty store is valid. The tools return empty results, not errors.

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes |
| Automatic execution capture | No |
| Memory-use guidance | Native plugin skill/commands, or roles / constitution instructions |

### How to use

Use retrieval when a build, test, or deployment failure may have happened
before. Once the cause or fix is confirmed, use `remember` to save the lesson
so future CodeWhale sessions (or any other agent sharing the store) can find
it. CodeWhale roles and the constitution carry standing instructions, so a
role can be told to consult MemoryWhale before debugging and to record
verified fixes. See CodeWhale's `docs/FLEET.md` and `docs/CONFIGURATION.md`.

### Automatic capture

MCP access is not automatic execution capture. Commands CodeWhale runs are
recorded by MemoryWhale only through the normal capture paths: `mw-run --`,
`mw-remember`, `mw --notes "project:…"` session recording, or an installed
shell hook. CodeWhale has its own lifecycle hooks (`docs/HOOKS.md`), but none
is verified to write into MemoryWhale; do not assume capture without one of
the paths above.

### Limitations

- The model must support tool calling; not every local model does.
- MCP tools consume context; CodeWhale's docs recommend enabling only the
  servers you need.
- Secret redaction on capture reduces accidental retention but is not a
  security boundary.

### Security

`mw-mcp` is read-mostly: of its six tools, `remember` is the only one that
writes, and it saves a single note. The other five only read. Review the
canonical [local stdio trust model](../../docs/reference/mcp.md#trust-model)
before connecting a store that contains sensitive output. Note that these MCP
tool permissions are separate from process-level filesystem permissions: the
OS user that spawns `mw-mcp` remains the file-level trust boundary.
`MEMORYWHALE_DATA_DIR` selects a store; it is not an access-control mechanism.

## Example prompt

> Use MemoryWhale to check whether I have encountered a similar build failure
> before. Search for `openssl` and explain which saved evidence is relevant
> before suggesting a fix.

## Troubleshooting

- Run `command -v mw-mcp` from the environment CodeWhale launches from; use
  an absolute path in `command` if they differ.
- Run `codewhale mcp list` and `codewhale mcp validate` to check the
  entry parses and the server connects.
- Run `/mcp reload` in the TUI after any edit; restart headless processes.
- Confirm the file is valid JSON at `~/.codewhale/mcp.json` (or your
  `mcp_config_path` / `DEEPSEEK_MCP_CONFIG` override).
- Run `mw doctor` to verify the MemoryWhale data directory and database.

## Uninstall

For the native bundle, use `/plugin disable memorywhale` to stop contributions,
or `/plugin revoke memorywhale` to remove trust. Use the host's reviewed uninstall
flow for the exact source you installed; do not delete unrelated plugin files
or native-memory data. Disabling/removing this plugin does not erase MemoryWhale
records. Capture/automatic-recall components, when installed separately, must
be disabled separately.

For manual MCP setup:

Remove the `"memorywhale"` entry from `~/.codewhale/mcp.json` (or run
`codewhale mcp remove memorywhale`, or `/mcp remove memorywhale` in the
TUI), then run `/mcp reload` in each TUI session. Running headless
`codewhale exec` processes do not hot-reload MCP configuration. Restart them
after removing the entry. This does not delete the MemoryWhale database; use
`mw rm` or the documented retention commands for data lifecycle.
