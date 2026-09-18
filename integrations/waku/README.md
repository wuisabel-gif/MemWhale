# Waku Agent + MemoryWhale

Waku Agent is an open-source personal assistant with its own local memory and
an MCP connector. This integration supplies a separate local debugging-memory
server and optional guidance, without replacing Waku's facts, conversations,
persona, or procedural memory.

## Status

Verified September 16, 2026 against Waku source
[`4c19b283`](https://github.com/ShenSeanChen/waku-agent/tree/4c19b28366df60c4c812e48851d19857cfb27398)
(declares **0.1.8**), Python 3.12.13, and MCP SDK 2.1.0/2.2.0. See
[verification evidence and limits](VERIFICATION.md). This identifies the exact
source tested, not every wheel or later release bearing a similar version.

The unmodified native Waku application, registry, and loop discovered six
MemoryWhale tools, saved a pending agent note, retrieved approved evidence in a
fresh process, and continued working after MCP disconnection. The model was
scripted through Waku's normal injection seam; live-model judgment and the
interactive gateways were not tested.

A cold SDK import race was reproduced on the pinned bridge. The small
[`launch.py`](launch.py) preloads the SDK's public entry points before invoking
unchanged Waku. The verified path uses that bootstrap; do not silently claim
unassisted cold startup is certified. No Waku source patch is required.

## Requirements

- Matching MemoryWhale helpers, with an explicitly selected local store.
- Python 3.11+ and Waku's optional `mcp` extra in a virtual environment, not a
  global installation. For the exact tested source, install in that environment:

  ```bash
  python -m pip install 'waku-agent[mcp] @ git+https://github.com/ShenSeanChen/waku-agent.git@4c19b28366df60c4c812e48851d19857cfb27398' 'mcp==2.2.0'
  ```

- Your existing Waku model setup for normal conversational use. The isolated
  verification does not need a provider account, API key, OAuth, or browser.
- Tested locally on macOS arm64. Other platforms and gateways remain unverified.

Waku Memory is a separate hosted product. **Do not run `waku connect
waku-memory` for this integration**: it is not the local MemoryWhale server.

## Setup

Waku reads MCP configuration from **`$WAKU_HOME/mcp.json`**. If `WAKU_HOME` is
unset, its default is **`.waku` relative to the launch directory**, not
necessarily `~/.waku`. Choose the intended home before editing configuration.

Add the following entry to the existing `servers` array; preserve other
servers and unrelated client settings. Unlike some clients, this contract uses
an **array**, not an object keyed by server name:

```json
{
  "servers": [
    {
      "name": "memorywhale",
      "command": "/absolute/path/to/mw-mcp",
      "args": [],
      "env": {
        "MEMORYWHALE_DATA_DIR": "/absolute/path/to/explicitly-selected-store"
      }
    }
  ]
}
```

Replace both placeholders with reviewed paths. For the first test use a new
synthetic store, not normal terminal history. Keep store selection in the server
`env`, not just in a setup shell. The name must be unique; changing it changes
the `memorywhale_*` tool prefix used by the guidance.

Launch with the virtual environment's Python, passing ordinary Waku arguments:

```bash
python /path/to/MemWhale/integrations/waku/launch.py
```

This launcher does not edit configuration, select a model, start an OAuth flow,
or replace the bridge. Waku retains its normal `.env` and provider behavior in
regular use; this is not an isolation wrapper. Restart the process after MCP
configuration changes rather than assuming a live registry reload.

### Optional guidance

Copy the reviewed `skills/memorywhale-debugging/` directory into an **unused**
`$WAKU_HOME/skills/memorywhale-debugging/` directory. Do not overwrite an existing
skill or native persona. Waku scans those `SKILL.md` files and matches keywords;
“MemoryWhale debugging” is an intentional trigger. The skill is prompt guidance,
not a write-authorization or confidentiality enforcement mechanism.

## Verify

The repeatable check uses real Waku code and real local `mw-mcp` processes with
scripted model responses. It creates a new test home/store and never deletes
runtime data. From the MemoryWhale root, in the isolated environment containing
the pinned Waku source install and MCP extra:

```bash
python scripts/verify-waku-mcp.py \
  --waku-source /path/to/pinned-waku-checkout \
  --bin-dir /absolute/path/to/memorywhale-binaries \
  --scratch "$PWD/.verification-waku/check-1"
```

The checkout must be the pinned, unmodified revision. The scratch directory
must not already exist; retain failed attempts rather than overwriting them.
The helper directory must contain matching `mw` and `mw-mcp` binaries.

Expected evidence:
- All six prefixed tools are discovered by Waku's actual registry.
- The native loop calls `memorywhale_remember` on an explicit synthetic request.
- A fresh process retrieves a CLI-seeded, approved synthetic note.
- The agent-written proposal persists but remains hidden until normal review.
- Removing only the test MCP entry leaves Waku working and both stores intact.

Normal user configuration and credentials are not read by this fixture. Its
child environment is sealed, `.env` traversal stops at a synthetic empty file,
and Python network connections are refused. It does not run web-search,
calendar, Apple automation, experimental delegation, or browser tools.

## Available capabilities

| Capability | Available |
| --- | --- |
| MCP memory access | Yes, verified native application/loop with SDK bootstrap |
| Automatic execution capture | No; MCP access does not record Waku's commands |
| Memory-use guidance | Optional skill; discovery and bounded trigger cases verified |

Tools appear as `memorywhale_recent_errors`, `memorywhale_search_memory`,
`memorywhale_get_context`, `memorywhale_remember`,
`memorywhale_similar_failures`, and `memorywhale_stats`.

**Approval and privacy:** by default, agent-written MemoryWhale notes await
approval in its normal TUI review pane. A save acknowledgement does not mean
immediate search visibility or verification of the explanation. Do not disable
review to make a test pass. No new `agent:waku` capture/provenance label or
`mw integrate waku` installer is introduced.

The server has the launching OS user's authority, and results can enter Waku's
model context and local chat/traces. Local stdio is not proof of offline model
inference. Registering tools exposes the note-writing tool too; neither the
skill nor the launcher adds a permission gate. Review the
[MemoryWhale trust model](../../docs/reference/mcp.md#trust-model).

## Example prompt

> MemoryWhale debugging: search for this exact compiler error in the selected
> project. Summarize relevant recorded evidence and distinguish observations
> from hypotheses. Do not save a note unless I explicitly ask.

For a deliberate write, ask to save a specific debugging lesson to
**MemoryWhale**, rather than using an ambiguous native Waku memory request.

## Troubleshooting

- A “missing mcp” message can also follow an SDK import failure. Verify the
  extra is installed in the same interpreter and use `launch.py` for the
  observed cold-import race; do not patch Waku or downgrade below its declared
  SDK requirement to hide the problem.
- Confirm the actual `WAKU_HOME`, JSON array shape, unique server name, executable
  path, and the store environment. A failed connection may leave Waku running
  without the memory tools; successful startup alone is insufficient.
- A saved agent note may be pending review. Inspect it using MemoryWhale's
  normal review interface; do not infer failed persistence from empty search.
- Skill matching is keyword-based, not proof a live model follows the guidance.
- Direct application restart was tested. Long-lived dashboard reconnection,
  graceful cleanup under every bridge failure, and hosted-provider behavior
  remain separate checks.

## Uninstall

Remove only the `memorywhale` entry from the chosen home's `servers` array and
restart Waku. Remove the optional skill only if it is the unmodified copy you
installed. Stop using the bootstrap launcher if no longer needed.

Do not delete `$WAKU_HOME`, `state.db`, chat/traces, or the MemoryWhale database.
Disconnecting one server does not require removing unrelated MCP servers or
changing Waku's own memory backend.
