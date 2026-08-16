# bash completion for the MemoryWhale helper binaries: mw-run, mw-remember,
# mw-serve, mw-view. (Completion for `mw` itself lives in mw.bash.)
# The MemoryWhale installer copies this file under each command name so
# bash-completion's on-demand loader picks it up.

_mw_serve_complete() {
  COMPREPLY=( $(compgen -W "--lan --host --port --token --help" -- "${COMP_WORDS[COMP_CWORD]}") )
}
complete -F _mw_serve_complete mw-serve

_mw_run_complete() {
  local cur="${COMP_WORDS[COMP_CWORD]}" prev="${COMP_WORDS[COMP_CWORD-1]}"
  case "$prev" in
    --cwd) COMPREPLY=( $(compgen -d -- "$cur") ); return ;;
  esac
  case "$cur" in
    -*) COMPREPLY=( $(compgen -W "--cwd --notes --help --" -- "$cur") ) ;;
    *)  COMPREPLY=( $(compgen -c -- "$cur") ) ;;
  esac
}
complete -F _mw_run_complete mw-run

_mw_remember_complete() {
  local cur="${COMP_WORDS[COMP_CWORD]}" prev="${COMP_WORDS[COMP_CWORD-1]}"
  case "$prev" in
    --cwd) COMPREPLY=( $(compgen -d -- "$cur") ); return ;;
  esac
  COMPREPLY=( $(compgen -W "--cwd --exit-code --stdout --stderr --notes --help --" -- "$cur") )
}
complete -F _mw_remember_complete mw-remember

_mw_view_complete() {
  [ "$COMP_CWORD" -eq 1 ] && COMPREPLY=( $(compgen -W "session command --no-open" -- "${COMP_WORDS[COMP_CWORD]}") )
}
complete -F _mw_view_complete mw-view
