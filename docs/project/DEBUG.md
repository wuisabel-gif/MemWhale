# MemoryWhale Debug Notes

This file records the real setup and debugging path used to bring MemoryWhale
up on a Jetson/Ubuntu machine. It is written for humans who hit the same
terminal problems later.

For deploying and testing MemoryWhale on the Jetson — the binaries, the
dashboard, global and in-container recording — see [JETSON_TESTING.md](JETSON_TESTING.md).

## Goal

MemoryWhale is a Rust/Tauri local-first terminal memory system. It stores
commands, arguments, working directories, exit codes, stdout, stderr, notes, and
imported text in a local SQLite database.

The important local database path on Linux is:

```bash
~/.local/share/MemoryWhale/memorywhale.sqlite3
```

## Jetson / Ubuntu Base Setup

Start in the project folder:

```bash
cd ~/barracuda_ws_isabella/MemoryWhale
```

Install base packages:

```bash
sudo apt update
sudo apt install -y nodejs npm build-essential pkg-config libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev
```

If `libwebkit2gtk-4.1-dev` is unavailable on Ubuntu 22.04, try:

```bash
sudo apt install -y libwebkit2gtk-4.0-dev
```

Check versions:

```bash
node --version
npm --version
rustc --version
cargo --version
```

## Problem: `bash: npm: command not found`

Node/npm is missing.

Fix:

```bash
sudo apt update
sudo apt install -y nodejs npm
```

## Problem: `npm WARN EBADENGINE Unsupported engine`

Ubuntu installed an old Node version, such as Node 12, while Vite and the React
tooling require Node 22 (22.12 or newer).

Fix with NodeSource:

```bash
sudo apt remove -y nodejs npm
curl -fsSL https://deb.nodesource.com/setup_22.x | sudo -E bash -
sudo apt install -y nodejs
node --version
npm --version
```

Then rebuild dependencies:

```bash
rm -rf node_modules
npm install
```

## Problem: Node 22 Install Fails Because Of `libnode-dev`

Error:

```txt
trying to overwrite '/usr/include/node/common.gypi',
which is also in package libnode-dev 12.22.9
```

Old Ubuntu Node 12 development packages are blocking the NodeSource Node 22
package.

Fix:

```bash
sudo apt remove -y libnode-dev libnode72 nodejs-doc
sudo apt --fix-broken install -y
sudo apt install -y nodejs
node --version
npm --version
```

## Problem: `tauri: Permission denied`

This can happen after dependency installation with a bad old Node/npm setup.

Fix:

```bash
rm -rf node_modules
npm install
npm run tauri:dev
```

If it still happens:

```bash
chmod +x node_modules/.bin/tauri
npm exec tauri dev
```

## Problem: `cargo metadata ... No such file or directory`

Rust/Cargo is missing or not loaded into the current shell.

Install Rust:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Choose option `1`, the standard install. Then run:

```bash
source "$HOME/.cargo/env"
rustc --version
cargo --version
```

Try the app again:

```bash
npm run tauri:dev
```

## Problem: Cargo Cannot Pick A Binary

Error:

```txt
cargo run could not determine which binary to run
available binaries: memorywhale, mw-remember
```

MemoryWhale has two Rust binaries:

- `memorywhale`, the Tauri desktop app
- `mw-remember`, the terminal memory helper

Cargo needs `memorywhale` as the default desktop binary.

Fix in `src-tauri/Cargo.toml`:

```toml
default-run = "memorywhale"
```

If a clone is missing that line:

```bash
grep -q '^default-run' src-tauri/Cargo.toml || sed -i '/^edition = "2021"/a default-run = "memorywhale"' src-tauri/Cargo.toml
npm run tauri:dev
```

## Problem: Missing `gdk-3.0`

Error:

```txt
No package 'gdk-3.0' found
The system library `gdk-3.0` required by crate `gdk-sys` was not found.
```

Fix:

```bash
sudo apt update
sudo apt install -y pkg-config libgtk-3-dev
```

Then:

```bash
npm run tauri:dev
```

## Problem: Missing `libsoup-3.0`

Error:

```txt
No package 'libsoup-3.0' found
The system library `libsoup-3.0` required by crate `soup3-sys` was not found.
```

Fix:

```bash
sudo apt update
sudo apt install -y libsoup-3.0-dev libwebkit2gtk-4.1-dev
```

If WebKit 4.1 is unavailable:

```bash
sudo apt install -y libsoup2.4-dev libwebkit2gtk-4.0-dev
```

## Problem: Tauri Compiles But GTK Fails To Open

Error:

```txt
Failed to initialize gtk backend!
Failed to initialize GTK
```

This means the app compiled, but the terminal does not have access to a desktop
display. This often happens over SSH or in a headless Jetson session.

Check:

```bash
echo $DISPLAY
```

If it prints nothing, the Tauri desktop window cannot open from that terminal.

Use the browser mode instead:

```bash
npm run dev -- --host 0.0.0.0
```

Find the Jetson IP:

```bash
hostname -I
```

Open the first LAN IP in a laptop browser. Example:

```txt
http://192.168.8.167:1420/
```

Ignore Docker/internal addresses like `172.17.0.1`, `172.18.0.1`, or
`172.19.0.1`.

## Vite Logs After The Prompt

A line like this is normal:

```txt
[vite] (client) [optimizer] bundling dependencies...
```

It means the frontend dev server is preparing JavaScript dependencies. If Tauri
or Cargo fails, Vite may still print one last log line after the prompt returns.
Press `Ctrl+C` to stop the dev server before rerunning commands.

## Browser Mode Versus Tauri Mode

This command starts only the web frontend:

```bash
npm run dev -- --host 0.0.0.0
```

It lets a laptop browser open the UI from the Jetson, but it does not provide
the full Tauri desktop bridge. The terminal memory helper can still write to
SQLite, but the browser page may not automatically show the saved entries until
a local HTTP API is added.

This command starts the full desktop app:

```bash
npm run tauri:dev
```

It needs a working graphical desktop session.

## Recording Terminal Memory Manually

Show the helper usage:

```bash
cd ~/barracuda_ws_isabella/MemoryWhale/src-tauri
cargo run --bin mw-remember -- --help
```

Expected shape:

```txt
mw-remember --cwd <path> --exit-code <code> --stdout <text> --stderr <text> --notes <text> -- <command> [args...]
```

Example:

```bash
cargo run --bin mw-remember -- \
  --cwd ~/barracuda_ws_isabella/MemoryWhale \
  --exit-code 0 \
  --stdout "MemoryWhale web UI started at http://192.168.8.167:1420" \
  --stderr "" \
  --notes "Started MemoryWhale browser UI from Jetson and opened it from laptop." \
  -- npm run dev -- --host 0.0.0.0
```

Success looks like:

```txt
remembered command run #1
```

## What Is Saved Automatically?

Nothing is captured automatically yet. MemoryWhale currently saves terminal
memory only when something is explicitly sent through `mw-remember`.

Once `mw-remember` prints `remembered command run #...`, the entry is durable in
the local SQLite database and survives terminal shutdown, SSH disconnection, and
reboot as long as the database file persists.

The next useful feature is an automatic shell wrapper, for example:

```bash
mw npm run dev -- --host 0.0.0.0
```

or a shell hook that records every command, exit code, cwd, stdout, and stderr.

## Finding The SQLite Database

The database file is:

```bash
~/.local/share/MemoryWhale/memorywhale.sqlite3
```

This will not find it because the file does not end in `.db`:

```bash
find ~/.local/share -name "*.db" | grep -i memory
```

Use:

```bash
find ~/.local/share -name "memorywhale.sqlite3"
ls -la ~/.local/share/MemoryWhale
```

## Querying Saved Terminal Memory

Install the SQLite viewer if needed:

```bash
sudo apt update
sudo apt install -y sqlite3
```

Do not run the `.sqlite3` file directly. It is data, not an executable.

Show command runs:

```bash
sqlite3 ~/.local/share/MemoryWhale/memorywhale.sqlite3 \
  "SELECT id, command, cwd, exit_code, notes, created_at FROM command_runs;"
```

Example output:

```txt
1|npm|/home/barracuda/barracuda_ws_isabella/MemoryWhale|0|Started MemoryWhale browser UI from Jetson and opened it from laptop.|2026-06-17T23:26:20.415072667+00:00
```

Meaning:

- `1` is the memory entry ID.
- `npm` is the command.
- The long path is the working directory.
- `0` means the command succeeded.
- The note explains the debugging context.
- The timestamp is when MemoryWhale saved it.

Show split command arguments:

```bash
sqlite3 ~/.local/share/MemoryWhale/memorywhale.sqlite3 \
  "SELECT command_run_id, position, value FROM command_arguments ORDER BY command_run_id, position;"
```

Example output:

```txt
1|0|npm
1|1|run
1|2|dev
1|3|--
1|4|--host
1|5|0.0.0.0
```

This means MemoryWhale remembered the full command:

```bash
npm run dev -- --host 0.0.0.0
```

## macOS: `Killed: 9` (exit 137) after copying a binary

On macOS, copying a freshly built binary (e.g. `cp target/release/mw-serve ~/.local/bin/`)
can invalidate its code signature, so the OS kills it immediately with `Killed: 9`
(exit code 137) — even for `mw-serve --help`. A binary run straight from
`target/debug` or `target/release` works, but the copy does not.

Fix: re-sign the copies ad-hoc after installing.

```bash
cp target/release/{mw,mw-remember,mw-serve,mw-view,mw-recover} ~/.local/bin/
codesign --force --sign - \
  ~/.local/bin/mw ~/.local/bin/mw-remember ~/.local/bin/mw-serve \
  ~/.local/bin/mw-view ~/.local/bin/mw-recover
```

Verify:

```bash
mw-serve --help    # should print usage and exit 0 (not "Killed: 9")
```

This is macOS code-signing only; it does not affect Linux/Jetson installs.
