# Stop-before-it-runs demo, rendered with `vhs assets/stopped-before-it-runs.tape`
# from the repo root after `cargo build --release -p memorywhale-cli --bins`.
# The story: an agent keeps sending a command that fails the same way. One
# saved sentence makes the pre-run hook (mw-remember --before-hook) stop that
# command once, before it runs, and hand the agent the lesson as the reason.
# Every line on screen is real output from the binaries in target/release.
Output assets/stopped-before-it-runs.gif

Set Shell "bash"
Set FontSize 20
Set Width 1240
Set Height 560
Set Padding 26
Set Theme "Catppuccin Mocha"
Set TypingSpeed 25ms
Set Framerate 20

# ---- silent setup: throwaway store with one recorded command ----------------
Hide
Type `export PATH="$PWD/target/release:$PATH"`
Enter
Type `export MEMORYWHALE_DATA_DIR="$(mktemp -d)"; cd "$(mktemp -d)"`
Enter
Type `mw-remember --cwd "$PWD" --exit-code 0 -- ls >/dev/null`
Enter
# the PreToolUse hook JSON an agent sends before it runs a Bash command
Type `jq -n '{hook_event_name:"PreToolUse",tool_name:"Bash",tool_input:{command:"ls src; echo ====; ls tests"}}' > before.json`
Enter
Type `export PS1='\[\e[38;5;39m\]$\[\e[0m\] '`
Enter
Type "clear"
Enter
Show

# ---- the demo ---------------------------------------------------------------
Type "# an agent keeps printing separators with echo ====, which zsh rejects."
Enter
Sleep 600ms
Type "# save one sentence about it:"
Enter
Sleep 400ms
Type "mw remember 'when `echo ==`: zsh reads =word as a lookup. Quote it.'"
Enter
Sleep 2500ms

Type "# next session, the agent is about to run:"
Enter
Sleep 400ms
Type "jq -r .tool_input.command before.json"
Enter
Sleep 2s

Type "# its pre-run hook answers before the command runs:"
Enter
Sleep 400ms
Type "mw-remember --before-hook claude < before.json | jq -r '.[][]' | fold -s -w 84"
Enter
Sleep 7s

Type "mw doctor | grep -o '[0-9]* stopped before running'"
Enter
Sleep 4s
