#!/usr/bin/env bash
# Offline Claude -> MemoryWhale -> Rho handoff demonstration.
#
# The Claude events are fixtures rather than live agent calls. Rho is exercised
# through the real mw-mcp stdio protocol, so this demo is deterministic and does
# not require provider credentials, network access, or a user's real database.
set -euo pipefail

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

echo "1. Claude's failing Bash command is captured by the verified Claude hook"
run_remember --from-hook claude < "$failure_fixture" >/dev/null
run_remember --from-hook claude < "$success_fixture" >/dev/null

claude_results="$(run_mw search "linker error" agent:claude)"
grep -Fq '[command · claude]' <<<"$claude_results"
grep -Fq 'linker error' <<<"$claude_results"
echo "   failure captured and searchable as agent:claude"

echo "2. Claude's verified fix is retained as agent-attributed evidence"
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

echo "4. Rho performs its real legacy MCP handshake and searches the shared store"
# Rho's streamable_http client starts with the legacy initialize lifecycle.
# Keep this sequence explicit so the demo catches regressions in handshake,
# notifications, tools/list, or tools/call handling.
rho_results="$(
  {
    printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"Rho","version":"handoff-demo"}}}'
    printf '%s\n' '{"jsonrpc":"2.0","method":"notifications/initialized","params":{}}'
    printf '%s\n' '{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
    printf '%s\n' '{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"search_memory","arguments":{"query":"cuda","agent":"claude"}}}'
  } | MEMORYWHALE_DATA_DIR="$data_dir" "$mcp_bin"
)"
grep -Fq '"id":1' <<<"$rho_results"
grep -Fq '"protocolVersion":"2025-11-25"' <<<"$rho_results"
grep -Fq '"id":2' <<<"$rho_results"
grep -Fq '"search_memory"' <<<"$rho_results"
grep -Fq '"id":3' <<<"$rho_results"
grep -Fq '"type":"text"' <<<"$rho_results"
grep -Fq 'agent: claude' <<<"$rho_results"
grep -Fq 'fix: rerun without --features cuda' <<<"$rho_results"
echo "   Rho completed initialize → initialized → tools/list → search_memory"
echo "   Rho retrieved Claude's failure and verified fix from the shared store"

echo
echo "Agent handoff complete: Claude captured the failure and fix; Rho found both without rediscovery."
