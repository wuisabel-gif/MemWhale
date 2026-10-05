#!/usr/bin/env bash
#
# Install and enable the MemoryWhale dashboard as a systemd --user service.
#
# Why: the dashboard (mw-serve) is a long-running process. Started by hand over
# SSH it dies the moment the session detaches (systemd-logind kills the user's
# processes on logout). A --user service plus lingering keeps it up across
# logout, disconnect, and reboot.
#
# Usage:
#   linux/systemd/enable-dashboard.sh           # enable + start, bound to 127.0.0.1
#   linux/systemd/enable-dashboard.sh --lan     # enable + start on the LAN, token required
#   linux/systemd/enable-dashboard.sh --disable # stop + disable
#
# --lan never puts the token in the unit: mw-serve reads (or creates, on first
# start) serve.token in the MemoryWhale data directory, readable only by you.
#
set -euo pipefail

UNIT="memorywhale-dashboard.service"
SRC_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DEST_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"

LISTEN="--host 127.0.0.1"
case "${1:-}" in
  --disable)
    systemctl --user disable --now "$UNIT" 2>/dev/null || true
    echo "MemoryWhale dashboard disabled."
    exit 0 ;;
  --lan) LISTEN="--lan" ;;
  "") ;;
  *) echo "usage: $0 [--lan | --disable]" >&2; exit 1 ;;
esac

# Resolve the mw-serve binary (PATH first, then the common user-install dir).
MW_SERVE="$(command -v mw-serve 2>/dev/null || true)"
if [ -z "$MW_SERVE" ] && [ -x "$HOME/.local/bin/mw-serve" ]; then
  MW_SERVE="$HOME/.local/bin/mw-serve"
fi
if [ -z "$MW_SERVE" ]; then
  echo "error: mw-serve not found on PATH or in ~/.local/bin." >&2
  echo "       Build/install it first:  linux/install.sh" >&2
  exit 1
fi

mkdir -p "$DEST_DIR"
sed -e "s|__MW_SERVE__|$MW_SERVE|g" -e "s|--host 127.0.0.1|$LISTEN|" \
  "$SRC_DIR/$UNIT" > "$DEST_DIR/$UNIT"

systemctl --user daemon-reload
systemctl --user enable --now "$UNIT"

# Keep the user manager (and thus the service) running with no active login.
if command -v loginctl >/dev/null 2>&1; then
  loginctl enable-linger "$USER" 2>/dev/null \
    || echo "note: could not enable linger (needs privileges); the service will" \
            "stop on logout until you run: sudo loginctl enable-linger $USER"
fi

echo
if [ "$LISTEN" = "--lan" ]; then
  echo "MemoryWhale dashboard enabled on the LAN  ->  http://<this-machine-ip>:7071"
  echo "  token  : $MW_SERVE --lan --print-token   (paste it into the sign-in form;"
  echo "           never put it in a URL)"
else
  echo "MemoryWhale dashboard enabled  ->  http://127.0.0.1:7071"
fi
echo "  binary : $MW_SERVE"
echo "  unit   : $DEST_DIR/$UNIT"
echo "  status : systemctl --user status $UNIT"
echo "  logs   : journalctl --user -u $UNIT -f"
if [ "$LISTEN" != "--lan" ]; then
  echo
  echo "For LAN access (e.g. open it from a laptop), rerun with --lan. The token"
  echo "is kept in serve.token in the data directory, not in the unit file."
fi
