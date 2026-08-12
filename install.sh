#!/bin/sh
# MemoryWhale one-line installer — downloads prebuilt CLI binaries, no Rust needed.
#
#   curl -fsSL https://raw.githubusercontent.com/wuisabel-gif/MemWhale/main/install.sh | sh
#
# Installs mw, mw-serve, mw-run, mw-remember, mw-view, mw-recover, mw-screenshot, mw-mcp
# into ~/.local/bin (override with PREFIX=/usr/local, needs write access).
set -eu

REPO="wuisabel-gif/MemWhale"
PREFIX="${PREFIX:-$HOME/.local}"
BIN_DIR="$PREFIX/bin"

os="$(uname -s)"
arch="$(uname -m)"
case "$os-$arch" in
  Linux-x86_64)            target="x86_64-unknown-linux-gnu" ;;
  Linux-aarch64|Linux-arm64) target="aarch64-unknown-linux-gnu" ;;
  Darwin-x86_64)           target="x86_64-apple-darwin" ;;
  Darwin-arm64)            target="aarch64-apple-darwin" ;;
  *) echo "unsupported platform: $os-$arch" >&2
     echo "build from source instead: cargo install --git https://github.com/$REPO mw-cli" >&2
     exit 1 ;;
esac

echo "==> Finding latest MemoryWhale release…"
tag="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
       | grep '"tag_name"' | head -1 | sed -E 's/.*"tag_name": *"([^"]+)".*/\1/')"
[ -n "$tag" ] || { echo "could not find a release; is one published yet?" >&2; exit 1; }
ver="${tag#v}"

asset="memorywhale-${ver}-${target}.tar.gz"
url="https://github.com/$REPO/releases/download/$tag/$asset"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

echo "==> Downloading $asset ($tag)…"
curl -fSL "$url" -o "$tmp/$asset"

# Verify the download against the release's published SHA256 checksum. Releases
# from v0.7.0 on ship "<asset>.sha256"; older releases have none, so we skip
# (with a warning) rather than fail. A checksum mismatch is always fatal.
echo "==> Verifying checksum…"
if curl -fsSL "$url.sha256" -o "$tmp/$asset.sha256" 2>/dev/null; then
  expected="$(cut -d' ' -f1 "$tmp/$asset.sha256")"
  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$tmp/$asset" | cut -d' ' -f1)"
  elif command -v shasum >/dev/null 2>&1; then
    actual="$(shasum -a 256 "$tmp/$asset" | cut -d' ' -f1)"
  else
    actual=""
    echo "   WARNING: no sha256 tool (sha256sum/shasum) found; cannot verify." >&2
  fi
  if [ -n "$actual" ]; then
    if [ "$actual" != "$expected" ]; then
      echo "   CHECKSUM MISMATCH — refusing to install." >&2
      echo "   expected: $expected" >&2
      echo "   actual:   $actual" >&2
      exit 1
    fi
    echo "   OK ($expected)"
  fi
else
  echo "   NOTE: no published checksum for $tag; skipping verification." >&2
fi

tar xzf "$tmp/$asset" -C "$tmp"

mkdir -p "$BIN_DIR"
cp "$tmp/memorywhale-${ver}-${target}/bin/"* "$BIN_DIR/"
chmod +x "$BIN_DIR/"mw*

echo "==> Installed to $BIN_DIR"
case ":$PATH:" in
  *":$BIN_DIR:"*) : ;;
  *) echo "   NOTE: $BIN_DIR is not on your PATH. Add this to your shell startup file:"
     echo "         export PATH=\"$BIN_DIR:\$PATH\"" ;;
esac
echo "==> Done. Run 'mw' to get started, or 'mw-serve' for the web dashboard."
