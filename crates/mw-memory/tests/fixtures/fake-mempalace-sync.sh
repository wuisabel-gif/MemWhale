#!/bin/sh
# A fake `mempalace-mcp` for the id-sync path: handshake + any number of
# `mempalace_add_drawer` / `mempalace_delete_drawer` calls over one session.
# Each response echoes the request's own id (the client increments it per call),
# adds hand out distinct drawer ids (drawer-1, drawer-2, ...), deletes succeed.
# No network, no real mempalace.
n=0
while IFS= read -r line; do
  id=$(printf '%s' "$line" | sed -n 's/.*"id":\([0-9][0-9]*\).*/\1/p')
  case "$line" in
    *'"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":'"$id"',"result":{"protocolVersion":"2024-11-05","capabilities":{},"serverInfo":{"name":"mempalace","version":"fake"}}}'
      ;;
    *'mempalace_add_drawer'*)
      n=$((n + 1))
      printf '%s\n' '{"jsonrpc":"2.0","id":'"$id"',"result":{"content":[{"type":"text","text":"{\"success\":true,\"drawer_id\":\"drawer-'"$n"'\",\"wing\":\"memorywhale\",\"room\":\"note\"}"}]}}'
      ;;
    *'mempalace_delete_drawer'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":'"$id"',"result":{"content":[{"type":"text","text":"{\"success\":true}"}]}}'
      ;;
  esac
done
