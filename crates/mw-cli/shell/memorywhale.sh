# MemoryWhale lightweight capture hook (bash + zsh).
#
# Records every interactive command — the command line, working directory,
# exit code and duration — into the local MemoryWhale database. It never
# captures output. For a full faithful transcript, use `mw --live`.
#
# Managed by `mw hooks install` / `mw hooks uninstall`; you normally don't
# source this by hand.
#
# Toggle off for one shell:   export MW_HOOK_OFF=1
# Everything stays local and is never uploaded.

# Interactive shells only; never load twice.
case $- in *i*) : ;; *) return 0 2>/dev/null || true ;; esac
[ -n "${__MW_HOOK_LOADED:-}" ] && return 0 2>/dev/null
__MW_HOOK_LOADED=1

# Record one command. $1 = exit code, $2 = command line, $3 = duration seconds.
# Fire-and-forget in a detached subshell so the prompt stays snappy and a
# hiccup — locked DB, missing DB, missing binary — never breaks the shell.
# Word-splitting turns the line into argv for mw-remember.
__mw_record() {
  [ -n "${MW_HOOK_OFF:-}" ] && return 0
  [ -n "${MW_RECORDING:-}" ] && return 0   # inside a full `mw` capture session
  [ -z "${2:-}" ] && return 0
  case "$2" in
    __mw_*|mw-remember*) return 0 ;;        # don't record our own bookkeeping
  esac
  local bin
  bin="$(command -v mw-remember 2>/dev/null || true)"
  [ -z "$bin" ] && [ -x "$HOME/.local/bin/mw-remember" ] && bin="$HOME/.local/bin/mw-remember"
  [ -x "$bin" ] || return 0
  # mw-remember applies the per-directory capture gate (.mwignore /
  # [capture.paths]) itself, before it opens the database.
  if [ -n "${ZSH_VERSION:-}" ]; then
    ( "$bin" --cwd "$PWD" --exit-code "$1" --capture-kind hook \
        --notes "shell hook dur:${3:-0}s" -- ${=2} >/dev/null 2>&1 & ) 2>/dev/null
  else
    ( "$bin" --cwd "$PWD" --exit-code "$1" --capture-kind hook \
        --notes "shell hook dur:${3:-0}s" -- $2 >/dev/null 2>&1 & ) 2>/dev/null
  fi
  return 0
}

if [ -n "${ZSH_VERSION:-}" ]; then
  autoload -Uz add-zsh-hook 2>/dev/null
  __mw_preexec() { __MW_LAST_CMD="$1"; __MW_START=$SECONDS; }
  __mw_precmd()  {
    local code=$?
    if [ -n "${__MW_LAST_CMD:-}" ]; then
      __mw_record "$code" "$__MW_LAST_CMD" "$((SECONDS - ${__MW_START:-$SECONDS}))"
    fi
    __MW_LAST_CMD=""
    return $code   # never clobber the exit code the user sees
  }
  add-zsh-hook preexec __mw_preexec
  add-zsh-hook precmd  __mw_precmd
elif [ -n "${BASH_VERSION:-}" ]; then
  __mw_preexec() {
    # The DEBUG trap also fires for the commands *inside* our own hook
    # functions; __MW_IN_HOOK keeps those out of the record. Without it the
    # first-word-wins latch below can be claimed by our own bookkeeping and
    # the user's actual command is silently dropped.
    [ -n "${__MW_IN_HOOK:-}" ] && return 0
    # Ignore completion expansions and the prompt command itself.
    [ -n "${COMP_LINE:-}" ] && return 0
    [ "$BASH_COMMAND" = "${PROMPT_COMMAND:-}" ] && return 0
    case "$BASH_COMMAND" in __mw_*|__MW_*) return 0 ;; esac
    [ -n "${__MW_LAST_CMD:-}" ] && return 0   # first word of the pipeline wins
    __MW_LAST_CMD="$BASH_COMMAND"
    __MW_START=$SECONDS
    return 0
  }
  __mw_precmd() {
    local code=$?
    __MW_IN_HOOK=1
    if [ -n "${__MW_LAST_CMD:-}" ]; then
      __mw_record "$code" "$__MW_LAST_CMD" "$((SECONDS - ${__MW_START:-$SECONDS}))"
    fi
    __MW_LAST_CMD=""
    __MW_IN_HOOK=""
    return $code
  }
  trap '__mw_preexec' DEBUG
  case "${PROMPT_COMMAND:-}" in
    *__mw_precmd*) : ;;
    *) PROMPT_COMMAND="__mw_precmd${PROMPT_COMMAND:+; $PROMPT_COMMAND}" ;;
  esac
  # The DEBUG trap is live for the rest of this file, so the `case` above just
  # latched itself as "the user's command". Drop it.
  __MW_LAST_CMD=""
fi
