<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="MemoryWhale logo" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>Persistent local debugging memory for developers and coding agents.</strong></p>

<p align="center"><a href="README.md">English README</a> · <a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="release"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="license MIT"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="local-first, nothing uploaded"/>
</p>

MemoryWhale records what actually happened while you debug: commands, output,
failures, and the fixes that worked. It stores that evidence in local SQLite so
you and your coding agents can find it after the terminal, SSH connection, or
agent session is gone.

**MemoryWhale 0.10.0 — Agent-Native Memory · September 6, 2026.**
The CLI, web UI, and desktop app share product version 0.10.0; the reusable
Rust core is version 0.5.0. See the [release notes](https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md)
for the upgrade guide and breaking Rust API change.

## Why MemoryWhale

- **Remember what actually happened.** Preserve the command, environment,
  output, failure, and lesson—not only a shell-history line.
- **Use one memory across coding agents.** Any compatible stdio MCP client can
  read and write the same local memory through `mw-mcp`.
- **Keep development history local.** MemoryWhale works without an account,
  hosted service, or per-token memory bill.

MemoryWhale records development experience, not everything. It is a debugging
memory layer, not an autonomous coding agent, a general-purpose personal memory
system, or a replacement for project documentation.

## New in Agent-Native Memory

- **Connect and inspect agents.** Install Claude Code or Rho MCP access,
  capture hooks, and memory-use guidance with `mw integrate`; `mw doctor`
  checks MCP, hooks, and skills independently.
- **Keep provenance explicit.** Schema 10 stores command agents as `claude`,
  `rho`, or `NULL`. The display/filter label `terminal` means terminal/manual
  or legacy provenance, not proof that a human ran it. Agent identity is
  separate from source type such as `command`, `session`, or `note`.
- **Share a repository, distinguish worktrees.** Canonical repository IDs
  group linked worktrees while preserving each worktree root and existing
  project tags. Discovery reads local Git metadata, not a remote service.
- **Use local interfaces.** `mw-serve` provides HTTP MCP at `POST /mcp`;
  `mw-serve --api` opts into the read-only JSON API. Both use the dashboard's
  listener; non-loopback access requires a token.
- **Fetch GitHub context explicitly.** `mw github context <pr>` reads PR
  metadata, checks, and reviews through your existing `gh` login. It prints
  bounded, redacted context without checking out code or automatically saving
  it to memory. There is no background GitHub sync.

## Install

Prebuilt binaries are available for Linux x86_64/aarch64 and macOS:

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

Or install with Cargo or Homebrew:

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

After installation or upgrade, check the version and local setup:

```bash
mw --version
mw doctor
```

Windows users can run MemoryWhale inside
[WSL](https://learn.microsoft.com/windows/wsl/). See the
[getting-started guide](docs/guides/getting-started.md) for package installs,
PATH setup, and platform notes.

## Sixty-second example

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

![mw pet mood demo](assets/pet-demo.gif)

For longer work, `mw --live` records a crash-resistant shell session. `mw tui`
opens an interactive terminal browser, while `mw-serve` starts the local web
dashboard.

## How it works

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

Capture and retrieval are independent. MCP gives an agent access to existing
memory; it does not automatically record normal terminal activity. See the
[architecture](docs/architecture.md) and
[capture concept](docs/concepts/capture.md) for the complete model.

## Works with your coding agent

`mw-mcp` is the common integration seam: a local stdio MCP server exposing six
memory tools, also available over HTTP through `mw-serve`. Existing guides
cover Claude Code, Rho, Claude Desktop, Cursor, VS
Code / GitHub Copilot, Windsurf, Zed, Codex CLI, Cline, Continue, Gemini CLI,
Goose, OpenClaw, CrowClaw, Hermes Agent, and other compatible clients.

```bash
mw integrate claude
mw integrate rho
mw doctor
```

Clients do not all provide the same capabilities. MCP supports memory access;
automatic execution capture requires a client-specific hook. The
[integration matrix](integrations/README.md) distinguishes access, capture, and
memory-use guidance and links every verified setup guide.

Rho's current hook payload lacks command text and stdout: failures can be
recorded as metadata with a sentinel command; successful calls without command
text are skipped. The [cross-agent handoff demo](docs/guides/cross-agent-handoff.md)
uses fixtures and a simulated Rho client against real MCP, not live agents or
a verified Cargo fix.

The bundled skill guides memory use; it does not implement automatic task-start
recall, failure lookup, or pre-compaction saving. Those lifecycle decisions
remain with the client. MCP-authored lessons are pending review by default.

## Who is MemoryWhale for?

MemoryWhale is for developers whose debugging context is scattered across
terminal scrollback, shell history, machines, and temporary agent sessions. It
is especially useful when you:

- debug builds, dependencies, Git, environments, or deployments;
- use coding agents across sessions or switch between tools;
- work over SSH or across multiple development machines;
- want recurring failures and their fixes to remain searchable;
- prefer local storage over a hosted memory service.

See [Use cases](docs/concepts/use-cases.md) for each of these as an
end-to-end scenario with real commands.

## Documentation

- [Documentation map](docs/README.md)
- [Getting started](docs/guides/getting-started.md)
- [`mw pet` reference](docs/reference/pet.md)
- [Terminal capture](docs/guides/terminal-capture.md)
- [Agent memory](docs/guides/agent-memory.md)
- [CLI reference](docs/reference/cli.md)
- [Local JSON API](docs/reference/api.md)
- [MCP reference](docs/reference/mcp.md)
- [Security and local threat model](docs/SECURITY.md)
- [Ecosystem](ECOSYSTEM.md) — Delphin, ContextGC, and MemoryWhale together
- [Integration guides and capability matrix](integrations/README.md)

## Contributing

MemoryWhale accepts changes that improve capturing, preserving, retrieving, or
sharing development experience. Read [CONTRIBUTING.md](CONTRIBUTING.md) for the
scope rule, development commands, and pull-request checklist.

Licensed under the [MIT License](LICENSE).
