# MemoryWhale on Linux

Everything needed to run MemoryWhale as a first-class Linux citizen: install the
CLI binaries, auto-record commands with a shell hook, keep the dashboard alive
as a service, and package it as a `.deb`.

All of MemoryWhale is local-first. Nothing here uploads anything — every command,
session, and note lands in `~/.local/share/MemoryWhale/memorywhale.sqlite3`.

## Quick install

```bash
# Build the binaries and install them into ~/.local/bin
linux/install.sh

# …or install everything (hook + dashboard service + completions + man pages)
linux/install.sh --all
```

Building compiles the crate, which pulls the Tauri/GTK dependency tree, so the
build host needs the system libraries (already covered in `DEBUG.md`):

```bash
sudo apt install -y build-essential pkg-config libssl-dev \
  libgtk-3-dev libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev libsoup-3.0-dev
```

## Pieces

| Path | What it does |
|------|--------------|
| `install.sh` | Build + install the `mw*` binaries into `~/.local/bin`; flags add the extras below. |
| `systemd/memorywhale-dashboard.service` | A `systemd --user` unit that runs `mw-serve`. |
| `systemd/enable-dashboard.sh` | Installs the unit, resolves the `mw-serve` path, enables it, and turns on lingering. |
| `crates/mw-cli/shell/memorywhale.sh` | A bash/zsh hook that records every command (cwd + exit code) via `mw-remember`. |
| `completions/mw.bash`, `completions/_mw` | Tab-completion for `mw` in bash and zsh. |
| `man/*.1` | Man pages for `mw`, `mw-serve`, `mw-remember`. |

## Dashboard as a service (survives SSH logout)

Started by hand over SSH, `mw-serve` dies when the session detaches —
`systemd-logind` reaps the user's processes on logout. A `--user` service plus
lingering keeps it running across logout and reboot:

```bash
linux/systemd/enable-dashboard.sh
#   -> http://127.0.0.1:7071
#   status: systemctl --user status memorywhale-dashboard
#   logs:   journalctl --user -u memorywhale-dashboard -f
#   stop:   linux/systemd/enable-dashboard.sh --disable
```

For LAN access (open the dashboard from another machine), change `--host` to
`0.0.0.0` in the installed unit and restart it.

## Per-command recording vs. whole-session

Two complementary layers:

- **`mw` / `mw global on`** — records a whole shell session as a faithful
  transcript (every command *and* its output). Best for debugging a build.
- **`crates/mw-cli/shell/memorywhale.sh`** — a lightweight index that records each command's
  line, working directory, and exit code (no output capture). Best for "what
  did I run, where, and did it work?" across every shell.

Enable the per-command hook:

```bash
echo '. /path/to/MemWhale/crates/mw-cli/shell/memorywhale.sh' >> ~/.bashrc   # or ~/.zshrc
# pause it in one shell:  export MW_PERCMD_OFF=1
```

## Debian package

With [`cargo-deb`](https://github.com/kornelski/cargo-deb):

```bash
cargo install cargo-deb
cd src-tauri
cargo deb            # writes target/debian/memorywhale_0.1.0_<arch>.deb
sudo dpkg -i target/debian/memorywhale_*.deb
```

The package installs the `mw*` binaries into `/usr/bin`, the man pages and
completions into the standard locations, and the hook + service files into
`/usr/share/memorywhale/`. After installing, enable the dashboard with
`/usr/share/memorywhale/enable-dashboard.sh`.
