#!/bin/sh
# A legacy fake `mempalace-mcp` for the checkpoint path: handshake + one
# `mempalace_checkpoint` call. Response ids match the client's request order
# (discovery = 1, initialize = 2, tools/call = 3). No network, no real mempalace.
while IFS= read -r line; do
  case "$line" in
    *'"server/discover"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":1,"error":{"code":-32601,"message":"Method not found"}}'
      ;;
    *'"initialize"'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":2,"result":{"protocolVersion":"2025-11-25","capabilities":{},"serverInfo":{"name":"mempalace","version":"fake"}}}'
      ;;
    *'mempalace_checkpoint'*)
      printf '%s\n' '{"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":"{\"added\":[{\"drawer_id\":\"a\"},{\"drawer_id\":\"b\"}],\"duplicates\":[{\"drawer_id\":\"c\"}],\"errors\":[]}"}]}}'
      ;;
  esac
done
