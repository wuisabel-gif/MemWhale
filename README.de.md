<!-- README-SOURCE-SHA256: 0a6b911696113a6221346fd4145b26a73f5a62493d8a8d34048d26a832c0c100 -->

<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="MemoryWhale-Logo" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>Dauerhaftes lokales Debugging-Gedächtnis für Entwickler und Coding-Agenten.</strong></p>

<p align="center" dir="ltr"><a href="README.md">English README</a> · <a href="README.ar.md" lang="ar">العربية</a> · <a href="README.de.md" lang="de">Deutsch</a> · <a href="README.fr.md">README français</a> · <a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="Release"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="MIT-Lizenz"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="Local-first, kein Upload"/>
</p>

MemoryWhale hält fest, was beim Debuggen tatsächlich passiert ist: Befehle,
Ausgaben, Fehler und die Lösungen, die funktioniert haben. Diese Informationen
werden lokal in SQLite gespeichert, damit du und deine Coding-Agenten sie auch
dann wiederfinden, wenn das Terminal, die SSH-Verbindung oder die Agentensitzung
längst beendet ist.

**MemoryWhale 0.10.0 — Agent-Native Memory · 6. September 2026.**
CLI, Weboberfläche und Desktop-App haben die gemeinsame Produktversion 0.10.0;
der wiederverwendbare Rust-Kern hat die Version 0.5.0. Die
[Release Notes](https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md)
enthalten Hinweise zum Upgrade und zur inkompatiblen Änderung der Rust-API.

## Warum MemoryWhale?

- **Behalte, was wirklich passiert ist.** Bewahre Befehl, Umgebung, Ausgabe,
  Fehler und Erkenntnis auf – nicht nur eine Zeile in der Shell-History.
- **Nutze ein gemeinsames Gedächtnis für deine Coding-Agenten.** Jeder kompatible
  stdio-MCP-Client kann über `mw-mcp` auf denselben lokalen Speicher zugreifen
  und darin schreiben.
- **Halte deine Entwicklungshistorie lokal.** MemoryWhale funktioniert ohne
  Konto, gehosteten Dienst oder tokenbasierte Abrechnung für den Speicher.

MemoryWhale speichert Entwicklungserfahrung, nicht alles. Es ist eine
Gedächtnisschicht fürs Debugging – kein autonomer Coding-Agent, kein universelles
persönliches Gedächtnis und kein Ersatz für Projektdokumentation.

## Neu in Agent-Native Memory

- **Agenten verbinden und ihre Einrichtung prüfen.** Mit `mw integrate`
  installierst du MCP-Zugriff, Capture-Hooks und Hinweise zur Speichernutzung
  für Claude Code oder Rho. `mw doctor` prüft MCP, Hooks und Skills getrennt.
- **Die Herkunft nachvollziehbar halten.** Schema 10 speichert den ausführenden
  Agenten einer Befehlsaufzeichnung als `claude`, `rho` oder `NULL`. Das Anzeige-
  und Filterlabel `terminal` steht für Terminal-, manuelle oder ältere
  Aufzeichnungen; es beweist nicht, dass ein Mensch den Befehl ausgeführt hat.
  Die Agentenidentität ist unabhängig vom Quelltyp, etwa `command`, `session`
  oder `note`.
- **Ein Repository teilen, Worktrees unterscheiden.** Kanonische Repository-IDs
  gruppieren verknüpfte Worktrees, ohne deren jeweilige Wurzelverzeichnisse oder
  vorhandene Projekt-Tags zu verlieren. Die Erkennung liest lokale Git-Metadaten
  und kontaktiert keinen entfernten Dienst.
- **Lokale Schnittstellen nutzen.** `mw-serve` stellt HTTP-MCP unter
  `POST /mcp` bereit. `mw-serve --api` aktiviert ausdrücklich die schreibgeschützte
  JSON-API. Beide nutzen den Listener des Dashboards; Zugriffe außerhalb von
  Loopback erfordern ein Token.
- **GitHub-Kontext ausdrücklich abrufen.** `mw github context <pr>` liest
  PR-Metadaten, Checks und Reviews über deine bestehende `gh`-Anmeldung. Der
  ausgegebene Kontext ist größenbegrenzt und um erkannte sensible Daten bereinigt.
  Die Funktion checkt keinen Code aus und speichert nichts automatisch als
  Erinnerung. Es gibt keine GitHub-Synchronisierung im Hintergrund.

## Installation

Vorgefertigte Binärdateien gibt es für Linux x86_64/aarch64 und macOS:

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

Alternativ kannst du Cargo oder Homebrew verwenden:

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

Unter Windows kannst du MemoryWhale in
[WSL](https://learn.microsoft.com/windows/wsl/) ausführen. Hinweise zur
Paketinstallation, zur Einrichtung des PATH und zu den unterstützten Plattformen
findest du in der [Einstiegsanleitung](docs/guides/getting-started.md).

## Ein Beispiel in 60 Sekunden

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

![Demo der Stimmungen von mw pet](assets/pet-demo.gif)

Für längere Arbeiten zeichnet `mw --live` eine Shell-Sitzung mit regelmäßiger
Sicherung auf, die auch nach einem Absturz verwertbar bleibt. `mw tui` öffnet
einen interaktiven Browser im Terminal; `mw-serve` startet das lokale
Web-Dashboard.

## So funktioniert es

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

Aufzeichnung und Abruf sind unabhängig voneinander. MCP gibt einem Agenten
Zugriff auf vorhandene Erinnerungen, zeichnet aber normale Terminalaktivität
nicht automatisch auf. Das vollständige Modell beschreiben die
[Architektur](docs/architecture.md) und das
[Konzept zur Aufzeichnung](docs/concepts/capture.md).

## Funktioniert mit deinem Coding-Agenten

`mw-mcp` ist der gemeinsame Integrationspunkt: ein lokaler stdio-MCP-Server mit
sechs Speicherwerkzeugen, die über `mw-serve` auch per HTTP verfügbar sind.
Es gibt Anleitungen für Claude Code, Rho, Claude Desktop, Cursor, VS Code /
GitHub Copilot, Windsurf, Zed, Codex CLI, Cline, Continue, Gemini CLI, Goose,
OpenClaw, CrowClaw, Hermes Agent und weitere kompatible Clients.

```bash
mw integrate claude
mw integrate rho
mw doctor
```

Nicht alle Clients bieten dieselben Funktionen. MCP ermöglicht den
Speicherzugriff; die automatische Aufzeichnung ausgeführter Befehle braucht
einen clientspezifischen Hook. Die [Integrationsmatrix](integrations/README.md)
unterscheidet Speicherzugriff, Aufzeichnung und Hinweise zur Speichernutzung
und verlinkt die jeweils geprüften Einrichtungsanleitungen.

Rhos aktuelle Hook-Payload enthält weder Befehlstext noch stdout: Fehler können
als Metadaten mit einem Platzhalterbefehl gespeichert werden; erfolgreiche
Aufrufe ohne Befehlstext werden übersprungen. Die
[Demo zur Übergabe zwischen Agenten](docs/guides/cross-agent-handoff.md)
verwendet Fixtures und einen simulierten Rho-Client mit echtem MCP – keine live
ausgeführten Agenten und keinen tatsächlich überprüften Cargo-Fix.

Der mitgelieferte Skill gibt Hinweise zur Speichernutzung. Er implementiert
keinen automatischen Abruf zum Aufgabenstart, keine automatische Fehlersuche
und kein Speichern vor einer Kontextkomprimierung. Diese Entscheidungen über
den Sitzungsablauf bleiben beim Client. Über MCP verfasste Erkenntnisse warten
standardmäßig auf eine Prüfung.

## Für wen ist MemoryWhale gedacht?

MemoryWhale richtet sich an Entwickler, deren Debugging-Kontext über den
Terminal-Scrollback, die Shell-History, mehrere Rechner und vorübergehende
Agentensitzungen verteilt ist. Es ist besonders hilfreich, wenn du:

- Builds, Abhängigkeiten, Git, Entwicklungsumgebungen oder Deployments untersuchst;
- Coding-Agenten über mehrere Sitzungen nutzt oder zwischen Tools wechselst;
- über SSH oder auf mehreren Entwicklungsrechnern arbeitest;
- wiederkehrende Fehler und ihre Lösungen wiederfinden möchtest;
- lokale Speicherung einem gehosteten Speicherdienst vorziehst.

Die [Anwendungsfälle](docs/concepts/use-cases.md) zeigen diese Szenarien
Schritt für Schritt mit echten Befehlen.

## Dokumentation

- [Dokumentationsübersicht](docs/README.md)
- [Erste Schritte](docs/guides/getting-started.md)
- [`mw pet`-Referenz](docs/reference/pet.md)
- [Terminalaufzeichnung](docs/guides/terminal-capture.md)
- [Agentengedächtnis](docs/guides/agent-memory.md)
- [CLI-Referenz](docs/reference/cli.md)
- [Lokale JSON-API](docs/reference/api.md)
- [MCP-Referenz](docs/reference/mcp.md)
- [Sicherheit und lokales Bedrohungsmodell](docs/SECURITY.md)
- [Ökosystem](ECOSYSTEM.md) – Delphin, ContextGC und MemoryWhale
- [Integrationsanleitungen und Funktionsmatrix](integrations/README.md)

## Mitwirken

MemoryWhale begrüßt Änderungen, die das Aufzeichnen, Bewahren, Abrufen oder
Teilen von Entwicklungserfahrung verbessern. In
[CONTRIBUTING.md](CONTRIBUTING.md) findest du den Projektumfang,
Entwicklungsbefehle und die Checkliste für Pull Requests.

Veröffentlicht unter der [MIT-Lizenz](LICENSE).
