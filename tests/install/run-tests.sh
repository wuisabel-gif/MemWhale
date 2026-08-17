#!/bin/sh
# Fixture-based tests for install.sh's release-tag parsing.
#
# Runs with a plain POSIX sh; makes no network requests. The tests exercise the
# exact `latest_release_tag` / `validate_tag` code the installer runs by
# extracting the marker-delimited library from install.sh verbatim, so there is
# a single source of truth for the parsing logic.
#
#   sh tests/install/run-tests.sh
set -eu

root="$(cd "$(dirname "$0")/../.." && pwd)"
install_sh="$root/install.sh"
fixtures="$root/tests/install/fixtures"

lib="$(sed -n '/^# --- release-tag library/,/^# --- end release-tag library/p' "$install_sh" | sed '1d;$d')"
eval "$lib"

failures=0
checks=0

# check <fixture-name> <accept|reject>
check() {
  fixture="$1"
  expected="$2"
  checks=$((checks + 1))

  tag="$(latest_release_tag "$fixtures/$fixture.json")" || tag=""
  if validate_tag "$tag"; then
    actual="accept"
  else
    actual="reject"
  fi

  if [ "$actual" != "$expected" ]; then
    printf 'FAIL  %-26s expected %-7s got %-7s (tag=%s)\n' \
      "$fixture" "$expected" "$actual" "$tag" >&2
    failures=$((failures + 1))
  else
    printf 'ok    %-26s %s\n' "$fixture" "$expected"
  fi
}

# jq path (jq is installed on the CI runner and common on dev machines).
echo "==> jq path"
check latest-valid          accept
check latest-prerelease     accept
check latest-null-tag       reject
check latest-empty-tag      reject
check latest-missing-tag    reject
check latest-multiline-tag  reject
check latest-path-escape    reject
check latest-not-a-version  reject

# Fallback path without jq: a non-executable jq stub in PATH is invisible to
# `command -v`, so latest_release_tag takes the portable grep/sed branch.
echo "==> no-jq fallback path"
no_jq_bin="$(mktemp -d)"
trap 'rm -rf "$no_jq_bin"' EXIT
: > "$no_jq_bin/jq"
chmod 644 "$no_jq_bin/jq"

check_fallback() {
  fixture="$1"
  expected="$2"
  checks=$((checks + 1))
  tag="$(PATH="$no_jq_bin:$PATH" latest_release_tag "$fixtures/$fixture.json")" || tag=""
  if validate_tag "$tag"; then actual="accept"; else actual="reject"; fi
  if [ "$actual" != "$expected" ]; then
    printf 'FAIL  %-26s expected %-7s got %-7s (tag=%s)\n' \
      "$fixture" "$expected" "$actual" "$tag" >&2
    failures=$((failures + 1))
  else
    printf 'ok    %-26s %s\n' "$fixture" "$expected"
  fi
}

check_fallback latest-valid         accept
check_fallback latest-prerelease    accept
check_fallback latest-null-tag      reject
check_fallback latest-empty-tag     reject
check_fallback latest-missing-tag   reject
check_fallback latest-multiline-tag reject
check_fallback latest-path-escape   reject
check_fallback latest-not-a-version reject

if [ "$failures" -eq 0 ]; then
  echo "==> all $checks installer tag tests passed"
else
  echo "==> $failures of $checks installer tag tests FAILED" >&2
  exit 1
fi