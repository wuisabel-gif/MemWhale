# OpenHands SDK verification for mw-mcp

These checks run MemoryWhale's `mw-mcp` under the real OpenHands Software Agent
SDK, on temporary synthetic stores. They never open the default MemoryWhale
store: importing `fixture.py` points `HOME` at a fresh temporary directory and
drops every `MEMORYWHALE_*` and `XDG_*` variable before the SDK loads, and the
stores, the throwaway git repository, and that `HOME` are deleted at exit.

Pinned SDK: `openhands-sdk==1.58.2` from PyPI (tag `v1.58.2`, commit
`d2ee0cdea60825608c2f8906eb54e1d4d264fb42`, released October 25, 2026). Only
the SDK is pinned; the last run resolved `fastmcp` 3.4.11, `mcp` 1.30.0, and
`litellm` 1.105.4.

## Run

From the repository root:

```bash
cargo build -p memorywhale-cli --bins
uv run --no-project --with-requirements scripts/verification/openhands-mcp/requirements.txt \
  python -I scripts/verification/openhands-mcp/test_sdk.py -v
```

`MW_BIN_DIR` selects another directory of built helpers (default
`target/debug`). No model, API key, or network access is used after the
packages are installed.

## What each check proves

| Check | Kind | What it shows |
| --- | --- | --- |
| `NativeDiscoveryAndDispatch` | Native SDK, no model | The SDK's `create_mcp_tools` starts `mw-mcp` over stdio, lists the six tools, and calls them through its own `MCPToolExecutor`. `search_memory`, `recent_errors`, and `similar_failures` return the seeded synthetic failure. With one server the tool names are unprefixed; with two, each is prefixed with its config key. |

## Not covered

- Container, VM, and remote Agent Server runtimes. Every path here is local.
