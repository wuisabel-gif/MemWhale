#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

cli_version="$(awk -F'"' '/^version = / { print $2; exit }' crates/mw-cli/Cargo.toml)"
package_version="$(node -p "require('./package.json').version")"
lock_version="$(node -p "require('./package-lock.json').version")"
tauri_version="$(node -p "require('./src-tauri/tauri.conf.json').version")"

failed=0
for entry in \
  "crates/mw-cli/Cargo.toml:$cli_version" \
  "package.json:$package_version" \
  "package-lock.json:$lock_version" \
  "src-tauri/tauri.conf.json:$tauri_version"
do
  file="${entry%%:*}"
  version="${entry#*:}"
  if [[ "$version" != "$cli_version" ]]; then
    echo "version mismatch: $file is $version; expected $cli_version" >&2
    failed=1
  fi
done

if [[ $# -gt 0 ]]; then
  expected="${1#v}"
  if [[ "$expected" != "$cli_version" ]]; then
    echo "tag mismatch: requested $expected; product version is $cli_version" >&2
    failed=1
  fi
fi

if [[ "$failed" -ne 0 ]]; then
  exit 1
fi

echo "release version $cli_version is consistent"
