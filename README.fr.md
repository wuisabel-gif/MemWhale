<!-- README-SOURCE-SHA256: 0c0dee340943c6650cc58749d96cb86c7728e6fc31cef1f4daba0be190a01965 -->

<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="Logo de MemoryWhale" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>Une mémoire locale et persistante pour le débogage, pensée pour les développeurs et les agents de code.</strong></p>

<p align="center"><a href="README.md">English README</a> · <a href="README.fr.md">README français</a> · <a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="release"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="licence MIT"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="local-first, aucun envoi de données"/>
</p>

MemoryWhale enregistre ce qui s’est réellement passé pendant vos sessions de
débogage : les commandes, leurs sorties, les erreurs rencontrées et les
correctifs qui ont fonctionné. Toutes ces informations sont conservées dans une
base SQLite locale afin que vous et vos agents de code puissiez les retrouver,
même une fois le terminal, la connexion SSH ou la session de l’agent terminés.

**MemoryWhale 0.10.0 — Agent-Native Memory · 6 septembre 2026.**
Le CLI, l’interface web et l’application de bureau partagent la version 0.10.0
du produit ; le cœur Rust réutilisable est en version 0.5.0. Consultez les
[notes de version](https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md)
pour le guide de mise à niveau et les détails concernant la modification
incompatible de l’API Rust.

## Pourquoi MemoryWhale ?

- **Gardez une trace de ce qui s’est réellement passé.** Conservez la commande,
  l’environnement, la sortie, l’erreur et la leçon à en tirer — pas seulement une
  ligne dans l’historique du shell.
- **Partagez une même mémoire entre vos agents de code.** Tout client MCP stdio
  compatible peut lire et écrire dans la même mémoire locale via `mw-mcp`.
- **Gardez votre historique de développement en local.** MemoryWhale fonctionne
  sans compte, sans service hébergé et sans facturation de la mémoire au nombre
  de tokens.

MemoryWhale mémorise votre expérience de développement, pas tout ce que vous
faites. Il s’agit d’une couche de mémoire dédiée au débogage, et non d’un agent
de code autonome, d’un système de mémoire personnelle généraliste ou d’un
substitut à la documentation de vos projets.

## Nouveautés d’Agent-Native Memory

- **Connectez et inspectez vos agents.** Installez l’accès MCP pour Claude Code
  ou Rho, les hooks de capture et les consignes d’utilisation de la mémoire avec
  `mw integrate` ; `mw doctor` vérifie séparément MCP, les hooks et les skills.
- **Gardez une provenance explicite.** Le schéma 10 enregistre l’agent à
  l’origine d’une commande sous la forme `claude`, `rho` ou `NULL`. Le libellé
  d’affichage et de filtrage `terminal` désigne une provenance terminal/manuelle
  ou historique ; il ne prouve pas qu’un humain a exécuté la commande.
  L’identité de l’agent est distincte du type de source, par exemple `command`,
  `session` ou `note`.
- **Partagez un dépôt tout en distinguant les worktrees.** Les identifiants
  canoniques de dépôt regroupent les worktrees liés tout en conservant la racine
  propre à chacun et les tags de projet existants. La détection repose sur les
  métadonnées Git locales, et non sur un service distant.
- **Utilisez des interfaces locales.** `mw-serve` expose MCP en HTTP via
  `POST /mcp` ; `mw-serve --api` active explicitement l’API JSON en lecture seule.
  Les deux utilisent le listener du dashboard ; tout accès hors loopback
  nécessite un token.
- **Récupérez explicitement le contexte GitHub.** `mw github context <pr>`
  récupère les métadonnées d’une PR, ses checks et ses reviews à l’aide de votre
  session `gh` existante. La commande affiche un contexte limité et expurgé des
  données sensibles, sans checkout du code ni enregistrement automatique dans
  la mémoire. Aucune synchronisation GitHub ne s’exécute en arrière-plan.

## Installation

Des binaires précompilés sont disponibles pour Linux x86_64/aarch64 et macOS :

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

Vous pouvez également l’installer avec Cargo ou Homebrew :

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

Après l’installation ou une mise à niveau, vérifiez la version et la
configuration locale :

```bash
mw --version
mw doctor
```

Sous Windows, MemoryWhale peut être utilisé via
[WSL](https://learn.microsoft.com/windows/wsl/). Consultez le
[guide de démarrage](docs/guides/getting-started.md) pour l’installation des
paquets, la configuration du PATH et les remarques propres à chaque plateforme.

## Exemple en 60 secondes

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

![Démonstration des humeurs de mw pet](assets/pet-demo.gif)

Pour les sessions plus longues, `mw --live` enregistre une session shell de
façon à résister aux crashs. `mw tui` ouvre un navigateur interactif dans le
terminal, tandis que `mw-serve` démarre le dashboard web local.

## Fonctionnement

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

La capture et la recherche sont indépendantes. MCP permet à un agent d’accéder
à la mémoire existante ; il n’enregistre pas automatiquement l’activité normale
du terminal. Consultez la documentation de
[l’architecture](docs/architecture.md) et le
[concept de capture](docs/concepts/capture.md) pour une présentation complète
du modèle.

## Fonctionne avec votre agent de code

`mw-mcp` constitue le point d’intégration commun : il s’agit d’un serveur MCP
stdio local qui expose six outils de mémoire, également accessibles en HTTP
via `mw-serve`. Des guides sont disponibles pour Claude Code, Rho, Claude
Desktop, Cursor, VS Code / GitHub Copilot, Windsurf, Zed, Codex CLI, Cline,
Continue, Gemini CLI, Goose, OpenClaw, CrowClaw, Hermes Agent ainsi que d’autres
clients compatibles.

```bash
mw integrate claude
mw integrate rho
mw doctor
```

Tous les clients n’offrent pas les mêmes fonctionnalités. MCP donne accès à la
mémoire ; la capture automatique des commandes exécutées nécessite un hook
spécifique au client. La [matrice d’intégration](integrations/README.md)
distingue l’accès à la mémoire, la capture et les consignes d’utilisation de la
mémoire, avec un lien vers chaque guide de configuration vérifié.

Le payload actuel des hooks de Rho ne contient ni le texte de la commande ni
stdout : les échecs peuvent être enregistrés sous forme de métadonnées avec
une commande sentinelle ; les appels réussis sans texte de commande sont
ignorés. La [démo de transfert entre agents](docs/guides/cross-agent-handoff.md)
utilise des fixtures et un client Rho simulé avec un véritable serveur MCP ;
elle ne repose ni sur des agents exécutés en conditions réelles ni sur un
correctif Cargo vérifié.

Le skill fourni indique à l’agent comment utiliser la mémoire ; il n’implémente
pas automatiquement le rappel au démarrage d’une tâche, la recherche d’erreurs
ou la sauvegarde avant compaction. Ces décisions de cycle de vie restent à la
charge du client. Les leçons créées via MCP sont, par défaut, placées en attente
de validation.

## À qui s’adresse MemoryWhale ?

MemoryWhale s’adresse aux développeurs dont le contexte de débogage se retrouve
dispersé entre le scrollback du terminal, l’historique du shell, plusieurs
machines et des sessions temporaires d’agents. Il est particulièrement utile
si vous :

- déboguez des builds, des dépendances, Git, des environnements ou des déploiements ;
- utilisez des agents de code sur plusieurs sessions ou passez régulièrement d’un outil à l’autre ;
- travaillez en SSH ou sur plusieurs machines de développement ;
- souhaitez pouvoir retrouver facilement les erreurs récurrentes et leurs correctifs ;
- préférez un stockage local à un service de mémoire hébergé.

Consultez les [cas d’usage](docs/concepts/use-cases.md) pour découvrir chacun de
ces scénarios de bout en bout, avec de vraies commandes.

## Documentation

- [Sommaire de la documentation](docs/README.md)
- [Bien démarrer](docs/guides/getting-started.md)
- [Référence de `mw pet`](docs/reference/pet.md)
- [Capture du terminal](docs/guides/terminal-capture.md)
- [Mémoire des agents](docs/guides/agent-memory.md)
- [Référence du CLI](docs/reference/cli.md)
- [API JSON locale](docs/reference/api.md)
- [Référence MCP](docs/reference/mcp.md)
- [Sécurité et modèle de menace local](docs/SECURITY.md)
- [Écosystème](ECOSYSTEM.md) — Delphin, ContextGC et MemoryWhale
- [Guides d’intégration et matrice des fonctionnalités](integrations/README.md)

## Contribuer

MemoryWhale accepte les contributions qui améliorent la capture, la
conservation, la recherche ou le partage de l’expérience de développement.
Consultez [CONTRIBUTING.md](CONTRIBUTING.md) pour connaître le périmètre du
projet, les commandes de développement et la checklist des pull requests.

Distribué sous [licence MIT](LICENSE).
