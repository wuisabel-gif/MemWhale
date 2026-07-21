# MemoryWhale lightweight capture hook (fish).
# Managed by `mw hooks install` / `mw hooks uninstall`.
# Records command, cwd, exit code and duration — never output.
# Toggle off for one shell:   set -x MW_HOOK_OFF 1

status is-interactive; or exit 0
set -q __mw_hook_loaded; and exit 0
set -g __mw_hook_loaded 1

function __mw_postexec --on-event fish_postexec
    set -l code $status              # capture first: nothing above may run
    set -q MW_HOOK_OFF; and return $code
    set -q MW_RECORDING; and return $code   # inside a full `mw` capture session
    test -n "$argv[1]"; or return $code
    string match -q 'mw-remember*' -- "$argv[1]"; and return $code
    set -l bin (command -v mw-remember 2>/dev/null)
    if test -z "$bin"; and test -x "$HOME/.local/bin/mw-remember"
        set bin "$HOME/.local/bin/mw-remember"
    end
    test -x "$bin"; or return $code
    # mw-remember applies the per-directory capture gate itself.
    fish -c "$bin --cwd '$PWD' --exit-code $code --capture-kind hook --notes 'shell hook dur:'(math -s0 $CMD_DURATION/1000)'s' -- $argv[1] >/dev/null 2>&1 &" >/dev/null 2>&1
    return $code
end
