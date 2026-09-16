#!/usr/bin/env python3
"""Offline asset/launcher checks; does not install or run Waku or a model."""
import ast
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
from types import ModuleType
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
GUIDE = ROOT / 'integrations/waku'


class WakuAssets(unittest.TestCase):
    def test_config_selects_one_explicit_local_store(self):
        value = json.loads((GUIDE / 'mcp.example.json').read_text())
        self.assertEqual(set(value), {'servers'})
        self.assertEqual(len(value['servers']), 1)
        server = value['servers'][0]
        self.assertEqual(set(server), {'name', 'command', 'args', 'env'})
        self.assertEqual(server['name'], 'memorywhale')
        self.assertTrue(Path(server['command']).is_absolute())
        self.assertTrue(Path(server['env']['MEMORYWHALE_DATA_DIR']).is_absolute())
        self.assertEqual(server['args'], [])

    def test_guide_uses_required_sections(self):
        sections = [line for line in (GUIDE / 'README.md').read_text().splitlines() if line.startswith('## ')]
        self.assertEqual(sections, ['## Status', '## Requirements', '## Setup', '## Verify',
                                   '## Available capabilities', '## Example prompt',
                                   '## Troubleshooting', '## Uninstall'])

    def test_skill_is_explicitly_scoped(self):
        skill = (GUIDE / 'skills/memorywhale-debugging/SKILL.md').read_text()
        self.assertTrue(skill.startswith('---\nname: memorywhale-debugging\n'))
        self.assertIn('memorywhale_search_memory', skill)
        self.assertIn('memorywhale_remember', skill)
        self.assertIn('pending', skill)
        self.assertIn('guidance, not a permission gate', skill)

    def test_launcher_delegates_without_replacing_waku_or_arguments(self):
        spec = importlib.util.spec_from_file_location('waku_launcher', GUIDE / 'launch.py')
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        sdk = ModuleType('mcp')
        sdk.ClientSession = object
        sdk.StdioServerParameters = object
        entry = ModuleType('waku.__main__')
        calls = []
        entry.main = lambda: calls.append(list(sys.argv))
        with patch.dict(sys.modules, {'mcp': sdk, 'waku': ModuleType('waku'), 'waku.__main__': entry}):
            with patch.object(sys, 'argv', ['launch.py', 'mcp']):
                module.main()
        self.assertEqual(calls, [['launch.py', 'mcp']])

    def test_verifier_syntax(self):
        ast.parse((ROOT / 'scripts/verify-waku-mcp.py').read_text())

    def test_verifier_rejects_changed_note_content_and_stderr(self):
        spec = importlib.util.spec_from_file_location('waku_verifier', ROOT / 'scripts/verify-waku-mcp.py')
        verifier = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(verifier)
        expected = [(verifier.HUMAN_TEXT, 'human', 1), (verifier.PROPOSED_TEXT, 'agent', 0)]
        verifier.check_notes(expected)
        for index in [0, 1]:
            changed = list(expected)
            changed[index] = ('substituted content', *changed[index][1:])
            with self.assertRaises(RuntimeError):
                verifier.check_notes(changed)
        with tempfile.TemporaryDirectory(prefix='mw-waku-assertions-') as directory:
            root = Path(directory)
            for name in ['write', 'read', 'disabled']:
                (root / (name + '.stderr')).write_text('')
            self.assertEqual(verifier.check_stderr(root), [])
            for name in ['write', 'read', 'disabled']:
                path = root / (name + '.stderr')
                path.write_text('unexpected warning')
                with self.assertRaises(RuntimeError):
                    verifier.check_stderr(root)
                path.write_text('')


if __name__ == '__main__':
    unittest.main()
