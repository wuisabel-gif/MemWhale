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

| `ScriptedConversation` | SDK agent loop, deterministic fake model | The SDK's `TestLLM` emits fixed tool calls that an `Agent` + `Conversation` dispatches to `mw-mcp`. This shows the dispatch path; it does not show that a real model would choose these calls. See the handoff below. |

| `Isolation` | Native SDK and SDK agent loop, negative tests | See [What isolation holds](#what-isolation-holds). |
| `LiveHarnessOffline` | Deterministic fake model | `live_check.py` refuses to run without opt-in, and its checks pass and fail correctly on scripted runs, which it labels as not live. |

### The fix handoff

1. A conversation calls `similar_failures` and `search_memory`, which return
   the seeded failure and a seeded lesson. The lesson says "verified, exit 0",
   and nothing treats that as evidence.
2. The harness, not the model, applies the hinted change. `verified_handoff`
   refuses until the tree is clean and committed, then runs the test itself.
   It writes a note only when that run exits 0. The note records the commit,
   branch, test command, and exit code, and cites the retrieved memory ids as
   hints. A wrong hinted fix (`a * b`) produces no note.
3. A second conversation saves the note with `remember`. It lands as an agent
   note awaiting review (`approved = 0`). It is not returned by
   `search_memory` until it is approved, after which a fresh conversation
   retrieves it, labelled "remembered by mcp" (the SDK's MCP client name).
4. Neither the MCP calls nor the harness's test runs add a command run.
   Memory access is separate from capture.

`mw-mcp` does not check what a note claims. A `remember` call that copies
"verified" from a lesson is saved like any other. The only MemoryWhale-side
guard is that the note stays out of retrieval until someone approves it.

### What isolation holds

Two stores are seeded: the calc store, and an unrelated store with a unique
marker. Each negative test has a positive control, so it cannot pass just
because a tool returned nothing.

- **A server sees only its own store.** Pointed at the calc store, every read
  tool (`search_memory` in both modes, `get_context`, `recent_errors` at its
  64-run maximum, `similar_failures` with the unrelated error, and `stats`)
  returns no trace of the marker. The same calls against the unrelated store
  do return it.
- **The parent's `MEMORYWHALE_DATA_DIR` does not reach the server.** The SDK
  starts stdio servers with a small default environment plus
  `MCPServer.env`. With the variable set in the parent and `env` naming the
  calc store, the calc store wins. With no `env` at all, `mw-mcp` falls back
  to the default store under the `HOME` it was given. Here that is the
  temporary `HOME`, so the calls fail with "failed to open". **In normal use
  it is your real store, so always set `MEMORYWHALE_DATA_DIR` in
  `MCPServer.env`.**
- **A read-only tool filter hides `remember` from the agent, and only from
  the agent.** With `filter_tools_regex` set to the five read tools, the
  agent's registry has no `remember`. A scripted `remember` call ends as
  "Tool 'remember' not found" and writes nothing. A second client started
  without the filter can still call `remember` on the same store. The filter
  is enforced in the SDK process, not by `mw-mcp`.
- **A separate store is not access control.** Every store file belongs to
  the current user, and a plain SQLite read of the unrelated store's file
  succeeds. `MEMORYWHALE_DATA_DIR` selects storage. It does not authenticate
  or sandbox. A tool that can run commands as the same user, such as a
  terminal tool, can read any store directly.

Also measured: `search_memory` has no relevance floor. A query with no
matching term still returns the store's top-ranked rows. Isolation therefore
has to come from the store a server opens, never from the query.

## Live check (opt-in, may cost money)

`live_check.py` puts a real model in the same setup. It refuses to run unless
you set `MW_OPENHANDS_LIVE=1` and `LLM_MODEL`, because a hosted provider may
bill you. It takes the same `LLM_*` variables as the SDK's MCP example:

```bash
MW_OPENHANDS_LIVE=1 LLM_MODEL=<model> LLM_API_KEY=<key> \
  uv run --no-project --with-requirements scripts/verification/openhands-mcp/requirements.txt \
  python -I scripts/verification/openhands-mcp/live_check.py
```

For a local model server, set `LLM_BASE_URL` and leave `LLM_API_KEY` unset.
The run is capped at 12 agent iterations and at `MW_OPENHANDS_MAX_USD`
(default 0.25), which works only when LiteLLM knows the model's price.

The model sees only the five read tools, so it cannot save memory. It is asked
to diagnose the seeded failure and propose a fix. The harness then:

1. checks that the model called a MemoryWhale tool, that the seeded failure or
   lesson came back, and that the reply proposes `return a + b`;
2. applies that fix, commits, and reruns the test itself;
3. saves the handoff note only when that fresh run passes, and checks that the
   note is pending review.

It prints a JSON report with `"kind": "live model"` and the accumulated cost,
and exits 1 if any check fails. The same function also runs offline, in
`LiveHarnessOffline`, with the scripted `TestLLM`. The report then says
`"scripted TestLLM, not live"` and costs 0. A scripted run is never labelled
live.

**Status: not run with a real model.** The machine this was built on had no
provider credentials in its environment. Its only local model was a 0.5B
model, behind an Ollama server that was not running. Running it would have
meant starting a new local listener, which was out of scope.

## Not covered

- A real model choosing these calls (see the live check above).
- Container, VM, and remote Agent Server runtimes. Every path here is local.
