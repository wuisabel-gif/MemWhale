# OpenHands SDK integration verification

Checked October 27, 2026 against `openhands-sdk` **1.58.2** from PyPI (tag
`v1.58.2`, commit `d2ee0cdea60825608c2f8906eb54e1d4d264fb42`), with
`fastmcp` 3.4.11 and `mcp` 1.30.0, on macOS with a debug build of `mw-mcp`
0.23.0.
Result: **native SDK discovery and dispatch PASS offline; real-model run
pending; container and remote runtimes not verified.**

The checks are in
[`scripts/verification/openhands-mcp`](../../scripts/verification/openhands-mcp/README.md)
(9 tests). They use only temporary synthetic stores, a temporary git
repository, and a temporary `HOME`, and never open the default MemoryWhale
store.

## Verified

Three kinds of evidence, kept separate:

| Kind | What ran | Result |
| --- | --- | --- |
| Native SDK, no model | `openhands.sdk.mcp.create_mcp_tools` started `mw-mcp` over stdio and called its tools through `MCPToolExecutor` | PASS: six tools listed; `search_memory`, `recent_errors`, and `similar_failures` returned the seeded synthetic failure |
| SDK agent loop, deterministic fake model | `Agent` + `Conversation` with the SDK's `openhands.sdk.testing.TestLLM` emitting fixed tool calls | PASS: calls dispatched to `mw-mcp`; fix handoff, review flow, and isolation behaved as below |
| Real model | Nothing | **Not run** |

The scripted model proves that the SDK dispatches the calls and returns
`mw-mcp`'s results into the conversation. It does not prove that a real model
would choose those calls.

Measured contract details:

- With one server the agent sees bare tool names (`search_memory`). With two,
  each is prefixed with its `mcp_config` key (`memorywhale_search_memory`).
  The earlier guide said the key always namespaces the tools. It does not.
- `MCPServer.env` must carry `MEMORYWHALE_DATA_DIR`. The stdio server does not
  inherit it from the SDK process. Without it, `mw-mcp` opens the default
  store under `HOME`.
- The SDK's MCP client identifies itself as `mcp`, so a note saved through
  `remember` shows "remembered by mcp".
- MCP access adds no command runs. Capture stays separate.

### Fix handoff

A seeded failure and a seeded lesson that claims "verified, exit 0" are
retrieved through `similar_failures` and `search_memory`. The harness applies
the hinted change and runs the test itself. It writes a handoff note (commit,
branch, test command, exit code, and the retrieved ids cited as hints) only
when the tree is clean and committed and that fresh run exits 0. A wrong
hinted fix produces no note.

`remember` saves the note as an agent note awaiting review (`approved = 0`).
`search_memory` does not return it until it is approved; afterwards a fresh
conversation retrieves it. `mw-mcp` does not check claims: a `remember` that
copies "verified" from a lesson is saved the same way, and only the review flow
keeps it out of retrieval.

### Isolation

- A server on one store returned no trace of a second store's unique marker
  from any read tool. The same calls against the second store did return it.
- With the parent's `MEMORYWHALE_DATA_DIR` set to the second store, a server
  whose `env` names the first store read only the first. A server with no
  `env` fell back to the default path under the temporary `HOME` and failed to
  open it.
- `filter_tools_regex` limited to the five read tools removed `remember` from
  the agent. A scripted `remember` call ended as "Tool 'remember' not found"
  and wrote nothing. An unfiltered client could still call `remember` on the
  same store, so this filter is enforced by the SDK, not by `mw-mcp`.
- `MEMORYWHALE_DATA_DIR` selects storage. It is not authentication or a
  sandbox: every store file belongs to the current user, and a direct SQLite
  read of another store's file succeeded.
- `search_memory` has no relevance floor. A query with no matching term still
  returns the store's top-ranked rows. Isolation has to come from the store a
  server opens, not from the query.

## Not yet verified

- A real model choosing these calls in a `Conversation`.
- Container, VM, or remote Agent Server runtimes. Whether `mw-mcp` and its
  store are reachable inside those runtimes is not tested.
- Linux and Windows. The checks ran on macOS only.
- Automatic command capture. The SDK exposes MCP tools only.

## Earlier transport check (September 26, 2026)

Before the SDK was installed, `mw-mcp` 0.12.1 was exercised directly with an
isolated `MEMORYWHALE_DATA_DIR` (`initialize` -> `notifications/initialized`
-> `tools/list`). That check returned `serverInfo` `{"name":"memorywhale"}` and
the same six tools, against the SDK example at revision
`bd88f050259276978dc31541d8099d98e4994428`.

## Source boundary

The SDK was installed from PyPI into a throwaway virtual environment and was
not modified. No OpenHands issue, pull request, push, or credential access was
performed.
