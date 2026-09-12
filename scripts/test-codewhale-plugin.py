#!/usr/bin/env python3
"""Offline checks for the shipped bundle; not a Codewhale runtime certification."""
import argparse
import json
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
BUNDLE = ROOT / "integrations/codewhale/plugin"

def unique(pairs):
    obj = {}
    for key, value in pairs:
        assert key not in obj, f"duplicate configuration key: {key}"
        obj[key] = value
    return obj

def main():
    if not __debug__:
        raise SystemExit("Run this verification without Python optimization")
    parser = argparse.ArgumentParser()
    parser.add_argument("--bin-dir", default=str(ROOT / "target/debug"))
    args = parser.parse_args()
    bins = Path(args.bin_dir).resolve()
    manifest = json.loads((BUNDLE / "plugin.json").read_text(), object_pairs_hook=unique)
    mcp = json.loads((BUNDLE / "mcp.json").read_text(), object_pairs_hook=unique)
    assert manifest["name"] == "memorywhale"
    assert manifest["$schema"] == "https://agent-plugins.org/schemas/plugin.json"
    extension = manifest["extensions"]["net.codewhale"]
    assert "hooks" not in extension and "native" not in extension
    assert (BUNDLE / extension["commands"]["path"]).is_dir()
    server = mcp["mcpServers"]["memory"]
    assert server["command"] == "sh"
    assert server["args"] == ["scripts/mcp.sh"]
    assert server["env"] == {"MEMORYWHALE_DATA_DIR": "${MEMORYWHALE_DATA_DIR}"}
    assert not any(p.is_symlink() for p in BUNDLE.rglob("*"))
    script = BUNDLE / server["args"][0]
    subprocess.run(["sh", "-n", str(script)], check=True)
    for name in ["mw", "mw-mcp"]:
        assert (bins / name).is_file(), f"build {name} before running this check"
    with tempfile.TemporaryDirectory(prefix="mw-codewhale-plugin-") as temporary:
        root = Path(temporary).resolve()
        env = {"HOME": str(root / "home"), "XDG_CONFIG_HOME": str(root / "config"),
               "XDG_DATA_HOME": str(root / "data"), "PATH": str(bins) + ":/usr/bin:/bin"}
        for value in [None, "", "relative-store"]:
            trial = env.copy()
            if value is not None:
                trial["MEMORYWHALE_DATA_DIR"] = value
            result = subprocess.run(["sh", str(script)], cwd=BUNDLE, env=trial,
                                    input="", capture_output=True, text=True, timeout=10)
            assert result.returncode == 64 and not result.stdout
            assert "explicit absolute MEMORYWHALE_DATA_DIR" in result.stderr
            assert not (root / "data").exists(), "missing selection must not open a default store"
        env["MEMORYWHALE_DATA_DIR"] = str(root / "synthetic-store")
        marker = "CODEWHALE_PLUGIN_SYNTHETIC"
        subprocess.run([str(bins / "mw"), "remember", marker], env=env, cwd=root,
                       capture_output=True, check=True, timeout=10)
        meta = {"io.modelcontextprotocol/protocolVersion": "2026-07-28",
                "io.modelcontextprotocol/clientInfo": {"name": "bundle-contract-check", "version": "1"},
                "io.modelcontextprotocol/clientCapabilities": {}}
        requests = [
            {"jsonrpc": "2.0", "id": 1, "method": "server/discover", "params": {"_meta": meta}},
            {"jsonrpc": "2.0", "id": 2, "method": "tools/call", "params": {
                "name": "search_memory", "arguments": {"query": marker}, "_meta": meta}},
        ]
        result = subprocess.run(["sh", str(script)], cwd=BUNDLE, env=env,
                                input="".join(json.dumps(v) + "\n" for v in requests),
                                capture_output=True, text=True, check=True, timeout=10)
        replies = [json.loads(line) for line in result.stdout.splitlines()]
        reply = next(r for r in replies if r.get("id") == 2)
        assert "error" not in reply and reply["result"]["isError"] is False
        assert marker in json.dumps(reply)
    print("PASS: bundle assets, shell syntax, explicit store selection, and isolated MCP retrieval (not a native Codewhale test)")

if __name__ == "__main__":
    main()
