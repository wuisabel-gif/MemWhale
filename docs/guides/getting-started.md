# Getting started

## 1. Install

```bash
curl -fsSL https://raw.githubusercontent.com/wuisabel-gif/MemWhale/main/install.sh | sh
```

The installer uses `~/.local/bin` by default. Ensure that directory is on
`PATH`, then verify the installation:

```bash
mw --version
mw doctor
```

Cargo and Homebrew installation commands are available in the root README.
Windows users should install inside WSL; native Windows session recording is
not currently supported.

## 2. Capture one command

```bash
mw-run -- cargo check
```

## 3. Find it

```bash
mw search "cargo check"
mw context --last-error
```

## 4. Enable ongoing capture

```bash
mw global on
mw doctor
```

Open a new shell after enabling the hook. Review the
[terminal-capture guide](terminal-capture.md) before enabling capture in
sensitive environments.

## 5. Connect an agent

Follow the [agent-memory guide](agent-memory.md) and select a verified client
from the [integration matrix](../../integrations/README.md).
