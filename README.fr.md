<!-- README-SOURCE-SHA256: a97cc1e8f27cad05e884ecb8f7d07e3350335fb2166f58a02447ec5ef9eaa57b -->

<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="Logo MemoryWhale" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>Une mémoire de débogage locale et persistante, pour les développeurs et les agents de code.</strong></p>

<p align="center" dir="ltr"><a href="README.md">English README</a> · <a href="README.ar.md" lang="ar">العربية</a> · <a href="README.de.md" lang="de">Deutsch</a> · <a href="README.fr.md">README français</a> · <a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="release"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="licence MIT"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="local-first, rien n’est envoyé"/>
</p>

MemoryWhale enregistre ce qui s’est réellement passé pendant que vous déboguez :
les commandes, leur sortie, les échecs et les correctifs qui ont fonctionné. Ces
traces sont stockées dans une base SQLite locale, pour que vous et vos agents de
code puissiez les retrouver une fois le terminal fermé, la connexion SSH coupée
ou la session de l’agent terminée.

**MemoryWhale 0.11.0 — Explainable Debugging Evidence · 20 septembre 2026.**
Le CLI, l’interface web et l’application de bureau partagent la version produit
0.11.0 ; le cœur Rust réutilisable est en version 0.5.0. Consultez les [notes de version](https://github.com/wuisabel-gif/MemWhale/blob/v0.11.0/docs/releases/0.11.0.md)
pour le guide de mise à niveau. Le schéma reste en version 10.

**Envie de contribuer ?** Commencez par l’[issue « Start here »](https://github.com/wuisabel-gif/MemWhale/issues/317).
Beaucoup de tâches ne demandent aucune connaissance de Rust, notamment la relecture
des traductions ou la correction de la documentation.

## Pourquoi MemoryWhale ?

- **Gardez une trace de ce qui s’est vraiment passé.** Conservez la commande,
  l’environnement, la sortie, l’échec et la leçon à retenir, pas seulement une
  ligne dans l’historique du shell.
- **Une seule mémoire pour tous vos agents de code.** Tout client MCP stdio
  compatible peut lire et écrire dans la même mémoire locale via `mw-mcp`.
- **Gardez votre historique de développement en local.** MemoryWhale fonctionne
  sans compte, sans service hébergé et sans mémoire facturée au token.

MemoryWhale enregistre votre expérience de développement, pas tout ce que vous
faites. C’est une couche de mémoire dédiée au débogage : ni un agent de code
autonome, ni un système de mémoire personnelle généraliste, ni un substitut à la
documentation de vos projets.

## Nouveautés d’Agent-Native Memory

- **Connectez et inspectez vos agents.** `mw integrate` installe l’accès MCP
  pour Claude Code ou Rho, les hooks de capture et les consignes d’utilisation
  de la mémoire ; `mw doctor` vérifie MCP, les hooks et les skills
  indépendamment les uns des autres.
- **Gardez une provenance explicite.** Le schéma 10 enregistre l’agent d’une
  commande sous la forme `claude`, `rho` ou `NULL`. Le libellé `terminal`,
  utilisé à l’affichage et dans les filtres, désigne une provenance
  terminal/manuelle ou héritée (legacy) ; il ne prouve pas qu’un humain a lancé
  la commande. L’identité de l’agent est distincte du type de source, comme
  `command`, `session` ou `note`.
- **Partagez un dépôt, distinguez les worktrees.** Des identifiants de dépôt
  canoniques regroupent les worktrees liés, tout en conservant la racine de
  chaque worktree et les tags de projet existants. La détection s’appuie sur les
  métadonnées Git locales, pas sur un service distant.
- **Utilisez des interfaces locales.** `mw-serve` expose MCP en HTTP sur
  `POST /mcp` ; `mw-serve --api` active, sur demande explicite, l’API JSON en
  lecture seule. Les deux passent par le listener du dashboard ; tout accès hors
  loopback exige un token.
- **Récupérez le contexte GitHub explicitement.** `mw github context <pr>` lit
  les métadonnées d’une PR, ses checks et ses reviews via votre session `gh`
  existante. La commande affiche un contexte borné et caviardé, sans faire de
  checkout du code ni l’enregistrer automatiquement en mémoire. Aucune
  synchronisation GitHub ne tourne en arrière-plan.

## Installation

Des binaires précompilés sont disponibles pour Linux x86_64/aarch64 et macOS :

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

Vous pouvez aussi passer par Cargo ou Homebrew :

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

Après une installation ou une mise à niveau, vérifiez la version et la
configuration locale :

```bash
mw --version
mw doctor
```

Sous Windows, vous pouvez utiliser MemoryWhale dans
[WSL](https://learn.microsoft.com/windows/wsl/). Le
[guide de démarrage](docs/guides/getting-started.md) détaille l’installation des
paquets, la configuration du PATH et les particularités de chaque plateforme.

## Exemple en soixante secondes

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

![Démonstration des humeurs de mw pet](assets/pet-demo.gif)

Pour les sessions plus longues, `mw --live` enregistre une session shell qui
résiste aux plantages. `mw tui` ouvre un navigateur interactif dans le terminal,
et `mw-serve` lance le dashboard web local.

## Fonctionnement

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

La capture et la recherche sont indépendantes. MCP donne à un agent l’accès à la
mémoire existante ; il n’enregistre pas automatiquement l’activité normale du
terminal. Le modèle complet est décrit dans [l’architecture](docs/architecture.md)
et dans le [concept de capture](docs/concepts/capture.md).

## Compatible avec votre agent de code

`mw-mcp` est le point d’intégration commun : un serveur MCP stdio local qui
expose six outils de mémoire, également accessibles en HTTP via `mw-serve`. Des
guides existent pour Claude Code, Rho, Claude Desktop, Cursor, VS Code / GitHub
Copilot, Windsurf, Zed, Codex CLI, Cline, Continue, Gemini CLI, Goose, OpenClaw,
CrowClaw, Hermes Agent et d’autres clients compatibles.

```bash
mw integrate claude
mw integrate rho
mw doctor
```

Les clients n’offrent pas tous les mêmes capacités. MCP permet d’accéder à la
mémoire ; la capture automatique des exécutions nécessite un hook propre à
chaque client. La [matrice d’intégration](integrations/README.md) distingue
l’accès, la capture et les consignes d’utilisation de la mémoire, et renvoie
vers chaque guide de configuration vérifié.

Le payload actuel des hooks de Rho ne contient ni le texte de la commande ni
stdout : les échecs peuvent être enregistrés sous forme de métadonnées avec une
commande sentinelle, et les appels réussis sans texte de commande sont ignorés.
La [démo de passage de relais entre agents](docs/guides/cross-agent-handoff.md)
utilise des fixtures et un client Rho simulé face à un vrai serveur MCP ; il ne
s’agit ni d’agents réels en fonctionnement, ni d’un correctif Cargo vérifié.

Le skill fourni guide l’utilisation de la mémoire ; il n’implémente aucun
mécanisme automatique de rappel au démarrage d’une tâche, de recherche des
échecs ou de sauvegarde avant compaction. Ces décisions de cycle de vie restent
du ressort du client. Par défaut, les leçons rédigées via MCP sont en attente de
validation.

## À qui s’adresse MemoryWhale ?

MemoryWhale s’adresse aux développeurs dont le contexte de débogage est éparpillé
entre le scrollback du terminal, l’historique du shell, différentes machines et
des sessions d’agent éphémères. Il est particulièrement utile si vous :

- déboguez des builds, des dépendances, Git, des environnements ou des déploiements ;
- utilisez des agents de code d’une session à l’autre ou passez d’un outil à l’autre ;
- travaillez en SSH ou sur plusieurs machines de développement ;
- voulez pouvoir retrouver par une recherche les échecs récurrents et leurs correctifs ;
- préférez un stockage local à un service de mémoire hébergé.

Les [cas d’usage](docs/concepts/use-cases.md) décrivent chacun de ces scénarios de
bout en bout, avec de vraies commandes.

## Documentation

- [Plan de la documentation](docs/README.md)
- [Bien démarrer](docs/guides/getting-started.md)
- [Référence de `mw pet`](docs/reference/pet.md)
- [Capture du terminal](docs/guides/terminal-capture.md)
- [Mémoire des agents](docs/guides/agent-memory.md)
- [Référence du CLI](docs/reference/cli.md)
- [API JSON locale](docs/reference/api.md)
- [Référence MCP](docs/reference/mcp.md)
- [Sécurité et modèle de menace local](docs/SECURITY.md)
- [Écosystème](ECOSYSTEM.md) — Delphin, ContextGC et MemoryWhale réunis
- [Guides d’intégration et matrice des capacités](integrations/README.md)

## Contribuer

MemoryWhale accepte les modifications qui améliorent la capture, la
conservation, la recherche ou le partage de l’expérience de développement.
Lisez [CONTRIBUTING.md](CONTRIBUTING.md) pour connaître la règle de périmètre,
les commandes de développement et la checklist des pull requests. Si vous
débutez sur le projet, choisissez une tâche dans l’[issue « Start here »](https://github.com/wuisabel-gif/MemWhale/issues/317).

Distribué sous [licence MIT](LICENSE).
