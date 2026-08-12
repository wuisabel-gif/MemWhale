#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

tools="$(cargo run --quiet -p memorywhale-cli --bin mw-mcp -- --list-tools)"
count=0
while IFS= read -r tool; do
  [[ -n "$tool" ]] || continue
  grep -Fq "\`$tool\`" docs/reference/mcp.md
  grep -Fq "\`$tool\`" integrations/README.md
  count=$((count + 1))
done <<< "$tools"

documented_count="$(sed -n '/^| `.*` |/p' docs/reference/mcp.md | wc -l | tr -d ' ')"
if [[ "$count" -ne "$documented_count" ]]; then
  echo "MCP tool count differs: runtime=$count docs=$documented_count" >&2
  exit 1
fi

for binary in crates/mw-cli/src/bin/*.rs; do
  name="$(basename "$binary" .rs)"
  grep -Fq "$name" docs/reference/cli.md
done

if grep -Eq 'Binaries \\(`src-tauri/src/bin/`\\)|Build the helper binaries from `src-tauri`' \
  AGENTS.md HANDOFF.md; then
  echo "obsolete CLI source path remains in operational documentation" >&2
  exit 1
fi

echo "documentation references match $count MCP tools and all CLI binaries"
