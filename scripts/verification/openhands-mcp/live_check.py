#!/usr/bin/env python3
"""Opt-in live check: a real model drives the OpenHands SDK against mw-mcp.

It refuses to run unless MW_OPENHANDS_LIVE=1 and LLM_MODEL are set, because a
hosted provider may bill you. It uses the same LLM_* names as the SDK's
07_mcp_integration.py example, and the same temporary synthetic store as
test_sdk.py:

  MW_OPENHANDS_LIVE=1 LLM_MODEL=<model> LLM_API_KEY=<key> [LLM_BASE_URL=<url>] \
    uv run --no-project --with-requirements scripts/verification/openhands-mcp/requirements.txt \
    python -I scripts/verification/openhands-mcp/live_check.py

The model gets only the five read tools. It cannot save memory. The harness
applies the fix the model proposes, reruns the test itself, and saves a handoff
note only if that run passes. MW_OPENHANDS_MAX_USD caps the spend (default 0.25).
"""
import json
import os
import re
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import fixture  # noqa: E402  (sets HOME before the SDK is imported)

PROMPT = """In the repository at {repo}, `python3 -m unittest -q` fails with:

{stderr}
Use the MemoryWhale tools to check whether this failure was seen before and
whether a fix was recorded. Retrieved lessons are unverified hints. Reply with
the one-line change you would make to calc.py and the memory ids you relied
on. Do not claim the fix works; it has not been tested."""


def run(llm, name="live"):
    """Drive one conversation with `llm` and report what happened. The report
    says "live model" only when `llm` is not the SDK's scripted TestLLM."""
    from openhands.sdk import Agent, LocalConversation
    from openhands.sdk.conversation import get_agent_final_response
    from openhands.sdk.event import ActionEvent, ObservationEvent
    from openhands.sdk.mcp import create_mcp_tools
    from openhands.sdk.testing import TestLLM

    store = fixture.SCRATCH / name
    repo = fixture.make_repo(f"{name}-calc")
    stderr = fixture.seed_failure(store, repo)
    fixture.mw(store, "mw", "remember", fixture.LESSON)

    events = []
    agent = Agent(llm=llm, tools=[], mcp_config={"memorywhale": fixture.server(store)},
                  filter_tools_regex=fixture.READ_ONLY)
    # LocalConversation, not the Conversation factory: only it takes a budget.
    conversation = LocalConversation(agent=agent, workspace=str(repo), callbacks=[events.append],
                                     visualizer=None, max_iteration_per_run=12,
                                     max_budget_per_run=float(os.environ.get("MW_OPENHANDS_MAX_USD", "0.25")))
    try:
        conversation.send_message(PROMPT.format(repo=repo, stderr=stderr))
        conversation.run()
    finally:
        conversation.close()

    called = [e.tool_name for e in events if isinstance(e, ActionEvent)]
    seen = "\n".join(e.observation.text for e in events if isinstance(e, ObservationEvent))
    reply = get_agent_final_response(events)
    checks = {
        "called a MemoryWhale tool": any(name in fixture.TOOLS for name in called),
        "retrieved the seeded failure or lesson": f"(cwd: {repo})" in seen or fixture.LESSON in seen,
        "proposed return a + b": bool(re.search(r"return\s+a\s*\+\s*b", reply)),
    }

    handoff = None
    if checks["proposed return a + b"]:
        (repo / "calc.py").write_text("def add(a, b):\n    return a + b\n")
        fixture.commit(repo, "apply the proposed fix")
        note = fixture.verified_handoff(repo, re.findall(r"command_run #\d+|note #\d+", seen))
        if note:
            with create_mcp_tools({"memorywhale": fixture.server(store)}, timeout=30) as client:
                tool = next(t for t in client.tools if t.name == "remember")
                saved = tool(tool.action_from_arguments({"text": note})).text
            note_id = int(re.search(r"Saved as memory #(\d+)", saved).group(1))
            handoff = {"note": note, "id": note_id, "approved": fixture.scalar(
                store, "SELECT approved FROM bookmarks WHERE id = ?", note_id)}
    checks["fresh test run passed and handoff saved pending"] = bool(handoff) and handoff["approved"] == 0

    return {
        "kind": "scripted TestLLM, not live" if isinstance(llm, TestLLM) else "live model",
        "model": llm.model,
        "cost_usd": llm.metrics.accumulated_cost,
        "tools_called": called,
        "reply": reply,
        "checks": checks,
        "handoff": handoff,
        "passed": all(checks.values()),
    }


def main():
    if os.environ.get("MW_OPENHANDS_LIVE") != "1" or not os.environ.get("LLM_MODEL"):
        print("refusing to call a model: set MW_OPENHANDS_LIVE=1 and LLM_MODEL (plus LLM_API_KEY "
              "or LLM_BASE_URL as your provider needs). A hosted provider may bill you.", file=sys.stderr)
        return 2
    from openhands.sdk import LLM
    from pydantic import SecretStr

    key = os.environ.get("LLM_API_KEY")
    llm = LLM(usage_id="mw-live-check", model=os.environ["LLM_MODEL"],
              base_url=os.environ.get("LLM_BASE_URL"), api_key=SecretStr(key) if key else None)
    report = run(llm)
    print(json.dumps(report, indent=2))
    return 0 if report["passed"] else 1


if __name__ == "__main__":
    sys.exit(main())
