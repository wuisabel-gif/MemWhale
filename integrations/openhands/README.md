# OpenHands Software Agent SDK + MemoryWhale

## Status

Configuration and boundaries were checked against the OpenHands
[`software-agent-sdk`](https://github.com/OpenHands/software-agent-sdk) example
[`examples/01_standalone_sdk/07_mcp_integration.py`](https://github.com/OpenHands/software-agent-sdk/blob/bd88f050259276978dc31541d8099d98e4994428/examples/01_standalone_sdk/07_mcp_integration.py)
at revision `bd88f050259276978dc31541d8099d98e4994428` on September 26, 2026.
OpenHands is a Python SDK, not a CLI: it constructs `MCPServer(command=..., args=...,
env=...)`, passes a `mcp_config` dict through `Agent(mcp_config=...)`, and runs a
normal `Conversation`. This integration targets that explicit SDK configuration,
not the OpenHands Agent Canvas frontend.

This guide is scoped to a **local stdio** child process reachable from the same
runtime that runs the agent. The MemoryWhale transport was verified directly (see
[VERIFICATION.md](VERIFICATION.md)); a native OpenHands SDK agent run that
discovers and dispatches the tools is pending. Container/VM/remote Agent Server
runtimes are **not verified here** — see the deployment note in Setup.

## Requirements

- The OpenHands `software-agent-sdk` from a pinned, reviewed release or revision,
  plus a configured model/provider for normal use.
- MemoryWhale `mw-mcp` on an absolute path and an explicitly selected local store.
  Build the helpers from the MemoryWhale repository root:
  `cargo build --release -p memorywhale-cli --bins`. On macOS, re-sign a copied
  binary (`codesign --force --sign - <path>`) or it is `Killed: 9` — see `DEBUG.md`.
- macOS or Linux for the documented local stdio path. Windows is not verified here.
- The `mw-mcp` binary **and its store must be reachable from the process that
  actually runs the agent**, which is not necessarily your host — see Setup.

## Setup

`MCPServer` lives in `openhands.sdk.mcp`. Build the config in Python; there is no
JSON/TOML file. The minimal wiring (placeholders only — full file at
[`mcp_config.example.py`](mcp_config.example.py)):

```python
from pydantic import SecretStr

from openhands.sdk import LLM, Agent, Conversation
from openhands.sdk.mcp import MCPServer

mcp_config = {
    "memorywhale": MCPServer(
        command="/absolute/path/to/mw-mcp",
        args=[],
        env={
            "MEMORYWHALE_DATA_DIR": SecretStr("/absolute/path/to/a-memorywhale-store"),
        },
    ),
}

llm = LLM(usage_id="agent", model="<your-model>", api_key=SecretStr("<your-key>"))
agent = Agent(llm=llm, tools=[], mcp_config=mcp_config)

conversation = Conversation(agent=agent, workspace=".")
conversation.send_message("Use MemoryWhale to check whether I hit this error before.")
conversation.run()
```

Notes on the current SDK contract:

- `mcp_config` is a **dict** of `name -> MCPServer`, not a list. The key
  (`"memorywhale"`) namespaces the tools the agent sees.
- `MCPServer.env` is typed `dict[str, SecretStr]`, so wrap the store path in
  `SecretStr` even though it is not a secret; a bare `str` is rejected.
- Use an **absolute** `command` path and an **absolute**, dedicated store path.
  Review the command and env before trusting them.

### Local vs container/remote deployment

OpenHands may run the agent in a local workspace, an ephemeral Docker workspace,
a VM, or a remote Agent Server. `MCPServer(command=...)` launches the command
**inside whatever runtime executes the agent**. The paths above are correct only
when a local process runs the agent.

- **Local SDK (supported here):** the paths resolve on the same machine that runs
  the SDK. This is what the Setup snippet and [VERIFICATION.md](VERIFICATION.md)
  cover.
- **Container / VM / remote Agent Server (not verified here):** the host
  filesystem is generally **not** visible inside the runtime. You must make both
  `mw-mcp` and its SQLite store reachable there — e.g. bake the binary into the
  image and bind-mount or volume-mount the store, then set `command`/
  `MEMORYWHALE_DATA_DIR` to the in-runtime paths. This repository does not verify
  any container or remote path.

## Verify

First run the direct MemoryWhale transport check with an isolated store (this
mirrors what [VERIFICATION.md](VERIFICATION.md) records):

```bash
STORE="$(mktemp -d)"
printf '%s\n' \
  '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"openhands-sdk-verify","version":"0.0.0"}}}' \
  '{"jsonrpc":"2.0","method":"notifications/initialized"}' \
  '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}' \
  | MEMORYWHALE_DATA_DIR="$STORE" /absolute/path/to/mw-mcp
```

Expect `serverInfo` `{"name":"memorywhale", ...}` and a `tools/list` result
listing all six tools. For native SDK validation, run the example above against a
fresh synthetic store, confirm the agent lists the six `memorywhale`-namespaced
tools through its MCP registry, ask it to `search_memory` for a known marker, then
ask it to `remember` a separate lesson. Confirm the proposed note stays pending in
MemoryWhale's ordinary review flow; a save acknowledgement is not approval.

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Local stdio transport verified; native SDK tool discovery/dispatch and container/remote runtimes pending |
| Automatic execution capture | No; the SDK exposes MCP tools, it does not record executed commands |
| Memory-use guidance | Example prompt only; no OpenHands-native skill/rule mechanism claimed |

The six MCP tools are `recent_errors`, `search_memory`, `get_context`,
`remember`, `similar_failures`, and `stats`. Local stdio and a local database do
not make model inference offline; retrieved excerpts enter the configured model's
context.

## Example prompt

> Use MemoryWhale to check whether I encountered a similar failure before. Report
> only recorded evidence, separate observations from hypotheses, and do not save a
> lesson unless I ask.

For an explicit write:

> Save this proposed debugging lesson to MemoryWhale, including the failed command,
> the fix, and the verification command. Do not treat it as verified until I
> approve it in MemoryWhale.

## Troubleshooting

- If the agent reports no MemoryWhale tools, verify the absolute `command` path,
  executable permissions, the `SecretStr`-wrapped `env`, and that the store path
  is writable. On macOS, re-sign a copied binary (`codesign --force --sign -`).
- If tools are missing only inside a container or remote backend, the host paths
  do not exist there — see the deployment note in Setup; this is expected and not
  verified by this guide.
- If the server connects but calls fail, run the direct `mw-mcp` handshake above
  and confirm the child `env` points to the intended store.
- Keep normal approvals for `remember`; do not grant a broad wildcard that hides
  an unreviewed write.
- If a call response is lost, treat the write outcome as unknown and inspect the
  store before retrying.

## Uninstall

Remove the `"memorywhale"` entry from your `mcp_config` dict (or delete the
config snippet). No global OpenHands configuration is written outside your
Python code. The configured MemoryWhale database created or updated by
`mw-mcp` remains on disk after you remove the entry; delete it yourself only
if you want to discard that memory. Do not delete the MemoryWhale database or
unrelated MCP entries as part of uninstalling.
