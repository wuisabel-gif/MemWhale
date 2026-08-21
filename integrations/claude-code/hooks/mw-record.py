#!/usr/bin/env python3
"""Claude Code PostToolUse hook: record every Bash command the agent runs into
MemoryWhale, so next week's session can see what this week's already tried.

Reads the hook JSON payload from stdin (Claude Code's documented PostToolUse
shape), and — only for the Bash tool — shells out to `mw-remember` with the
command, its output, and its exit status. Field names are read defensively
with fallbacks, since hook payload shape can vary slightly across Claude Code
versions; if a field is missing this still records what it can.

Never fails the tool call: any error here is swallowed and the hook exits 0,
so a MemoryWhale hiccup can't block your agent session.

Install: see integrations/claude-code/README.md.
"""
import json
import subprocess
import sys
import shutil

MAX_OUTPUT = 20_000  # cap what we pass as args; mw-remember still redacts secrets


def first(d, *keys, default=""):
    for k in keys:
        v = d.get(k)
        if v:
            return v
    return default


def main():
    try:
        payload = json.load(sys.stdin)
    except Exception:
        return  # not JSON, or nothing to read — nothing to record

    if payload.get("tool_name") != "Bash":
        return

    tool_input = payload.get("tool_input") or {}
    tool_response = payload.get("tool_response") or {}

    command = tool_input.get("command", "").strip()
    if not command:
        return

    cwd = payload.get("cwd") or tool_input.get("cwd") or ""
    stdout = str(first(tool_response, "stdout", "output"))[:MAX_OUTPUT]
    stderr = str(first(tool_response, "stderr"))[:MAX_OUTPUT]
    is_error = bool(
        tool_response.get("is_error")
        or tool_response.get("isError")
        or tool_response.get("interrupted")
    )
    exit_code = "1" if is_error else "0"

    mw_remember = shutil.which("mw-remember")
    if not mw_remember:
        return  # MemoryWhale not installed/on PATH — silently skip

    try:
        subprocess.run(
            [
                mw_remember,
                "--cwd", cwd,
                "--exit-code", exit_code,
                "--stdout", stdout,
                "--stderr", stderr,
                "--notes", "agent:claude-code",
                "--",
                command,
            ],
            capture_output=True,
            timeout=10,
        )
    except Exception:
        pass  # never let a recording failure interrupt the agent


def _selftest():
    """`python3 mw-record.py --selftest` — sanity-checks the payload parsing
    without touching mw-remember or subprocess."""
    assert first({"a": "x"}, "a", "b") == "x"
    assert first({"b": "y"}, "a", "b") == "y"
    assert first({}, "a", "b", default="z") == "z"
    assert first({"a": ""}, "a", "b") == "" or first({"a": "", "b": "y"}, "a", "b") == "y"

    long_text = "x" * 100
    assert len(long_text[:MAX_OUTPUT]) <= MAX_OUTPUT

    print("mw-record.py: selftest OK")


if __name__ == "__main__":
    if len(sys.argv) > 1 and sys.argv[1] == "--selftest":
        _selftest()
    else:
        main()
