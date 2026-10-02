# Testing MemoryWhale on the Jetson

MemoryWhale runs headless on the Jetson. You record terminal work on the device
and read it back from a laptop browser over the LAN. This is the runbook for
deploying it there and checking each layer works — plus the snags worth knowing
before you hit them.

## What this covers

- Host recording — one command (`mw-remember`) and whole sessions (`mw`)
- The web dashboard, reachable across the network
- Global recording — every new terminal auto-records
- In-container recording — a recorded shell inside an onboard container

## Prerequisites

- SSH access to the Jetson (a `jetson` host alias in `~/.ssh/config` is handy).
- On the Jetson: `cargo` (rustup), `node`/`npm`, and `script` (util-linux, used
  for session capture). All ship on the standard Jetson image.

## 1. Deploy the binaries

The Jetson's checkout predates the current flat `main` layout and its history has
diverged, so `git pull` is not the path of least resistance. Copy the current
source straight from `main` into the Jetson's bin directory instead, then build
with the Jetson's own cargo. Each `mw-*.rs` is a standalone binary that only needs
crates already in `Cargo.toml`, so this just works.

From a machine that has the repo (paths below are the flat `main` layout):

```bash
REPO=/home/barracuda/barracuda_ws_isabella/MemWhale/MemoryWhale
for f in mw mw-remember mw-serve mw-view mw-recover; do
  git show origin/main:src-tauri/src/bin/$f.rs \
    | ssh jetson "cat > $REPO/src-tauri/src/bin/$f.rs"
done
```

On the Jetson:

```bash
cd ~/barracuda_ws_isabella/MemWhale/MemoryWhale/src-tauri
cargo build --bin mw --bin mw-remember --bin mw-serve --bin mw-view --bin mw-recover
mkdir -p ~/.local/bin
cp target/debug/{mw,mw-remember,mw-serve,mw-view,mw-recover} ~/.local/bin/
```

No `codesign` step here — that one is macOS-only.

## 2. Run the dashboard

Bind to all interfaces so the laptop can reach it, and detach it so it survives
your SSH session:

```bash
setsid mw-serve --host 0.0.0.0 --port 7071 >~/mw-serve.log 2>&1 </dev/null &
```

Open it from a laptop on the same network at `http://<jetson-ip>:7071/`. Find the
address with `hostname -I` and use the LAN one — ignore the Docker `172.x`
bridges.

Check it from the Jetson:

```bash
curl -s -o /dev/null -w "%{http_code}\n" http://127.0.0.1:7071/   # expect 200
```

## 3. Test host recording

```bash
# one command, recorded by hand:
mw-remember --cwd "$(pwd)" --exit-code 0 --notes "smoke test" -- echo hello

# a whole session — type exit (wait for "recorded session #N") to save:
mw --notes "project:auv debugging"

# a live-autosaved session — useful if SSH may disconnect before you can exit:
mw --live --notes "project:auv live debugging"

mw list          # what's recorded
mw show 1        # replay a session transcript
```

Anything you record appears on the dashboard on refresh.

## 4. Test global recording

Make every new terminal record itself:

```bash
mw global on        # wires a guarded hook into ~/.bashrc
mw global status
```

Open a new SSH session — it auto-execs into a recorded `mw` session, one per
terminal. Turn it off with `mw global off`.

Recovery: the hook only fires in interactive shells, so a non-interactive command
bypasses it. If a terminal ever misbehaves, this always works from anywhere:

```bash
ssh jetson 'mw global off'
```

## 5. Test in-container recording

The host-built `mw` runs inside the onboard containers (same aarch64 Ubuntu), and
`script` is present in the image, so you can mount `mw` in and bind-mount the host
store so container sessions land in the same database the dashboard serves.

A wrapper that opens a recorded shell inside a container, unified with the host
dashboard:

```bash
docker run --rm -it --entrypoint /bin/bash \
  -v "$HOME/.local/share/MemoryWhale:/root/.local/share/MemoryWhale" \
  -v "$HOME/.local/bin/mw:/usr/local/bin/mw:ro" \
  <your-onboard-image>:latest \
  -lc 'exec mw --notes "container:<name>"'
```

Run it, work normally, `exit` to save. The session shows up on the host dashboard
because the store is bind-mounted. This records an on-demand shell with the
entrypoint overridden to bash; recording every compose-launched container shell
would mean baking `mw` into the image or wiring the compose service.

## Gotchas

- **Nested vs flat git.** The device's old nested checkout has unrelated history
  to flat `main`. Copy files from `main` rather than pulling.
- **Tilde over SSH.** `ssh jetson 'cat > ~/x'` expands `~` on the Jetson;
  `BIN=~/x; ssh jetson "cat > $BIN"` expands it on the *local* machine first. Use
  absolute remote paths, or keep the `~` inside the single-quoted remote command.
- **Detaching a server over SSH.** Plain backgrounding can hold the SSH channel
  open and return a non-zero exit even though the server started. Use
  `setsid … >log 2>&1 </dev/null &`, then verify with a fresh connection.
- **glibc compatibility.** The host-built `mw` runs in the onboard containers
  because they share the host's aarch64 Ubuntu base, so it can be bind-mounted in
  rather than rebuilt.
- **Unified container memory.** Bind-mount the host store to
  `/root/.local/share/MemoryWhale` in the container (root's home) so in-container
  recordings reach the host dashboard. Files written from the container are owned
  by root on the host mount.
- **No nested recordings.** With global mode on, `mw` sets `MW_RECORDING` for the
  shell it records, so the startup hook sees the guard and doesn't start a second
  recording inside the first.

## Data

Everything is local: `~/.local/share/MemoryWhale/memorywhale.sqlite3` on the
Jetson, with raw session transcripts under `~/.local/share/MemoryWhale/sessions/`.
Nothing is uploaded. See `SOP.md` for general usage and `DEBUG.md` for Jetson
setup and troubleshooting.
