# MemoryWhale README demo — rendered with `vhs assets/demo.tape` from the repo root.
# The story: an agent's build fails, you record the fix once, and later any
# session (or the agent itself, over MCP) can recall the error *and* the fix.
Output assets/demo.gif

Set Shell "bash"
Set FontSize 20
Set Width 1240
Set Height 640
Set Padding 26
Set Theme "Catppuccin Mocha"
Set TypingSpeed 35ms

# ---- silent setup ----------------------------------------------------------
Hide
Type `export PATH="$PWD/target/release:$PATH"`
Enter
Type `export MEMORYWHALE_DATA_DIR="$(mktemp -d)"`
Enter
# a throwaway crate that fails to compile the same way a real one would
Type `proj="$(mktemp -d)/camera-driver"; cargo new -q "$proj"; cd "$proj"`
Enter
Type `printf 'fn main() {\n    tokio::runtime::Runtime::new().unwrap();\n}\n' > src/main.rs`
Enter
Type `export PS1='\[\e[38;5;39m\]$\[\e[0m\] '`
Enter
Type "clear"
Enter
Show

# ---- the demo --------------------------------------------------------------
Type "# an agent's build fails. capture it the moment it happens:"
Enter
Sleep 500ms
Type "mw-run --notes 'project:auv camera-driver' -- cargo build"
Enter
Sleep 5s

Type "# jot the fix once, in plain words:"
Enter
Sleep 700ms
Type "mw remember 'project:auv fix: add tokio to [dependencies] in camera-driver'"
Enter
Sleep 2s

Type "# a new session weeks later: what was that, and did we ever solve it?"
Enter
Sleep 900ms
Type "mw search 'tokio unresolved import'"
Enter
Sleep 4500ms

Type "# or let your coding agent recall the same memory over MCP:"
Enter
Sleep 800ms
Type "# claude mcp add memorywhale -- mw-mcp"
Enter
Sleep 3s
