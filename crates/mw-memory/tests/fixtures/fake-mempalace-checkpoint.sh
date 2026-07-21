#!/bin/sh
# A fake `mempalace-mcp` for the checkpoint path: handshake + one
# `mempalace_checkpoint` call. Response ids match the client's request order
# (initialize = 1, tools/call = 2). No network, no real mempalace.
while IFS= read -r line; do
  case "$line" in
    *'"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{},"serverInfo":{"name":"mempalace","version":"fake"}}}'
      ;;
    *'mempalace_checkpoint'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"{\"added\":[{\"drawer_id\":\"a\"},{\"drawer_id\":\"b\"}],\"duplicates\":[{\"drawer_id\":\"c\"}],\"errors\":[]}"}]}}'
      ;;
  esac
done
