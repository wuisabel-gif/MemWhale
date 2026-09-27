# MemoryWhale recall demo. Regenerate from the repo root with an installed
# `mw` on PATH:  vhs scripts/demo/recall.tape
Output assets/recall-demo.gif
Set Shell "bash"
Set FontSize 18
Set Width 1500
Set Height 580
Set Padding 24
Set Theme "Catppuccin Mocha"
Set TypingSpeed 35ms

Hide
Type "export MEMORYWHALE_DATA_DIR=$(mktemp -d) PS1='$ ' && clear" Enter
Sleep 300ms
Show

Type "# A build fails. MemoryWhale's capture hook records it:" Enter
Sleep 400ms
Type "mw-remember --cwd /work/api --exit-code 101 --stderr 'error: linker `cc` not found' -- cargo build" Enter
Sleep 1s
Type "# You fix it and save the lesson:" Enter
Sleep 300ms
Type "mw remember 'linker cc not found -> run: xcode-select --install'" Enter
Sleep 1s

Hide
Type "mw-remember --cwd /work/api --exit-code 0 -- cargo build >/dev/null; mw-remember --cwd /work/api --exit-code 101 --stderr 'error: linker `cc` not found' -- cargo build >/dev/null; clear" Enter
Sleep 400ms
Show

Type "# ...three weeks later, the same error is back. Have I seen this?" Enter
Sleep 500ms
Type "mw context --last-error" Enter
Sleep 3s
Type "mw search linker cc not found" Enter
Sleep 4s
