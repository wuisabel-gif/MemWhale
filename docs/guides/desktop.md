# The desktop app

The MemoryWhale app is a window onto the same local store the CLI and the agent
hooks write to. It shows what your coding agents were told, the commands and
output behind it, and the lessons you saved. Nothing leaves your machine.

## Install

Each release from 0.23 on attaches the app next to the CLI archives on the
[releases page](https://github.com/wuisabel-gif/MemWhale/releases).

**macOS (Apple silicon and Intel).** Download `MemoryWhale-<version>-macos.dmg`,
open it, and drag MemoryWhale to Applications. The app is signed ad hoc but not
notarized, so the first time macOS says it cannot check it. Open
**System Settings → Privacy & Security**, scroll to the message about
MemoryWhale, and choose **Open Anyway**. From a terminal, this does the same:

```bash
xattr -dr com.apple.quarantine /Applications/MemoryWhale.app
```

**Linux (x86_64).** Install the Debian package, or run the AppImage:

```bash
sudo apt install ./memorywhale-desktop_<version>_amd64.deb
```

```bash
chmod +x MemoryWhale-<version>-x86_64.AppImage && ./MemoryWhale-<version>-x86_64.AppImage
```

The app does not install the CLI. Install that as the
[README](../../README.md#install) describes, then connect your agents with
`mw integrate claude` (or `codex`, `cursor`, `gemini`, `rho`).

## Build it yourself

You need Rust, Node 22, and on Linux the WebKitGTK libraries listed in
[DEBUG.md](../project/DEBUG.md).

```bash
npm ci && npm run tauri:build
```

The bundles land in `target/release/bundle/`. `npm run tauri:dev` runs it with
live reload, and `npm run dev` previews the interface in a browser with sample
data.

## Where the data comes from

The app opens the default store, or the one in `MEMORYWHALE_DATA_DIR`, and runs
the same schema migrations as the CLI. Captured text is redacted again before it
is shown. Search ranks memories on this machine; if a local
[Ollama](https://ollama.com) is running with `nomic-embed-text`, it also uses
those embeddings and caches them in the store.
