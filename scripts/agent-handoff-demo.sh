#!/usr/bin/env bash
# Offline Claude -> MemoryWhale -> Rho handoff demonstration.
#
# The Claude events and Rho-style client are simulated. They exercise the real
# mw-mcp stdio protocol, so this demo is deterministic and does
# not require provider credentials, network access, or a user's real database.
set -euo pipefail

command -v python3 >/dev/null || { echo "python3 is required to validate MCP replies" >&2; exit 1; }

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mw_bin="${MW_BIN:-$root/target/debug/mw}"
remember_bin="${MW_REMEMBER_BIN:-$root/target/debug/mw-remember}"
mcp_bin="${MW_MCP_BIN:-$root/target/debug/mw-mcp}"
failure_fixture="$root/tests/fixtures/agent-handoff/claude-post-tool-use-failure.json"
success_fixture="$root/tests/fixtures/agent-handoff/claude-post-tool-use-success.json"

for binary in "$mw_bin" "$remember_bin" "$mcp_bin"; do
  if [[ ! -x "$binary" ]]; then
    echo "missing executable: $binary" >&2
    echo "build with: cargo build -p memorywhale-cli --bins" >&2
    exit 1
  fi
done

for fixture in "$failure_fixture" "$success_fixture"; do
  if [[ ! -r "$fixture" ]]; then
    echo "missing fixture: $fixture" >&2
    exit 1
  fi
done

data_dir="$(mktemp -d "${TMPDIR:-/tmp}/memorywhale-agent-handoff.XXXXXX")"
trap 'rm -rf "$data_dir"' EXIT

run_mw() {
  MEMORYWHALE_DATA_DIR="$data_dir" "$mw_bin" "$@"
}

run_remember() {
  MEMORYWHALE_DATA_DIR="$data_dir" "$remember_bin" "$@"
}

echo "1. A Claude failure fixture is captured through the Claude hook parser"
run_remember --from-hook claude < "$failure_fixture" >/dev/null
run_remember --from-hook claude < "$success_fixture" >/dev/null

claude_results="$(run_mw search "linker error" agent:claude)"
grep -Fq '[command · claude]' <<<"$claude_results"
grep -Fq 'linker error' <<<"$claude_results"
echo "   failure captured and searchable as agent:claude"

echo "2. A Claude success fixture retains the example fix with provenance"
fix_results="$(run_mw search 'cuda' agent:claude)"
grep -Fq 'fix: rerun without --features cuda' <<<"$fix_results"
echo "   fix retained with Claude provenance"

echo "3. An unrelated terminal command is not attributed to Claude"
run_remember \
  --cwd /tmp/memorywhale-agent-handoff \
  --exit-code 0 \
  --stdout 'terminal-only handoff sentinel' \
  -- terminal-only-command >/dev/null
claude_filtered="$(run_mw search 'terminal-only handoff sentinel' agent:claude)"
if grep -Fq 'terminal-only-command' <<<"$claude_filtered"; then
  echo "agent:claude unexpectedly included a terminal-only command" >&2
  exit 1
fi
terminal_results="$(run_mw search 'terminal-only handoff sentinel' agent:terminal)"
grep -Fq '[command · terminal]' <<<"$terminal_results"
grep -Fq 'terminal-only-command' <<<"$terminal_results"
echo "   terminal-only command remains terminal-attributed"

echo "4. A simulated Rho-style client initializes and searches via real mw-mcp stdio"
# This is a protocol fixture, not a live Rho invocation or HTTP transport test.
rho_results="$(
  {
    printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"Rho","version":"handoff-demo"}}}'
    printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}'
    printf '%s\n' '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
    printf '%s\n' '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search_memory","arguments":{"query":"cuda","agent":"claude"}}}'
  } | MEMORYWHALE_DATA_DIR="$data_dir" "$mcp_bin"
)"
printf '%s\n' "$rho_results" | python3 -c '
import json, sys
replies = [json.loads(line) for line in sys.stdin if line.strip()]
assert len(replies) == 3, "unexpected MCP reply count"
by_id = {reply.get("id"): reply for reply in replies}
assert set(by_id) == {1, 2, 3}, "missing MCP response IDs"
for reply in replies:
    assert "error" not in reply, "MCP returned an error"
    assert not reply["result"].get("isError", False), "MCP tool returned an error"
assert by_id[1]["result"]["protocolVersion"] == "2025-11-25", "unexpected negotiated protocol"
assert "search_memory" in [tool["name"] for tool in by_id[2]["result"]["tools"]], "search tool missing"
text = "\n".join(item["text"] for item in by_id[3]["result"]["content"] if item["type"] == "text")
assert "linker error: cannot find -lcudart" in text, "failure evidence missing"
assert "fix: rerun without --features cuda" in text, "fix evidence missing"
assert text.count("agent: claude") == 2, "Claude provenance missing"
assert "terminal-only-command" not in text, "terminal capture leaked into agent-filtered results"
'
echo "   initialize → initialized → tools/list → search_memory succeeded"
echo "   MCP returned both Claude fixture records with provenance"

echo
echo "Offline handoff verified. Agent events and command outcomes were simulated, not executed."
