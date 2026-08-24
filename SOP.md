# MemoryWhale — Standard Operating Procedure

How to use MemoryWhale day to day. Everything is local; nothing is uploaded.

## 0. One-time setup (per machine)

```bash
cd src-tauri
cargo build --release --bin mw --bin mw-remember --bin mw-serve --bin mw-view --bin mw-recover
mkdir -p ~/.local/bin
cp target/release/{mw,mw-remember,mw-serve,mw-view,mw-recover} ~/.local/bin/
# optional: put ~/.local/bin on PATH so you can drop the full path
echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.zshrc && source ~/.zshrc
```

On Ubuntu/Jetson, install Node/Tauri system deps first — see [DEBUG.md](DEBUG.md).

## 1. Record a single command (manual, fast)

Use when you already have the output and want to jot one command.

```bash
mw-remember --cwd "$(pwd)" --exit-code 0 \
  --stdout "..." --stderr "..." \
  --notes "what this was / why it matters" -- <command> [args]
```

Success prints `remembered command run #N`. It records *what you type*, so be accurate.

## 2. Record a whole session (automatic, truthful)

```bash
mw --notes "what you're debugging"
#   ...work normally; every command + real output is captured...
```

**To stop — do this cleanly:**

1. If a pager is open (`git log`, `less`, `man`) press **`q`**.
2. Type **`exit`** (or Ctrl-D).
3. **Wait for** `mw: recorded session #N (… bytes …)`. Only then is it saved.
4. Don't close the terminal before that line — if you do, the transcript file still
   survives and is auto-recovered later (see §6), but exiting cleanly is best.

Browse recorded sessions: `mw list` · replay one: `mw show <id>`.

## 3. View your memory in a browser (the dashboard)

```bash
mw-serve --host 127.0.0.1 --port 7071     # this machine only
#   open http://localhost:7071/
```

On a **headless Jetson**, bind to the LAN and open from your laptop:

```bash
mw-serve                                   # binds 0.0.0.0:7071
#   laptop browser: http://<jetson-ip>:7071/   (find the IP with: hostname -I)
```

Click any command/session for the detail page + suggested next steps.
A single memory page can also be generated with `mw-view <id>`.
The dashboard also has a `/graph` view of commands linked to their arguments.

## 3a. Check your memory pet (optional)

`mw pet` shows a read-only whale whose mood reflects how recently you used your
memory store. Run `mw pet --watch` to animate it until you press Ctrl-C. See the
[`mw pet` reference](docs/reference/pet.md) for the mood rules.

## 3b. Record across multiple terminals (projects)

Each `mw` records only its own terminal, so multi-terminal work is captured as
separate sessions. Give them the same `project:<name>` tag and the dashboard
groups them automatically:

```bash
# terminal 1
mw --notes "project:pop_playlist git history"
# terminal 2
mw --notes "project:pop_playlist server testing"
```

On the dashboard, the **Projects** section lists each project; opening one merges
every command run and session tagged with it into a single time-ordered timeline,
across all terminals.

- Keep project names a single token: `project:pop_playlist` (underscores fine),
  not `project:"two words"` (only the first word is captured).
- To auto-record every new terminal without typing `mw`, use `mw global on`.

## 4. Query directly (power users)

```bash
# Linux/Jetson:
sqlite3 ~/.local/share/MemoryWhale/memorywhale.sqlite3 \
  "SELECT id, command, exit_code, notes FROM command_runs;"
# macOS:
sqlite3 ~/Library/Application\ Support/MemoryWhale/memorywhale.sqlite3 \
  "SELECT id, started_at, notes FROM sessions;"
```

## 5. Always-on recording (optional)

```bash
mw global on        # every new interactive terminal auto-records (guarded, safe)
mw global status
mw global off       # stop
```

⚠️ Records **everything**, including sessions that may contain secrets/tokens.
Turn it off for sensitive work.

## 6. Recover an interrupted recording

If an `mw` session is cut off before it saved, the raw transcript still exists on
disk. Importing it back is automatic in two places, and also available manually:

```bash
mw-recover          # import any orphaned session transcripts
```

`mw-serve` also runs this automatically on startup, so just opening the dashboard
self-heals. Recovery is idempotent — safe to run anytime.

## 7. Golden rules

- **`mw` = whole session. `mw-remember` = one command.** Pick the right tool.
- **Wait for `recorded session #N`** before closing a terminal.
- **Per-machine and local.** The Jetson and your laptop have separate stores; back
  up the `.sqlite3` file to preserve or move memory between machines.
- **Nothing is uploaded** — but treat the store as sensitive (it can hold secrets).

## Data locations

| | Path |
|---|---|
| Database | `<data_local>/MemoryWhale/memorywhale.sqlite3` |
| Session transcripts | `<data_local>/MemoryWhale/sessions/` |
| Screenshots | `<data_local>/MemoryWhale/screenshots/` |

`<data_local>` = `~/.local/share` on Linux/Jetson, `~/Library/Application Support` on macOS.

Deploying and testing on a headless Jetson (binaries, dashboard over the LAN,
global and in-container recording) is covered in [JETSON_TESTING.md](JETSON_TESTING.md).
