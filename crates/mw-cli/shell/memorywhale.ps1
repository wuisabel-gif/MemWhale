# MemoryWhale lightweight capture hook (PowerShell).
#
# Records every interactive command — the command line, working directory,
# exit code and duration — into the local MemoryWhale database. It never
# captures output. For a full faithful transcript, use `mw --live`.
#
# Managed by `mw hooks install pwsh` / `mw hooks uninstall pwsh`; you normally
# don't dot-source this by hand.
#
# Toggle off for one session:   $env:MW_HOOK_OFF = 1
# Everything stays local and is never uploaded.
#
# Mechanism: we wrap the `prompt` function. PowerShell has no "command
# executed" event that is portable across Windows PowerShell 5.1 and
# PowerShell 7+ (PSReadLine handlers and Register-EngineEvent differ between
# them), but `prompt` runs once before every prompt is drawn — i.e. right
# after the previous command finishes. `Get-History` then gives us that
# command's text and start/end time (duration), and we read $LASTEXITCODE/$?
# for its result. This is the widely-compatible approach.

if (-not $global:__MW_HOOK_LOADED) {
    $global:__MW_HOOK_LOADED = 1
    $global:__MW_LAST_HISTORY_ID = 0
    # Keep the user's existing prompt so we don't change how it looks. Fall
    # back to the PowerShell default if there isn't one.
    if (Test-Path Function:\prompt) {
        $global:__MW_ORIG_PROMPT = (Get-Item Function:\prompt).ScriptBlock
    } else {
        $global:__MW_ORIG_PROMPT = {
            "PS $($executionContext.SessionState.Path.CurrentLocation)$('>' * ($nestedPromptLevel + 1)) "
        }
    }

    function global:prompt {
        # FIRST line: snapshot the result of the command the user just ran,
        # before anything below can clobber it. We hand both back untouched
        # at the end so the exit code the user sees is never changed.
        $mwLastSuccess = $?
        $mwLastExit = $global:LASTEXITCODE

        try {
            # MW_RECORDING = inside a full `mw` capture session (double-capture
            # guard). MW_HOOK_OFF = user disabled this shell.
            if (-not $env:MW_HOOK_OFF -and -not $env:MW_RECORDING) {
                $h = Get-History -Count 1 -ErrorAction SilentlyContinue
                # Only record a *new* history entry (prompt fires on empty
                # ENTER too, which leaves history unchanged).
                if ($h -and $h.Id -gt $global:__MW_LAST_HISTORY_ID) {
                    $global:__MW_LAST_HISTORY_ID = $h.Id
                    $cmd = $h.CommandLine
                    # Don't record our own bookkeeping.
                    if ($cmd -and $cmd -notmatch '^\s*mw-remember') {
                        $bin = (Get-Command mw-remember -ErrorAction SilentlyContinue).Source
                        if (-not $bin) {
                            $local = Join-Path $HOME '.local/bin/mw-remember'
                            if (Test-Path $local) { $bin = $local }
                        }
                        if ($bin) {
                            # Exit code: a native program sets $LASTEXITCODE;
                            # cmdlets don't, so fall back to 0/1 from $?.
                            if ($null -ne $mwLastExit) { $code = $mwLastExit }
                            elseif ($mwLastSuccess) { $code = 0 } else { $code = 1 }
                            $dur = [int]([Math]::Max(0, ($h.EndExecutionTime - $h.StartExecutionTime).TotalSeconds))
                            # Fire-and-forget in a hidden process so the prompt
                            # stays snappy. mw-remember applies the per-directory
                            # capture gate (.mwignore / [capture.paths]) itself.
                            # We pass the whole command line as one argument;
                            # -- keeps mw-remember from parsing it as flags.
                            Start-Process -FilePath $bin -WindowStyle Hidden `
                                -ArgumentList @(
                                    '--cwd', "$($PWD.Path)",
                                    '--exit-code', "$code",
                                    '--capture-kind', 'hook',
                                    '--notes', "shell hook dur:${dur}s",
                                    '--', "$cmd"
                                ) -ErrorAction SilentlyContinue | Out-Null
                        }
                    }
                }
            }
        } catch {
            # Never let a hiccup — locked DB, missing binary — break the prompt.
        }

        # Restore the user's exit state, then hand off to their prompt.
        # $LASTEXITCODE is what scripts and the visible exit code read, so we
        # always put it back verbatim.
        # ponytail: $? is read-only and can't be re-set; after the prompt
        # redraw it reads $true, which is the normal post-prompt state. If a
        # user's prompt logic depends on the previous command's $?, they read
        # $LASTEXITCODE (restored above) instead.
        $global:LASTEXITCODE = $mwLastExit
        & $global:__MW_ORIG_PROMPT
    }
}
