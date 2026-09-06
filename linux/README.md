# MemoryWhale on Linux

Everything needed to run MemoryWhale as a first-class Linux citizen: install the
CLI binaries, auto-record commands with a shell hook, keep the dashboard alive
as a service, and package it as a `.deb`.

MemoryWhale 0.10.0 — Agent-Native Memory — uses the same product version on
Linux, macOS, the CLI, and the UI. See the
[release notes](../docs/releases/0.10.0.md).

Storage is local-first: commands, sessions, and notes land in
`~/.local/share/MemoryWhale/memorywhale.sqlite3` unless
`MEMORYWHALE_DATA_DIR` selects another directory. Export, SSH transfer, and
network access are explicit choices.

## Quick install

For prebuilt Linux x86_64/aarch64 binaries, use the root installer:

```bash
(
  set -eu
  installer="$(mktemp)"
  trap 'rm -f "$installer"' EXIT
  curl -fsSL https://raw.githubusercontent.com/wuisabel-gif/MemWhale/7c3864c743cec9a8fa813dcc0b2459cc2859c849/install.sh -o "$installer"
  printf '%s  %s\n' '3e0cad72b29c1894d5ff5f7c30b099537f96501801c14b6320c12e169a3ac8d6' "$installer" | shasum -a 256 -c -
  sh "$installer"
)
```

Or, from a source checkout's repository root:

```bash
# Build the binaries and install them into ~/.local/bin
linux/install.sh

# …or install everything (hook + dashboard service + completions + man pages)
linux/install.sh --all
```

The CLI is separate from the Tauri desktop shell: it does not require GTK or
WebKit. With a Rust toolchain installed, a Debian/Ubuntu build host needs a C
build toolchain for bundled SQLite:

```bash
sudo apt install -y build-essential pkg-config
cargo build --release -p memorywhale-cli --bins
```

After installing, ensure `~/.local/bin` (or Cargo's `~/.cargo/bin`) is on
`PATH`, then verify the version and each integration component:

```bash
mw --version
mw doctor
mw integrate claude     # optional Claude Code hook, skill, and MCP setup
mw integrate rho        # optional Rho hook, skill, and MCP setup
mw doctor
```

## Pieces

| Path | What it does |
|------|--------------|
| `install.sh` | Build + install the `mw*` binaries into `~/.local/bin`; flags add the extras below. |
| `systemd/memorywhale-dashboard.service` | A `systemd --user` unit that runs `mw-serve`. |
| `systemd/enable-dashboard.sh` | Installs the unit, resolves the `mw-serve` path, enables it, and turns on lingering. |
| `crates/mw-cli/shell/memorywhale.sh` | A bash/zsh hook that records every command (cwd + exit code) via `mw-remember`. |
| `completions/mw.bash`, `completions/_mw` | Tab-completion for `mw` in bash and zsh. |
| `man/*.1` | Man pages for `mw`, `mw-run`, `mw-remember`, `mw-serve`, `mw-mcp`, `mw-view`, and `mw-recover`. |

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

For LAN access (open the dashboard from another machine), add `--lan`. If no
token is set, `mw-serve` mints `serve.token` in the data directory. `mw-serve
--lan --print-token` prints that LAN token. MCP clients send it as
`Authorization: Bearer …` to `POST /mcp`.

HTTP MCP is available on the dashboard listener without an extra flag;
`mw-serve --api` additionally enables the read-only `/api/v1` JSON API.
Explicit loopback tokens also protect MCP. See the [MCP](../docs/reference/mcp.md)
and [JSON API](../docs/reference/api.md) references before exposing the service.

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
cargo deb -p memorywhale-cli   # run from the repository root
# writes target/debian/memorywhale_0.10.0-1_<arch>.deb (revision may vary)
sudo dpkg -i target/debian/memorywhale_*.deb
```

The package installs the `mw*` binaries into `/usr/bin`, the man pages and
completions into the standard locations, and the hook + service files into
`/usr/share/memorywhale/`. After installing, enable the dashboard with
`/usr/share/memorywhale/enable-dashboard.sh`.
