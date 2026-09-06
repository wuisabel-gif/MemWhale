const EN = {
  meta: {
    title: "MemoryWhale — terminal memory for you and your AI agent",
    description:
      "MemoryWhale captures development evidence into local SQLite so people and trusted tools can retrieve past failures and lessons. Local-first, with explicit export and transfer.",
    jsonLdDescription:
      "Persistent local debugging memory for developers and coding agents. Captures terminal evidence into local SQLite and serves it over MCP."
  },
  "nav.label": "Page navigation",
  "brand.home": "MemoryWhale home",
  "nav.terminal": "Terminal Memory",
  "nav.how": "How It Works",
  "nav.agents": "AI Agents",
  "nav.who": "Who It's For",
  "nav.install": "Install",
  "nav.docs": "Docs",
  "nav.releases": "Releases",
  "nav.github": "GitHub ↗",
  "nav.delphin": "Delphin ↗",
  "nav.contextgc": "ContextGC ↗",
  "language.label": "Language",
  "language.en": "English",
  "language.fr": "Français",
  "language.zh-CN": "简体中文",
  "language.zh-TW": "繁體中文",
  "language.ko": "한국어",
  "language.ja": "日本語",
  "release.banner": "🐋 v0.10.0 — Agent-Native Memory · September 6, 2026 · release notes and upgrade guide →",
  "hero.eyebrow": "Local-first terminal memory",
  "hero.title": "MemoryWhale remembers what your terminal forgets.",
  "hero.lead":
    "Capture terminal evidence, preserve it in local SQLite, and retrieve the failures and lessons that matter. MemoryWhale is local-first: it does not silently upload or synchronize your data.",
  "hero.demoCta": "See the 60-second demo",
  "hero.installCta": "Install MemoryWhale",
  "hero.securityCta": "Read the security model",
  "hero.memoryChip": "Terminal memory live",
  "hero.whaleAlt": "Luminous whale swimming through knowledge graph nodes",
  "release.eyebrow": "New in 0.10.0",
  "release.title": "Shared memory. Explicit provenance.",
  "release.copy":
    "Product 0.10.0 spans the CLI, web UI, and desktop app. The reusable Rust core is 0.5.0: Rust <code>Memory</code> literals now require <code>agent: Option&lt;String&gt;</code>; older JSON remains readable through its serde default.",
  "release.connectTitle": "Connect Claude Code and Rho",
  "release.connectBody":
    "<code>mw integrate claude</code> and <code>mw integrate rho</code> install MCP access, capture hooks, and a skill. <code>mw doctor</code> checks those components independently.",
  "release.provenanceTitle": "Know where evidence came from",
  "release.provenanceBody":
    "Schema 10 stores command agents as <code>claude</code>, <code>rho</code>, or <code>NULL</code>, displayed as <code>terminal</code>. Agent is separate from source type. Canonical repository IDs group linked worktrees without losing each worktree's path.",
  "release.interfaceTitle": "Choose your local interface",
  "release.interfaceBody":
    "<code>mw-serve</code> adds HTTP MCP at <code>POST /mcp</code>. <code>--api</code> opts into a read-only JSON API. <code>mw github context &lt;pr&gt;</code> explicitly reads PR metadata, checks, commit statuses, and reviews via your <code>gh</code> login: no checkout, automatic save, or background sync.",
  "who.eyebrow": "Who it's for",
  "who.title": "Built for three ways of working.",
  "who.copy":
    "MemoryWhale serves developers whose debugging context is scattered across terminal scrollback, shell history, machines, and temporary agent sessions. See the full <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">use-case walkthroughs</a> with real command transcripts.",
  "who.shellTitle": "🔍 The shell-centric debugger",
  "who.shellBody":
    "You hit the same build, linker, or dependency error twice. Shell history remembers the command — not the output, the error tail, or the fix. <code>mw search</code> returns the old failing run <em>and</em> the lesson linked to it.",
  "who.multiTitle": "🛰️ The multi-machine worker",
  "who.multiBody":
    "Jetson, lab server, laptop — sessions drop and each machine keeps a private, incomplete history. <code>mw --live</code> autosaves through disconnects; <code>mw push</code> / <code>mw pull</code> move memory between machines explicitly.",
  "who.agentTitle": "🤖 The coding-agent user",
  "who.agentBody":
    "Claude Code, Codex, Cursor — every session starts with re-explaining your environment. With <code>mw-mcp</code>, the agent can query prior evidence and explicitly save a lesson with <code>remember</code>. You still need to verify that a fix works.",
  "terminal.eyebrow": "Terminal memory",
  "terminal.title": "A memory palace for command-line work.",
  "terminal.copy":
    "MemoryWhale stores terminal sessions as structured local memory. Instead of keeping one giant text dump, it saves the command, every argument, the working directory, the exit code, stdout, stderr, and your own notes.",
  "terminal.argsTitle": "Arguments Become Searchable",
  "terminal.argsBody":
    "Flags like <code>--manifest-path</code>, paths, subcommands, package names, and model options are split into their own rows.",
  "terminal.errorsTitle": "Error Logs Stay Attached",
  "terminal.errorsBody":
    "stderr is preserved beside the command that produced it, so the cause and context remain together.",
  "terminal.liveTitle": "Live Autosave for Sessions",
  "terminal.liveBody":
    "<code>mw --live</code> writes the active shell transcript into SQLite every few seconds, so a disconnect can still leave a usable memory trail.",
  "terminal.graphTitle": "Graph Nodes for Failures",
  "terminal.graphBody":
    "Failed commands appear in the knowledge galaxy and connect to extracted concepts like cargo, Tauri, SQLite, ports, and builds.",
  "how.eyebrow": "How it works",
  "how.title": "Capture, store, extract, explore.",
  "how.captureTitle": "Capture",
  "how.captureBody": "Paste a terminal run, call the Rust helper, or start a live-autosaved shell.",
  "how.storeTitle": "Store",
  "how.storeBody": "SQLite saves command runs and arguments locally on your machine.",
  "how.extractTitle": "Extract",
  "how.extractBody": "Rust extracts keywords from commands, notes, and error text.",
  "how.exploreTitle": "Explore",
  "how.exploreBody": "Search or click command nodes in the glowing graph interface.",
  "agents.eyebrow": "AI agents",
  "agents.title": "Give your agent memory of what already failed.",
  "agents.copy":
    "Coding-agent sessions can lose context and repeat debugging you already did. <code>mw-mcp</code> is a Model Context Protocol server over your local memory — register it once and Claude Code, Rho, Codex, or Cursor can query past failures directly. Trust the client with the evidence it retrieves, including any model provider to which that client sends context.",
  "agents.clientsLabel": "Clients with integration guides",
  "agents.matrix": "More in the matrix",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">Setup guides for clients and tools</a> — the capability matrix documents MCP support, auto-capture, and verification status per client, including model gateways like OpenRouter and CLIProxyAPI.",
  "agents.setupLabel": "Setup",
  "agents.setupValue": "One command",
  "agents.toolsLabel": "Tools",
  "agents.toolsValue": "6 local MCP tools: recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "No agent?",
  "agents.noAgentValue": "mw context prints a paste-ready digest",
  "demo.eyebrow": "Capture → memory → retrieval",
  "demo.title": "See the core loop with synthetic data.",
  "demo.copy":
    "Capture one command, save the explanation that fixed it, then search the local store when the same failure returns. MCP provides retrieval and explicit writing; it does not capture ordinary terminal activity automatically.",
  "demo.handoff":
    "The <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">offline Claude-to-Rho handoff demo</a> imports fixtures and simulates a Rho client against real MCP. It does not run live agents or execute and verify the fixture's Cargo fix. Rho hooks currently retain failure metadata when command text is absent; successful calls without command text are skipped. Automatic task-start recall, failure lookup, and pre-compaction saving remain client-orchestration work, not shipped automation.",
  "demo.imageAlt": "Synthetic MemoryWhale terminal and dashboard demo",
  "data.eyebrow": "Your data",
  "data.title": "Local-first means visible choices.",
  "data.copy":
    "The database lives on your machine: typically <code>~/.local/share/MemoryWhale/</code> on Linux or <code>~/Library/Application Support/MemoryWhale/</code> on macOS. Set <code>MEMORYWHALE_DATA_DIR</code> to choose another location.",
  "data.captureLabel": "Capture controls",
  "data.captureValue": "<code>.mwignore</code>, path policy, commands-only",
  "data.redactionLabel": "Redaction",
  "data.redactionValue": "Helps with common secrets; it is not a security boundary",
  "data.sizeLabel": "Size limit",
  "data.sizeValue": "Captured text fields default to 1 MiB with truncation",
  "data.inspectLabel": "Inspect / delete",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "Transfer",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code> or explicit SSH transfer",
  "data.stewardshipLabel": "Stewardship",
  "data.stewardshipValue": "<code>mw memory compact</code> — dry-run first, rows preserved",
  "security.eyebrow": "Security model",
  "security.title": "Local by default, explicit when shared.",
  "security.copy":
    "The CLI, TUI, MCP server, web dashboard, and desktop shell use the local store. <code>mw-mcp</code> is a trusted local stdio process; the dashboard binds to loopback by default. A non-loopback dashboard requires a token, and should only be exposed on a trusted network. Protected HTTP MCP requires Bearer authentication; the opt-in JSON API shares the dashboard's access controls. HTTP does not encrypt the connection. Neither interface makes client access automatic capture. See the <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">local data threat model</a>.",
  "run.eyebrow": "Install",
  "run.title": "One line. No Rust needed.",
  "run.copy":
    "Prebuilt binaries are available for Linux x86_64/aarch64 and macOS. The installer verifies published SHA256 files when a release provides them; older releases may not have a checksum. Start with one explicit capture, inspect it, and only then consider <code>mw global on</code>. Windows is not a native target; WSL can use the Linux build.",
  "run.tryLabel": "Try first",
  "run.tryValue": "<code>mw demo</code> — writes sample data to the selected store",
  "run.prebuiltLabel": "Prebuilt install",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md#install-or-upgrade\">Pinned, checksum-verified installer instructions</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.10.0 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": ".deb on the releases page",
  "run.securityLabel": "Security",
  "run.securityValue": "<a href=\"#security\">Read the model</a>",
  "run.verifyLabel": "Verify",
  "run.verifyValue": "<code>mw --version</code> · <code>mw doctor</code>",
  "footer.copyright": "Copyright (c) 2026 wuisabel-gif. MemoryWhale - Rust/Tauri terminal memory and knowledge graph.",
  "footer.docs": "Documentation",
  "footer.useCases": "Use cases",
  "footer.cli": "CLI reference",
  "footer.security": "Security policy",
  "footer.integrations": "Integrations"
};

const FR = {
  meta: {
    title: "MemoryWhale — une mémoire de terminal pour vous et votre agent IA",
    description:
      "MemoryWhale capture les preuves de développement dans SQLite local afin que les personnes et les outils de confiance puissent retrouver les échecs et les leçons utiles. Local par défaut, avec export et transfert explicites.",
    jsonLdDescription:
      "Mémoire locale et persistante du débogage pour les développeurs et les agents de code. Capture les preuves du terminal dans SQLite local et les sert via MCP."
  },
  "nav.label": "Navigation de la page",
  "brand.home": "Accueil de MemoryWhale",
  "nav.terminal": "Mémoire du terminal",
  "nav.how": "Fonctionnement",
  "nav.agents": "Agents IA",
  "nav.who": "Pour qui ?",
  "nav.install": "Installer",
  "nav.docs": "Documentation",
  "nav.releases": "Versions",
  "nav.github": "GitHub ↗",
  "nav.delphin": "Delphin ↗",
  "nav.contextgc": "ContextGC ↗",
  "language.label": "Langue",
  "language.en": "Anglais",
  "language.fr": "Français",
  "language.zh-CN": "Chinois simplifié",
  "language.zh-TW": "Chinois traditionnel",
  "language.ko": "Coréen",
  "language.ja": "Japonais",
  "release.banner": "🐋 v0.10.0 — Agent-Native Memory · 6 septembre 2026 · notes de version et guide de mise à niveau →",
  "hero.eyebrow": "Mémoire de terminal locale par défaut",
  "hero.title": "MemoryWhale se souvient de ce que votre terminal oublie.",
  "hero.lead":
    "Capturez les preuves du terminal, conservez-les dans SQLite local, puis retrouvez les échecs et les leçons qui comptent. MemoryWhale privilégie le local : vos données ne sont ni téléversées ni synchronisées silencieusement.",
  "hero.demoCta": "Voir la démo en 60 secondes",
  "hero.installCta": "Installer MemoryWhale",
  "hero.securityCta": "Lire le modèle de sécurité",
  "hero.memoryChip": "Mémoire du terminal active",
  "hero.whaleAlt": "Baleine lumineuse nageant parmi des nœuds d'un graphe de connaissances",
  "release.eyebrow": "Nouveau dans 0.10.0",
  "release.title": "Mémoire partagée. Provenance explicite.",
  "release.copy":
    "Le produit 0.10.0 couvre le CLI, l'interface web et l'application de bureau. Le cœur Rust réutilisable est en version 0.5.0 : les littéraux Rust <code>Memory</code> exigent désormais <code>agent: Option&lt;String&gt;</code> ; l'ancien JSON reste lisible grâce à la valeur par défaut de serde.",
  "release.connectTitle": "Connectez Claude Code et Rho",
  "release.connectBody":
    "<code>mw integrate claude</code> et <code>mw integrate rho</code> installent l'accès MCP, les hooks de capture et un skill. <code>mw doctor</code> vérifie ces composants séparément.",
  "release.provenanceTitle": "Sachez d'où viennent les preuves",
  "release.provenanceBody":
    "Le schéma 10 enregistre les agents des commandes sous la forme <code>claude</code>, <code>rho</code> ou <code>NULL</code>, affichés comme <code>terminal</code>. L'agent est distinct du type de source. Les identifiants canoniques des dépôts regroupent les worktrees liés sans perdre le chemin de chacun.",
  "release.interfaceTitle": "Choisissez votre interface locale",
  "release.interfaceBody":
    "<code>mw-serve</code> ajoute HTTP MCP sur <code>POST /mcp</code>. <code>--api</code> active explicitement une API JSON en lecture seule. <code>mw github context &lt;pr&gt;</code> lit explicitement les métadonnées de PR, les checks, les statuts de commit et les reviews via votre session <code>gh</code> : aucun checkout, enregistrement automatique ou synchronisation en arrière-plan.",
  "who.eyebrow": "À qui s'adresse-t-il ?",
  "who.title": "Conçu pour trois façons de travailler.",
  "who.copy":
    "MemoryWhale s'adresse aux développeurs dont le contexte de débogage est dispersé entre le scrollback du terminal, l'historique du shell, plusieurs machines et des sessions d'agents temporaires. Consultez les <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">cas d'usage détaillés</a> avec de vrais transcriptions de commandes.",
  "who.shellTitle": "🔍 Le débogueur centré sur le shell",
  "who.shellBody":
    "Vous rencontrez deux fois la même erreur de build, de linker ou de dépendance. L'historique du shell retient la commande — pas la sortie, la fin de l'erreur ni le correctif. <code>mw search</code> renvoie l'exécution en échec <em>et</em> la leçon qui lui est liée.",
  "who.multiTitle": "🛰️ La personne qui travaille sur plusieurs machines",
  "who.multiBody":
    "Jetson, serveur de labo, ordinateur portable : les sessions se coupent et chaque machine garde un historique privé et incomplet. <code>mw --live</code> enregistre automatiquement malgré les déconnexions ; <code>mw push</code> / <code>mw pull</code> déplacent explicitement la mémoire entre machines.",
  "who.agentTitle": "🤖 L'utilisateur d'agents de code",
  "who.agentBody":
    "Claude Code, Codex, Cursor : chaque session commence par une nouvelle explication de votre environnement. Avec <code>mw-mcp</code>, l'agent peut interroger les preuves précédentes et enregistrer explicitement une leçon avec <code>remember</code>. Vous devez toujours vérifier qu'un correctif fonctionne.",
  "terminal.eyebrow": "Mémoire du terminal",
  "terminal.title": "Un palais de mémoire pour le travail en ligne de commande.",
  "terminal.copy":
    "MemoryWhale stocke les sessions du terminal comme une mémoire locale structurée. Au lieu de conserver un énorme dump de texte, il enregistre la commande, chaque argument, le répertoire de travail, le code de sortie, stdout, stderr et vos propres notes.",
  "terminal.argsTitle": "Les arguments deviennent consultables",
  "terminal.argsBody":
    "Les options comme <code>--manifest-path</code>, les chemins, sous-commandes, noms de paquets et options de modèle sont séparés dans leurs propres lignes.",
  "terminal.errorsTitle": "Les journaux d'erreur restent attachés",
  "terminal.errorsBody":
    "stderr est conservé à côté de la commande qui l'a produit : la cause et le contexte restent réunis.",
  "terminal.liveTitle": "Sauvegarde en direct des sessions",
  "terminal.liveBody":
    "<code>mw --live</code> écrit la transcription du shell actif dans SQLite toutes les quelques secondes ; une déconnexion peut donc laisser une trace mémoire exploitable.",
  "terminal.graphTitle": "Nœuds du graphe pour les échecs",
  "terminal.graphBody":
    "Les commandes en échec apparaissent dans la galaxie de connaissances et se relient à des concepts extraits comme cargo, Tauri, SQLite, ports et builds.",
  "how.eyebrow": "Fonctionnement",
  "how.title": "Capturer, stocker, extraire, explorer.",
  "how.captureTitle": "Capturer",
  "how.captureBody": "Collez une exécution du terminal, appelez l'aide Rust ou démarrez un shell sauvegardé en direct.",
  "how.storeTitle": "Stocker",
  "how.storeBody": "SQLite enregistre localement les exécutions et les arguments des commandes sur votre machine.",
  "how.extractTitle": "Extraire",
  "how.extractBody": "Rust extrait les mots-clés des commandes, des notes et du texte des erreurs.",
  "how.exploreTitle": "Explorer",
  "how.exploreBody": "Recherchez ou cliquez sur les nœuds de commandes dans l'interface de graphe lumineuse.",
  "agents.eyebrow": "Agents IA",
  "agents.title": "Donnez à votre agent la mémoire de ce qui a déjà échoué.",
  "agents.copy":
    "Les sessions d'agents de code peuvent perdre leur contexte et répéter un débogage déjà effectué. <code>mw-mcp</code> est un serveur Model Context Protocol installé sur votre mémoire locale : enregistrez-le une fois et Claude Code, Rho, Codex ou Cursor pourront interroger directement les échecs passés. Faites confiance au client pour les preuves qu'il récupère, y compris au fournisseur de modèles auquel il transmet ce contexte.",
  "agents.clientsLabel": "Clients avec guides d'intégration",
  "agents.matrix": "Plus dans la matrice",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">Guides de configuration pour les clients et outils</a> — la matrice des capacités documente le support MCP, la capture automatique et l'état de vérification de chaque client, y compris les passerelles de modèles comme OpenRouter et CLIProxyAPI.",
  "agents.setupLabel": "Configuration",
  "agents.setupValue": "Une commande",
  "agents.toolsLabel": "Outils",
  "agents.toolsValue": "6 outils MCP locaux : recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "Pas d'agent ?",
  "agents.noAgentValue": "mw context imprime un digest prêt à coller",
  "demo.eyebrow": "Capture → mémoire → recherche",
  "demo.title": "Voyez la boucle centrale avec des données synthétiques.",
  "demo.copy":
    "Capturez une commande, enregistrez l'explication qui l'a corrigée, puis recherchez le stockage local lorsque le même échec revient. MCP fournit la recherche et l'écriture explicite ; il ne capture pas automatiquement l'activité ordinaire du terminal.",
  "demo.handoff":
    "La <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">démo hors ligne de transfert Claude vers Rho</a> importe des fixtures et simule un client Rho contre un vrai MCP. Elle n'exécute pas d'agents en direct et ne lance ni ne vérifie le correctif Cargo de la fixture. Les hooks Rho conservent actuellement les métadonnées d'échec lorsque le texte de commande est absent ; les appels réussis sans texte de commande sont ignorés. Le rappel au début d'une tâche, la recherche d'échecs et la sauvegarde avant compaction restent des décisions d'orchestration du client, pas une automatisation livrée.",
  "demo.imageAlt": "Démo synthétique du terminal et du dashboard MemoryWhale",
  "data.eyebrow": "Vos données",
  "data.title": "Le local par défaut rend les choix visibles.",
  "data.copy":
    "La base de données vit sur votre machine : généralement <code>~/.local/share/MemoryWhale/</code> sous Linux ou <code>~/Library/Application Support/MemoryWhale/</code> sous macOS. Définissez <code>MEMORYWHALE_DATA_DIR</code> pour choisir un autre emplacement.",
  "data.captureLabel": "Contrôles de capture",
  "data.captureValue": "<code>.mwignore</code>, politique de chemins, commandes uniquement",
  "data.redactionLabel": "Caviardage",
  "data.redactionValue": "Aide à traiter les secrets courants ; ce n'est pas une frontière de sécurité",
  "data.sizeLabel": "Limite de taille",
  "data.sizeValue": "Les champs de texte capturés font par défaut 1 MiB et sont tronqués",
  "data.inspectLabel": "Inspecter / supprimer",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "Transfert",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code> ou transfert SSH explicite",
  "data.stewardshipLabel": "Entretien",
  "data.stewardshipValue": "<code>mw memory compact</code> — dry-run d'abord, lignes conservées",
  "security.eyebrow": "Modèle de sécurité",
  "security.title": "Local par défaut, explicite lorsqu'il est partagé.",
  "security.copy":
    "Le CLI, le TUI, le serveur MCP, le dashboard web et le shell de bureau utilisent le stockage local. <code>mw-mcp</code> est un processus stdio local de confiance ; le dashboard se lie à loopback par défaut. Un dashboard hors loopback exige un token et ne devrait être exposé que sur un réseau de confiance. MCP HTTP protégé exige une authentification Bearer ; l'API JSON activée volontairement partage les contrôles d'accès du dashboard. HTTP ne chiffre pas la connexion. Aucune des deux interfaces ne transforme l'accès du client en capture automatique. Consultez le <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">modèle de menace des données locales</a>.",
  "run.eyebrow": "Installer",
  "run.title": "Une ligne. Rust n'est pas nécessaire.",
  "run.copy":
    "Des binaires précompilés sont disponibles pour Linux x86_64/aarch64 et macOS. L'installateur vérifie les fichiers SHA256 publiés lorsqu'une version en fournit ; les anciennes versions peuvent ne pas avoir de checksum. Commencez par une capture explicite, inspectez-la, puis envisagez seulement <code>mw global on</code>. Windows n'est pas une cible native ; WSL peut utiliser la version Linux.",
  "run.tryLabel": "À essayer d'abord",
  "run.tryValue": "<code>mw demo</code> — écrit des données d'exemple dans le stockage choisi",
  "run.prebuiltLabel": "Installation précompilée",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md#install-or-upgrade\">Instructions d'installation épinglées et vérifiées par checksum</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.10.0 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": ".deb sur la page des versions",
  "run.securityLabel": "Sécurité",
  "run.securityValue": "<a href=\"#security\">Lire le modèle</a>",
  "run.verifyLabel": "Vérifier",
  "run.verifyValue": "<code>mw --version</code> · <code>mw doctor</code>",
  "footer.copyright": "Copyright (c) 2026 wuisabel-gif. MemoryWhale - mémoire de terminal et graphe de connaissances Rust/Tauri.",
  "footer.docs": "Documentation",
  "footer.useCases": "Cas d'usage",
  "footer.cli": "Référence CLI",
  "footer.security": "Politique de sécurité",
  "footer.integrations": "Intégrations"
};

const ZH_CN = {
  meta: {
    title: "MemoryWhale — 为你和 AI 智能体提供的终端记忆",
    description:
      "MemoryWhale 将开发证据采集到本地 SQLite，让人和受信任的工具找回过去的失败与经验。本地优先，明确导出和传输。",
    jsonLdDescription:
      "面向开发者和编程智能体的持久化本地调试记忆。将终端证据采集到本地 SQLite，并通过 MCP 提供服务。"
  },
  "nav.label": "页面导航",
  "brand.home": "MemoryWhale 首页",
  "nav.terminal": "终端记忆",
  "nav.how": "工作原理",
  "nav.agents": "AI 智能体",
  "nav.who": "适用对象",
  "nav.install": "安装",
  "nav.docs": "文档",
  "nav.releases": "发布版本",
  "nav.github": "GitHub ↗",
  "nav.delphin": "Delphin ↗",
  "nav.contextgc": "ContextGC ↗",
  "language.label": "语言",
  "language.en": "英语",
  "language.fr": "法语",
  "language.zh-CN": "简体中文",
  "language.zh-TW": "繁体中文",
  "language.ko": "韩语",
  "language.ja": "日语",
  "release.banner": "🐋 v0.10.0 — Agent-Native Memory · 2026 年 9 月 6 日 · 发布说明与升级指南 →",
  "hero.eyebrow": "本地优先的终端记忆",
  "hero.title": "MemoryWhale 记住终端忘记的事。",
  "hero.lead":
    "采集终端证据，将其保存在本地 SQLite 中，再找回真正重要的失败与经验。MemoryWhale 本地优先：不会在你不知情时上传或同步数据。",
  "hero.demoCta": "查看 60 秒演示",
  "hero.installCta": "安装 MemoryWhale",
  "hero.securityCta": "阅读安全模型",
  "hero.memoryChip": "终端记忆运行中",
  "hero.whaleAlt": "在知识图谱节点间游动的发光鲸鱼",
  "release.eyebrow": "0.10.0 新功能",
  "release.title": "共享记忆。来源明确。",
  "release.copy":
    "产品 0.10.0 覆盖 CLI、Web UI 和桌面应用。可复用的 Rust 核心版本为 0.5.0：Rust <code>Memory</code> 字面量现在要求 <code>agent: Option&lt;String&gt;</code>；旧 JSON 仍可通过 serde 默认值读取。",
  "release.connectTitle": "连接 Claude Code 和 Rho",
  "release.connectBody":
    "<code>mw integrate claude</code> 和 <code>mw integrate rho</code> 安装 MCP 访问、采集钩子和技能。<code>mw doctor</code> 会分别检查这些组件。",
  "release.provenanceTitle": "了解证据来自哪里",
  "release.provenanceBody":
    "结构版本 10 将命令智能体保存为 <code>claude</code>、<code>rho</code> 或 <code>NULL</code>，显示为 <code>terminal</code>。智能体身份独立于来源类型。规范化仓库 ID 会归组关联工作树，同时保留每个工作树的路径。",
  "release.interfaceTitle": "选择本地接口",
  "release.interfaceBody":
    "<code>mw-serve</code> 在 <code>POST /mcp</code> 提供 HTTP MCP。<code>--api</code> 可显式启用只读 JSON API。<code>mw github context &lt;pr&gt;</code> 通过你的 <code>gh</code> 登录明确读取 PR 元数据、检查结果、提交状态和评审：不检出代码、不自动保存，也没有后台同步。",
  "who.eyebrow": "适合谁",
  "who.title": "为三种工作方式而构建。",
  "who.copy":
    "MemoryWhale 面向调试上下文散落在终端滚动记录、Shell 历史、多台机器和临时智能体会话中的开发者。查看包含真实命令记录的<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">完整使用场景</a>。",
  "who.shellTitle": "🔍 以 Shell 为中心的调试者",
  "who.shellBody":
    "同一个构建、链接器或依赖错误出现了两次。Shell 历史记得命令，却不记得输出、错误末尾或修复方法。<code>mw search</code> 会返回旧的失败运行，<em>以及</em>与之关联的经验。",
  "who.multiTitle": "🛰️ 多机器工作者",
  "who.multiBody":
    "Jetson、实验室服务器、笔记本——会话可能中断，每台机器都只保留私有且不完整的历史。<code>mw --live</code> 可在断连时自动保存；<code>mw push</code> / <code>mw pull</code> 明确地在机器之间移动记忆。",
  "who.agentTitle": "🤖 编程智能体用户",
  "who.agentBody":
    "Claude Code、Codex、Cursor——每次会话都要重新解释环境。借助 <code>mw-mcp</code>，智能体可以查询过去的证据，并用 <code>remember</code> 明确保存经验。修复是否有效仍需你验证。",
  "terminal.eyebrow": "终端记忆",
  "terminal.title": "命令行工作的记忆宫殿。",
  "terminal.copy":
    "MemoryWhale 将终端会话保存为结构化的本地记忆。它不保留一个巨大的文本转储，而是保存命令、每个参数、工作目录、退出码、stdout、stderr 以及你的笔记。",
  "terminal.argsTitle": "参数变得可搜索",
  "terminal.argsBody":
    "<code>--manifest-path</code> 之类的标志、路径、子命令、包名和模型选项都会拆分到各自的记录中。",
  "terminal.errorsTitle": "错误日志始终关联",
  "terminal.errorsBody": "stderr 会保存在产生它的命令旁边，因此原因和上下文始终在一起。",
  "terminal.liveTitle": "会话实时自动保存",
  "terminal.liveBody":
    "<code>mw --live</code> 每隔几秒将活动 Shell 转录写入 SQLite，因此断线后仍可能留下可用的记忆轨迹。",
  "terminal.graphTitle": "失败的图谱节点",
  "terminal.graphBody":
    "失败的命令会出现在知识星系中，并连接到 cargo、Tauri、SQLite、端口和构建等提取出的概念。",
  "how.eyebrow": "工作原理",
  "how.title": "采集、存储、提取、探索。",
  "how.captureTitle": "采集",
  "how.captureBody": "粘贴一次终端运行，调用 Rust 辅助程序，或启动实时自动保存的 Shell。",
  "how.storeTitle": "存储",
  "how.storeBody": "SQLite 会将命令运行和参数保存在你的机器本地。",
  "how.extractTitle": "提取",
  "how.extractBody": "Rust 从命令、笔记和错误文本中提取关键词。",
  "how.exploreTitle": "探索",
  "how.exploreBody": "在发光的图谱界面中搜索或点击命令节点。",
  "agents.eyebrow": "AI 智能体",
  "agents.title": "让智能体记住已经失败过的事情。",
  "agents.copy":
    "编程智能体会话可能丢失上下文，重复你已经做过的调试。<code>mw-mcp</code> 是运行在本地记忆之上的 Model Context Protocol 服务器——注册一次后，Claude Code、Rho、Codex 或 Cursor 就能直接查询过去的失败。请信任客户端处理它检索到的证据，包括客户端发送上下文的模型提供商。",
  "agents.clientsLabel": "提供集成指南的客户端",
  "agents.matrix": "矩阵中还有更多",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">客户端与工具设置指南</a>——能力矩阵记录每个客户端的 MCP 支持、自动采集和验证状态，也包括 OpenRouter 与 CLIProxyAPI 等模型网关。",
  "agents.setupLabel": "设置",
  "agents.setupValue": "一条命令",
  "agents.toolsLabel": "工具",
  "agents.toolsValue": "6 个本地 MCP 工具：recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "没有智能体？",
  "agents.noAgentValue": "mw context 输出可直接粘贴的摘要",
  "demo.eyebrow": "采集 → 记忆 → 检索",
  "demo.title": "用合成数据查看核心循环。",
  "demo.copy":
    "采集一条命令，保存修复它的解释；同一失败再次出现时，再搜索本地存储。MCP 提供检索和明确写入，但不会自动采集普通终端活动。",
  "demo.handoff":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">离线 Claude 到 Rho 交接演示</a>导入 fixtures，并让模拟的 Rho 客户端连接真实 MCP。它不会运行真实智能体，也不会执行或验证 fixture 中的 Cargo 修复。当前 Rho 钩子在缺少命令文本时仍保留失败元数据；没有命令文本的成功调用会被跳过。任务开始时的自动回忆、失败查找和压缩前保存仍由客户端编排，不是已提供的自动化。",
  "demo.imageAlt": "MemoryWhale 终端与仪表盘合成演示",
  "data.eyebrow": "你的数据",
  "data.title": "本地优先意味着选择透明可见。",
  "data.copy":
    "数据库位于你的机器上：Linux 通常是 <code>~/.local/share/MemoryWhale/</code>，macOS 通常是 <code>~/Library/Application Support/MemoryWhale/</code>。设置 <code>MEMORYWHALE_DATA_DIR</code> 可选择其他位置。",
  "data.captureLabel": "采集控制",
  "data.captureValue": "<code>.mwignore</code>、路径策略、仅命令",
  "data.redactionLabel": "脱敏",
  "data.redactionValue": "有助于处理常见机密；它不是安全边界",
  "data.sizeLabel": "大小限制",
  "data.sizeValue": "采集的文本字段默认限制为 1 MiB，超出部分会截断",
  "data.inspectLabel": "检查 / 删除",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "传输",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code> 或明确的 SSH 传输",
  "data.stewardshipLabel": "维护",
  "data.stewardshipValue": "<code>mw memory compact</code>——先 dry-run，保留记录行",
  "security.eyebrow": "安全模型",
  "security.title": "默认本地，共享时明确授权。",
  "security.copy":
    "CLI、TUI、MCP 服务器、Web 仪表盘和桌面 Shell 都使用本地存储。<code>mw-mcp</code> 是受信任的本地 stdio 进程；仪表盘默认绑定 loopback。非 loopback 的仪表盘需要 token，只应暴露在受信任的网络上。受保护的 HTTP MCP 需要 Bearer 身份验证；选择启用的 JSON API 共享仪表盘访问控制。HTTP 不会加密连接。任何接口都不会让客户端访问自动变成采集。请参阅<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">本地数据威胁模型</a>。",
  "run.eyebrow": "安装",
  "run.title": "一行命令。不需要 Rust。",
  "run.copy":
    "Linux x86_64/aarch64 和 macOS 提供预编译二进制文件。发布版本提供 SHA256 文件时，安装器会验证它们；较旧版本可能没有校验和。先进行一次明确采集并检查结果，然后再考虑 <code>mw global on</code>。Windows 不是原生目标；WSL 可以使用 Linux 构建。",
  "run.tryLabel": "先试试",
  "run.tryValue": "<code>mw demo</code>——将示例数据写入所选存储",
  "run.prebuiltLabel": "预编译安装",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md#install-or-upgrade\">固定版本、已验证校验和的安装说明</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.10.0 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": "发布页面上的 .deb",
  "run.securityLabel": "安全",
  "run.securityValue": "<a href=\"#security\">阅读模型</a>",
  "run.verifyLabel": "验证",
  "run.verifyValue": "<code>mw --version</code> · <code>mw doctor</code>",
  "footer.copyright": "Copyright (c) 2026 wuisabel-gif。MemoryWhale - Rust/Tauri 终端记忆与知识图谱。",
  "footer.docs": "文档",
  "footer.useCases": "使用场景",
  "footer.cli": "CLI 参考",
  "footer.security": "安全政策",
  "footer.integrations": "集成"
};

const ZH_TW = {
  meta: {
    title: "MemoryWhale — 為你與 AI 代理提供的終端機記憶",
    description:
      "MemoryWhale 將開發證據擷取到本機 SQLite，讓人與受信任的工具找回過去的失敗與經驗。本機優先，明確匯出與傳輸。",
    jsonLdDescription:
      "為開發者與程式設計代理提供的持久本機除錯記憶。將終端機證據擷取到本機 SQLite，並透過 MCP 提供服務。"
  },
  "nav.label": "頁面導覽",
  "brand.home": "MemoryWhale 首頁",
  "nav.terminal": "終端機記憶",
  "nav.how": "運作方式",
  "nav.agents": "AI 代理",
  "nav.who": "適合誰",
  "nav.install": "安裝",
  "nav.docs": "文件",
  "nav.releases": "發行版本",
  "nav.github": "GitHub ↗",
  "nav.delphin": "Delphin ↗",
  "nav.contextgc": "ContextGC ↗",
  "language.label": "語言",
  "language.en": "英文",
  "language.fr": "法文",
  "language.zh-CN": "簡體中文",
  "language.zh-TW": "繁體中文",
  "language.ko": "韓文",
  "language.ja": "日文",
  "release.banner": "🐋 v0.10.0 — Agent-Native Memory · 2026 年 9 月 6 日 · 發行說明與升級指南 →",
  "hero.eyebrow": "本機優先的終端機記憶",
  "hero.title": "MemoryWhale 記住終端機忘記的事。",
  "hero.lead":
    "擷取終端機證據，將它保存在本機 SQLite，再找回真正重要的失敗與經驗。MemoryWhale 採本機優先：不會在你不知情時上傳或同步資料。",
  "hero.demoCta": "查看 60 秒示範",
  "hero.installCta": "安裝 MemoryWhale",
  "hero.securityCta": "閱讀安全模型",
  "hero.memoryChip": "終端機記憶運作中",
  "hero.whaleAlt": "在知識圖譜節點間游動的發光鯨魚",
  "release.eyebrow": "0.10.0 新功能",
  "release.title": "共用記憶。來源明確。",
  "release.copy":
    "產品 0.10.0 涵蓋 CLI、Web UI 與桌面應用程式。可重用的 Rust 核心版本為 0.5.0：Rust <code>Memory</code> 字面值現在要求 <code>agent: Option&lt;String&gt;</code>；舊版 JSON 仍可透過 serde 預設值讀取。",
  "release.connectTitle": "連接 Claude Code 與 Rho",
  "release.connectBody":
    "<code>mw integrate claude</code> 與 <code>mw integrate rho</code> 會安裝 MCP 存取、擷取掛鉤與技能。<code>mw doctor</code> 會分別檢查這些元件。",
  "release.provenanceTitle": "了解證據來自何處",
  "release.provenanceBody":
    "結構版本 10 將指令代理儲存為 <code>claude</code>、<code>rho</code> 或 <code>NULL</code>，顯示為 <code>terminal</code>。代理身分獨立於來源類型。標準化儲存庫 ID 會歸組關聯工作樹，同時保留每個工作樹的路徑。",
  "release.interfaceTitle": "選擇本機介面",
  "release.interfaceBody":
    "<code>mw-serve</code> 在 <code>POST /mcp</code> 提供 HTTP MCP。<code>--api</code> 可明確啟用唯讀 JSON API。<code>mw github context &lt;pr&gt;</code> 透過你的 <code>gh</code> 登入明確讀取 PR 中繼資料、檢查結果、提交狀態與審查：不簽出程式碼、不自動儲存，也沒有背景同步。",
  "who.eyebrow": "適合誰",
  "who.title": "為三種工作方式打造。",
  "who.copy":
    "MemoryWhale 適合除錯上下文散落在終端機捲動記錄、Shell 歷史、不同機器與臨時代理工作階段中的開發者。查看包含真實指令記錄的<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">完整使用情境</a>。",
  "who.shellTitle": "🔍 以 Shell 為中心的除錯者",
  "who.shellBody":
    "同一個建置、連結器或相依套件錯誤出現兩次。Shell 歷史記得指令，卻不記得輸出、錯誤尾端或修正方法。<code>mw search</code> 會回傳舊的失敗執行，<em>以及</em>與它連結的經驗。",
  "who.multiTitle": "🛰️ 多機器工作者",
  "who.multiBody":
    "Jetson、實驗室伺服器、筆電——工作階段可能中斷，每台機器都只保留私有且不完整的歷史。<code>mw --live</code> 可在斷線時自動儲存；<code>mw push</code> / <code>mw pull</code> 明確地在機器之間搬移記憶。",
  "who.agentTitle": "🤖 程式設計代理使用者",
  "who.agentBody":
    "Claude Code、Codex、Cursor——每個工作階段都要重新說明你的環境。透過 <code>mw-mcp</code>，代理可以查詢過去的證據，並用 <code>remember</code> 明確儲存經驗。修正是否有效仍需由你驗證。",
  "terminal.eyebrow": "終端機記憶",
  "terminal.title": "命令列工作的記憶宮殿。",
  "terminal.copy":
    "MemoryWhale 將終端機工作階段儲存為結構化的本機記憶。它不保留一大段文字傾印，而是儲存指令、每個引數、工作目錄、結束碼、stdout、stderr 與你的筆記。",
  "terminal.argsTitle": "引數變得可搜尋",
  "terminal.argsBody":
    "像 <code>--manifest-path</code> 這樣的旗標、路徑、子命令、套件名稱與模型選項，都會拆分到各自的記錄中。",
  "terminal.errorsTitle": "錯誤記錄保持關聯",
  "terminal.errorsBody": "stderr 會保存在產生它的指令旁邊，因此原因與上下文始終在一起。",
  "terminal.liveTitle": "工作階段即時自動儲存",
  "terminal.liveBody":
    "<code>mw --live</code> 每隔幾秒將目前的 Shell 轉錄寫入 SQLite，因此斷線後仍可能留下可用的記憶軌跡。",
  "terminal.graphTitle": "失敗的圖譜節點",
  "terminal.graphBody":
    "失敗的指令會出現在知識星系中，並連結到 cargo、Tauri、SQLite、連接埠與建置等擷取出的概念。",
  "how.eyebrow": "運作方式",
  "how.title": "擷取、儲存、提取、探索。",
  "how.captureTitle": "擷取",
  "how.captureBody": "貼上一段終端機執行記錄、呼叫 Rust 輔助程式，或啟動即時自動儲存的 Shell。",
  "how.storeTitle": "儲存",
  "how.storeBody": "SQLite 會將指令執行記錄與引數儲存在你的機器本機。",
  "how.extractTitle": "提取",
  "how.extractBody": "Rust 從指令、筆記與錯誤文字中擷取關鍵字。",
  "how.exploreTitle": "探索",
  "how.exploreBody": "在發光的圖譜介面中搜尋或點擊指令節點。",
  "agents.eyebrow": "AI 代理",
  "agents.title": "讓代理記住已經失敗過的事。",
  "agents.copy":
    "程式設計代理工作階段可能失去上下文，重複你已經做過的除錯。<code>mw-mcp</code> 是建立在本機記憶上的 Model Context Protocol 伺服器——註冊一次後，Claude Code、Rho、Codex 或 Cursor 就能直接查詢過去的失敗。請信任用戶端處理它取回的證據，包括用戶端傳送上下文的模型供應商。",
  "agents.clientsLabel": "提供整合指南的用戶端",
  "agents.matrix": "矩陣中還有更多",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">用戶端與工具設定指南</a>——能力矩陣記錄每個用戶端的 MCP 支援、自動擷取與驗證狀態，也包括 OpenRouter 與 CLIProxyAPI 等模型閘道。",
  "agents.setupLabel": "設定",
  "agents.setupValue": "一個指令",
  "agents.toolsLabel": "工具",
  "agents.toolsValue": "6 個本機 MCP 工具：recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "沒有代理？",
  "agents.noAgentValue": "mw context 輸出可直接貼上的摘要",
  "demo.eyebrow": "擷取 → 記憶 → 檢索",
  "demo.title": "用合成資料查看核心循環。",
  "demo.copy":
    "擷取一個指令，儲存修正它的說明；同一個失敗再次出現時，再搜尋本機儲存。MCP 提供檢索與明確寫入，但不會自動擷取一般終端機活動。",
  "demo.handoff":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">離線 Claude 到 Rho 交接示範</a>會匯入 fixtures，並讓模擬的 Rho 用戶端連接真正的 MCP。它不會執行真正的代理，也不會執行或驗證 fixture 中的 Cargo 修正。目前 Rho 掛鉤在缺少指令文字時仍保留失敗中繼資料；沒有指令文字的成功呼叫會被略過。工作開始時自動回憶、失敗查找與壓縮前儲存仍是用戶端編排工作，不是已提供的自動化。",
  "demo.imageAlt": "MemoryWhale 終端機與儀表板合成示範",
  "data.eyebrow": "你的資料",
  "data.title": "本機優先代表選擇清楚可見。",
  "data.copy":
    "資料庫位於你的機器上：Linux 通常是 <code>~/.local/share/MemoryWhale/</code>，macOS 通常是 <code>~/Library/Application Support/MemoryWhale/</code>。設定 <code>MEMORYWHALE_DATA_DIR</code> 可選擇其他位置。",
  "data.captureLabel": "擷取控制",
  "data.captureValue": "<code>.mwignore</code>、路徑政策、僅指令",
  "data.redactionLabel": "遮蔽",
  "data.redactionValue": "有助於處理常見密碼；它不是安全邊界",
  "data.sizeLabel": "大小限制",
  "data.sizeValue": "擷取的文字欄位預設為 1 MiB，超出部分會截斷",
  "data.inspectLabel": "檢查 / 刪除",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "傳輸",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code> 或明確的 SSH 傳輸",
  "data.stewardshipLabel": "維護",
  "data.stewardshipValue": "<code>mw memory compact</code>——先 dry-run，保留資料列",
  "security.eyebrow": "安全模型",
  "security.title": "預設本機，共用時明確授權。",
  "security.copy":
    "CLI、TUI、MCP 伺服器、Web 儀表板與桌面 Shell 都使用本機儲存。<code>mw-mcp</code> 是受信任的本機 stdio 程序；儀表板預設繫結到 loopback。非 loopback 的儀表板需要 token，只應暴露在受信任的網路上。受保護的 HTTP MCP 需要 Bearer 驗證；選擇啟用的 JSON API 共用儀表板的存取控制。HTTP 不會加密連線。任何介面都不會讓用戶端存取自動變成擷取。請參閱<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">本機資料威脅模型</a>。",
  "run.eyebrow": "安裝",
  "run.title": "一行指令。不需要 Rust。",
  "run.copy":
    "Linux x86_64/aarch64 與 macOS 提供預先編譯的二進位檔。發行版本提供 SHA256 檔案時，安裝程式會驗證它們；較舊版本可能沒有檢查碼。先進行一次明確擷取並檢查結果，再考慮 <code>mw global on</code>。Windows 不是原生目標；WSL 可以使用 Linux 建置。",
  "run.tryLabel": "先試試",
  "run.tryValue": "<code>mw demo</code>——將範例資料寫入選定的儲存區",
  "run.prebuiltLabel": "預先編譯安裝",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md#install-or-upgrade\">固定版本、已驗證檢查碼的安裝說明</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.10.0 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": "發行頁面上的 .deb",
  "run.securityLabel": "安全性",
  "run.securityValue": "<a href=\"#security\">閱讀模型</a>",
  "run.verifyLabel": "驗證",
  "run.verifyValue": "<code>mw --version</code> · <code>mw doctor</code>",
  "footer.copyright": "Copyright (c) 2026 wuisabel-gif。MemoryWhale - Rust/Tauri 終端機記憶與知識圖譜。",
  "footer.docs": "文件",
  "footer.useCases": "使用情境",
  "footer.cli": "CLI 參考",
  "footer.security": "安全政策",
  "footer.integrations": "整合"
};

const KO = {
  meta: {
    title: "MemoryWhale — 나와 AI 에이전트를 위한 터미널 메모리",
    description:
      "MemoryWhale은 개발 증거를 로컬 SQLite에 저장해 사람과 신뢰할 수 있는 도구가 과거의 실패와 교훈을 다시 찾도록 합니다. 로컬 우선이며 내보내기와 전송은 명시적으로 수행합니다.",
    jsonLdDescription:
      "개발자와 코딩 에이전트를 위한 지속적인 로컬 디버깅 메모리입니다. 터미널 증거를 로컬 SQLite에 저장하고 MCP로 제공합니다."
  },
  "nav.label": "페이지 탐색",
  "brand.home": "MemoryWhale 홈",
  "nav.terminal": "터미널 메모리",
  "nav.how": "작동 방식",
  "nav.agents": "AI 에이전트",
  "nav.who": "누구를 위한 도구인가요",
  "nav.install": "설치",
  "nav.docs": "문서",
  "nav.releases": "릴리스",
  "nav.github": "GitHub ↗",
  "nav.delphin": "Delphin ↗",
  "nav.contextgc": "ContextGC ↗",
  "language.label": "언어",
  "language.en": "영어",
  "language.fr": "프랑스어",
  "language.zh-CN": "중국어 간체",
  "language.zh-TW": "중국어 번체",
  "language.ko": "한국어",
  "language.ja": "일본어",
  "release.banner": "🐋 v0.10.0 — Agent-Native Memory · 2026년 9월 6일 · 릴리스 노트와 업그레이드 안내 →",
  "hero.eyebrow": "로컬 우선 터미널 메모리",
  "hero.title": "MemoryWhale은 터미널이 잊는 것을 기억합니다.",
  "hero.lead":
    "터미널 증거를 캡처하고 로컬 SQLite에 보존한 다음, 중요한 실패와 교훈을 다시 찾으세요. MemoryWhale은 로컬 우선입니다. 데이터를 몰래 업로드하거나 동기화하지 않습니다.",
  "hero.demoCta": "60초 데모 보기",
  "hero.installCta": "MemoryWhale 설치",
  "hero.securityCta": "보안 모델 읽기",
  "hero.memoryChip": "터미널 메모리 작동 중",
  "hero.whaleAlt": "지식 그래프 노드 사이를 헤엄치는 빛나는 고래",
  "release.eyebrow": "0.10.0의 새로운 기능",
  "release.title": "공유 메모리. 명확한 출처.",
  "release.copy":
    "제품 0.10.0은 CLI, 웹 UI, 데스크톱 앱을 아우릅니다. 재사용 가능한 Rust 코어는 0.5.0입니다. Rust <code>Memory</code> 리터럴에는 이제 <code>agent: Option&lt;String&gt;</code>이 필요하며, 이전 JSON은 serde 기본값으로 계속 읽을 수 있습니다.",
  "release.connectTitle": "Claude Code와 Rho 연결",
  "release.connectBody":
    "<code>mw integrate claude</code>와 <code>mw integrate rho</code>는 MCP 접근, 캡처 훅, 스킬을 설치합니다. <code>mw doctor</code>는 각 구성 요소를 따로 점검합니다.",
  "release.provenanceTitle": "증거가 어디서 왔는지 확인",
  "release.provenanceBody":
    "스키마 10은 명령을 만든 에이전트를 <code>claude</code>, <code>rho</code> 또는 <code>NULL</code>로 저장하고 <code>terminal</code>로 표시합니다. 에이전트는 소스 유형과 별개입니다. 정규화된 저장소 ID는 연결된 작업 트리를 묶으면서 각 작업 트리의 경로를 보존합니다.",
  "release.interfaceTitle": "로컬 인터페이스 선택",
  "release.interfaceBody":
    "<code>mw-serve</code>는 <code>POST /mcp</code>에서 HTTP MCP를 제공합니다. <code>--api</code>는 읽기 전용 JSON API를 명시적으로 활성화합니다. <code>mw github context &lt;pr&gt;</code>는 <code>gh</code> 로그인으로 PR 메타데이터, 검사 결과, 커밋 상태와 리뷰를 명시적으로 읽습니다. 체크아웃, 자동 저장, 백그라운드 동기화는 없습니다.",
  "who.eyebrow": "누구를 위한 도구인가요",
  "who.title": "세 가지 작업 방식에 맞게 만들었습니다.",
  "who.copy":
    "MemoryWhale은 디버깅 컨텍스트가 터미널 스크롤백, 셸 기록, 여러 머신과 임시 에이전트 세션에 흩어진 개발자를 위한 도구입니다. 실제 명령 기록이 담긴 <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">전체 사용 사례</a>를 확인하세요.",
  "who.shellTitle": "🔍 셸 중심 디버거",
  "who.shellBody":
    "같은 빌드, 링커 또는 의존성 오류를 두 번 만납니다. 셸 기록에는 명령만 남고 출력, 오류 끝부분, 해결책은 남지 않습니다. <code>mw search</code>는 이전 실패 실행과 연결된 교훈을 <em>함께</em> 반환합니다.",
  "who.multiTitle": "🛰️ 여러 머신에서 일하는 작업자",
  "who.multiBody":
    "Jetson, 연구실 서버, 노트북—세션이 끊기면 각 머신에는 불완전하고 사적인 기록만 남습니다. <code>mw --live</code>는 연결이 끊겨도 자동 저장하고, <code>mw push</code> / <code>mw pull</code>은 머신 사이에서 메모리를 명시적으로 옮깁니다.",
  "who.agentTitle": "🤖 코딩 에이전트 사용자",
  "who.agentBody":
    "Claude Code, Codex, Cursor를 쓸 때마다 환경을 다시 설명해야 합니다. <code>mw-mcp</code>를 사용하면 에이전트가 이전 증거를 조회하고 <code>remember</code>로 교훈을 명시적으로 저장할 수 있습니다. 수정이 실제로 작동하는지는 직접 확인해야 합니다.",
  "terminal.eyebrow": "터미널 메모리",
  "terminal.title": "명령줄 작업을 위한 메모리 궁전.",
  "terminal.copy":
    "MemoryWhale은 터미널 세션을 구조화된 로컬 메모리로 저장합니다. 거대한 텍스트 덤프 하나를 보관하는 대신 명령, 모든 인수, 작업 디렉터리, 종료 코드, stdout, stderr와 직접 작성한 메모를 저장합니다.",
  "terminal.argsTitle": "인수를 검색할 수 있습니다",
  "terminal.argsBody":
    "<code>--manifest-path</code> 같은 플래그, 경로, 하위 명령, 패키지 이름과 모델 옵션을 각각의 행으로 나눕니다.",
  "terminal.errorsTitle": "오류 로그가 명령에 연결됩니다",
  "terminal.errorsBody": "stderr는 이를 만든 명령 옆에 보존되므로 원인과 컨텍스트가 함께 남습니다.",
  "terminal.liveTitle": "세션 실시간 자동 저장",
  "terminal.liveBody":
    "<code>mw --live</code>는 몇 초마다 활성 셸 기록을 SQLite에 씁니다. 연결이 끊겨도 사용할 수 있는 메모리 흔적이 남을 수 있습니다.",
  "terminal.graphTitle": "실패를 나타내는 그래프 노드",
  "terminal.graphBody":
    "실패한 명령은 지식 은하에 나타나고 cargo, Tauri, SQLite, 포트, 빌드 같은 추출된 개념과 연결됩니다.",
  "how.eyebrow": "작동 방식",
  "how.title": "캡처하고, 저장하고, 추출하고, 탐색합니다.",
  "how.captureTitle": "캡처",
  "how.captureBody": "터미널 실행을 붙여 넣거나 Rust 도우미를 호출하거나 실시간 자동 저장 셸을 시작하세요.",
  "how.storeTitle": "저장",
  "how.storeBody": "SQLite는 명령 실행과 인수를 사용자의 머신에 로컬로 저장합니다.",
  "how.extractTitle": "추출",
  "how.extractBody": "Rust는 명령, 메모와 오류 텍스트에서 키워드를 추출합니다.",
  "how.exploreTitle": "탐색",
  "how.exploreBody": "빛나는 그래프 인터페이스에서 검색하거나 명령 노드를 클릭하세요.",
  "agents.eyebrow": "AI 에이전트",
  "agents.title": "에이전트가 이미 실패한 일을 기억하게 하세요.",
  "agents.copy":
    "코딩 에이전트 세션은 컨텍스트를 잃고 이미 수행한 디버깅을 반복할 수 있습니다. <code>mw-mcp</code>는 로컬 메모리 위에서 동작하는 Model Context Protocol 서버입니다. 한 번 등록하면 Claude Code, Rho, Codex 또는 Cursor가 과거의 실패를 직접 조회할 수 있습니다. 검색한 증거를 전달받는 모델 제공자를 포함해 클라이언트가 증거를 처리한다는 점을 신뢰해야 합니다.",
  "agents.clientsLabel": "통합 가이드가 있는 클라이언트",
  "agents.matrix": "매트릭스에서 더 보기",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">클라이언트와 도구 설정 가이드</a>—기능 매트릭스에서 각 클라이언트의 MCP 지원, 자동 캡처와 검증 상태를 확인할 수 있으며 OpenRouter와 CLIProxyAPI 같은 모델 게이트웨이도 포함합니다.",
  "agents.setupLabel": "설정",
  "agents.setupValue": "한 명령",
  "agents.toolsLabel": "도구",
  "agents.toolsValue": "6개 로컬 MCP 도구: recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "에이전트가 없나요?",
  "agents.noAgentValue": "mw context가 바로 붙여 넣을 수 있는 요약을 출력합니다",
  "demo.eyebrow": "캡처 → 메모리 → 검색",
  "demo.title": "합성 데이터로 핵심 순환을 확인하세요.",
  "demo.copy":
    "명령 하나를 캡처하고 이를 해결한 설명을 저장한 다음, 같은 실패가 다시 발생하면 로컬 저장소를 검색하세요. MCP는 검색과 명시적 기록을 제공하지만 일반 터미널 활동을 자동으로 캡처하지는 않습니다.",
  "demo.handoff":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">오프라인 Claude-to-Rho 인계 데모</a>는 fixture를 가져와 실제 MCP에 연결된 Rho 클라이언트를 시뮬레이션합니다. 실제 에이전트를 실행하지 않으며 fixture의 Cargo 수정도 실행하거나 검증하지 않습니다. 현재 Rho 훅은 명령 텍스트가 없을 때 실패 메타데이터를 보존하고, 명령 텍스트가 없는 성공 호출은 건너뜁니다. 작업 시작 자동 회상, 실패 조회와 압축 전 저장은 출시된 자동화가 아니라 클라이언트 오케스트레이션의 책임입니다.",
  "demo.imageAlt": "합성 MemoryWhale 터미널 및 대시보드 데모",
  "data.eyebrow": "내 데이터",
  "data.title": "로컬 우선은 선택을 투명하게 만듭니다.",
  "data.copy":
    "데이터베이스는 사용자의 머신에 있습니다. 일반적으로 Linux에서는 <code>~/.local/share/MemoryWhale/</code>, macOS에서는 <code>~/Library/Application Support/MemoryWhale/</code>입니다. <code>MEMORYWHALE_DATA_DIR</code>를 설정해 다른 위치를 선택하세요.",
  "data.captureLabel": "캡처 제어",
  "data.captureValue": "<code>.mwignore</code>, 경로 정책, 명령만",
  "data.redactionLabel": "비식별화",
  "data.redactionValue": "일반적인 비밀값에 도움이 되지만 보안 경계는 아닙니다",
  "data.sizeLabel": "크기 제한",
  "data.sizeValue": "캡처된 텍스트 필드는 기본 1 MiB이며 초과분은 잘립니다",
  "data.inspectLabel": "검사 / 삭제",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "전송",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code> 또는 명시적인 SSH 전송",
  "data.stewardshipLabel": "관리",
  "data.stewardshipValue": "<code>mw memory compact</code>—먼저 dry-run하고 행은 보존합니다",
  "security.eyebrow": "보안 모델",
  "security.title": "기본은 로컬, 공유할 때는 명시적으로.",
  "security.copy":
    "CLI, TUI, MCP 서버, 웹 대시보드와 데스크톱 셸은 로컬 저장소를 사용합니다. <code>mw-mcp</code>는 신뢰하는 로컬 stdio 프로세스이며 대시보드는 기본적으로 loopback에 바인딩됩니다. loopback이 아닌 대시보드에는 token이 필요하고 신뢰할 수 있는 네트워크에만 노출해야 합니다. 보호된 HTTP MCP에는 Bearer 인증이 필요하며, 선택적으로 활성화한 JSON API는 대시보드의 접근 제어를 공유합니다. HTTP는 연결을 암호화하지 않습니다. 어느 인터페이스도 클라이언트 접근을 자동 캡처로 바꾸지 않습니다. <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">로컬 데이터 위협 모델</a>을 확인하세요.",
  "run.eyebrow": "설치",
  "run.title": "한 줄이면 됩니다. Rust는 필요 없습니다.",
  "run.copy":
    "Linux x86_64/aarch64와 macOS용 사전 빌드 바이너리를 제공합니다. 릴리스에 게시된 SHA256 파일이 있으면 설치 프로그램이 검증하며, 이전 릴리스에는 체크섬이 없을 수 있습니다. 먼저 명시적인 캡처 하나를 실행하고 검사한 다음 <code>mw global on</code>을 고려하세요. Windows는 네이티브 대상이 아니며 WSL에서는 Linux 빌드를 사용할 수 있습니다.",
  "run.tryLabel": "먼저 시도",
  "run.tryValue": "<code>mw demo</code>—선택한 저장소에 샘플 데이터를 씁니다",
  "run.prebuiltLabel": "사전 빌드 설치",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md#install-or-upgrade\">고정 버전·체크섬 검증 설치 안내</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.10.0 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": "릴리스 페이지의 .deb",
  "run.securityLabel": "보안",
  "run.securityValue": "<a href=\"#security\">모델 읽기</a>",
  "run.verifyLabel": "확인",
  "run.verifyValue": "<code>mw --version</code> · <code>mw doctor</code>",
  "footer.copyright": "Copyright (c) 2026 wuisabel-gif. MemoryWhale - Rust/Tauri 터미널 메모리 및 지식 그래프.",
  "footer.docs": "문서",
  "footer.useCases": "사용 사례",
  "footer.cli": "CLI 레퍼런스",
  "footer.security": "보안 정책",
  "footer.integrations": "통합"
};

const JA = {
  meta: {
    title: "MemoryWhale — あなたと AI エージェントのためのターミナルメモリ",
    description:
      "MemoryWhale は開発の証拠をローカル SQLite に保存し、人と信頼できるツールが過去の失敗や教訓を見つけられるようにします。ローカル優先で、エクスポートと転送は明示的に行います。",
    jsonLdDescription:
      "開発者とコーディングエージェントのための永続的なローカルデバッグメモリです。ターミナルの証拠をローカル SQLite に保存し、MCP 経由で提供します。"
  },
  "nav.label": "ページナビゲーション",
  "brand.home": "MemoryWhale ホーム",
  "nav.terminal": "ターミナルメモリ",
  "nav.how": "仕組み",
  "nav.agents": "AI エージェント",
  "nav.who": "対象ユーザー",
  "nav.install": "インストール",
  "nav.docs": "ドキュメント",
  "nav.releases": "リリース",
  "nav.github": "GitHub ↗",
  "nav.delphin": "Delphin ↗",
  "nav.contextgc": "ContextGC ↗",
  "language.label": "言語",
  "language.en": "英語",
  "language.fr": "フランス語",
  "language.zh-CN": "簡体字中国語",
  "language.zh-TW": "繁体字中国語",
  "language.ko": "韓国語",
  "language.ja": "日本語",
  "release.banner": "🐋 v0.10.0 — Agent-Native Memory · 2026年9月6日 · リリースノートとアップグレードガイド →",
  "hero.eyebrow": "ローカル優先のターミナルメモリ",
  "hero.title": "MemoryWhale はターミナルが忘れることを覚えています。",
  "hero.lead":
    "ターミナルの証拠を取得してローカル SQLite に保存し、重要な失敗や教訓を後から見つけます。MemoryWhale はローカル優先です。データを知らないうちにアップロードしたり同期したりしません。",
  "hero.demoCta": "60秒のデモを見る",
  "hero.installCta": "MemoryWhale をインストール",
  "hero.securityCta": "セキュリティモデルを読む",
  "hero.memoryChip": "ターミナルメモリ稼働中",
  "hero.whaleAlt": "知識グラフのノード間を泳ぐ光るクジラ",
  "release.eyebrow": "0.10.0 の新機能",
  "release.title": "共有メモリ。明確な由来。",
  "release.copy":
    "製品 0.10.0 は CLI、Web UI、デスクトップアプリにまたがります。再利用可能な Rust コアは 0.5.0 です。Rust の <code>Memory</code> リテラルには <code>agent: Option&lt;String&gt;</code> が必要になり、以前の JSON は serde のデフォルトによって引き続き読み取れます。",
  "release.connectTitle": "Claude Code と Rho を接続",
  "release.connectBody":
    "<code>mw integrate claude</code> と <code>mw integrate rho</code> は MCP アクセス、キャプチャフック、スキルをインストールします。<code>mw doctor</code> は各コンポーネントを個別に確認します。",
  "release.provenanceTitle": "証拠の出所を把握",
  "release.provenanceBody":
    "スキーマ 10 はコマンドのエージェントを <code>claude</code>、<code>rho</code>、または <code>NULL</code> として保存し、<code>terminal</code> と表示します。エージェントはソース種別とは別です。正規化されたリポジトリ ID はリンクされたワークツリーをまとめながら、それぞれのパスを失いません。",
  "release.interfaceTitle": "ローカルインターフェースを選ぶ",
  "release.interfaceBody":
    "<code>mw-serve</code> は <code>POST /mcp</code> で HTTP MCP を提供します。<code>--api</code> で読み取り専用 JSON API を明示的に有効化できます。<code>mw github context &lt;pr&gt;</code> は既存の <code>gh</code> ログインを通じて PR メタデータ、チェック、コミットステータス、レビューを明示的に読み取ります。チェックアウト、自動保存、バックグラウンド同期はありません。",
  "who.eyebrow": "対象ユーザー",
  "who.title": "3つの働き方のために作られています。",
  "who.copy":
    "MemoryWhale は、デバッグのコンテキストがターミナルのスクロールバック、シェル履歴、複数のマシン、一時的なエージェントセッションに散らばっている開発者のためのものです。実際のコマンド記録を含む<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">ユースケース全体</a>をご覧ください。",
  "who.shellTitle": "🔍 シェル中心のデバッガー",
  "who.shellBody":
    "同じビルド、リンカー、依存関係のエラーに二度遭遇します。シェル履歴が覚えているのはコマンドだけで、出力、エラーの末尾、修正方法は残りません。<code>mw search</code> は以前の失敗した実行と、それに紐づく教訓を<em>両方</em>返します。",
  "who.multiTitle": "🛰️ 複数マシンで働く人",
  "who.multiBody":
    "Jetson、ラボサーバー、ノートパソコン。セッションが切れると、各マシンには不完全で個別の履歴しか残りません。<code>mw --live</code> は切断中も自動保存し、<code>mw push</code> / <code>mw pull</code> はマシン間でメモリを明示的に移動します。",
  "who.agentTitle": "🤖 コーディングエージェントの利用者",
  "who.agentBody":
    "Claude Code、Codex、Cursor では、セッションごとに環境を説明し直すことになります。<code>mw-mcp</code> を使えば、エージェントは過去の証拠を照会し、<code>remember</code> で教訓を明示的に保存できます。修正が機能することは自分で確認する必要があります。",
  "terminal.eyebrow": "ターミナルメモリ",
  "terminal.title": "コマンドライン作業のための記憶の宮殿。",
  "terminal.copy":
    "MemoryWhale はターミナルセッションを構造化されたローカルメモリとして保存します。巨大なテキストダンプを1つ残す代わりに、コマンド、すべての引数、作業ディレクトリ、終了コード、stdout、stderr、そして自分のメモを保存します。",
  "terminal.argsTitle": "引数を検索できる",
  "terminal.argsBody":
    "<code>--manifest-path</code> のようなフラグ、パス、サブコマンド、パッケージ名、モデルオプションをそれぞれの行に分けます。",
  "terminal.errorsTitle": "エラーログがコマンドに紐づく",
  "terminal.errorsBody": "stderr はそれを生成したコマンドの隣に保存されるため、原因とコンテキストが一緒に残ります。",
  "terminal.liveTitle": "セッションのライブ自動保存",
  "terminal.liveBody":
    "<code>mw --live</code> は数秒ごとにアクティブなシェルの記録を SQLite に書き込みます。切断されても利用できるメモリの軌跡が残ります。",
  "terminal.graphTitle": "失敗を表すグラフノード",
  "terminal.graphBody":
    "失敗したコマンドは知識の銀河に現れ、cargo、Tauri、SQLite、ポート、ビルドなど抽出された概念とつながります。",
  "how.eyebrow": "仕組み",
  "how.title": "取得、保存、抽出、探索。",
  "how.captureTitle": "取得",
  "how.captureBody": "ターミナルの実行結果を貼り付けるか、Rust ヘルパーを呼び出すか、ライブ自動保存シェルを開始します。",
  "how.storeTitle": "保存",
  "how.storeBody": "SQLite はコマンドの実行と引数をマシン上のローカルに保存します。",
  "how.extractTitle": "抽出",
  "how.extractBody": "Rust はコマンド、メモ、エラーテキストからキーワードを抽出します。",
  "how.exploreTitle": "探索",
  "how.exploreBody": "光るグラフインターフェースで検索するか、コマンドノードをクリックします。",
  "agents.eyebrow": "AI エージェント",
  "agents.title": "すでに失敗したことをエージェントに覚えさせる。",
  "agents.copy":
    "コーディングエージェントのセッションはコンテキストを失い、すでに行ったデバッグを繰り返すことがあります。<code>mw-mcp</code> はローカルメモリ上で動作する Model Context Protocol サーバーです。一度登録すれば、Claude Code、Rho、Codex、Cursor が過去の失敗を直接照会できます。取得した証拠を送るモデルプロバイダーを含め、そのクライアントが証拠を扱うことを信頼してください。",
  "agents.clientsLabel": "連携ガイドのあるクライアント",
  "agents.matrix": "マトリクスでもっと見る",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">クライアントとツールのセットアップガイド</a> — 機能マトリクスでは、各クライアントの MCP 対応、自動キャプチャ、検証状況を、OpenRouter や CLIProxyAPI などのモデルゲートウェイも含めて確認できます。",
  "agents.setupLabel": "セットアップ",
  "agents.setupValue": "1つのコマンド",
  "agents.toolsLabel": "ツール",
  "agents.toolsValue": "6つのローカル MCP ツール: recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "エージェントがない場合は？",
  "agents.noAgentValue": "mw context が貼り付け可能なダイジェストを出力します",
  "demo.eyebrow": "取得 → メモリ → 検索",
  "demo.title": "合成データでコアのループを見る。",
  "demo.copy":
    "1つのコマンドを取得し、それを直した説明を保存します。同じ失敗が戻ってきたらローカルストアを検索できます。MCP は検索と明示的な書き込みを提供しますが、通常のターミナル操作を自動取得しません。",
  "demo.handoff":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">オフラインの Claude から Rho への引き継ぎデモ</a>は fixture を取り込み、実際の MCP に対して Rho クライアントをシミュレートします。実際のエージェントを実行したり、fixture の Cargo 修正を実行・検証したりはしません。現在の Rho フックはコマンドテキストがない場合も失敗メタデータを保持し、コマンドテキストなしの成功呼び出しはスキップします。タスク開始時の自動想起、失敗検索、コンパクション前の保存は、提供済みの自動化ではなくクライアントのオーケストレーションです。",
  "demo.imageAlt": "MemoryWhale のターミナルとダッシュボードの合成デモ",
  "data.eyebrow": "あなたのデータ",
  "data.title": "ローカル優先なら選択が見える。",
  "data.copy":
    "データベースはマシン上にあります。通常、Linux では <code>~/.local/share/MemoryWhale/</code>、macOS では <code>~/Library/Application Support/MemoryWhale/</code> です。<code>MEMORYWHALE_DATA_DIR</code> を設定して別の場所を選べます。",
  "data.captureLabel": "取得の制御",
  "data.captureValue": "<code>.mwignore</code>、パスのポリシー、コマンドのみ",
  "data.redactionLabel": "編集・秘匿",
  "data.redactionValue": "一般的な秘密情報には役立ちますが、セキュリティ境界ではありません",
  "data.sizeLabel": "サイズ制限",
  "data.sizeValue": "取得するテキストフィールドはデフォルトで 1 MiB、超過分は切り詰められます",
  "data.inspectLabel": "確認 / 削除",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "転送",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code> または明示的な SSH 転送",
  "data.stewardshipLabel": "管理",
  "data.stewardshipValue": "<code>mw memory compact</code> — まず dry-run、行は保持",
  "security.eyebrow": "セキュリティモデル",
  "security.title": "デフォルトはローカル、共有時は明示的に。",
  "security.copy":
    "CLI、TUI、MCP サーバー、Web ダッシュボード、デスクトップシェルはローカルストアを使います。<code>mw-mcp</code> は信頼されたローカル stdio プロセスで、ダッシュボードはデフォルトで loopback にバインドされます。非 loopback のダッシュボードには token が必要で、信頼できるネットワークだけに公開してください。保護された HTTP MCP には Bearer 認証が必要です。オプトインの JSON API はダッシュボードのアクセス制御を共有します。HTTP は接続を暗号化しません。どちらのインターフェースもクライアントアクセスを自動キャプチャにはしません。<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">ローカルデータの脅威モデル</a>をご覧ください。",
  "run.eyebrow": "インストール",
  "run.title": "1行だけ。Rust は不要です。",
  "run.copy":
    "Linux x86_64/aarch64 と macOS 向けにビルド済みバイナリを提供しています。リリースに公開 SHA256 ファイルがある場合、インストーラーが検証します。古いリリースにはチェックサムがないことがあります。まず明示的な取得を1つ行って確認し、その後で <code>mw global on</code> を検討してください。Windows はネイティブ対象ではありませんが、WSL では Linux ビルドを使えます。",
  "run.tryLabel": "まず試す",
  "run.tryValue": "<code>mw demo</code> — 選択したストアにサンプルデータを書き込みます",
  "run.prebuiltLabel": "ビルド済みインストール",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md#install-or-upgrade\">固定版・チェックサム検証済みのインストール手順</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.10.0 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": "リリースページの .deb",
  "run.securityLabel": "セキュリティ",
  "run.securityValue": "<a href=\"#security\">モデルを読む</a>",
  "run.verifyLabel": "確認",
  "run.verifyValue": "<code>mw --version</code> · <code>mw doctor</code>",
  "footer.copyright": "Copyright (c) 2026 wuisabel-gif. MemoryWhale - Rust/Tauri ターミナルメモリと知識グラフ。",
  "footer.docs": "ドキュメント",
  "footer.useCases": "ユースケース",
  "footer.cli": "CLI リファレンス",
  "footer.security": "セキュリティポリシー",
  "footer.integrations": "連携"
};

const translations = {
  en: EN,
  fr: FR,
  "zh-CN": ZH_CN,
  "zh-TW": ZH_TW,
  ko: KO,
  ja: JA
};
const supportedLanguages = Object.freeze(["en", "fr", "zh-CN", "zh-TW", "ko", "ja"]);

globalThis.MEMORYWHALE_I18N = Object.freeze({ supportedLanguages, translations });

(() => {
  if (typeof document === "undefined") return;

  const storageKey = "memorywhale.language";
  const languageSelect = document.getElementById("language-select");
  const languageSet = new Set(supportedLanguages);

  const normalizeLanguage = (value) => {
    if (typeof value !== "string") return null;
    const normalized = value.trim().replace(/_/g, "-").toLowerCase();
    if (normalized === "zh-hans" || normalized.startsWith("zh-hans-")) return "zh-CN";
    if (normalized === "zh-hant" || normalized.startsWith("zh-hant-")) return "zh-TW";
    if (normalized === "zh-tw" || normalized === "zh-hk" || normalized === "zh-mo") return "zh-TW";
    if (normalized === "zh-cn" || normalized === "zh-sg" || normalized === "zh") return "zh-CN";
    const primary = normalized.split("-")[0];
    return ["en", "fr", "ko", "ja"].includes(primary) ? primary : null;
  };

  const readStoredLanguage = () => {
    try {
      return normalizeLanguage(window.localStorage.getItem(storageKey));
    } catch {
      return null;
    }
  };

  const browserLanguage = () => {
    const candidates = Array.isArray(navigator.languages) && navigator.languages.length
      ? navigator.languages
      : [navigator.language];
    for (const candidate of candidates) {
      const language = normalizeLanguage(candidate);
      if (language && languageSet.has(language)) return language;
    }
    return "en";
  };

  const queryLanguage = () => {
    try {
      const language = normalizeLanguage(new URL(window.location.href).searchParams.get("lang"));
      return language && languageSet.has(language) ? language : null;
    } catch {
      return null;
    }
  };

  const applyJsonLd = (dictionary, language) => {
    const element = document.querySelector('script[type="application/ld+json"]');
    if (!element) return;
    try {
      const structuredData = JSON.parse(element.textContent);
      structuredData.description = dictionary.meta.jsonLdDescription;
      structuredData.inLanguage = language;
      element.textContent = JSON.stringify(structuredData);
    } catch {
      // Keep the inline English JSON-LD intact if a future edit makes it invalid.
    }
  };

  const updateUrlLanguage = (language) => {
    try {
      const url = new URL(window.location.href);
      url.searchParams.set("lang", language);
      window.history.replaceState(window.history.state, "", `${url.pathname}${url.search}${url.hash}`);
    } catch {
      // The page still changes language when History API access is unavailable.
    }
  };

  const applyLanguage = (language, { persist = false, updateUrl = false } = {}) => {
    const selectedLanguage = languageSet.has(language) ? language : "en";
    const dictionary = translations[selectedLanguage] || translations.en;
    document.documentElement.lang = selectedLanguage;
    document.title = dictionary.meta.title;
    const description = document.querySelector('meta[name="description"]');
    if (description) description.setAttribute("content", dictionary.meta.description);
    applyJsonLd(dictionary, selectedLanguage);

    document.querySelectorAll("[data-i18n]").forEach((element) => {
      const value = dictionary[element.dataset.i18n];
      if (typeof value === "string") element.innerHTML = value;
    });
    document.querySelectorAll("[data-i18n-aria-label]").forEach((element) => {
      const value = dictionary[element.dataset.i18nAriaLabel];
      if (typeof value === "string") element.setAttribute("aria-label", value);
    });
    document.querySelectorAll("[data-i18n-alt]").forEach((element) => {
      const value = dictionary[element.dataset.i18nAlt];
      if (typeof value === "string") element.setAttribute("alt", value);
    });
    if (languageSelect) languageSelect.value = selectedLanguage;

    if (persist) {
      try {
        window.localStorage.setItem(storageKey, selectedLanguage);
      } catch {
        // Private browsing and blocked storage must not break language selection.
      }
    }
    if (updateUrl) updateUrlLanguage(selectedLanguage);
  };

  const initialLanguage = queryLanguage() || readStoredLanguage() || browserLanguage();
  applyLanguage(initialLanguage);
  languageSelect?.addEventListener("change", (event) => {
    const selectedLanguage = normalizeLanguage(event.target.value) || "en";
    applyLanguage(selectedLanguage, { persist: true, updateUrl: true });
  });
})();
