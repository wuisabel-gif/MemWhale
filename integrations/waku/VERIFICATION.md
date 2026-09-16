# Waku MCP integration verification

Checked **September 16, 2026**. Result: **PASS with the documented SDK preload**.
This is real Waku application/loop and stdio MCP evidence with a scripted model,
not a live-provider or interactive-gateway certification.

## Versions and provenance

- Waku source `4c19b28366df60c4c812e48851d19857cfb27398`, declaring 0.1.8.
  The verifier compares installed Python source files with the pinned checkout.
- Python 3.12.13, macOS arm64.
- MCP SDK 2.1.0 and 2.2.0 passed the native-loop scenario with preload.
  The final recorded run used SDK 2.2.0.
- Matching local MemoryWhale development helpers; `mw-mcp` SHA-256:
  `116e678e6b327ca8305f86f9b369a995a696860b7d65d1e5aac9a04d49bef8e7`.
- Waku was installed only in the task-owned virtual environment from that source.
  The Waku checkout remained clean. No upstream edit, issue, PR, or push occurred.

Primary contracts inspected in that revision:
[`mcp_client.py`](https://github.com/ShenSeanChen/waku-agent/blob/4c19b28366df60c4c812e48851d19857cfb27398/waku/tools/mcp_client.py),
[`app.py`](https://github.com/ShenSeanChen/waku-agent/blob/4c19b28366df60c4c812e48851d19857cfb27398/waku/app.py),
[`config.py`](https://github.com/ShenSeanChen/waku-agent/blob/4c19b28366df60c4c812e48851d19857cfb27398/waku/config.py), and
[`SkillLoader`](https://github.com/ShenSeanChen/waku-agent/blob/4c19b28366df60c4c812e48851d19857cfb27398/waku/memory/procedural/loader.py).

## Observed outcomes

| Check | Actual evidence |
| --- | --- |
| Discovery | `Waku(...)` built its normal registry and exposed all six `memorywhale_*` tools through its actual `MCPBridge`. |
| Deliberate write | `Waku.respond()` ran the real agent loop; its injected scripted model selected `memorywhale_remember`, which returned a real save acknowledgement for note #2. |
| Review policy | A read-only SQL check confirmed note #2 had agent provenance and `approved=0`. No review configuration or approval bit was changed. |
| Fresh retrieval | A separate Python process created a new Waku instance and dispatched `memorywhale_search_memory`; it retrieved the approved synthetic note #1 seeded through `mw remember`. The pending proposal was not exposed. |
| Disconnect | A third process used an empty MCP server array in the same test home. MemoryWhale tools disappeared, Waku still completed its turn, and both databases remained. |
| Guidance | The real skill loader found `memorywhale-debugging` for the explicit debugging trigger, but not four ordinary greeting/calendar/personal-preference prompts. |
| Basic cleanup | All three processes exited successfully, with empty stderr in the passing runs. This does not certify every long-lived bridge failure/cleanup path. |

## Findings that changed the setup

**Cold MCP import race.** Unassisted `Waku(...)` initialization failed with both
SDK versions during concurrent initial imports, mentioning a partially
initialized `mcp.client._input_required`. Waku's subsequent message suggested the
MCP extra was missing even though it was installed. A single-threaded import of
`ClientSession` and `StdioServerParameters` succeeded. The tiny MemoryWhale-side
launcher performs those imports before delegating to unchanged Waku; the same
bootstrap is exercised by the verifier. No SDK monkeypatch or Waku fork is used.

**A saved proposal is not an approved note.** An initial expectation that the
agent's freshly saved note would immediately appear in search was incorrect.
The fixture was changed to respect the normal review boundary: explicit human
seed visible, agent proposal persisted but hidden. It does not auto-approve,
modify SQLite approval fields, or weaken `review_agent_memories`.

## Repeatable evidence

Run `scripts/verify-waku-mcp.py` as documented in the guide. Its fresh scratch
contains per-phase stdout/stderr, `result.json`, and only synthetic Waku and
MemoryWhale stores. The local passing runs were `run-5` (SDK 2.1.0), `run-6`
(SDK 2.2.0), and `run-8` (final skill, source-identity, network/isolation guards,
and native conversation-persistence check).
These runtime artifacts, the installed environment, and the source clone are
not part of the PR. The fixture never deletes them.

The final fixture uses a sealed environment, a task-owned empty `.env` traversal
boundary, disabled dotenv loading, no credentials, and a Python audit hook
refusing network connections. Apple/calendar/GitHub/experimental integrations
and trace export are disabled. Only the selected local stdio server is configured.
The scripted client uses Waku's existing injection seam; neither MCP dispatch
nor persistence is mocked.

Five dependency-free asset/launcher checks run in MemoryWhale CI through
`scripts/test-waku-integration.py`. The optional native Waku scenario is run
separately; a passing asset check alone is not a native integration pass.

## Not demonstrated

No claim is made about live LLM judgment, every SDK or Waku release, interactive
CLI/dashboard behavior, remote HTTP/OAuth, Windows, hosted Waku Memory, or
automatic execution capture. All six tools were discovered; the scenario
specifically dispatched `remember` and `search_memory`. Other tool behavior is
covered by MemoryWhale's own tests, not falsely counted as Waku-specific calls.
The skill does not enforce consent or confidentiality, and native Waku memory
remains a distinct system.
