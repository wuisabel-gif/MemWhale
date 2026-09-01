#!/usr/bin/env bash
# Offline Claude -> MemoryWhale -> Rho handoff demonstration.
#
# The Claude event is a fixture rather than a live agent call. Rho is exercised
# through the real mw-mcp stdio protocol, so this demo is deterministic and does
# not require provider credentials, network access, or a user's real database.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
mw_bin="${MW_BIN:-$root/target/debug/mw}"
remember_bin="${MW_REMEMBER_BIN:-$root/target/debug/mw-remember}"
mcp_bin="${MW_MCP_BIN:-$root/target/debug/mw-mcp}"
fixture="$root/tests/fixtures/agent-handoff/claude-post-tool-use-failure.json"

for binary in "$mw_bin" "$remember_bin" "$mcp_bin"; do
  if [[ ! -x "$binary" ]]; then
    echo "missing executable: $binary" >&2
    echo "build with: cargo build -p memorywhale-cli --bins" >&2
    exit 1
  fi
done

if [[ ! -r "$fixture" ]]; then
  echo "missing fixture: $fixture" >&2
  exit 1
fi

data_dir="$(mktemp -d "${TMPDIR:-/tmp}/memorywhale-agent-handoff.XXXXXX")"
trap 'rm -rf "$data_dir"' EXIT

run_mw() {
  MEMORYWHALE_DATA_DIR="$data_dir" "$mw_bin" "$@"
}

run_remember() {
  MEMORYWHALE_DATA_DIR="$data_dir" "$remember_bin" "$@"
}

echo "1. Claude's Bash failure is captured by the verified Claude hook"
run_remember --from-hook claude < "$fixture" >/dev/null

claude_results="$(run_mw search "linker error" agent:claude)"
grep -Fq '[command · claude]' <<<"$claude_results"
grep -Fq 'linker error' <<<"$claude_results"
echo "   captured and searchable as agent:claude"

echo "2. An unrelated terminal command is not attributed to Claude"
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

echo "3. Rho searches the same local store through MCP"
meta='{"io.modelcontextprotocol/protocolVersion":"2026-07-28","io.modelcontextprotocol/clientInfo":{"name":"Rho","version":"handoff-demo"},"io.modelcontextprotocol/clientCapabilities":{}}'
rho_results="$(
  {
    printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"server/discover\",\"params\":{\"_meta\":$meta}}"
    printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"tools/list\",\"params\":{\"_meta\":$meta}}"
    printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":3,\"method\":\"tools/call\",\"params\":{\"name\":\"search_memory\",\"arguments\":{\"query\":\"linker error\",\"agent\":\"claude\"},\"_meta\":$meta}}"
  } | MEMORYWHALE_DATA_DIR="$data_dir" "$mcp_bin"
)"
grep -Fq '"id":3' <<<"$rho_results"
grep -Fq '"isError":false' <<<"$rho_results"
grep -Fq 'agent: claude' <<<"$rho_results"
grep -Fq 'linker error' <<<"$rho_results"
echo "   Rho retrieved Claude's prior failure from the shared store"

echo
echo "Agent handoff complete: Claude captured the failure; Rho found it without rediscovery."
