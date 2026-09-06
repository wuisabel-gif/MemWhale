# bash completion for mw (MemoryWhale).
# Install:  cp mw.bash ~/.local/share/bash-completion/completions/mw
#       or: source it from ~/.bashrc

_mw_complete() {
  local cur prev
  cur="${COMP_WORDS[COMP_CWORD]}"
  prev="${COMP_WORDS[COMP_CWORD-1]}"
  COMPREPLY=()

  if [ "$COMP_CWORD" -eq 1 ]; then
    COMPREPLY=( $(compgen -W "\
list show mark remember memory rm prune audit share discard replay demo \
export import push pull context agent ask search explain link unlink links \
pet tui sync-mempalace git-fix github doctor global status hooks integrate \
--live --autosave --notes --version --help" -- "$cur") )
    return
  fi

  case "${COMP_WORDS[1]}" in
    global)
      [ "$COMP_CWORD" -eq 2 ] && COMPREPLY=( $(compgen -W "on off status" -- "$cur") )
      ;;
    memory)
      if [ "$COMP_CWORD" -eq 2 ]; then
        COMPREPLY=( $(compgen -W "stale supersede compact" -- "$cur") )
      elif [ "${COMP_WORDS[2]}" = compact ]; then
        case "$prev" in
          --min-session-bytes|--stale-days|--max-output-bytes) return ;;
        esac
        COMPREPLY=( $(compgen -W "--apply --min-session-bytes --stale-days --max-output-bytes --help" -- "$cur") )
      fi
      ;;
    github)
      [ "$COMP_CWORD" -eq 2 ] && COMPREPLY=( $(compgen -W "context" -- "$cur") )
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
      if [ "$COMP_CWORD" -eq 2 ]; then
        COMPREPLY=( $(compgen -W "claude claude-code hermes rho" -- "$cur") )
      else
        case "${COMP_WORDS[2]}" in
          claude|claude-code)
            [ "$COMP_CWORD" -eq 3 ] && COMPREPLY=( $(compgen -W "--revert" -- "$cur") )
            ;;
          rho)
            [ "$prev" = --token ] && return
            if [ "$prev" = --http ]; then
              COMPREPLY=( $(compgen -W "http://127.0.0.1:7071/mcp --token --revert" -- "$cur") )
            else
              COMPREPLY=( $(compgen -W "--revert --http --token" -- "$cur") )
            fi
            ;;
        esac
      fi
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
    search)
      case "$prev" in
        --project|--machine|--since) return ;;
        agent:) COMPREPLY=( $(compgen -W "claude rho terminal" -- "$cur") ); return ;;
        source:) COMPREPLY=( $(compgen -W "command session note document conversation" -- "$cur") ); return ;;
        :)
          case "${COMP_WORDS[COMP_CWORD-2]}" in
            agent) COMPREPLY=( $(compgen -W "claude rho terminal" -- "$cur") ); return ;;
            source) COMPREPLY=( $(compgen -W "command session note document conversation" -- "$cur") ); return ;;
          esac
          ;;
      esac
      COMPREPLY=( $(compgen -W "--explain --project --machine --since tag: source:command source:session source:note source:document source:conversation agent:claude agent:rho agent:terminal before: after: limit:" -- "$cur") )
      # Readline treats ':' as a word break by default; do not insert it twice.
      if [[ "$cur" == *:* && "$COMP_WORDBREAKS" == *:* ]]; then
        COMPREPLY=( "${COMPREPLY[@]#${cur%:*}:}" )
      fi
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
