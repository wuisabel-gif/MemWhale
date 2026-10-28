#!/usr/bin/env python3
"""Offline checks for the pstack source playbook; not a pstack runtime test.

Every `tool {json}` call in integrations/pstack/sources/memorywhale.md is run
against mw-mcp on a temporary synthetic store, and the strings the playbook
tells an investigator to read are checked against what mw-mcp returns.
"""
import argparse
import json
from pathlib import Path
import re
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
GUIDE = ROOT / "integrations/pstack"
PLAYBOOK = GUIDE / "sources/memorywhale.md"
HEADINGS = ["## Status", "## Requirements", "## Setup", "## Verify", "## Available capabilities",
            "## Example prompt", "## Troubleshooting", "## Uninstall"]
COMMAND = "cargo test -p calc"
ERROR = "error[E0308]: mismatched types in calc::add PSTACK-SYNTHETIC"
FILL = {"<error text>": ERROR, "<command>": COMMAND, "<term>": "calc add", "<project>": "pstack-demo"}


def documented_calls(text):
    """Each `name {json}` line inside a fenced block, in order."""
    blocks = re.findall(r"```\n(.*?)```", text, re.S)
    return [(name, json.loads(args)) for block in blocks
            for name, args in re.findall(r"^\s*(\w+) (\{.*\})$", block, re.M)]


def fill(value):
    return FILL.get(value, value) if isinstance(value, str) else value


def mcp(bins, env, calls):
    """Legacy handshake, tools/list, then one tools/call per (name, args)."""
    requests = [
        {"jsonrpc": "2.0", "id": 0, "method": "initialize", "params": {
            "protocolVersion": "2025-11-25", "capabilities": {},
            "clientInfo": {"name": "pstack-playbook-check", "version": "1"}}},
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        {"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}},
    ] + [{"jsonrpc": "2.0", "id": i + 2, "method": "tools/call",
          "params": {"name": name, "arguments": args}} for i, (name, args) in enumerate(calls)]
    done = subprocess.run([str(bins / "mw-mcp")], env=env, capture_output=True, text=True, timeout=30,
                          input="".join(json.dumps(r) + "\n" for r in requests), check=True)
    replies = {r["id"]: r for r in map(json.loads, done.stdout.splitlines()) if "id" in r}
    schemas = {t["name"]: t["inputSchema"] for t in replies[1]["result"]["tools"]}
    texts = []
    for i in range(len(calls)):
        reply = replies[i + 2]
        assert "error" not in reply and not reply["result"].get("isError"), reply
        texts.append("".join(c["text"] for c in reply["result"]["content"]))
    return schemas, texts


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--bin-dir", default=str(ROOT / "target/debug"))
    bins = Path(parser.parse_args().bin_dir).resolve()
    for name in ["mw", "mw-remember", "mw-mcp"]:
        assert (bins / name).is_file(), f"build {name} before running this check"

    guide = (GUIDE / "README.md").read_text()
    assert [l for l in guide.splitlines() if l.startswith("## ")] == HEADINGS
    assert "Lauren Tan" in guide and "github.com/cursor/plugins/tree/main/pstack" in guide
    text = PLAYBOOK.read_text()
    assert "Never call `remember`" in text
    calls = documented_calls(text)
    tools = subprocess.run([str(bins / "mw-mcp"), "--list-tools"], capture_output=True, text=True,
                           check=True).stdout.split()
    assert calls and {name for name, _ in calls} <= set(tools) - {"remember"}, calls

    with tempfile.TemporaryDirectory(prefix="mw-pstack-") as temporary:
        root = Path(temporary).resolve()
        repo = root / "calc"
        repo.mkdir()
        env = {"HOME": str(root / "home"), "XDG_CONFIG_HOME": str(root / "config"),
               "XDG_DATA_HOME": str(root / "data"), "PATH": "/usr/bin:/bin",
               "MEMORYWHALE_DATA_DIR": str(root / "store")}

        def run(code, *command, stderr=""):
            subprocess.run([str(bins / "mw-remember"), "--cwd", str(repo), "--exit-code", str(code),
                            "--stderr", stderr, "--notes", "project:pstack-demo", "--", *command],
                           env=env, capture_output=True, check=True, timeout=10)

        for filler in (["git", "status"], ["ls"], ["cargo", "fmt"]):
            run(0, *filler)
        # Failed, passed, failed again: a fix that did not hold.
        run(101, *COMMAND.split(), stderr=ERROR)
        run(0, *COMMAND.split())
        run(101, *COMMAND.split(), stderr=ERROR)
        subprocess.run([str(bins / "mw"), "remember", "fix: calc add needs i64 (verified)"],
                       env=env, capture_output=True, check=True, timeout=10)

        filled = [(name, {k: fill(v) for k, v in args.items()}) for name, args in calls]
        nonsense = ("search_memory", {"query": "qqq-nothing-zzz"})
        schemas, out = mcp(bins, env, filled + [nonsense])
        for name, args in calls:
            properties = schemas[name].get("properties", {})
            assert set(args) <= set(properties), (name, args)
            for key, value in args.items():
                assert value in properties[key].get("enum", [value]), (name, key, value)
        by_call = dict(zip([f"{n} {json.dumps(a, sort_keys=True)}" for n, a in filled], out))

        def result(name, **args):
            return by_call[f"{name} {json.dumps({k: fill(v) for k, v in args.items()}, sort_keys=True)}"]

        stats = json.loads(result("stats"))
        assert stats["memories"] == 7 and stats["errors"] == 2, stats  # six runs, one lesson
        similar = result("similar_failures", error_text="<error text>", command="<command>")
        assert "You've hit this 2 times: a later run succeeded 1 of 2 times. [" in similar, similar
        assert "the latest run is green" not in similar
        errors = result("recent_errors", limit=64)
        assert f"`{COMMAND}` (exit 101" in errors and f"cwd: {repo}" in errors, errors
        # Both failure rows contain the terms, so BM25 scores them 0 within
        # the failures view: the snippet, not the score, shows the match.
        failures = result("search_memory", query="<term>", mode="failures")
        assert failures.count("calc::add") == 2, failures
        evidence = result("search_memory", query="<term>", mode="evidence", project="<project>")
        assert "keyword match (BM25" in evidence.split("\n- ")[0], evidence
        lessons = result("search_memory", query="<term>", mode="lessons")
        assert "[note #" in lessons and "remembered by you on " in lessons, lessons
        assert "cargo" in result("get_context", project="<project>")
        # The relevance-floor warning: a query that matches nothing still
        # returns rows, and their reasons name no keyword match.
        assert "- [" in out[-1] and "keyword match" not in out[-1], out[-1]
        assert "qqq" not in out[-1].lower()
    print(f"PASS: guide headings and credit, {len(calls)} playbook calls against an isolated "
          "synthetic store (not a pstack runtime test)")


if __name__ == "__main__":
    main()
