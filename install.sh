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
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
release_json="$tmp/latest-release.json"

# --- release-tag library (extracted verbatim by tests/install/run-tests.sh) ---
# Pulls the latest release tag out of GitHub's /releases/latest JSON payload.
# Uses `jq` when available; falls back to a portable single-line grep/sed
# pipeline so the one-line installer keeps working on machines without jq.
latest_release_tag() {
  file="$1"
  if command -v jq >/dev/null 2>&1; then
    jq -r '.tag_name? // empty' "$file" 2>/dev/null || true
  else
    grep '"tag_name"' "$file" | head -1 | sed -E 's/.*"tag_name" *: *"([^"]+)".*/\1/'
  fi
}

# Validates a candidate tag before it is used to construct asset URLs.
# MemoryWhale publishes semver tags with a "v" prefix (e.g. v0.7.0). The core
# MAJOR.MINOR.PATCH shape is required; an optional "-prerelease"/"+build"
# suffix is accepted to stay aligned with semver. GitHub's /releases/latest
# endpoint already excludes prereleases and drafts, so the tag the installer
# sees is effectively always stable. Empty, null (empty after extraction),
# multiline, and other unexpected values are rejected here so a malformed
# response cannot reach URL construction. The shape gate also guarantees the
# tag cannot carry path-breaking characters.
validate_tag() {
  tag="$1"
  [ -n "$tag" ] || return 1
  case "$tag" in
    *"
"*) return 1 ;;
  esac
  printf '%s\n' "$tag" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$'
}
# --- end release-tag library ---

if [ -n "${GITHUB_TOKEN:-}" ]; then
  release_status=0
  curl -fsSL -H "Authorization: Bearer $GITHUB_TOKEN" \
    "https://api.github.com/repos/$REPO/releases/latest" -o "$release_json" || release_status=$?
else
  release_status=0
  curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
    -o "$release_json" || release_status=$?
fi
if [ "$release_status" -ne 0 ]; then
  echo "== ERROR: could not fetch release metadata from the GitHub API." >&2
  echo "   Check your network connection. Unauthenticated GitHub API requests" >&2
  echo "   are rate-limited; if you are hitting the limit, set GITHUB_TOKEN." >&2
  echo "   Alternatively build from source: cargo install --git https://github.com/$REPO mw-cli" >&2
  exit 1
fi

tag="$(latest_release_tag "$release_json")" || tag=""
if ! validate_tag "$tag"; then
  echo "== ERROR: could not determine a valid release tag (is a release published yet?)." >&2
  [ -n "$tag" ] && echo "   unexpected value from the API: $tag" >&2
  exit 1
fi
ver="${tag#v}"

asset="memorywhale-${ver}-${target}.tar.gz"
url="https://github.com/$REPO/releases/download/$tag/$asset"

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
