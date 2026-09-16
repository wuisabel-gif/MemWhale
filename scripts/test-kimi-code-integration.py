#!/usr/bin/env python3
"""Offline checks for the Kimi Code integration assets and contract."""
import json
from pathlib import Path
import re
import unittest

ROOT = Path(__file__).resolve().parents[1]
GUIDE = ROOT / 'integrations/kimi-code'
EXPECTED = ['recent_errors', 'search_memory', 'get_context', 'remember', 'similar_failures', 'stats']


class KimiCodeIntegrationTests(unittest.TestCase):
    def test_mcp_example_matches_documented_kimi_shape(self):
        config = json.loads((GUIDE / 'mcp.example.json').read_text())
        self.assertEqual(set(config), {'mcpServers'})
        server = config['mcpServers']['memorywhale']
        self.assertEqual(server['args'], [])
        self.assertEqual(server['enabledTools'], EXPECTED)
        self.assertTrue(server['command'].startswith('/'))
        self.assertTrue(server['env']['MEMORYWHALE_DATA_DIR'].startswith('/'))
        self.assertEqual(server['startupTimeoutMs'], 30000)
        self.assertEqual(server['toolTimeoutMs'], 120000)

    def test_skill_is_scoped_and_mentions_review(self):
        skill = (GUIDE / 'skills/memorywhale-debugging/SKILL.md').read_text()
        self.assertTrue(skill.startswith('---\nname: memorywhale-debugging\n'))
        self.assertIn('mcp__memorywhale__search_memory', skill)
        self.assertIn('mcp__memorywhale__remember', skill)
        self.assertIn('stays pending', skill)
        self.assertIn('not a permission gate, hook, or automatic capture path', skill)

    def test_guide_uses_template_headings_in_order(self):
        headings = [line for line in (GUIDE / 'README.md').read_text().splitlines() if line.startswith('## ')]
        self.assertEqual(headings, ['## Status', '## Requirements', '## Setup', '## Verify',
                                     '## Available capabilities', '## Example prompt',
                                     '## Troubleshooting', '## Uninstall'])

    def test_guide_does_not_claim_capture_or_live_verification(self):
        guide = (GUIDE / 'README.md').read_text().lower()
        self.assertIn('not implemented or claimed', guide)
        self.assertIn('native client call pending', guide)
        self.assertIn('live', guide)
        self.assertIn('provider-backed', guide)
        self.assertNotIn('automatic execution capture | yes', guide)

    def test_no_secret_or_home_path_in_assets(self):
        text = '\n'.join(p.read_text() for p in GUIDE.rglob('*') if p.is_file())
        self.assertNotRegex(text, r'(?i)(sk-[a-z0-9]{20,}|/users/[^/]+/\.kimi-code|/home/[^/]+/\.kimi-code)')
        self.assertNotIn('MEMORYWHALE_DATA_DIR": "~', text)


if __name__ == '__main__':
    unittest.main()
