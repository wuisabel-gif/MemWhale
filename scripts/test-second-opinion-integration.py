#!/usr/bin/env python3
"""Offline contract checks for the Second-Opinion MemoryWhale guide."""

from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[1]
GUIDE = ROOT / "integrations" / "second-opinion" / "README.md"
WORKFLOW = ROOT / ".github" / "workflows" / "second-opinion.yml"
PIN = "wuisabel-gif/second-opinion@e5cce405ea722fb7c7c5cfa28224c5735127cf6d"
HEADINGS = [
    "## Status",
    "## Requirements",
    "## Setup",
    "## Verify",
    "## Available capabilities",
    "## Example prompt",
    "## Troubleshooting",
    "## Uninstall",
]


class SecondOpinionGuide(unittest.TestCase):
    def test_template_headings_in_order(self):
        text = GUIDE.read_text()
        positions = [text.find(h) for h in HEADINGS]
        self.assertTrue(all(p >= 0 for p in positions), positions)
        self.assertEqual(positions, sorted(positions))

    def test_workflow_stays_pull_request_target_and_pinned(self):
        workflow = WORKFLOW.read_text()
        self.assertIn("pull_request_target", workflow)
        self.assertIn(PIN, workflow)
        self.assertIn("REVIEW_API_KEY", workflow)
        self.assertNotRegex(workflow, r"uses:\s+actions/checkout")

    def test_guide_does_not_claim_mcp_or_store_access(self):
        text = GUIDE.read_text()
        self.assertIn("| MCP memory access | No |", text)
        self.assertIn("does not read `memorywhale.sqlite3`", text)
        self.assertIn("mw github context", text)
        self.assertIn("mw remember", text)


if __name__ == "__main__":
    unittest.main()
