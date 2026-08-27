#!/bin/sh
# A fake `mempalace-mcp`: newline-delimited JSON-RPC 2.0 on stdio, just enough
# to exercise the client (discovery + one tools/call). No network, no mempalace.
RESULTS='[{"id":143,"text":"I use Rust for systems software.","score":0.87,"tags":["rust"],"created_at":"2026-06-07T12:00:00Z","last_used":"2026-06-27T12:00:00Z","mentions":27,"importance":0.98},{"id":22,"text":"Use Tokio for the async runtime.","score":0.41}]'
ESCAPED=$(printf '%s' "$RESULTS" | sed 's/"/\\"/g')
while IFS= read -r line; do
  case "$line" in
    *'"method":"server/discover"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"resultType":"complete","supportedVersions":["2026-07-28"],"capabilities":{"tools":{}}}}'
      ;;
    *'"method":"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{}},"serverInfo":{"name":"mempalace","version":"0.0.0-fake"}}}'
      ;;
    *'"method":"tools/list"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"tools":[{"name":"search","inputSchema":{"type":"object","properties":{"query":{"type":"string"}}}}]}}'
      ;;
    *'"method":"tools/call"'*)
      printf '%s\n' "{\"jsonrpc\":\"2.0\",\"id\":3,\"result\":{\"content\":[{\"type\":\"text\",\"text\":\"$ESCAPED\"}]}}"
      ;;
  esac
done
