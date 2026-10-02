# Proactive recall demo, rendered with `vhs assets/proactive-recall.tape` from the
# repo root after `cargo build --release -p memorywhale-cli --bins`.
# The story: a build error was fixed once. When an agent hits it again, the
# agent capture hook (mw-remember --from-hook) hands the agent the past fix in one line.
# Every line on screen is real output from the binaries in target/release.
Output assets/proactive-recall.gif

Set Shell "bash"
Set FontSize 20
Set Width 1240
Set Height 470
Set Padding 26
Set Theme "Catppuccin Mocha"
Set TypingSpeed 25ms
Set Framerate 20

# ---- silent setup: throwaway store, seeded with a past failure and its fix ----
Hide
Type `export PATH="$PWD/target/release:$PATH"`
Enter
Type `export MEMORYWHALE_DATA_DIR="$(mktemp -d)"; cd "$(mktemp -d)"`
Enter
Type `err='xcrun: error: invalid active developer path (/Library/Developer/CommandLineTools), missing xcrun at: /Library/Developer/CommandLineTools/usr/bin/xcrun'`
Enter
Type `mw-remember --cwd "$PWD" --exit-code 2 --stderr "$err" -- make >/dev/null`
Enter
Type `mw-remember --cwd "$PWD" --exit-code 0 -- xcode-select --install >/dev/null`
Enter
Type `mw-remember --cwd "$PWD" --exit-code 0 -- make >/dev/null`
Enter
# the PostToolUseFailure hook JSON an agent sends when a Bash call fails
Type `jq -n --arg e "Exit code 2"$'\n'"$err" '{hook_event_name:"PostToolUseFailure",tool_name:"Bash",tool_input:{command:"make"},error:$e}' > failure.json`
Enter
Type `export PS1='\[\e[38;5;39m\]$\[\e[0m\] '`
Enter
Type "clear"
Enter
Show

# ---- the demo ---------------------------------------------------------------
Type "# earlier: make failed with this xcrun error; xcode-select --install fixed it."
Enter
Sleep 600ms
Type "# now an agent's make fails the same way. Its hook payload:"
Enter
Sleep 400ms
Type "jq . failure.json"
Enter
Sleep 3s

Type "mw-remember --from-hook claude < failure.json | jq -r '.[].additionalContext'"
Enter
Sleep 5s
