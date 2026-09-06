#!/usr/bin/env python3
"""Exercise installed release binaries without using the user's real database.

Usage: python3 scripts/smoke-release.py <installed-bin-directory> <version>
Only loopback HTTP is used. No model, GitHub API, or credentials are needed.
"""

import json
import os
from pathlib import Path
import socket
import sqlite3
import subprocess
import sys
import tempfile
import time
from urllib.error import URLError
from urllib.request import Request, build_opener, ProxyHandler


BINARIES = (
    "mw", "mw-remember", "mw-serve", "mw-view", "mw-recover", "mw-run",
    "mw-screenshot", "mw-mcp",
)


def check(condition, message):
    if not condition:
        raise RuntimeError(message)


def main():
    if len(sys.argv) != 3:
        raise SystemExit(__doc__)
    bin_dir = Path(sys.argv[1]).resolve()
    version = sys.argv[2]
    for name in BINARIES:
        check(os.access(bin_dir / name, os.X_OK), f"missing executable: {name}")

    with tempfile.TemporaryDirectory(prefix="memorywhale-release-smoke-") as temp:
        work = Path(temp)
        data = work / "data"
        home = work / "home"
        home.mkdir()
        # Do not inherit capture rules, model credentials, or server tokens.
        env = {
            "PATH": f"{bin_dir}{os.pathsep}{os.defpath}",
            "HOME": str(home),
            "MEMORYWHALE_DATA_DIR": str(data),
            "LANG": "en_US.UTF-8",
        }
        for key in ("SystemRoot", "WINDIR", "TMPDIR", "TMP", "TEMP"):
            if key in os.environ:
                env[key] = os.environ[key]
        (work / ".mwignore").write_text('capture = "full"\n', encoding="utf-8")

        def run(binary, *args, payload=None):
            result = subprocess.run(
                [str(bin_dir / binary), *args], cwd=work, env=env,
                input=payload, text=True, capture_output=True, timeout=30,
            )
            check(result.returncode == 0, f"{binary} failed: {result.stderr}")
            return result.stdout

        check(run("mw", "--version").strip() == f"mw {version}", "version mismatch")
        check(not data.exists(), "--version should not create a database")
        check("github context" in run("mw", "--help"), "new CLI help missing")

        failure = "linker error: release smoke failure"
        run("mw-remember", "--from-hook", "claude", payload=json.dumps({
            "hook_event_name": "PostToolUseFailure", "tool_name": "Bash",
            "cwd": temp, "tool_input": {"command": "smoke-claude-failure"},
            "error": failure, "exit_code": 1,
        }))
        run("mw-remember", "--from-hook", "rho", payload=json.dumps({
            "event": "after_tool_use", "workspace": {"root": temp},
            "payload": {"tool": {"name": "bash"}, "status": "failed",
                        "failure": {"message": "rho release smoke failure"}},
        }))
        run("mw-remember", "--cwd", temp, "--exit-code", "0", "--stdout", "verified",
            "--", "smoke-terminal-command")
        check("[command · claude]" in run("mw", "search", "agent:claude"), "Claude provenance missing")
        check("[command · rho]" in run("mw", "search", "agent:rho"), "Rho provenance missing")
        check("[command · terminal]" in run("mw", "search", "agent:terminal"), "terminal provenance missing")
        with sqlite3.connect(data / "memorywhale.sqlite3") as conn:
            check(conn.execute("PRAGMA user_version").fetchone()[0] == 10, "migration 10 missing")
            agents = [row[0] for row in conn.execute("SELECT agent FROM command_runs ORDER BY id")]
            check(agents == ["claude", "rho", None], f"unexpected agents: {agents}")

        requests = [
            {"jsonrpc": "2.0", "id": 1, "method": "initialize", "params": {
                "protocolVersion": "2025-11-25", "capabilities": {},
                "clientInfo": {"name": "release-smoke", "version": "1"}}},
            {"jsonrpc": "2.0", "method": "notifications/initialized", "params": {}},
            {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
            {"jsonrpc": "2.0", "id": 3, "method": "tools/call", "params": {
                "name": "search_memory", "arguments": {"query": failure, "agent": "claude"}}},
        ]
        replies = [json.loads(line) for line in run("mw-mcp", payload="".join(
            json.dumps(request) + "\n" for request in requests
        )).splitlines() if line.strip()]
        by_id = {reply["id"]: reply for reply in replies}
        check(set(by_id) == {1, 2, 3}, "missing MCP replies")
        for reply in replies:
            check("error" not in reply, f"MCP error: {reply}")
            check(not reply["result"].get("isError", False), f"MCP tool error: {reply}")
        check(by_id[1]["result"]["protocolVersion"] == "2025-11-25", "MCP negotiation failed")
        check(len(by_id[2]["result"]["tools"]) == 6, "MCP tool discovery failed")
        check("smoke-claude-failure" in by_id[3]["result"]["content"][0]["text"], "MCP recall failed")

        # Choose a currently unused loopback port. Readiness is bounded and a
        # bind failure is reported rather than silently talking to another app.
        with socket.socket() as reserve:
            reserve.bind(("127.0.0.1", 0))
            port = reserve.getsockname()[1]
        opener = build_opener(ProxyHandler({}))
        with (work / "server.log").open("w+") as log:
            server = subprocess.Popen(
                [str(bin_dir / "mw-serve"), "--host", "127.0.0.1", "--port", str(port), "--api"],
                cwd=work, env=env, stdin=subprocess.DEVNULL, stdout=log, stderr=log,
            )
            try:
                url = f"http://127.0.0.1:{port}"
                deadline = time.monotonic() + 15
                while True:
                    check(server.poll() is None, "mw-serve exited during startup")
                    try:
                        with opener.open(url + "/api/v1/health", timeout=1) as response:
                            health = json.load(response)
                        break
                    except (URLError, TimeoutError):
                        check(time.monotonic() < deadline, "mw-serve did not become ready")
                        time.sleep(0.05)
                check(health["data"]["version"] == version, "API version mismatch")
                with opener.open(url + "/api/v1/search?q=smoke&agent=claude", timeout=5) as response:
                    hits = json.load(response)["data"]["results"]
                check(len(hits) == 1 and hits[0]["agent"] == "claude", "API agent filtering failed")
                with opener.open(url + "/", timeout=5) as response:
                    check("agent:claude" in response.read().decode(), "dashboard provenance missing")
                request = Request(url + "/mcp", data=json.dumps(requests[0]).encode(), headers={
                    "Content-Type": "application/json", "Accept": "application/json, text/event-stream"})
                with opener.open(request, timeout=5) as response:
                    check(json.load(response)["result"]["protocolVersion"] == "2025-11-25", "HTTP MCP failed")
            except Exception:
                log.seek(0)
                print(log.read(), file=sys.stderr)
                raise
            finally:
                server.terminate()
                try:
                    server.wait(timeout=5)
                except subprocess.TimeoutExpired:
                    server.kill()
                    server.wait(timeout=5)
    print(f"release {version} smoke passed: binaries, schema, capture, CLI, MCP, API, dashboard")


if __name__ == "__main__":
    main()
