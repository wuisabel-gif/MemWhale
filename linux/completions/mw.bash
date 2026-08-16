# bash completion for mw (MemoryWhale).
# Install:  cp mw.bash ~/.local/share/bash-completion/completions/mw
#       or: source it from ~/.bashrc

_mw_complete() {
  local cur prev
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"

  if [ "$COMP_CWORD" -eq 1 ]; then
    COMPREPLY=( $(compgen -W "\
list show mark remember memory rm prune audit share discard replay demo \
export import push pull context agent ask search explain link unlink links \
pet tui sync-mempalace git-fix doctor global status hooks integrate \
--live --autosave --notes --version --help" -- "$cur") )
    return
  fi

  case "${COMP_WORDS[1]}" in
    global)
      [ "$COMP_CWORD" -eq 2 ] && COMPREPLY=( $(compgen -W "on off status" -- "$cur") )
      ;;
    memory)
      [ "$COMP_CWORD" -eq 2 ] && COMPREPLY=( $(compgen -W "stale supersede" -- "$cur") )
      ;;
    hooks)
      if [ "$COMP_CWORD" -eq 2 ]; then
        COMPREPLY=( $(compgen -W "install uninstall remove" -- "$cur") )
      elif [ "$COMP_CWORD" -eq 3 ] && [[ " install uninstall remove " == *" ${COMP_WORDS[2]} "* ]]; then
        COMPREPLY=( $(compgen -W "pwsh" -- "$cur") )
      fi
      ;;
    pet)
      [ "$COMP_CWORD" -eq 2 ] && COMPREPLY=( $(compgen -W "--watch" -- "$cur") )
      ;;
    integrate)
      [ "$COMP_CWORD" -eq 2 ] && COMPREPLY=( $(compgen -W "hermes" -- "$cur") )
      ;;
    sync-mempalace)
      case "$prev" in
        --wing|--limit) COMPREPLY=(); return ;;
      esac
      COMPREPLY=( $(compgen -W "--wing --limit --dry-run" -- "$cur") )
      ;;
    show)
      # session ids come from `mw list`; offer them when available.
      if [ "$COMP_CWORD" -eq 2 ] && command -v mw >/dev/null 2>&1; then
        local ids
        ids="$(mw list 2>/dev/null | sed -n 's/^#\([0-9][0-9]*\).*/\1/p')"
        COMPREPLY=( $(compgen -W "$ids" -- "$cur") )
      fi
      ;;
    git-fix)
      # command_run ids come from `mw search`; best-effort, no dedicated lister.
      if [ "$COMP_CWORD" -eq 2 ] && command -v mw >/dev/null 2>&1; then
        local ids
        ids="$(mw search git 2>/dev/null | sed -n 's/^- #\([0-9][0-9]*\).*/\1/p')"
        COMPREPLY=( $(compgen -W "$ids" -- "$cur") )
      fi
      ;;
    rm|share)
      [ "$COMP_CWORD" -eq 2 ] && COMPREPLY=( $(compgen -W "session command" -- "$cur") )
      ;;
    import)
      # a bundle directory or an exported .sqlite3 file
      COMPREPLY=( $(compgen -f -- "$cur") )
      ;;
    push|pull)
      # ssh hosts from ~/.ssh/config
      local hosts
      hosts="$(sed -n 's/^[Hh]ost[[:space:]]\{1,\}\(.*\)/\1/p' ~/.ssh/config 2>/dev/null | tr ' ' '\n' | grep -v '[*?]')"
      COMPREPLY=( $(compgen -W "$hosts" -- "$cur") )
      ;;
    context)
      COMPREPLY=( $(compgen -W "--last-error --limit project:" -- "$cur") )
      ;;
    ask)
      case "$prev" in
        --chat) COMPREPLY=( $(compgen -W "chatgpt claude gemini" -- "$cur") ); return ;;
      esac
      COMPREPLY=( $(compgen -W "--chat --session --no-open" -- "$cur") )
      ;;
    export)
      COMPREPLY=( $(compgen -W "project:" -- "$cur") )
      ;;
  esac
}
complete -F _mw_complete mw
