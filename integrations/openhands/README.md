# OpenHands Software Agent SDK + MemoryWhale

## Status

Checked against the OpenHands
[`software-agent-sdk`](https://github.com/OpenHands/software-agent-sdk) release
`openhands-sdk` **1.58.2** (tag `v1.58.2`, commit
`d2ee0cdea60825608c2f8906eb54e1d4d264fb42`) on October 27, 2026, including its
example
[`examples/01_standalone_sdk/07_mcp_integration.py`](https://github.com/OpenHands/software-agent-sdk/blob/d2ee0cdea60825608c2f8906eb54e1d4d264fb42/examples/01_standalone_sdk/07_mcp_integration.py).
OpenHands is a Python SDK, not a CLI: it constructs `MCPServer(command=..., args=...,
env=...)`, passes a `mcp_config` dict through `Agent(mcp_config=...)`, and runs a
normal `Conversation`. This integration targets that explicit SDK configuration,
not the OpenHands Agent Canvas frontend.

This guide is scoped to a **local stdio** child process reachable from the same
runtime that runs the agent. With the pinned SDK, native tool discovery and
dispatch pass offline on synthetic stores, both through the SDK's MCP client and
through an `Agent` + `Conversation` driven by the SDK's scripted `TestLLM` (see
[VERIFICATION.md](VERIFICATION.md)). A run with a real model is still pending.
Container/VM/remote Agent Server runtimes are **not verified here** — see the
deployment note in Setup.

## Requirements

- The OpenHands `software-agent-sdk` from a pinned, reviewed release or revision,
  plus a configured model/provider for normal use.
- MemoryWhale `mw-mcp` on an absolute path and an explicitly selected local store.
  Build the helpers from the MemoryWhale repository root:
  `cargo build --release -p memorywhale-cli --bins`. On macOS, re-sign a copied
  binary (`codesign --force --sign - <path>`) or it is `Killed: 9` — see `docs/project/DEBUG.md`.
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

- `mcp_config` is a **dict** of `name -> MCPServer`, not a list. With one
  server the agent sees the bare tool names (`search_memory`). With two or more,
  each name is prefixed with its key (`memorywhale_search_memory`).
- `MCPServer.env` is typed `dict[str, SecretStr]`, so wrap the store path in
  `SecretStr` even though it is not a secret; a bare `str` is rejected.
- Use an **absolute** `command` path and an **absolute**, dedicated store path.
  Review the command and env before trusting them.
- **Always set `MEMORYWHALE_DATA_DIR` in `MCPServer.env`.** The SDK starts stdio
  servers with a small default environment plus `env`, so the variable is not
  inherited from your Python process. Without it, `mw-mcp` opens the default
  store under `HOME`, which is your real one.

To keep the agent from writing memory, filter `remember` out of its tools:

```python
agent = Agent(
    llm=llm,
    tools=[],
    mcp_config=mcp_config,
    filter_tools_regex=r"^(memorywhale_)?(recent_errors|search_memory|get_context|similar_failures|stats)$",
)
```

The SDK enforces this filter, not `mw-mcp`. Any other client that starts the
same server can still call `remember`, and a store directory is not an access
boundary: a tool that runs commands as your user can read any store.

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
listing all six tools.

Then run the native SDK checks. They need the pinned SDK and use only temporary
synthetic stores and a temporary `HOME`:

```bash
cargo build -p memorywhale-cli --bins
uv run --no-project --with-requirements scripts/verification/openhands-mcp/requirements.txt \
  python -I scripts/verification/openhands-mcp/test_sdk.py -v
```

They cover discovery and dispatch, a fix handoff that is saved only after a
fresh passing test run, the pending review flow, and isolation. See
[`scripts/verification/openhands-mcp`](../../scripts/verification/openhands-mcp/README.md).
A save acknowledgement is not approval: an agent note stays out of retrieval
until it is approved.

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Local stdio: native SDK discovery and dispatch verified offline with a scripted model; real-model run and container/remote runtimes pending |
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
