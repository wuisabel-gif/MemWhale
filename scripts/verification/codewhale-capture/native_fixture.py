#!/usr/bin/env python3
"""Offline native-TUI driver. Only stdlib, one task-owned scratch, no credentials.

serve --scratch ABS --host ABS --bin-dir ABS starts a loopback model fixture and
real PTY host. send --scratch ABS --text '/plugin list' submits one TUI line;
keys sends literal keys without Return; screen reads accumulated terminal text.
restart starts --fresh in the same isolated profile; stop requests /exit.
Review real output before sending trust tokens or approving individual tools.
"""
import argparse
import fcntl
import hashlib
import http.server
import json
import os
from pathlib import Path
import pty
import re
import shutil
import struct
import subprocess
import termios
import threading
import time
import urllib.request

ANSI = re.compile(r"\x1b\][^\x07]*(?:\x07|\x1b\\)|\x1b\[[0-?]*[ -/]*[@-~]|\x1b[=>]")


def checksum(path):
    digest = hashlib.sha256()
    with path.open('rb') as source:
        for block in iter(lambda: source.read(1024 * 1024), b''):
            digest.update(block)
    return digest.hexdigest()


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('action', choices=['serve', 'send', 'keys', 'screen', 'restart', 'stop'])
    p.add_argument('--scratch', required=True, type=Path)
    p.add_argument('--host', type=Path)
    p.add_argument('--bin-dir', type=Path)
    p.add_argument('--text', default='')
    a = p.parse_args()
    root = a.scratch.resolve()
    if a.action != 'serve':
        port = int((root / 'port').read_text())
        actions = [('keys', a.text), ('keys', '\r')] if a.action == 'send' else [(a.action, a.text)]
        if a.action == 'stop':
            actions = [('keys', '/exit'), ('keys', '\r')]
        # Separate text and Return so the native terminal paste detector has
        # finished before submission. One burst can leave Return in the input.
        for action, text in actions:
            req = urllib.request.Request(f'http://127.0.0.1:{port}/control',
                json.dumps({'action': action, 'text': text}).encode(),
                {'Content-Type': 'application/json'})
            with urllib.request.build_opener(urllib.request.ProxyHandler({})).open(req, timeout=15) as r:
                result = r.read().decode()
        print(result)
        return
    assert a.host and a.bin_dir
    root.mkdir(mode=0o700, parents=True, exist_ok=False)
    for name in ['home', 'config', 'workspace', 'store', 'cache', 'tmp', 'codewhale', 'workspace/child']:
        (root / name).mkdir(parents=True, exist_ok=True)
    os.chmod(root / 'codewhale', 0o700)
    integration = Path(__file__).resolve().parents[3] / 'integrations/codewhale'
    plugin_root = root / 'codewhale/plugins'
    plugin_root.mkdir(mode=0o700)
    for source, name in [('plugin', 'memorywhale'), ('capture-plugin', 'memorywhale-capture')]:
        assert not any(p.is_symlink() for p in (integration / source).rglob('*'))
        shutil.copytree(integration / source, plugin_root / name)
    # Copies are only discovered, never trusted or enabled by this harness.
    env = {'HOME': str(root / 'home'), 'CODEWHALE_HOME': str(root / 'codewhale'),
        'XDG_CONFIG_HOME': str(root / 'config'), 'XDG_CACHE_HOME': str(root / 'cache'),
        'XDG_DATA_HOME': str(root / 'data'), 'GIT_CONFIG_NOSYSTEM': '1',
        'TMPDIR': str(root / 'tmp'), 'MEMORYWHALE_DATA_DIR': str(root / 'store'),
        'MEMORYWHALE_HOOK_DIAGNOSTICS': '1', 'CODEWHALE_TELEMETRY': '0',
        'PATH': f'{a.bin_dir.resolve()}:/usr/bin:/bin:/usr/sbin:/sbin',
        'TERM': 'xterm-256color', 'LANG': 'en_US.UTF-8', 'SHELL': '/bin/bash'}
    # Prevent the native client and capture helper from discovering the real
    # parent repository when this task-owned scratch is inside a checkout.
    subprocess.run(['/usr/bin/git', 'init', '--quiet', str(root / 'workspace')],
                   env=env, check=True)
    state = {'fd': None, 'pid': None, 'session': 0, 'text': '', 'requests': 0}
    lock = threading.Lock()

    def log(name, obj):
        with (root / name).open('a') as f:
            f.write(json.dumps(obj, ensure_ascii=False) + '\n')

    def launch():
        state['session'] += 1
        session = state['session']
        state['text'] = ''
        pid, fd = pty.fork()
        if pid == 0:
            os.chdir(root / 'workspace')
            os.execve(str(a.host.resolve()), [str(a.host.resolve()), '--fresh',
                '--skip-onboarding', '--no-project-config', '--no-mouse-capture',
                '--config', str(root / 'codewhale/config.toml')], env)
        fcntl.ioctl(fd, termios.TIOCSWINSZ, struct.pack('HHHH', 40, 140, 0, 0))
        state.update(pid=pid, fd=fd)
        log('actions.jsonl', {'action': 'launch', 'pid': pid, 'session': session})
        def reader():
            with (root / f'terminal-{session}.raw').open('ab') as raw:
                while True:
                    try:
                        data = os.read(fd, 65536)
                        if not data:
                            break
                    except OSError:
                        break
                    raw.write(data); raw.flush()
                    if b'\x1b[6n' in data:
                        os.write(fd, b'\x1b[1;1R')
                    text = ANSI.sub('', data.decode('utf-8', errors='replace'))
                    with lock:
                        state['text'] += text
                    with (root / f'terminal-{session}.txt').open('a') as out:
                        out.write(text)
        threading.Thread(target=reader, daemon=True).start()

    class Handler(http.server.BaseHTTPRequestHandler):
        def log_message(self, *args):
            pass

        def do_GET(self):
            self.respond({'object': 'list', 'data': [{'id': 'capture-fixture', 'object': 'model', 'owned_by': 'local-fixture'}]})

        def respond(self, body):
            data = json.dumps(body).encode()
            self.send_response(200); self.send_header('Content-Type', 'application/json')
            self.send_header('Content-Length', str(len(data))); self.end_headers(); self.wfile.write(data)

        def do_POST(self):
            body = json.loads(self.rfile.read(int(self.headers.get('Content-Length', '0'))))
            if self.path == '/control':
                action = body['action']; text = body.get('text', '')
                log('actions.jsonl', {'action': action, 'text': text, 'session': state['session'], 'time': time.time()})
                if action in ['send', 'keys', 'stop']:
                    data = '/exit\r' if action == 'stop' else text + ('\r' if action == 'send' else '')
                    os.write(state['fd'], data.encode())
                    time.sleep(1.5)
                if action == 'restart':
                    # Only restart after normal /exit; never kill an unknown process.
                    pid, status = os.waitpid(state['pid'], os.WNOHANG)
                    if not pid:
                        self.respond({'error': 'host still running; send /exit first'}); return
                    log('actions.jsonl', {'action': 'host-exit', 'status': status})
                    launch(); time.sleep(2)
                self.respond({'session': state['session'], 'terminal_tail': state['text'][-22000:]})
                return
            state['requests'] += 1
            n = state['requests']
            log('model-requests.jsonl', {'n': n, 'session': state['session'], 'path': self.path, 'body': body})
            messages = body.get('messages', [])
            markers = ('CAPTURE_FAIL', 'CAPTURE_SUCCESS', 'CAPTURE_DISABLED', 'RECALL_CAPTURE')
            latest = next((i for i in range(len(messages)-1, -1, -1)
                if messages[i].get('role') == 'user'
                and any(marker in str(messages[i].get('content', '')) for marker in markers)), 0)
            prompt = str(messages[latest].get('content', '')) if messages else ''
            results = [m for m in messages[latest+1:] if m.get('role') == 'tool']
            names = [t.get('function', {}).get('name', '') for t in body.get('tools', [])]
            target = None; args = None
            if 'CAPTURE_FAIL' in prompt:
                target = 'bash'; args = {'command': 'printf MW_NATIVE_FAIL; printf MW_NATIVE_STDERR >&2; exit 7'}
            elif 'CAPTURE_SUCCESS' in prompt:
                target = 'bash'; args = {'command': 'printf MW_NATIVE_SUCCESS; pwd'}
            elif 'CAPTURE_DISABLED' in prompt:
                target = 'bash'; args = {'command': 'printf MW_NATIVE_DISABLED'}
            elif 'RECALL_CAPTURE' in prompt:
                target = next((x for x in names if x.endswith('search_memory')), 'mcp_plugin-11-memorywhale-memory_search_memory')
                args = {'query': 'MW_NATIVE_FAIL', 'agent': 'codewhale'}
            # Deferred tool loads are not execution. Retry once with a distinct id.
            deferred = results and ('defer' in str(results[-1]).lower() or 'retry' in str(results[-1]).lower())
            call = target and (not results or (deferred and len(results) < 3))
            delta = {'role': 'assistant'}
            if call:
                delta['tool_calls'] = [{'index': 0, 'id': f'fixture_{n}', 'type': 'function',
                    'function': {'name': target, 'arguments': json.dumps(args)}}]
                finish = 'tool_calls'
            else:
                delta['content'] = 'LOCAL_FIXTURE_TURN_COMPLETE'; finish = 'stop'
            log('model-responses.jsonl', {'n': n, 'delta': delta, 'finish': finish})
            if body.get('stream'):
                chunks = [{'id': f'fixture-{n}', 'object': 'chat.completion.chunk', 'created': 0,
                    'model': 'capture-fixture', 'choices': [{'index': 0, 'delta': delta, 'finish_reason': None}]},
                    {'id': f'fixture-{n}', 'object': 'chat.completion.chunk', 'created': 0,
                    'model': 'capture-fixture', 'choices': [{'index': 0, 'delta': {}, 'finish_reason': finish}]}]
                data = ''.join('data: ' + json.dumps(c) + '\n\n' for c in chunks).encode() + b'data: [DONE]\n\n'
                self.send_response(200); self.send_header('Content-Type', 'text/event-stream')
                self.send_header('Content-Length', str(len(data))); self.end_headers(); self.wfile.write(data)
            else:
                msg = dict(delta)
                for call in msg.get('tool_calls', []):
                    call.pop('index', None)
                self.respond({'id': f'fixture-{n}', 'object': 'chat.completion', 'model': 'capture-fixture',
                    'choices': [{'index': 0, 'message': msg, 'finish_reason': finish}]})

    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), Handler)
    port = server.server_port
    (root / 'port').write_text(str(port))
    (root / 'codewhale/config.toml').write_text(f'''provider = "capture-fixture"
default_text_model = "capture-fixture"
approval_policy = "on-request"
sandbox_mode = "workspace-write"
telemetry = false
[providers.capture-fixture]
kind = "openai-compatible"
base_url = "http://127.0.0.1:{port}/v1"
api_key = "synthetic-fixture-not-a-provider-credential"
model = "capture-fixture"
[hooks]
enabled = true
''')
    evidence = {'host': str(a.host.resolve()), 'host_sha256': checksum(a.host),
        'harness_sha256': checksum(Path(__file__)),
        'bin_dir': str(a.bin_dir.resolve()), 'env': env, 'fixture_port': port,
        'helper_sha256': {name: checksum(a.bin_dir / name) for name in ['mw', 'mw-remember', 'mw-mcp']},
        'plugin_files': {str(p.relative_to(plugin_root)): checksum(p)
                         for p in sorted(plugin_root.rglob('*')) if p.is_file()}}
    (root / 'launch.json').write_text(json.dumps(evidence, indent=2))
    launch()
    server.serve_forever()

if __name__ == '__main__':
    main()
