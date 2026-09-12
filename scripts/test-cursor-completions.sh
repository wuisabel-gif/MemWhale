#!/usr/bin/env bash
# Offline completion regression checks; never invokes mw or reads client configs.
set -eo pipefail
cd "$(dirname "$0")/.."
source linux/completions/mw.bash
source linux/completions/mw-extra.bash

complete_words() {
  local function_name=$1
  shift
  COMP_WORDS=("$@")
  COMP_CWORD=$((${#COMP_WORDS[@]} - 1))
  COMPREPLY=()
  "$function_name"
}
has() {
  local reply
  for reply in "${COMPREPLY[@]}"; do
    [ "$reply" != "$1" ] || return 0
  done
  printf 'Missing completion: %s\n' "$1" >&2
  exit 1
}
lacks() {
  local reply
  for reply in "${COMPREPLY[@]}"; do
    if [ "$reply" = "$1" ]; then
      printf 'Unexpected completion: %s\n' "$1" >&2
      exit 1
    fi
  done
}

complete_words _mw_complete mw integrate ''
for client in claude claude-code rho hermes codex cursor; do has "$client"; done
complete_words _mw_complete mw integrate cursor ''
for flag in --config --capture --skill --dry-run --check --revert; do has "$flag"; done
complete_words _mw_complete mw integrate cursor --capture ''
for flag in --hooks-file --dry-run --check --revert; do has "$flag"; done
for flag in --config --skill --capture; do lacks "$flag"; done
complete_words _mw_complete mw integrate cursor --capture --hooks-file linux/man/mw-rem
has linux/man/mw-remember.1
for client in cursor codex rho; do
  complete_words _mw_complete mw integrate "$client" --skill linux/man/mw.1 ''
  for flag in --skills-dir --dry-run --check --revert; do has "$flag"; done
  for flag in --capture --config --http; do lacks "$flag"; done
  complete_words _mw_complete mw integrate "$client" --skill linux/man/mw-rem
  has linux/man/mw-remember.1
  complete_words _mw_complete mw integrate "$client" --skill skill.md --skills-dir linux/comp
  has linux/completions
done
complete_words _mw_complete mw integrate cursor --config linux/man/mw-rem
has linux/man/mw-remember.1
complete_words _mw_complete mw integrate rho ''
for flag in --http --token --revert --skill; do has "$flag"; done
complete_words _mw_complete mw integrate rho --http ''
has http://127.0.0.1:7071/mcp
complete_words _mw_complete mw integrate claude ''
has --revert

COMP_WORDBREAKS=' :'
complete_words _mw_complete mw search agent:cu
has cursor
complete_words _mw_complete mw search agent: cu
has cursor
complete_words _mw_complete mw search agent : cu
has cursor
COMP_WORDBREAKS=' '
complete_words _mw_complete mw search agent:cu
has agent:cursor
complete_words _mw_remember_complete mw-remember --from-hook ''
for client in claude rho cursor; do has "$client"; done
complete_words _mw_remember_complete mw-remember ''
for flag in --cwd --exit-code --stdout --stderr --notes --capture-kind --from-hook --help --; do has "$flag"; done
printf 'Cursor completion regression checks passed.\n'
