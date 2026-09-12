#!/bin/sh
# Invoked from Codewhale's reviewed, staged plugin tree. Never select a store implicitly.
set -eu
case "${MEMORYWHALE_DATA_DIR:-}" in
  /*) ;;
  *) printf '%s\n' 'MemoryWhale requires an explicit absolute MEMORYWHALE_DATA_DIR for this plugin.' >&2; exit 64 ;;
esac
exec mw-mcp
