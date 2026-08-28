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

python3 - <<'PY'
import json
from pathlib import Path

spec = json.loads(Path("docs/reference/api.openapi.json").read_text())
assert spec.get("openapi", "").startswith("3.")
assert "mw_token" in spec["components"]["securitySchemes"]
for response in spec["components"]["responses"].values():
    media = response.get("content", {}).get("application/json")
    if media is not None:
        assert "schema" in media
for path_item in spec["paths"].values():
    for operation in path_item.values():
        if not isinstance(operation, dict):
            continue
        assert any(
            isinstance(requirement.get("mw_token"), list)
            for requirement in operation.get("security", [])
        )
        assert "401" in operation.get("responses", {})
        for response in operation.get("responses", {}).values():
            media = response.get("content", {}).get("application/json")
            if media is not None:
                assert "schema" in media
assert spec["components"]["schemas"]["SearchResponse"]["allOf"][1]["properties"]["data"]["properties"]["results"]["maxItems"] == 50
assert spec["components"]["schemas"]["CommandResponse"]["allOf"][1]["properties"]["data"]["$ref"] == "#/components/schemas/Command"
assert "command_id" in spec["components"]["schemas"]["SearchResult"]["required"]
PY

echo "documentation references match $count MCP tools and all CLI binaries"
