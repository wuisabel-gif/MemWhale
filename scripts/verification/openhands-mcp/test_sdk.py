#!/usr/bin/env python3
"""OpenHands SDK checks for mw-mcp, offline, on temporary synthetic stores.

Run from the repository root after `cargo build -p memorywhale-cli --bins`:
  uv run --no-project --with-requirements scripts/verification/openhands-mcp/requirements.txt \
    python -I scripts/verification/openhands-mcp/test_sdk.py -v
"""
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import fixture  # noqa: E402  (sets HOME before the SDK is imported)
from fixture import TOOLS, TEST_CMD, MARKER  # noqa: E402

from openhands.sdk.mcp import MCPToolExecutor, create_mcp_tools  # noqa: E402


def call(client, name, **args):
    tool = next(t for t in client.tools if t.name == name)
    return tool(tool.action_from_arguments(args))


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


if __name__ == "__main__":
    unittest.main()
