#!/usr/bin/env python3
"""Check a completed native_fixture run, not generated/mocked database rows."""
import argparse
import hashlib
import json
from pathlib import Path
import sqlite3


def check(condition, message):
    if not condition:
        raise SystemExit('FAIL: ' + message)


def lines(path):
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--scratch', type=Path, required=True)
    args = parser.parse_args()
    root = args.scratch.resolve()
    launch = json.loads((root / 'launch.json').read_text())
    requests = lines(root / 'model-requests.jsonl')
    responses = lines(root / 'model-responses.jsonl')
    actions = lines(root / 'actions.jsonl')
    started = [a for a in actions if a['action'] == 'launch']
    check(len(started) == 2 and started[0]['pid'] != started[1]['pid'], 'two distinct native processes required')
    check(any(a['action'] == 'host-exit' and a['status'] == 0 for a in actions), 'first process must exit normally before restart')
    check('CAPTURE_FAIL' not in str(next(r for r in requests if r['session'] == 2)['body']['messages']), 'fresh conversation must not inherit first-session failure')
    check(all(r['path'] == '/v1/chat/completions' for r in requests), 'unexpected fixture protocol request')
    check(all('Unexpected fields:' not in str(r['body']) for r in requests), 'fixture used an unsupported tool argument')

    connection = sqlite3.connect(f'file:{root / "store/memorywhale.sqlite3"}?mode=ro', uri=True)
    connection.row_factory = sqlite3.Row
    rows = [dict(row) for row in connection.execute('SELECT * FROM command_runs ORDER BY id')]
    check(len(rows) == 2, 'exactly two executions should be captured')
    expected = [('printf MW_NATIVE_FAIL; printf MW_NATIVE_STDERR >&2; exit 7', 7),
                ('printf MW_NATIVE_SUCCESS; pwd', 0)]
    for row, (command, exit_code) in zip(rows, expected):
        check(row['command'] == command and row['exit_code'] == exit_code, 'executed identity/exit mismatch')
        check(row['cwd'] == str(root / 'workspace'), 'record must use actual launch cwd')
        check(row['agent'] == 'codewhale', 'producing agent must be Codewhale')
        check(row['stdout'].startswith('[codewhale: combined stdout/stderr preview]'), 'combined output is not explicitly labeled')
        check(row['stderr'] == '' and '"output_mode":"combined"' in row['notes'], 'combined representation mismatch')
    check('MW_NATIVE_STDERR' in rows[0]['stdout'], 'actual stderr missing from combined capture')
    check(str(root / 'workspace') in rows[1]['stdout'], 'pwd did not independently confirm launch cwd')
    check(not any('MW_NATIVE_DISABLED' in str(row) for row in rows), 'capture persisted a post-disable execution')

    calls = {}
    for response in responses:
        for call in response['delta'].get('tool_calls', []):
            calls[call['id']] = (call['function']['name'], json.loads(call['function']['arguments']))
    results = {}
    for request in requests:
        for message in request['body']['messages']:
            if message.get('role') == 'tool':
                results.setdefault(message['tool_call_id'], (request['session'], request['n'], message['content']))
    disabled = [value for key, value in results.items()
                if calls.get(key, ('', {}))[1].get('command') == 'printf MW_NATIVE_DISABLED']
    check(len(disabled) == 1 and disabled[0][0] == 2, 'disabled command must actually run in second session')
    check('MW_NATIVE_DISABLED' in disabled[0][2] and '[approval]' in disabled[0][2], 'disabled command did not return approved real output')
    # The deferred alias uses literal hyphens; the advertised loaded tool
    # escapes server-name hyphens by doubling them.
    memory_tools = {'mcp_plugin-11-memorywhale-memory_search_memory',
                    'mcp_plugin--11--memorywhale--memory_search_memory'}
    recalls = [value for key, value in results.items()
               if calls.get(key, ('', {}))[0] in memory_tools
               and '[command #1]' in value[2] and 'MW_NATIVE_FAIL' in value[2]]
    check(any(s == 2 and n < disabled[0][1] for s, n, _ in recalls), 'fresh native session did not retrieve captured evidence')
    check(any(s == 2 and n > disabled[0][1] for s, n, _ in recalls), 'MCP retrieval stopped with capture disablement')
    check(all('[approval]' in text for _, _, text in recalls), 'native MCP approval evidence absent')
    terminal = (root / 'terminal-2.txt').read_text()
    check("Plugin bundle 'memorywhale-capture': disabled." in terminal, 'native disable confirmation missing')
    check('memorywhale — active' in terminal and 'computer-use — disabled' in terminal, 'final plugin scope not confirmed')
    check('To resume this session, run codewhale resume' in terminal, 'second native process did not report normal exit')

    summary = {'status': 'PASS', 'native_sessions': 2, 'captured_commands': 2,
        'captured_exit_codes': [r['exit_code'] for r in rows], 'output_mode': 'combined',
        'fresh_session_retrieval': True, 'post_disable_command_executed': True,
        'post_disable_new_records': 0, 'post_disable_mcp_retrieval': True,
        'host_sha256': launch['host_sha256'], 'helper_sha256': launch['helper_sha256'],
        'harness_sha256': launch['harness_sha256'],
        'artifacts_sha256': {name: hashlib.sha256((root / name).read_bytes()).hexdigest()
                            for name in ['actions.jsonl', 'model-requests.jsonl', 'model-responses.jsonl']}}
    (root / 'verification-result.json').write_text(json.dumps(summary, indent=2) + '\n')
    print(json.dumps(summary, indent=2))


if __name__ == '__main__':
    main()
