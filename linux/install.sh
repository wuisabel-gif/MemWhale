#!/usr/bin/env bash
#
# Build the MemoryWhale CLI binaries and install them into ~/.local/bin, with
# optional extras (shell hook, dashboard service, completions, man pages).
#
# Usage:
#   linux/install.sh                 # build + install the binaries only
#   linux/install.sh --all           # binaries + hook + service + completions + man
#   linux/install.sh --hook          # also enable per-command recording (this shell rc)
#   linux/install.sh --service       # also enable the dashboard systemd --user service
#   linux/install.sh --completions   # also install bash/zsh completions
#   linux/install.sh --man           # also install man pages
#
# Override the install prefix with PREFIX=... (default ~/.local).
#
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
LINUX_DIR="$REPO_ROOT/linux"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"

DO_HOOK=0 DO_SERVICE=0 DO_COMPLETIONS=0 DO_MAN=0
for arg in "$@"; do
  case "$arg" in
    --all) DO_HOOK=1; DO_SERVICE=1; DO_COMPLETIONS=1; DO_MAN=1 ;;
    --hook) DO_HOOK=1 ;;
    --service) DO_SERVICE=1 ;;
    --completions) DO_COMPLETIONS=1 ;;
    --man) DO_MAN=1 ;;
    -h|--help) sed -n '2,18p' "$0"; exit 0 ;;
    *) echo "unknown option: $arg (try --help)" >&2; exit 1 ;;
  esac
done

BINS=(mw mw-remember mw-serve mw-view mw-recover mw-run mw-screenshot mw-mcp)

echo "==> Building release binaries (CLI only — no Tauri/GTK needed)…"
( cd "$REPO_ROOT" && cargo build --release -p memorywhale-cli )

echo "==> Installing into $BIN_DIR"
mkdir -p "$BIN_DIR"
for b in "${BINS[@]}"; do
  install -m 0755 "$REPO_ROOT/target/release/$b" "$BIN_DIR/$b"
  echo "    $b"
done

case ":$PATH:" in
  *":$BIN_DIR:"*) : ;;
  *) echo "note: $BIN_DIR is not on your PATH — add it in your shell rc." ;;
esac

if [ "$DO_COMPLETIONS" = 1 ]; then
  echo "==> Installing shell completions"
  bash_dir="$PREFIX/share/bash-completion/completions"
  zsh_dir="$PREFIX/share/zsh/site-functions"
  mkdir -p "$bash_dir" "$zsh_dir"
  install -m 0644 "$LINUX_DIR/completions/mw.bash" "$bash_dir/mw"
  install -m 0644 "$LINUX_DIR/completions/_mw" "$zsh_dir/_mw"
  # helper binaries: bash loads completions by command name, so copy under each
  for c in mw-run mw-remember mw-serve mw-view; do
    install -m 0644 "$LINUX_DIR/completions/mw-extra.bash" "$bash_dir/$c"
  done
  install -m 0644 "$LINUX_DIR/completions/_mw-extra" "$zsh_dir/_mw-extra"
  echo "    bash -> $bash_dir/{mw,mw-run,mw-remember,mw-serve,mw-view}"
  echo "    zsh  -> $zsh_dir/{_mw,_mw-extra}  (ensure that dir is on \$fpath)"
fi

if [ "$DO_MAN" = 1 ]; then
  echo "==> Installing man pages"
  man_dir="$PREFIX/share/man/man1"
  mkdir -p "$man_dir"
  install -m 0644 "$LINUX_DIR"/man/*.1 "$man_dir/"
  echo "    $man_dir/mw*.1"
fi

if [ "$DO_HOOK" = 1 ]; then
  echo "==> Enabling per-command recording hook"
  hook_src="$LINUX_DIR/../crates/mw-cli/shell/memorywhale.sh"
  rc="$HOME/.bashrc"; [ -n "${ZSH_VERSION:-}" ] || case "${SHELL:-}" in *zsh) rc="$HOME/.zshrc";; esac
  line=". \"$hook_src\"  # memorywhale-percmd"
  if ! grep -qF "memorywhale-percmd" "$rc" 2>/dev/null; then
    printf '\n%s\n' "$line" >> "$rc"
    echo "    wired into $rc — open a new shell to start recording per command."
  else
    echo "    already wired into $rc."
  fi
fi

if [ "$DO_SERVICE" = 1 ]; then
  echo "==> Enabling dashboard systemd --user service"
  "$LINUX_DIR/systemd/enable-dashboard.sh"
fi

echo
echo "Done. Try:  mw --help   ·   mw-serve --host 127.0.0.1   ·   mw global on"
