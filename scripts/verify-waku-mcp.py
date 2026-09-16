#!/usr/bin/env python3
"""Pinned Waku application/loop + real MemoryWhale MCP, scripted model only.

Run with the isolated Python environment containing the pinned Waku + MCP extra.
Creates a NEW scratch directory; never deletes runtime state or edits Waku.
"""
import argparse
import hashlib
import importlib.metadata
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sqlite3
import subprocess
import sys
from types import SimpleNamespace as NS

PIN = '4c19b28366df60c4c812e48851d19857cfb27398'
MARKER = 'WAKU_MEMORYWHALE_SYNTHETIC_20260916'
HUMAN_TEXT = MARKER + ': synthetic human-approved fixture.'
PROPOSED_TEXT = MARKER + '_PROPOSED: synthetic lesson; awaiting normal review.'
TOOLS = {'recent_errors', 'search_memory', 'get_context', 'remember', 'similar_failures', 'stats'}
REPO = Path(__file__).resolve().parents[1]


def require(condition, message):
    if not condition:
        raise RuntimeError(message)


def check_notes(notes):
    require(notes == [(HUMAN_TEXT, 'human', 1), (PROPOSED_TEXT, 'agent', 0)],
            'stored note content, approval, or provenance differs from the requested fixtures')


def check_stderr(root):
    nonempty = [name for name in ['write', 'read', 'disabled']
                if (root / (name + '.stderr')).stat().st_size]
    require(not nonempty, 'a passing verification must have empty stderr: ' + ', '.join(nonempty))
    return nonempty


def phase(root, name):
    # Imports happen only in this fresh subprocess after its environment/cwd
    # have been sealed. A task-owned empty .env stops find_dotenv traversal.
    require(Path.cwd().resolve() == root / 'workspace'
            and os.environ.get('HOME') == str(root / 'home')
            and os.environ.get('WAKU_HOME') == str(root / 'waku')
            and os.environ.get('PYTHON_DOTENV_DISABLED') == '1',
            'internal phases must run through the isolated parent driver')
    def no_network(event, arguments):
        if event in ('socket.connect', 'socket.getaddrinfo'):
            raise RuntimeError('network access is forbidden in the scripted Waku verification')
    sys.addaudithook(no_network)
    spec = importlib.util.spec_from_file_location('memorywhale_waku_launch', REPO / 'integrations/waku/launch.py')
    launcher = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(launcher)
    launcher.prepare_mcp()
    from waku.app import Waku
    from waku.config import Settings

    class ScriptedModel:
        def __init__(self):
            self.messages = self
            self.calls = 0

        def create(self, **kwargs):
            if 'tools' not in kwargs:
                text = '{"retrieve": false, "query": "", "reason": "isolated transport check"}'
                return NS(content=[NS(type='text', text=text)])
            self.calls += 1
            require(self.calls <= 2, 'unexpected agent-loop request')
            if name != 'disabled' and self.calls == 1:
                tool = 'memorywhale_remember' if name == 'write' else 'memorywhale_search_memory'
                args = {'text': PROPOSED_TEXT} if name == 'write' else {'query': MARKER}
                content = [NS(type='tool_use', id='synthetic-call', name=tool, input=args)]
                reason = 'tool_use'
            else:
                content = [NS(type='text', text='OFFLINE_WAKU_TURN_COMPLETE')]
                reason = 'end_turn'
            return NS(content=content, stop_reason=reason, usage=NS(input_tokens=0, output_tokens=0))

    settings = Settings(home=root / 'waku', provider='anthropic', model='scripted',
        small_model='scripted-gate', consolidate_every=1000, graph_workflows=False,
        experimental=False, apple_tools=False, apple_calendar=False,
        google_calendar=False, gh_tool=False, otel_endpoint='')
    app = Waku(settings=settings, client=ScriptedModel())
    try:
        names = {tool['name'] for tool in app.tools.schemas()}
        expected = {'memorywhale_' + tool for tool in TOOLS}
        if name == 'disabled':
            require(not (names & expected), 'removed MCP tools remain registered')
        else:
            require(expected <= names, 'native Waku registry did not discover all six MCP tools')
        matched = app.memory.skills.match('MemoryWhale debugging: compiler failures')
        require(any(s.name == 'memorywhale-debugging' for s in matched), 'guidance skill not discovered')
        for ordinary in ['Hello, how are you?', 'Remember Alex prefers morning meetings', 'Book a meeting on Friday', 'What should I cook for dinner?']:
            require(not any(s.name == 'memorywhale-debugging' for s in app.memory.skills.match(ordinary)), 'skill triggers on unrelated conversation')
        prompt = {'write': 'MemoryWhale debugging: explicitly save this synthetic lesson.',
                  'read': 'MemoryWhale debugging: search the previously saved synthetic lesson.',
                  'disabled': 'Hello, answer this self-contained request.'}[name]
        result = app.respond(prompt)
        require(result.reply == 'OFFLINE_WAKU_TURN_COMPLETE', 'native application turn did not complete')
        if name != 'disabled':
            require(len(result.tool_calls) == 1, 'unexpected tool invocation count')
            output = result.tool_calls[0]['output']
            require(not output.startswith(('MCP call ', 'MCP server ', 'Error')), 'MCP dispatch failed: ' + output)
            if name == 'read':
                require(MARKER in output and '#1]' in output, 'fresh Waku process did not retrieve memory #1: ' + output)
                require('_PROPOSED' not in output, 'pending agent note escaped the normal review policy')
            else:
                require(output.startswith('Saved as memory #2.'), 'expected pending proposal was not saved')
        print('WAKU_RESULT ' + json.dumps({'phase': name, 'pid': os.getpid(),
            'tools': sorted(names & expected), 'tool_calls': result.tool_calls,
            'skill_loaded': True, 'reply': result.reply}), flush=True)
    finally:
        app.close()
        app.conn.close()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--scratch', type=Path, required=True)
    p.add_argument('--bin-dir', type=Path)
    p.add_argument('--waku-source', type=Path)
    p.add_argument('--phase', choices=['write', 'read', 'disabled'], help=argparse.SUPPRESS)
    a = p.parse_args()
    root = a.scratch.resolve()
    if a.phase:
        phase(root, a.phase)
        return
    require(a.bin_dir is not None and a.waku_source is not None, '--bin-dir and --waku-source required')
    revision = subprocess.check_output(['git', '-C', str(a.waku_source), 'rev-parse', 'HEAD'], text=True).strip()
    require(revision == PIN, 'unexpected Waku revision; review the contract before changing the pin')
    require(not subprocess.check_output(['git', '-C', str(a.waku_source), 'status', '--porcelain'], text=True).strip(), 'Waku source must remain unmodified')
    installed = Path(importlib.util.find_spec('waku').origin).parent
    for source in (a.waku_source / 'waku').rglob('*.py'):
        packaged = installed / source.relative_to(a.waku_source / 'waku')
        require(packaged.is_file() and source.read_bytes() == packaged.read_bytes(),
                'installed Waku Python sources differ from the pinned checkout')
    binary = (a.bin_dir / 'mw-mcp').resolve()
    require(binary.is_file(), 'build mw-mcp first')
    root.mkdir(parents=True, mode=0o700, exist_ok=False)
    for folder in ['home', 'workspace', 'waku', 'store', 'config', 'cache', 'tmp']:
        (root / folder).mkdir(mode=0o700)
    (root / 'workspace/.env').write_text('# Empty synthetic traversal boundary. No secrets.\n')
    shutil.copytree(REPO / 'integrations/waku/skills/memorywhale-debugging', root / 'waku/skills/memorywhale-debugging')
    config = root / 'waku/mcp.json'
    config.write_text(json.dumps({'servers': [{'name': 'memorywhale', 'command': str(binary), 'args': [],
        'env': {'MEMORYWHALE_DATA_DIR': str(root / 'store'), 'HOME': str(root / 'home'),
                'XDG_CONFIG_HOME': str(root / 'config'), 'XDG_DATA_HOME': str(root / 'store')}}]}, indent=2))
    env = {'HOME': str(root / 'home'), 'WAKU_HOME': str(root / 'waku'),
        'XDG_CONFIG_HOME': str(root / 'config'), 'XDG_CACHE_HOME': str(root / 'cache'),
        'XDG_DATA_HOME': str(root / 'store'), 'TMPDIR': str(root / 'tmp'),
        'PATH': '/usr/bin:/bin', 'PYTHON_DOTENV_DISABLED': '1', 'PYTHONNOUSERSITE': '1',
        'MEMORYWHALE_DATA_DIR': str(root / 'store')}
    # Seed only explicit synthetic human evidence through the public CLI. Do
    # not turn off review_agent_memories or edit an approval bit in SQLite.
    subprocess.run([str((a.bin_dir / 'mw').resolve()), 'remember', HUMAN_TEXT],
                   cwd=root / 'workspace', env=env, capture_output=True, check=True, timeout=15)
    outcomes = []
    for name in ['write', 'read', 'disabled']:
        if name == 'disabled':
            config.write_text('{"servers": []}\n')  # Only this fresh test profile.
        result = subprocess.run([sys.executable, str(Path(__file__).resolve()), '--scratch', str(root), '--phase', name],
            cwd=root / 'workspace', env=env, capture_output=True, text=True, timeout=75)
        (root / (name + '.stdout')).write_text(result.stdout)
        (root / (name + '.stderr')).write_text(result.stderr)
        require(result.returncode == 0, f'{name} failed; inspect task-owned stdout/stderr')
        records = [json.loads(line[len('WAKU_RESULT '):]) for line in result.stdout.splitlines() if line.startswith('WAKU_RESULT ')]
        require(len(records) == 1, f'{name}: missing native-loop result')
        outcomes += records
    require(len({r['pid'] for r in outcomes}) == 3, 'verification must use fresh processes')
    require((root / 'store/memorywhale.sqlite3').is_file(), 'disconnect deleted MemoryWhale data')
    require((root / 'waku/state.db').is_file(), 'native Waku memory is missing')
    with sqlite3.connect(f'file:{root / "waku/state.db"}?mode=ro', uri=True) as conn:
        require(conn.execute('SELECT COUNT(*) FROM chat_log').fetchone()[0] == 6,
                'native Waku conversation persistence did not survive the three phases')
    with sqlite3.connect(f'file:{root / "store/memorywhale.sqlite3"}?mode=ro', uri=True) as conn:
        notes = conn.execute('SELECT label, author_kind, approved FROM bookmarks ORDER BY id').fetchall()
    check_notes(notes)
    stderr_nonempty = check_stderr(root)
    report = {'status': 'PASS', 'waku_commit': revision, 'waku_version': importlib.metadata.version('waku-agent'),
        'mcp_version': importlib.metadata.version('mcp'), 'python_version': sys.version.split()[0],
        'mw_mcp_sha256': hashlib.sha256(binary.read_bytes()).hexdigest(),
        'phases': outcomes, 'pending_note_preserved': True,
        'stderr_nonempty': stderr_nonempty}
    (root / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
    print(json.dumps(report, indent=2))


if __name__ == '__main__':
    main()
