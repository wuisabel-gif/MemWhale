<!-- README-SOURCE-SHA256: a97cc1e8f27cad05e884ecb8f7d07e3350335fb2166f58a02447ec5ef9eaa57b -->

<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="MemoryWhale-Logo" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>Dauerhaftes, lokales Debugging-Gedächtnis für Entwickler und Coding-Agenten.</strong></p>

<p align="center" dir="ltr"><a href="README.md">English README</a> · <a href="README.ar.md" lang="ar">العربية</a> · <a href="README.de.md" lang="de">Deutsch</a> · <a href="README.fr.md">README français</a> · <a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="Release"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="MIT-Lizenz"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="Local-first, kein Upload"/>
</p>

MemoryWhale hält fest, was beim Debuggen tatsächlich passiert ist: Befehle,
Ausgaben, Fehlschläge und die Fixes, die funktioniert haben. Diese Belege landen
lokal in SQLite, sodass du und deine Coding-Agenten sie auch dann noch finden,
wenn das Terminal, die SSH-Verbindung oder die Agentensitzung längst weg ist.

**MemoryWhale 0.11.0 — Explainable Debugging Evidence · 20. September 2026.**
CLI, Weboberfläche und Desktop-App teilen sich die Produktversion 0.11.0; der
wiederverwendbare Rust-Kern hat die Version 0.5.0. Hinweise zum Upgrade findest
du in den [Release Notes](https://github.com/wuisabel-gif/MemWhale/blob/v0.11.0/docs/releases/0.11.0.md).
Das Schema bleibt bei Version 10.

**Du willst mitmachen?** Fang mit dem [„Start here“-Issue](https://github.com/wuisabel-gif/MemWhale/issues/317) an.
Für viele Aufgaben brauchst du kein Rust, etwa für das Prüfen von Übersetzungen
oder Korrekturen an der Dokumentation.

## Warum MemoryWhale?

- **Festhalten, was wirklich passiert ist.** Bewahre Befehl, Umgebung, Ausgabe,
  Fehlschlag und Erkenntnis auf – nicht nur eine Zeile in der Shell-History.
- **Ein Gedächtnis für alle Coding-Agenten.** Jeder kompatible stdio-MCP-Client
  kann über `mw-mcp` denselben lokalen Speicher lesen und beschreiben.
- **Die Entwicklungshistorie bleibt lokal.** MemoryWhale funktioniert ohne
  Konto, ohne gehosteten Dienst und ohne tokenbasierte Abrechnung für das
  Gedächtnis.

MemoryWhale speichert Entwicklungserfahrung, nicht alles. Es ist eine
Gedächtnisschicht fürs Debugging – kein autonomer Coding-Agent, kein
universelles persönliches Gedächtnis und kein Ersatz für Projektdokumentation.

## Neu in Agent-Native Memory

- **Agenten anbinden und prüfen.** Mit `mw integrate` richtest du für Claude
  Code oder Rho den MCP-Zugriff, Capture-Hooks und Hinweise zur
  Gedächtnisnutzung ein; `mw doctor` prüft MCP, Hooks und Skills unabhängig
  voneinander.
- **Herkunft explizit halten.** Schema 10 speichert den Agenten eines Befehls
  als `claude`, `rho` oder `NULL`. Das Anzeige- und Filterlabel `terminal`
  steht für Terminal-, manuelle oder ältere Aufzeichnungen – es ist kein Beleg
  dafür, dass ein Mensch den Befehl ausgeführt hat. Die Agentenidentität ist
  unabhängig vom Quelltyp wie `command`, `session` oder `note`.
- **Ein Repository, getrennte Worktrees.** Kanonische Repository-IDs fassen
  verknüpfte Worktrees zusammen und behalten dabei das Wurzelverzeichnis jedes
  Worktrees sowie vorhandene Projekt-Tags bei. Die Erkennung liest lokale
  Git-Metadaten und fragt keinen entfernten Dienst ab.
- **Lokale Schnittstellen nutzen.** `mw-serve` stellt HTTP-MCP unter
  `POST /mcp` bereit; mit `mw-serve --api` schaltest du bewusst die
  schreibgeschützte JSON-API zu. Beide laufen über den Listener des Dashboards;
  Zugriff von außerhalb des Loopback-Interfaces erfordert ein Token.
- **GitHub-Kontext gezielt abrufen.** `mw github context <pr>` liest
  PR-Metadaten, Checks und Reviews über deinen bestehenden `gh`-Login. Der
  Befehl gibt größenbegrenzten, um sensible Daten bereinigten Kontext aus,
  ohne Code auszuchecken oder ihn automatisch im Gedächtnis zu speichern. Eine
  GitHub-Synchronisierung im Hintergrund gibt es nicht.

## Installation

Vorkompilierte Binaries gibt es für Linux x86_64/aarch64 und macOS:

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

Alternativ installierst du über Cargo oder Homebrew:

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

Prüfe nach der Installation oder einem Upgrade die Version und die lokale
Einrichtung:

```bash
mw --version
mw doctor
```

Unter Windows läuft MemoryWhale in
[WSL](https://learn.microsoft.com/windows/wsl/). Hinweise zur Installation
über Paketmanager, zur PATH-Konfiguration und zu den einzelnen Plattformen
stehen in der [Anleitung für den Einstieg](docs/guides/getting-started.md).

## Beispiel in 60 Sekunden

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

![Demo der Stimmungen von mw pet](assets/pet-demo.gif)

Für längere Arbeiten zeichnet `mw --live` eine Shell-Sitzung auf, die auch einen
Absturz übersteht. `mw tui` öffnet einen interaktiven Browser im Terminal,
`mw-serve` startet das lokale Web-Dashboard.

## So funktioniert es

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

Aufzeichnung und Abruf sind voneinander unabhängig. MCP gibt einem Agenten
Zugriff auf bereits vorhandene Erinnerungen, zeichnet aber normale
Terminalaktivität nicht automatisch auf. Das vollständige Modell beschreiben
die [Architektur](docs/architecture.md) und das
[Konzept zur Aufzeichnung](docs/concepts/capture.md).

## Funktioniert mit deinem Coding-Agenten

`mw-mcp` ist die gemeinsame Integrationsschnittstelle: ein lokaler
stdio-MCP-Server mit sechs Gedächtniswerkzeugen, die über `mw-serve` auch per
HTTP erreichbar sind. Anleitungen gibt es bereits für Claude Code, Rho, Claude
Desktop, Cursor, VS Code / GitHub Copilot, Windsurf, Zed, Codex CLI, Cline,
Continue, Gemini CLI, Goose, OpenClaw, CrowClaw, Hermes Agent und weitere
kompatible Clients.

```bash
mw integrate claude
mw integrate rho
mw doctor
```

Nicht jeder Client kann dasselbe. MCP ermöglicht den Zugriff aufs Gedächtnis;
um ausgeführte Befehle automatisch aufzuzeichnen, braucht es einen
clientspezifischen Hook. Die [Integrationsmatrix](integrations/README.md)
unterscheidet zwischen Zugriff, Aufzeichnung und Hinweisen zur
Gedächtnisnutzung und verlinkt alle geprüften Einrichtungsanleitungen.

Die aktuelle Hook-Payload von Rho enthält weder den Befehlstext noch stdout:
Fehlschläge lassen sich als Metadaten mit einem Platzhalterbefehl speichern,
erfolgreiche Aufrufe ohne Befehlstext werden übersprungen. Die
[Demo zur Übergabe zwischen Agenten](docs/guides/cross-agent-handoff.md)
arbeitet mit Fixtures und einem simulierten Rho-Client gegen echtes MCP – nicht
mit live laufenden Agenten und nicht mit einem verifizierten Cargo-Fix.

Der mitgelieferte Skill gibt Hinweise zur Gedächtnisnutzung. Er implementiert
weder einen automatischen Abruf beim Start einer Aufgabe noch ein automatisches
Nachschlagen von Fehlschlägen oder ein Speichern vor der Kontextkomprimierung.
Solche Entscheidungen über den Sitzungsablauf bleiben Sache des Clients. Über
MCP angelegte Erkenntnisse müssen standardmäßig erst geprüft werden.

## Für wen ist MemoryWhale gedacht?

MemoryWhale richtet sich an Entwickler, deren Debugging-Kontext über
Terminal-Scrollback, Shell-History, verschiedene Rechner und kurzlebige
Agentensitzungen verstreut ist. Besonders nützlich ist es, wenn du:

- Builds, Abhängigkeiten, Git, Umgebungen oder Deployments debuggst;
- Coding-Agenten über mehrere Sitzungen hinweg nutzt oder zwischen Tools wechselst;
- per SSH oder auf mehreren Entwicklungsrechnern arbeitest;
- wiederkehrende Fehlschläge samt ihren Fixes durchsuchbar halten willst;
- lokale Speicherung einem gehosteten Gedächtnisdienst vorziehst.

Unter [Anwendungsfälle](docs/concepts/use-cases.md) findest du jedes dieser
Szenarien durchgespielt – von Anfang bis Ende und mit echten Befehlen.

## Dokumentation

- [Überblick über die Dokumentation](docs/README.md)
- [Erste Schritte](docs/guides/getting-started.md)
- [`mw pet`-Referenz](docs/reference/pet.md)
- [Terminal-Aufzeichnung](docs/guides/terminal-capture.md)
- [Agentengedächtnis](docs/guides/agent-memory.md)
- [CLI-Referenz](docs/reference/cli.md)
- [Lokale JSON-API](docs/reference/api.md)
- [MCP-Referenz](docs/reference/mcp.md)
- [Sicherheit und lokales Bedrohungsmodell](docs/SECURITY.md)
- [Ökosystem](ECOSYSTEM.md) – Delphin, ContextGC und MemoryWhale im Zusammenspiel
- [Integrationsanleitungen und Funktionsmatrix](integrations/README.md)

## Mitwirken

MemoryWhale nimmt Änderungen an, die das Aufzeichnen, Bewahren, Abrufen oder
Teilen von Entwicklungserfahrung verbessern. In
[CONTRIBUTING.md](CONTRIBUTING.md) findest du die Regeln zum Projektumfang, die
Entwicklungsbefehle und die Checkliste für Pull Requests. Wenn du neu dabei
bist, such dir eine Aufgabe aus dem [„Start here“-Issue](https://github.com/wuisabel-gif/MemWhale/issues/317) aus.

Veröffentlicht unter der [MIT-Lizenz](LICENSE).
