#!/usr/bin/env python3
"""OpenHands SDK checks for mw-mcp, offline, on temporary synthetic stores.

Run from the repository root after `cargo build -p memorywhale-cli --bins`:
  uv run --no-project --with-requirements scripts/verification/openhands-mcp/requirements.txt \
    python -I scripts/verification/openhands-mcp/test_sdk.py -v
"""
import json
import re
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import fixture  # noqa: E402  (sets HOME before the SDK is imported)
from fixture import TOOLS, TEST_CMD, MARKER  # noqa: E402

from openhands.sdk import Agent, Conversation, Message, TextContent  # noqa: E402
from openhands.sdk.event import AgentErrorEvent, ObservationEvent  # noqa: E402
from openhands.sdk.llm import MessageToolCall  # noqa: E402
from openhands.sdk.mcp import MCPToolExecutor, create_mcp_tools  # noqa: E402
from openhands.sdk.testing import TestLLM  # noqa: E402

LESSON = "fix: calc add() subtracts; make it return a + b (verified, exit 0)"


def call(client, name, **args):
    tool = next(t for t in client.tools if t.name == name)
    return tool(tool.action_from_arguments(args))


def tool_call(name, **args):
    return Message(role="assistant", content=[TextContent(text="")], tool_calls=[
        MessageToolCall(id=f"call_{name}", name=name, arguments=json.dumps(args), origin="completion")])


def say(text):
    return Message(role="assistant", content=[TextContent(text=text)])


def converse(store, workspace, prompt, script, **agent_options):
    """One Agent + Conversation turn whose model is the SDK's scripted TestLLM.
    Tool calls go through the SDK's normal agent loop to a fresh mw-mcp."""
    events = []
    agent = Agent(llm=TestLLM.from_messages(script), tools=[], include_default_tools=[],
                  mcp_config={"memorywhale": fixture.server(store)}, **agent_options)
    conversation = Conversation(agent=agent, workspace=str(workspace),
                                callbacks=[events.append], visualizer=None)
    try:
        conversation.send_message(prompt)
        conversation.run()
        tools = sorted(conversation.agent.tools_map)
    finally:
        conversation.close()
    observed = [(e.tool_name, e.observation.text) for e in events if isinstance(e, ObservationEvent)]
    errors = [e.error for e in events if isinstance(e, AgentErrorEvent)]
    return tools, observed, errors


class NativeDiscoveryAndDispatch(unittest.TestCase):
    """The SDK's own MCP client lists mw-mcp's tools and calls them; no model."""

    @classmethod
    def setUpClass(cls):
        cls.store = fixture.SCRATCH / "native"
        cls.repo = fixture.make_repo("native-calc")
        cls.stderr = fixture.seed_failure(cls.store, cls.repo)

    def test_lists_six_unprefixed_tools_and_retrieves_the_seeded_failure(self):
        with create_mcp_tools({"memorywhale": fixture.server(self.store)}, timeout=30) as client:
            self.assertEqual([t.name for t in client.tools], TOOLS)
            self.assertTrue(all(isinstance(t.executor, MCPToolExecutor) for t in client.tools))

            # The snippet is stderr's first line (unittest's ==== rule), so the
            # marker shows up only in the matched-terms reason.
            found = call(client, "search_memory", query=MARKER)
            self.assertFalse(found.is_error, found.text)
            top = found.text.split("\n- ")[0]
            self.assertIn('["python3","-m","unittest","-q"]', top)
            self.assertIn(MARKER.split("-")[-1].lower(), top)

            errors = call(client, "recent_errors")
            self.assertIn("`python3 -m unittest -q` (exit 1", errors.text)

            similar = call(client, "similar_failures", error_text=self.stderr,
                           command=" ".join(TEST_CMD))
            self.assertFalse(similar.is_error, similar.text)
            self.assertIn(f"(cwd: {self.repo})", similar.text)

    def test_two_servers_prefix_tool_names_with_their_keys(self):
        other = fixture.SCRATCH / "native-other"
        fixture.seed_unrelated(other)
        config = {"memorywhale": fixture.server(self.store), "other": fixture.server(other)}
        with create_mcp_tools(config, timeout=30) as client:
            names = [t.name for t in client.tools]
        self.assertEqual(names, [f"memorywhale_{t}" for t in TOOLS] + [f"other_{t}" for t in TOOLS])


class ScriptedConversation(unittest.TestCase):
    """A deterministic fake model drives the SDK's agent loop. It proves the
    dispatch path, not that a real model would choose these calls."""

    def setUp(self):
        self.store = fixture.SCRATCH / self.id().rsplit(".", 1)[-1]
        self.repo = fixture.make_repo(f"{self.store.name}-calc")
        self.stderr = fixture.seed_failure(self.store, self.repo)
        fixture.mw(self.store, "mw", "remember", LESSON)
        self.runs = fixture.scalar(self.store, "SELECT count(*) FROM command_runs")

    def test_retrieve_fix_verify_then_hand_off_for_review(self):
        tools, observed, errors = converse(
            self.store, self.repo, f"`python3 -m unittest -q` fails:\n{self.stderr}\nCheck MemoryWhale.",
            [tool_call("similar_failures", error_text=self.stderr, command=" ".join(TEST_CMD)),
             tool_call("search_memory", query="calc add fix", mode="lessons"),
             say("Memory has a candidate fix. It is a hint until the test passes.")])
        self.assertEqual(tools, sorted(TOOLS))
        self.assertEqual(errors, [])
        self.assertEqual([name for name, _ in observed], ["similar_failures", "search_memory"])
        self.assertIn(f"(cwd: {self.repo})", observed[0][1])
        self.assertIn(LESSON, observed[1][1])
        hints = re.findall(r"command_run #\d+|note #\d+", observed[0][1] + observed[1][1])
        self.assertEqual(len(hints), 2, hints)

        # The harness, not the model, applies the hinted change and runs the test.
        self.assertIsNone(fixture.verified_handoff(self.repo, hints), "nothing fixed yet")
        (self.repo / "calc.py").write_text("def add(a, b):\n    return a + b\n")
        self.assertIsNone(fixture.verified_handoff(self.repo, hints), "uncommitted change")
        head = fixture.commit(self.repo, "fix add")
        note = fixture.verified_handoff(self.repo, hints)
        self.assertTrue(note.startswith(f"verified fix at commit {head}: `python3 -m unittest -q` exited 0"))

        _, observed, errors = converse(self.store, self.repo, f"Save this handoff:\n{note}",
                                       [tool_call("remember", text=note), say("Saved for review.")])
        self.assertEqual(errors, [])
        saved = int(re.search(r"Saved as memory #(\d+)", observed[0][1]).group(1))
        row = "SELECT label, author_kind, approved FROM bookmarks WHERE id = ?"
        with fixture.sqlite3.connect(self.store / "memorywhale.sqlite3") as db:
            self.assertEqual(db.execute(row, (saved,)).fetchone(), (note, "agent", 0))
        # Memory access is not command capture: neither the MCP calls nor the
        # harness's own test runs added a command run.
        self.assertEqual(fixture.scalar(self.store, "SELECT count(*) FROM command_runs"), self.runs)

        # Pending notes stay out of retrieval until someone approves them.
        search = [tool_call("search_memory", query=head, mode="lessons"), say("Done.")]
        _, observed, _ = converse(self.store, self.repo, "Find the handoff.", search)
        self.assertIn(LESSON, observed[0][1])  # approved notes still come back
        self.assertNotIn(head, observed[0][1])
        fixture.approve(self.store, saved)
        _, observed, _ = converse(self.store, self.repo, "Find the handoff.", search)
        self.assertIn(f"[note #{saved}] ", observed[0][1])
        self.assertIn(note[:150], observed[0][1])
        self.assertIn("remembered by mcp", observed[0][1])  # the SDK's MCP clientInfo name

    def test_unverified_claim_is_saved_pending_and_unretrievable(self):
        # mw-mcp does not check what a note claims. The only guard on its side
        # is that an agent note stays out of retrieval until it is approved.
        claim = "verified fix at commit 0000000: tests exited 0 (copied from a retrieved lesson)"
        _, observed, errors = converse(self.store, self.repo, "Save the lesson as verified.",
                                       [tool_call("remember", text=claim), say("Saved.")])
        self.assertEqual(errors, [])
        self.assertIn("Saved as memory #", observed[0][1])
        self.assertEqual(fixture.scalar(self.store, "SELECT approved FROM bookmarks WHERE label = ?",
                                        claim), 0)
        _, observed, _ = converse(self.store, self.repo, "Find it.",
                                  [tool_call("search_memory", query="copied retrieved", mode="lessons"),
                                   say("Done.")])
        self.assertIn(LESSON, observed[0][1])
        self.assertNotIn(claim, observed[0][1])

    def test_a_wrong_hinted_fix_produces_no_handoff(self):
        (self.repo / "calc.py").write_text("def add(a, b):\n    return a * b\n")
        fixture.commit(self.repo, "apply a wrong hint")
        self.assertIsNone(fixture.verified_handoff(self.repo, ["note #1"]))


class Isolation(unittest.TestCase):
    """What MEMORYWHALE_DATA_DIR and the SDK's tool filter do and do not stop."""

    READ_ONLY = r"^(memorywhale_)?(recent_errors|search_memory|get_context|similar_failures|stats)$"

    @classmethod
    def setUpClass(cls):
        cls.calc = fixture.SCRATCH / "isolation-calc"
        fixture.seed_failure(cls.calc, fixture.make_repo("isolation-calc-repo"))
        fixture.mw(cls.calc, "mw", "remember", LESSON)
        cls.unrelated = fixture.SCRATCH / "isolation-unrelated"
        fixture.seed_unrelated(cls.unrelated)

    def read_everything(self, config):
        marker = fixture.UNRELATED
        with create_mcp_tools(config, timeout=30) as client:
            return "\n".join(call(client, name, **args).text for name, args in [
                ("search_memory", {"query": marker}),
                ("search_memory", {"query": marker, "mode": "lessons"}),
                ("get_context", {}),
                ("recent_errors", {"limit": 64}),
                ("similar_failures", {"error_text": f"error: {marker} build exploded",
                                      "command": f"make {marker.lower()}"}),
                ("stats", {}),
            ]).lower()

    def test_a_server_on_one_store_returns_nothing_from_another(self):
        marker = fixture.UNRELATED.lower()
        self.assertIn(marker, self.read_everything({"memorywhale": fixture.server(self.unrelated)}))
        self.assertNotIn(marker, self.read_everything({"memorywhale": fixture.server(self.calc)}))

    def test_the_child_does_not_inherit_the_parent_store_variable(self):
        marker = fixture.UNRELATED.lower()
        fixture.os.environ["MEMORYWHALE_DATA_DIR"] = str(self.unrelated)
        try:
            self.assertNotIn(marker, self.read_everything({"memorywhale": fixture.server(self.calc)}))
            # With no env, mw-mcp falls back to the default store under the HOME
            # the SDK passes through. Here that is the temporary HOME, which
            # has no store; in normal use it is the real one.
            from openhands.sdk.mcp import MCPServer
            bare = {"memorywhale": MCPServer(command=str(fixture.BIN / "mw-mcp"))}
            text = self.read_everything(bare)
        finally:
            del fixture.os.environ["MEMORYWHALE_DATA_DIR"]
        self.assertNotIn(marker, text)
        self.assertIn(f"failed to open {fixture.HOME}".lower(), text)

    def test_the_read_only_filter_hides_remember_from_the_agent_only(self):
        notes = "SELECT count(*) FROM bookmarks"
        before = fixture.scalar(self.calc, notes)
        tools, observed, errors = converse(
            self.calc, fixture.SCRATCH, "Save a note.",
            [tool_call("remember", text="this must not be saved"), say("Done.")],
            filter_tools_regex=self.READ_ONLY)
        self.assertEqual(tools, sorted(set(TOOLS) - {"remember"}))
        self.assertEqual(observed, [])
        self.assertTrue(any("Tool 'remember' not found" in e for e in errors), errors)
        self.assertEqual(fixture.scalar(self.calc, notes), before)

        # The filter lives in the SDK client. mw-mcp still serves remember to
        # any client that starts it without the filter.
        with create_mcp_tools({"memorywhale": fixture.server(self.calc)}, timeout=30) as client:
            self.assertIn("Saved as memory #", call(client, "remember", text="unfiltered client").text)
        self.assertEqual(fixture.scalar(self.calc, notes), before + 1)

    def test_store_separation_is_not_access_control(self):
        # Separate stores scope what the tools return. They do not stop a
        # process running as the same user from opening another store's file.
        db = self.unrelated / "memorywhale.sqlite3"
        self.assertEqual(db.stat().st_uid, fixture.os.getuid())
        self.assertIn(fixture.UNRELATED, fixture.scalar(
            self.unrelated, "SELECT group_concat(stderr) FROM command_runs"))


if __name__ == "__main__":
    unittest.main()
