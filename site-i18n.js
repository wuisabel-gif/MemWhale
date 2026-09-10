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
  "nav.terminal": "Features",
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
  "language.ar": "العربية",
  "language.de": "Deutsch",
  "language.fr": "Français",
  "language.zh-CN": "简体中文",
  "language.zh-TW": "繁體中文",
  "language.ko": "한국어",
  "language.ja": "日本語",
  "search.openLabel": "Search this page",
  "search.title": "Find on this page",
  "search.label": "Search homepage",
  "search.placeholder": "Try MCP, install, privacy…",
  "search.close": "Close search",
  "search.hint": "Search headings and content on this page.",
  "search.empty": "No matching sections.",
  "search.results": "Matching sections",
  "hero.releaseBadge": "v0.10.0 is here",
  "hero.title": "Make your terminal <span class=\"hero-accent\">remember</span>.",
  "hero.lead":
    "Keep useful terminal context locally. Let coding agents recover debugging knowledge from previous sessions.",
  "hero.demoCta": "View Demo",
  "hero.installCta": "Quick Start",
  "hero.localTitle": "Local-first",
  "hero.localBody": "Memory stays on your machine",
  "hero.privateTitle": "Private",
  "hero.privateBody": "No cloud required",
  "hero.agentTitle": "Built for coding agents",
  "hero.agentBody": "Debugging knowledge survives sessions",
  "hero.terminalTitle": "Connect your coding agent",
  "hero.terminalNote": "Setup example with Cargo and Claude Code; output shortened.",
  "hero.whaleAlt": "MemoryWhale whale mascot",
  "integrations.label": "Works with",
  "integrations.via": "via MCP",
  "integrations.more": "All integrations",
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
  "features.label": "A memory layer for terminal work",
  "features.terminalTitle": "Terminal Memory",
  "features.terminalBody": "Capture useful debugging context while you work.",
  "features.retrievalTitle": "Intelligent Retrieval",
  "features.retrievalBody": "Recover previous commands, failures, and fixes when they matter.",
  "features.localTitle": "Completely Local",
  "features.localBody": "Keep MemoryWhale data on your own machine.",
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

const AR = {
  meta: {
    title: "MemoryWhale — ذاكرة الطرفية لك ولوكيل الذكاء الاصطناعي الخاص بك",
    description:
      "يلتقط MemoryWhale شواهد العمل التطويري في قاعدة SQLite محلية، لتمكين الأشخاص والأدوات الموثوقة من استرجاع الإخفاقات السابقة والدروس المستفادة. يعطي الأولوية للتخزين المحلي، مع تصدير ونقل بطلب صريح.",
    jsonLdDescription:
      "ذاكرة محلية دائمة لتصحيح الأخطاء، للمطورين ووكلاء البرمجة. تلتقط شواهد الطرفية في قاعدة SQLite محلية وتتيحها عبر MCP."
  },
  "nav.label": "التنقل في الصفحة",
  "brand.home": "الصفحة الرئيسية لـ MemoryWhale",
  "nav.terminal": "الميزات",
  "nav.how": "كيف يعمل",
  "nav.agents": "وكلاء الذكاء الاصطناعي",
  "nav.who": "لمن صُمّم",
  "nav.install": "التثبيت",
  "nav.docs": "التوثيق",
  "nav.releases": "الإصدارات",
  "nav.github": "GitHub ↗",
  "nav.delphin": "Delphin ↗",
  "nav.contextgc": "ContextGC ↗",
  "language.label": "اللغة",
  "language.en": "الإنجليزية",
  "language.ar": "العربية",
  "language.de": "Deutsch",
  "language.fr": "الفرنسية",
  "language.zh-CN": "الصينية المبسطة",
  "language.zh-TW": "الصينية التقليدية",
  "language.ko": "الكورية",
  "language.ja": "اليابانية",
  "search.openLabel": "ابحث في هذه الصفحة",
  "search.title": "البحث في هذه الصفحة",
  "search.label": "البحث في الصفحة الرئيسية",
  "search.placeholder": "جرّب MCP أو التثبيت أو الخصوصية…",
  "search.close": "إغلاق البحث",
  "search.hint": "ابحث في عناوين هذه الصفحة ومحتواها.",
  "search.empty": "لا توجد أقسام مطابقة.",
  "search.results": "الأقسام المطابقة",
  "hero.releaseBadge": "الإصدار v0.10.0 متاح الآن",
  "hero.title": "اجعل طرفيتك <span class=\"hero-accent\">تتذكّر</span>.",
  "hero.lead":
    "احفظ سياق الطرفية المفيد محليًا. دع وكلاء البرمجة يسترجعون خبرات تصحيح الأخطاء من جلسات سابقة.",
  "hero.demoCta": "شاهد العرض التجريبي",
  "hero.installCta": "البدء السريع",
  "hero.localTitle": "محلي أولًا",
  "hero.localBody": "تبقى الذاكرة على جهازك",
  "hero.privateTitle": "خصوصية",
  "hero.privateBody": "لا حاجة إلى السحابة",
  "hero.agentTitle": "مصمّم لوكلاء البرمجة",
  "hero.agentBody": "خبرات تصحيح الأخطاء تبقى بين الجلسات",
  "hero.terminalTitle": "اربط وكيل البرمجة",
  "hero.terminalNote": "مثال إعداد باستخدام Cargo وClaude Code؛ المخرجات مختصرة.",
  "hero.whaleAlt": "حوت MemoryWhale، تميمة المشروع",
  "integrations.label": "يعمل مع",
  "integrations.via": "عبر MCP",
  "integrations.more": "جميع التكاملات",
  "release.eyebrow": "الجديد في 0.10.0",
  "release.title": "ذاكرة مشتركة. مصادر واضحة.",
  "release.copy":
    "يشمل إصدار المنتج 0.10.0 واجهة سطر الأوامر (CLI)، وواجهة الويب، وتطبيق سطح المكتب. أما النواة القابلة لإعادة الاستخدام بلغة Rust فإصدارها 0.5.0: تتطلب القيم الحرفية لـ <code>Memory</code> في Rust الآن <code>agent: Option&lt;String&gt;</code>؛ وتظل بيانات JSON القديمة قابلة للقراءة بفضل القيمة الافتراضية في serde.",
  "release.connectTitle": "اربط Claude Code وRho",
  "release.connectBody":
    "يثبّت كلٌّ من <code>mw integrate claude</code> و<code>mw integrate rho</code> إمكانية الوصول عبر MCP، وخطافات الالتقاط، ومهارة. يفحص <code>mw doctor</code> هذه المكوّنات كلًّا على حدة.",
  "release.provenanceTitle": "اعرف مصدر الشواهد",
  "release.provenanceBody":
    "يخزّن المخطط 10 هوية الوكيل لكل أمر بالقيم <code>claude</code> أو <code>rho</code> أو <code>NULL</code>؛ وتُعرض القيمة الأخيرة باسم <code>terminal</code>. هوية الوكيل مستقلة عن نوع المصدر. وتجمع معرّفات المستودعات الموحّدة أشجار العمل المرتبطة (worktrees) دون فقدان مسار أيٍّ منها.",
  "release.interfaceTitle": "اختر واجهتك المحلية",
  "release.interfaceBody":
    "يضيف <code>mw-serve</code> دعم HTTP MCP على <code>POST /mcp</code>. يفعّل <code>--api</code> واجهة JSON API للقراءة فقط باختيار صريح. يقرأ <code>mw github context &lt;pr&gt;</code> صراحةً البيانات الوصفية لطلبات السحب (PR)، والفحوص، وحالات الإيداعات، والمراجعات باستخدام تسجيل دخولك إلى <code>gh</code>: دون إجراء checkout أو حفظ تلقائي أو مزامنة في الخلفية.",
  "who.eyebrow": "لمن صُمّم",
  "who.title": "مصمّم لثلاث طرق للعمل.",
  "who.copy":
    "يخدم MemoryWhale المطورين الذين يتوزّع سياق تصحيح الأخطاء لديهم بين سجل مخرجات الطرفية، وسجل الصدفة، والأجهزة، وجلسات الوكلاء المؤقتة. اطّلع على <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">الشروحات التفصيلية لحالات الاستخدام</a> مع سجلات أوامر حقيقية.",
  "who.shellTitle": "🔍 من يصحّح الأخطاء عبر سطر الأوامر",
  "who.shellBody":
    "واجهت خطأ البناء أو الربط أو الاعتماديات نفسه مرتين. يحتفظ سجل الصدفة بالأمر — لا بالمخرجات، أو نهاية رسالة الخطأ، أو الحل. يعيد <code>mw search</code> التنفيذ السابق الذي أخفق <em>والدرس</em> المرتبط به.",
  "who.multiTitle": "🛰️ من يعمل على أجهزة متعددة",
  "who.multiBody":
    "Jetson، وخادم المختبر، والحاسوب المحمول — تنقطع الجلسات ويحتفظ كل جهاز بسجل خاص وغير مكتمل. يحفظ <code>mw --live</code> تلقائيًا رغم انقطاع الاتصال؛ وينقل <code>mw push</code> / <code>mw pull</code> الذاكرة بين الأجهزة بطلب صريح.",
  "who.agentTitle": "🤖 مستخدم وكلاء البرمجة",
  "who.agentBody":
    "<bdi dir=\"ltr\">Claude Code, Codex, Cursor</bdi> — تبدأ كل جلسة بشرح بيئتك من جديد. مع <code>mw-mcp</code>، يستطيع الوكيل الاستعلام عن الشواهد السابقة وحفظ درس صراحةً باستخدام <code>remember</code>. لا يزال عليك التحقق من نجاح أي إصلاح.",
  "features.label": "طبقة ذاكرة للعمل في الطرفية",
  "features.terminalTitle": "ذاكرة الطرفية",
  "features.terminalBody": "التقط سياق تصحيح الأخطاء المفيد أثناء عملك.",
  "features.retrievalTitle": "استرجاع ذكي",
  "features.retrievalBody": "استرجع الأوامر والإخفاقات والحلول السابقة عند الحاجة.",
  "features.localTitle": "محلي بالكامل",
  "features.localBody": "احتفظ ببيانات MemoryWhale على جهازك.",
  "how.eyebrow": "كيف يعمل",
  "how.title": "التقط، خزّن، استخرج، استكشف.",
  "how.captureTitle": "التقاط",
  "how.captureBody": "ألصق سجل تنفيذ من الطرفية، أو استدعِ أداة Rust المساعدة، أو ابدأ جلسة صدفة تُحفظ تلقائيًا باستمرار.",
  "how.storeTitle": "تخزين",
  "how.storeBody": "يحفظ SQLite عمليات تنفيذ الأوامر ووسيطاتها محليًا على جهازك.",
  "how.extractTitle": "استخراج",
  "how.extractBody": "يستخرج Rust الكلمات المفتاحية من الأوامر والملاحظات ونصوص الأخطاء.",
  "how.exploreTitle": "استكشاف",
  "how.exploreBody": "ابحث أو انقر عُقد الأوامر في واجهة الرسم البياني المضيئة.",
  "agents.eyebrow": "وكلاء الذكاء الاصطناعي",
  "agents.title": "امنح وكيلك ذاكرة لما أخفق من قبل.",
  "agents.copy":
    "قد تفقد جلسات وكلاء البرمجة السياق وتكرّر تصحيح أخطاء سبق أن عالجتها. <code>mw-mcp</code> خادم Model Context Protocol يتيح الوصول إلى ذاكرتك المحلية — سجّله مرة واحدة ليتمكن <bdi dir=\"ltr\">Claude Code, Rho, Codex, Cursor</bdi> من الاستعلام مباشرةً عن الإخفاقات السابقة. يجب أن تثق بالعميل الذي يطّلع على الشواهد المسترجعة، وكذلك بأي مزوّد نموذج يرسل إليه ذلك العميل السياق.",
  "agents.clientsLabel": "عملاء تتوفر لهم أدلة تكامل",
  "agents.matrix": "المزيد في جدول الإمكانات",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">أدلة إعداد العملاء والأدوات</a> — يوضّح جدول الإمكانات دعم MCP والالتقاط التلقائي وحالة التحقق لكل عميل، بما في ذلك بوابات النماذج مثل OpenRouter وCLIProxyAPI.",
  "agents.setupLabel": "الإعداد",
  "agents.setupValue": "أمر واحد",
  "agents.toolsLabel": "الأدوات",
  "agents.toolsValue": "6 أدوات MCP محلية: <bdi dir=\"ltr\">recent_errors · search_memory · get_context · remember · similar_failures · stats</bdi>",
  "agents.noAgentLabel": "لا تستخدم وكيلًا؟",
  "agents.noAgentValue": "يطبع <bdi dir=\"ltr\">mw context</bdi> ملخصًا جاهزًا للصق",
  "demo.eyebrow": "التقاط ← ذاكرة ← استرجاع",
  "demo.title": "شاهد دورة العمل الأساسية ببيانات اصطناعية.",
  "demo.copy":
    "التقط أمرًا واحدًا، واحفظ الشرح الذي أدى إلى إصلاحه، ثم ابحث في المخزن المحلي عندما يتكرر الإخفاق نفسه. يوفّر MCP الاسترجاع والكتابة الصريحة؛ ولا يلتقط نشاط الطرفية العادي تلقائيًا.",
  "demo.handoff":
    "يستورد <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">العرض التجريبي دون اتصال لتسليم السياق من Claude إلى Rho</a> بيانات اختبار جاهزة ويحاكي عميل Rho باستخدام MCP حقيقي. لا يشغّل وكلاء فعليين، ولا ينفّذ إصلاح Cargo الوارد في بيانات الاختبار أو يتحقق منه. تحتفظ خطافات Rho حاليًا بالبيانات الوصفية للإخفاق عند غياب نص الأمر؛ وتتجاوز الاستدعاءات الناجحة التي لا تحتوي على نص الأمر. ولا يزال الاسترجاع التلقائي عند بدء المهمة، والبحث عن الإخفاقات، والحفظ قبل ضغط السياق مهامًا على مستوى تنسيق العميل، وليست أتمتة متاحة ضمن المنتج.",
  "demo.imageAlt": "عرض تجريبي لطرفية MemoryWhale ولوحة التحكم ببيانات اصطناعية",
  "data.eyebrow": "بياناتك",
  "data.title": "الأولوية للتخزين المحلي تعني خيارات واضحة.",
  "data.copy":
    "توجد قاعدة البيانات على جهازك: عادةً في <code>~/.local/share/MemoryWhale/</code> على Linux أو <code>~/Library/Application Support/MemoryWhale/</code> على macOS. اضبط <code>MEMORYWHALE_DATA_DIR</code> لاختيار موقع آخر.",
  "data.captureLabel": "ضوابط الالتقاط",
  "data.captureValue": "<code>.mwignore</code>، سياسة المسارات، التقاط الأوامر فقط",
  "data.redactionLabel": "حجب البيانات الحساسة",
  "data.redactionValue": "يساعد على حجب الأسرار الشائعة؛ لكنه ليس حدًا أمنيًا",
  "data.sizeLabel": "حد الحجم",
  "data.sizeValue": "الحد الافتراضي للحقول النصية الملتقطة هو 1 MiB، مع اقتطاع ما يتجاوزه",
  "data.inspectLabel": "فحص / حذف",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "النقل",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code> أو نقل صريح عبر SSH",
  "data.stewardshipLabel": "إدارة الذاكرة",
  "data.stewardshipValue": "<code>mw memory compact</code> — معاينة دون تنفيذ أولًا، مع الحفاظ على الصفوف",
  "security.eyebrow": "نموذج الأمان",
  "security.title": "محلي افتراضيًا، وبقرار صريح عند المشاركة.",
  "security.copy":
    "تستخدم واجهة سطر الأوامر (CLI)، والواجهة النصية (TUI)، وخادم MCP، ولوحة الويب، وواجهة سطح المكتب المخزن المحلي. يعمل <code>mw-mcp</code> كعملية محلية موثوقة عبر stdio؛ وترتبط لوحة الويب افتراضيًا بعنوان loopback. تتطلب إتاحة اللوحة على عنوان غير loopback رمز وصول، وينبغي قصر إتاحتها على شبكة موثوقة. يتطلب HTTP MCP المحمي مصادقة Bearer؛ وتشترك واجهة JSON API، التي تُفعّل باختيار صريح، في ضوابط الوصول الخاصة بلوحة الويب. لا يشفّر HTTP الاتصال. ولا تحوّل أيٌّ من الواجهتين وصول العميل إلى التقاط تلقائي. راجع <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">نموذج تهديدات البيانات المحلية</a>.",
  "run.eyebrow": "التثبيت",
  "run.title": "سطر واحد. لا حاجة إلى Rust.",
  "run.copy":
    "تتوفر ملفات تنفيذية جاهزة لأنظمة Linux x86_64/aarch64 وmacOS. يتحقق برنامج التثبيت من ملفات SHA256 المنشورة عندما يوفّرها الإصدار؛ وقد لا تتوفر قيمة تحقق للإصدارات الأقدم. ابدأ بالتقاط واحد بطلب صريح، وافحصه، ثم فكّر في <code>mw global on</code>. لا يُعد Windows هدفًا مدعومًا بصورة أصلية؛ ويمكن لـ WSL استخدام نسخة Linux.",
  "run.tryLabel": "جرّب أولًا",
  "run.tryValue": "<code>mw demo</code> — يكتب بيانات نموذجية في المخزن المحدد",
  "run.prebuiltLabel": "تثبيت ملفات تنفيذية جاهزة",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md#install-or-upgrade\">تعليمات تثبيت لإصدار محدد مع التحقق من المجموع الاختباري</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.10.0 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": ".deb على صفحة الإصدارات",
  "run.securityLabel": "الأمان",
  "run.securityValue": "<a href=\"#security\">اقرأ النموذج</a>",
  "run.verifyLabel": "التحقق",
  "run.verifyValue": "<code>mw --version</code> · <code>mw doctor</code>",
  "footer.copyright": "حقوق النشر (c) 2026 wuisabel-gif. MemoryWhale - ذاكرة طرفية ورسم بياني معرفي باستخدام Rust/Tauri.",
  "footer.docs": "التوثيق",
  "footer.useCases": "حالات الاستخدام",
  "footer.cli": "مرجع CLI",
  "footer.security": "سياسة الأمان",
  "footer.integrations": "التكاملات"
};

const DE = {
  meta: {
    title: "MemoryWhale — Terminal-Gedächtnis für dich und deinen KI-Agenten",
    description:
      "MemoryWhale erfasst Belege aus der Entwicklungsarbeit in einer lokalen SQLite-Datenbank, damit Menschen und vertrauenswürdige Werkzeuge frühere Fehler und Erkenntnisse abrufen können. Lokale Speicherung hat Vorrang; Export und Übertragung erfolgen auf ausdrücklichen Wunsch.",
    jsonLdDescription:
      "Dauerhaftes lokales Debugging-Gedächtnis für Entwickler und Programmieragenten. Erfasst Belege aus dem Terminal in einer lokalen SQLite-Datenbank und stellt sie über MCP bereit."
  },
  "nav.label": "Seitennavigation",
  "brand.home": "MemoryWhale-Startseite",
  "nav.terminal": "Funktionen",
  "nav.how": "Funktionsweise",
  "nav.agents": "KI-Agenten",
  "nav.who": "Für wen?",
  "nav.install": "Installieren",
  "nav.docs": "Dokumentation",
  "nav.releases": "Versionen",
  "nav.github": "GitHub ↗",
  "nav.delphin": "Delphin ↗",
  "nav.contextgc": "ContextGC ↗",
  "language.label": "Sprache",
  "language.en": "Englisch",
  "language.ar": "العربية",
  "language.de": "Deutsch",
  "language.fr": "Französisch",
  "language.zh-CN": "Chinesisch (vereinfacht)",
  "language.zh-TW": "Chinesisch (traditionell)",
  "language.ko": "Koreanisch",
  "language.ja": "Japanisch",
  "search.openLabel": "Diese Seite durchsuchen",
  "search.title": "Auf dieser Seite suchen",
  "search.label": "Startseite durchsuchen",
  "search.placeholder": "Zum Beispiel MCP, Installation, Datenschutz…",
  "search.close": "Suche schließen",
  "search.hint": "Durchsuche Überschriften und Inhalte dieser Seite.",
  "search.empty": "Keine passenden Abschnitte.",
  "search.results": "Passende Abschnitte",
  "hero.releaseBadge": "v0.10.0 ist da",
  "hero.title": "Gib deinem Terminal ein <span class=\"hero-accent\">Gedächtnis</span>.",
  "hero.lead":
    "Bewahre nützlichen Terminal-Kontext lokal auf. Lass Programmieragenten Wissen zur Fehlersuche aus früheren Sitzungen abrufen.",
  "hero.demoCta": "Demo ansehen",
  "hero.installCta": "Schnellstart",
  "hero.localTitle": "Lokal zuerst",
  "hero.localBody": "Das Gedächtnis bleibt auf deinem Rechner",
  "hero.privateTitle": "Privat",
  "hero.privateBody": "Keine Cloud nötig",
  "hero.agentTitle": "Für Programmieragenten",
  "hero.agentBody": "Wissen zur Fehlersuche überdauert Sitzungen",
  "hero.terminalTitle": "Verbinde deinen Programmieragenten",
  "hero.terminalNote": "Einrichtung mit Cargo und Claude Code als Beispiel; Ausgabe gekürzt.",
  "hero.whaleAlt": "Das Wal-Maskottchen von MemoryWhale",
  "integrations.label": "Kompatibel mit",
  "integrations.via": "über MCP",
  "integrations.more": "Alle Integrationen",
  "release.eyebrow": "Neu in 0.10.0",
  "release.title": "Geteiltes Gedächtnis. Klare Herkunft.",
  "release.copy":
    "Der Produktstand 0.10.0 umfasst CLI, Weboberfläche und Desktop-App. Der wiederverwendbare Rust-Kern hat die Version 0.5.0: Rust-<code>Memory</code>-Literale erfordern jetzt <code>agent: Option&lt;String&gt;</code>; älteres JSON bleibt dank des serde-Standardwerts lesbar.",
  "release.connectTitle": "Claude Code und Rho anbinden",
  "release.connectBody":
    "<code>mw integrate claude</code> und <code>mw integrate rho</code> installieren MCP-Zugriff, Erfassungs-Hooks und einen Skill. <code>mw doctor</code> prüft diese Komponenten unabhängig voneinander.",
  "release.provenanceTitle": "Wissen, woher die Belege stammen",
  "release.provenanceBody":
    "Schema 10 speichert die Agenten von Befehlen als <code>claude</code>, <code>rho</code> oder <code>NULL</code>; Letzteres wird als <code>terminal</code> angezeigt. Der Agent ist vom Quelltyp getrennt. Kanonische Repository-IDs fassen verknüpfte Worktrees zusammen, ohne den Pfad des einzelnen Worktrees zu verlieren.",
  "release.interfaceTitle": "Wähle deine lokale Oberfläche",
  "release.interfaceBody":
    "<code>mw-serve</code> ergänzt HTTP MCP unter <code>POST /mcp</code>. <code>--api</code> aktiviert ausdrücklich eine schreibgeschützte JSON-API. <code>mw github context &lt;pr&gt;</code> liest über deine <code>gh</code>-Anmeldung gezielt PR-Metadaten, Prüfungen, Commit-Status und Reviews: kein Checkout, kein automatisches Speichern und keine Hintergrundsynchronisierung.",
  "who.eyebrow": "Für wen?",
  "who.title": "Für drei Arbeitsweisen entwickelt.",
  "who.copy":
    "MemoryWhale ist für Entwickler gedacht, deren Debugging-Kontext über den Terminal-Rücklauf, den Shell-Verlauf, verschiedene Rechner und kurzlebige Agentensitzungen verstreut ist. Die ausführlichen <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">Anwendungsbeispiele</a> zeigen echte Befehlsprotokolle.",
  "who.shellTitle": "🔍 Fehlersuche vor allem in der Shell",
  "who.shellBody":
    "Derselbe Build-, Linker- oder Abhängigkeitsfehler tritt zum zweiten Mal auf. Der Shell-Verlauf merkt sich den Befehl — nicht seine Ausgabe, das Ende der Fehlermeldung oder die Lösung. <code>mw search</code> liefert den früheren fehlgeschlagenen Lauf <em>und</em> die zugehörige Erkenntnis.",
  "who.multiTitle": "🛰️ Arbeit auf mehreren Rechnern",
  "who.multiBody":
    "Jetson, Laborserver, Laptop — Sitzungen brechen ab, und jeder Rechner behält seinen eigenen, unvollständigen Verlauf. <code>mw --live</code> speichert auch bei Verbindungsabbrüchen automatisch; <code>mw push</code> / <code>mw pull</code> übertragen das Gedächtnis gezielt zwischen Rechnern.",
  "who.agentTitle": "🤖 Entwicklung mit Programmieragenten",
  "who.agentBody":
    "Claude Code, Codex, Cursor — jede Sitzung beginnt damit, deine Umgebung erneut zu erklären. Mit <code>mw-mcp</code> kann der Agent frühere Belege abfragen und mit <code>remember</code> ausdrücklich eine Erkenntnis speichern. Ob eine Fehlerbehebung funktioniert, musst du weiterhin selbst überprüfen.",
  "features.label": "Ein Gedächtnis für die Terminalarbeit",
  "features.terminalTitle": "Terminal-Gedächtnis",
  "features.terminalBody": "Erfasse nützlichen Kontext zur Fehlersuche während der Arbeit.",
  "features.retrievalTitle": "Intelligente Suche",
  "features.retrievalBody": "Finde frühere Befehle, Fehler und Lösungen, wenn du sie brauchst.",
  "features.localTitle": "Vollständig lokal",
  "features.localBody": "Behalte MemoryWhale-Daten auf deinem eigenen Rechner.",
  "how.eyebrow": "Funktionsweise",
  "how.title": "Erfassen, speichern, extrahieren, erkunden.",
  "how.captureTitle": "Erfassen",
  "how.captureBody": "Füge ein Terminalprotokoll ein, rufe das Rust-Hilfsprogramm auf oder starte eine Shell mit laufender automatischer Speicherung.",
  "how.storeTitle": "Speichern",
  "how.storeBody": "SQLite speichert Befehlsausführungen und Argumente lokal auf deinem Rechner.",
  "how.extractTitle": "Extrahieren",
  "how.extractBody": "Rust extrahiert Schlüsselwörter aus Befehlen, Notizen und Fehlertexten.",
  "how.exploreTitle": "Erkunden",
  "how.exploreBody": "Suche oder klicke auf Befehlsknoten in der leuchtenden Graphoberfläche.",
  "agents.eyebrow": "KI-Agenten",
  "agents.title": "Gib deinem Agenten ein Gedächtnis für frühere Fehler.",
  "agents.copy":
    "Sitzungen mit Programmieragenten können Kontext verlieren und bereits erledigte Fehlersuche wiederholen. <code>mw-mcp</code> ist ein Model Context Protocol-Server für dein lokales Gedächtnis — registriere ihn einmal, damit Claude Code, Rho, Codex oder Cursor frühere Fehler direkt abfragen können. Du musst dem Client die abgerufenen Belege anvertrauen können — ebenso jedem Modellanbieter, an den er Kontext sendet.",
  "agents.clientsLabel": "Clients mit Integrationsanleitungen",
  "agents.matrix": "Mehr in der Übersicht",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">Einrichtungsanleitungen für Clients und Werkzeuge</a> — die Funktionsmatrix dokumentiert MCP-Unterstützung, automatische Erfassung und Prüfstatus pro Client, einschließlich Modell-Gateways wie OpenRouter und CLIProxyAPI.",
  "agents.setupLabel": "Einrichtung",
  "agents.setupValue": "Ein Befehl",
  "agents.toolsLabel": "Werkzeuge",
  "agents.toolsValue": "6 lokale MCP-Werkzeuge: recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "Kein Agent?",
  "agents.noAgentValue": "mw context gibt eine Übersicht zum direkten Einfügen aus",
  "demo.eyebrow": "Erfassung → Gedächtnis → Abruf",
  "demo.title": "Der Kernablauf mit synthetischen Daten.",
  "demo.copy":
    "Erfasse einen Befehl, speichere die Erklärung zur funktionierenden Lösung und durchsuche den lokalen Speicher, wenn derselbe Fehler wieder auftritt. MCP ermöglicht das Abrufen und ausdrücklich angestoßene Schreiben von Daten; normale Terminalaktivität erfasst es nicht automatisch.",
  "demo.handoff":
    "Die <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">Offline-Demo zur Übergabe von Claude an Rho</a> importiert Testdaten und simuliert einen Rho-Client mit echtem MCP. Sie startet keine echten Agenten und führt die in den Testdaten enthaltene Cargo-Fehlerbehebung weder aus noch überprüft sie diese. Rho-Hooks bewahren derzeit Fehlermetadaten auf, wenn der Befehlstext fehlt; erfolgreiche Aufrufe ohne Befehlstext werden übersprungen. Automatisches Abrufen zu Aufgabenbeginn, Nachschlagen von Fehlern und Speichern vor der Kontextverdichtung bleiben Aufgaben der Client-Orchestrierung und sind keine mitgelieferte Automatisierung.",
  "demo.imageAlt": "MemoryWhale-Demo von Terminal und Dashboard mit synthetischen Daten",
  "data.eyebrow": "Deine Daten",
  "data.title": "Lokale Speicherung hat Vorrang — mit transparenten Optionen.",
  "data.copy":
    "Die Datenbank liegt auf deinem Rechner: unter Linux meist in <code>~/.local/share/MemoryWhale/</code>, unter macOS in <code>~/Library/Application Support/MemoryWhale/</code>. Mit <code>MEMORYWHALE_DATA_DIR</code> wählst du einen anderen Speicherort.",
  "data.captureLabel": "Erfassung steuern",
  "data.captureValue": "<code>.mwignore</code>, Pfadrichtlinie, nur Befehle",
  "data.redactionLabel": "Schwärzung",
  "data.redactionValue": "Hilft beim Schwärzen üblicher Geheimnisse; ist keine Sicherheitsgrenze",
  "data.sizeLabel": "Größenlimit",
  "data.sizeValue": "Erfasste Textfelder sind standardmäßig auf 1 MiB begrenzt; Überlängen werden abgeschnitten",
  "data.inspectLabel": "Prüfen / löschen",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "Übertragung",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code> oder gezielte SSH-Übertragung",
  "data.stewardshipLabel": "Gedächtnispflege",
  "data.stewardshipValue": "<code>mw memory compact</code> — zuerst ein Probelauf, Zeilen bleiben erhalten",
  "security.eyebrow": "Sicherheitsmodell",
  "security.title": "Standardmäßig lokal, bewusst geteilt.",
  "security.copy":
    "CLI, TUI, MCP-Server, Web-Dashboard und Desktop-Hülle nutzen den lokalen Speicher. <code>mw-mcp</code> ist ein vertrauenswürdiger lokaler stdio-Prozess; das Dashboard bindet sich standardmäßig an Loopback. Ein Dashboard außerhalb von Loopback benötigt ein Token und sollte nur in einem vertrauenswürdigen Netzwerk erreichbar sein. Geschütztes HTTP MCP erfordert Bearer-Authentifizierung; die ausdrücklich aktivierte JSON-API nutzt dieselben Zugriffskontrollen wie das Dashboard. HTTP verschlüsselt die Verbindung nicht. Keine der beiden Schnittstellen macht den Clientzugriff zu einer automatischen Erfassung. Siehe das <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">Bedrohungsmodell für lokale Daten</a>.",
  "run.eyebrow": "Installieren",
  "run.title": "Eine Zeile. Kein Rust nötig.",
  "run.copy":
    "Vorkompilierte Programme sind für Linux x86_64/aarch64 und macOS verfügbar. Das Installationsprogramm prüft die veröffentlichten SHA256-Dateien, sofern die jeweilige Version sie bereitstellt; bei älteren Versionen kann eine Prüfsumme fehlen. Beginne mit einer gezielten Erfassung, prüfe sie und ziehe erst danach <code>mw global on</code> in Betracht. Windows wird nicht nativ unterstützt; WSL kann den Linux-Build verwenden.",
  "run.tryLabel": "Zuerst ausprobieren",
  "run.tryValue": "<code>mw demo</code> — schreibt Beispieldaten in den ausgewählten Speicher",
  "run.prebuiltLabel": "Vorkompiliert installieren",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md#install-or-upgrade\">Anleitung zum versionsgebundenen Installer mit Prüfsummenprüfung</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.10.0 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": ".deb auf der Versionsseite",
  "run.securityLabel": "Sicherheit",
  "run.securityValue": "<a href=\"#security\">Modell lesen</a>",
  "run.verifyLabel": "Überprüfen",
  "run.verifyValue": "<code>mw --version</code> · <code>mw doctor</code>",
  "footer.copyright": "Urheberrecht (c) 2026 wuisabel-gif. MemoryWhale - Terminal-Gedächtnis und Wissensgraph mit Rust/Tauri.",
  "footer.docs": "Dokumentation",
  "footer.useCases": "Anwendungsfälle",
  "footer.cli": "CLI-Referenz",
  "footer.security": "Sicherheitsrichtlinie",
  "footer.integrations": "Integrationen"
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
  "nav.terminal": "Fonctionnalités",
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
  "language.ar": "العربية",
  "language.de": "Deutsch",
  "language.fr": "Français",
  "language.zh-CN": "Chinois simplifié",
  "language.zh-TW": "Chinois traditionnel",
  "language.ko": "Coréen",
  "language.ja": "Japonais",
  "search.openLabel": "Rechercher sur cette page",
  "search.title": "Trouver sur cette page",
  "search.label": "Rechercher sur la page d'accueil",
  "search.placeholder": "Essayez MCP, installation, confidentialité…",
  "search.close": "Fermer la recherche",
  "search.hint": "Recherchez dans les titres et le contenu de cette page.",
  "search.empty": "Aucune section correspondante.",
  "search.results": "Sections correspondantes",
  "hero.releaseBadge": "La v0.10.0 est là",
  "hero.title": "Donnez à votre terminal une <span class=\"hero-accent\">mémoire</span>.",
  "hero.lead":
    "Gardez le contexte utile du terminal en local. Vos agents de code retrouvent les acquis du débogage des sessions précédentes.",
  "hero.demoCta": "Voir la démo",
  "hero.installCta": "Démarrage rapide",
  "hero.localTitle": "Priorité au local",
  "hero.localBody": "La mémoire reste sur votre machine",
  "hero.privateTitle": "Confidentiel",
  "hero.privateBody": "Pas besoin de cloud",
  "hero.agentTitle": "Conçu pour les agents de code",
  "hero.agentBody": "Les acquis du débogage survivent aux sessions",
  "hero.terminalTitle": "Connectez votre agent de code",
  "hero.terminalNote": "Exemple de configuration avec Cargo et Claude Code ; sortie abrégée.",
  "hero.whaleAlt": "La mascotte baleine de MemoryWhale",
  "integrations.label": "Compatible avec",
  "integrations.via": "par MCP",
  "integrations.more": "Toutes les intégrations",
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
  "features.label": "Une mémoire pour le travail au terminal",
  "features.terminalTitle": "Mémoire du terminal",
  "features.terminalBody": "Capturez le contexte utile du débogage pendant votre travail.",
  "features.retrievalTitle": "Recherche intelligente",
  "features.retrievalBody": "Retrouvez commandes, échecs et correctifs passés au bon moment.",
  "features.localTitle": "Entièrement local",
  "features.localBody": "Gardez les données de MemoryWhale sur votre propre machine.",
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
    title: "MemoryWhale — 你和 AI 编程助手的终端记忆",
    description:
      "MemoryWhale 将开发过程中的命令、输出等记录保存在本地 SQLite，方便你和可信工具查找以往的错误与调试经验。数据默认留在本机，由你主动导出或传输。",
    jsonLdDescription:
      "为开发者和 AI 编程助手长期保存调试经验。终端记录存入本地 SQLite，可通过 MCP 检索。"
  },
  "nav.label": "页面导航",
  "brand.home": "MemoryWhale 首页",
  "nav.terminal": "功能",
  "nav.how": "工作原理",
  "nav.agents": "AI 编程助手",
  "nav.who": "适用对象",
  "nav.install": "安装",
  "nav.docs": "文档",
  "nav.releases": "发布版本",
  "nav.github": "GitHub ↗",
  "nav.delphin": "Delphin ↗",
  "nav.contextgc": "ContextGC ↗",
  "language.label": "语言",
  "language.en": "English",
  "language.ar": "العربية",
  "language.de": "Deutsch",
  "language.fr": "Français",
  "language.zh-CN": "简体中文",
  "language.zh-TW": "繁體中文",
  "language.ko": "한국어",
  "language.ja": "日本語",
  "search.openLabel": "搜索本页",
  "search.title": "在本页查找",
  "search.label": "搜索首页",
  "search.placeholder": "试试 MCP、安装、隐私…",
  "search.close": "关闭搜索",
  "search.hint": "搜索本页的标题和内容。",
  "search.empty": "没有找到相关内容。",
  "search.results": "相关内容",
  "hero.releaseBadge": "v0.10.0 现已发布",
  "hero.title": "让终端<span class=\"hero-accent\">记住</span>调试经验。",
  "hero.lead":
    "在本地保存有用的命令、输出和调试背景，让 AI 编程助手在新会话中也能接着上次的经验工作。",
  "hero.demoCta": "查看演示",
  "hero.installCta": "快速上手",
  "hero.localTitle": "本地优先",
  "hero.localBody": "记忆留在你的设备上",
  "hero.privateTitle": "隐私保护",
  "hero.privateBody": "无需云服务",
  "hero.agentTitle": "支持 AI 编程助手",
  "hero.agentBody": "换个会话，经验不丢",
  "hero.terminalTitle": "接入你的 AI 编程助手",
  "hero.terminalNote": "使用 Cargo 和 Claude Code 的配置示例；输出已精简。",
  "hero.whaleAlt": "MemoryWhale 鲸鱼吉祥物",
  "integrations.label": "兼容",
  "integrations.via": "通过 MCP",
  "integrations.more": "所有集成",
  "release.eyebrow": "0.10.0 新功能",
  "release.title": "记忆可共享，来源可追溯。",
  "release.copy":
    "CLI、Web UI 和桌面应用均已更新至 0.10.0，可复用的 Rust 核心库版本为 0.5.0。在 Rust 中以结构体字面量创建 <code>Memory</code> 时，现在必须提供 <code>agent: Option&lt;String&gt;</code> 字段；serde 会为旧 JSON 中缺失的字段填入默认值，因此旧数据仍可读取。",
  "release.connectTitle": "连接 Claude Code 和 Rho",
  "release.connectBody":
    "<code>mw integrate claude</code> 和 <code>mw integrate rho</code> 会配置 MCP 连接，并安装采集钩子和技能文件。<code>mw doctor</code> 会逐项检查这些组件。",
  "release.provenanceTitle": "了解证据来自哪里",
  "release.provenanceBody":
    "数据库结构版本 10 用独立字段记录命令由哪个编程助手执行，取值为 <code>claude</code>、<code>rho</code> 或 <code>NULL</code>；只有空值显示为 <code>terminal</code>。这一身份字段与记录的来源类型分开保存。统一的仓库 ID 将关联的工作树归到同一仓库下，同时保留各工作树的路径。",
  "release.interfaceTitle": "选择本地接口",
  "release.interfaceBody":
    "<code>mw-serve</code> 通过 <code>POST /mcp</code> 提供 HTTP MCP 接口。只读 JSON API 需用 <code>--api</code> 主动开启。运行 <code>mw github context &lt;pr&gt;</code> 时，才会使用你的 <code>gh</code> 登录身份读取 PR 元数据、检查结果、提交状态和评审信息；不会检出代码、自动保存或在后台同步。",
  "who.eyebrow": "适合谁",
  "who.title": "三种开发日常，都用得上。",
  "who.copy":
    "调试线索散落在终端输出、Shell 历史、不同机器和临时的 AI 会话里？MemoryWhale 帮你把它们留存下来。查看附有真实命令记录的<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">完整使用场景</a>。",
  "who.shellTitle": "🔍 经常在终端里调试",
  "who.shellBody":
    "又遇到相同的构建、链接器或依赖错误？Shell 历史只有命令，没有当时的输出、末尾报错和解决方法。<code>mw search</code> 能找回上次失败的执行记录，<em>以及</em>关联的调试经验。",
  "who.multiTitle": "🛰️ 在多台机器之间切换",
  "who.multiBody":
    "Jetson、实验室服务器、笔记本各留着一部分历史，连接一断，线索也容易丢。<code>mw --live</code> 持续自动保存会话，减少断连造成的记录丢失；需要跨机器使用时，由你运行 <code>mw push</code> / <code>mw pull</code> 传输记忆。",
  "who.agentTitle": "🤖 使用 AI 编程助手",
  "who.agentBody":
    "用 Claude Code、Codex、Cursor 时，不必每次都从头说明环境。接入 <code>mw-mcp</code> 后，编程助手可以查询以往的记录，并主动调用 <code>remember</code> 保存经验。修复是否有效，仍需要你实际验证。",
  "features.label": "为终端工作保留记忆",
  "features.terminalTitle": "终端记忆",
  "features.terminalBody": "在开发过程中记录有用的命令、输出和调试背景。",
  "features.retrievalTitle": "智能检索",
  "features.retrievalBody": "需要时找回以往的命令、失败记录和修复方法。",
  "features.localTitle": "本地存储",
  "features.localBody": "将 MemoryWhale 数据保存在你自己的设备上。",
  "how.eyebrow": "工作原理",
  "how.title": "记录下来，存好，再按需找回。",
  "how.captureTitle": "采集",
  "how.captureBody": "粘贴一段终端执行记录、调用 Rust 辅助程序，或启动持续自动保存的 Shell 会话。",
  "how.storeTitle": "存储",
  "how.storeBody": "命令执行记录和参数都存入本机的 SQLite 数据库。",
  "how.extractTitle": "提取",
  "how.extractBody": "Rust 程序从命令、备注和报错文本中提取关键词。",
  "how.exploreTitle": "探索",
  "how.exploreBody": "搜索历史记录，或在图谱中点击命令节点查看详情。",
  "agents.eyebrow": "AI 编程助手",
  "agents.title": "踩过的坑，让编程助手也记住。",
  "agents.copy":
    "AI 编程助手换个会话就可能丢失上下文，把你做过的调试再做一遍。<code>mw-mcp</code> 通过 Model Context Protocol 提供本地记忆访问；配置后，Claude Code、Rho、Codex 或 Cursor 就能查询以往的失败记录。接入前，请确认你信任客户端处理这些记录；如果客户端会把上下文发送给模型提供商，也需要信任该提供商处理相应数据。本地存储并不意味着客户端取出的数据不会离开本机。",
  "agents.clientsLabel": "提供集成指南的客户端",
  "agents.matrix": "查看完整支持对照表",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">客户端与工具配置指南</a>中的支持对照表逐项列出 MCP 支持、自动采集能力及验证状态，也涵盖 OpenRouter 和 CLIProxyAPI 等模型网关。",
  "agents.setupLabel": "设置",
  "agents.setupValue": "一条命令",
  "agents.toolsLabel": "工具",
  "agents.toolsValue": "6 个本地 MCP 工具：recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "不用 AI 编程助手？",
  "agents.noAgentValue": "mw context 输出可直接粘贴的摘要",
  "demo.eyebrow": "采集 → 记忆 → 检索",
  "demo.title": "用模拟数据走一遍记录与检索流程。",
  "demo.copy":
    "记录一次命令执行，再保存解决问题的方法。下次遇到相同错误，就能从本地记录中查找。MCP 支持检索和主动写入，但仅接入 MCP 不会自动采集日常终端操作。",
  "demo.handoff":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">Claude 向 Rho 移交记录的离线演示</a>会导入预设测试数据（fixtures），用模拟的 Rho 客户端连接实际运行的 MCP 服务。演示不运行真实的编程助手，也不执行或验证测试数据中的 Cargo 修复方法。目前，Rho 钩子遇到缺少命令文本的失败调用时会保留失败元数据；同样缺少命令文本的成功调用则会跳过。任务开始时自动检索历史、出错时查找相关记录，以及上下文压缩前保存记忆，都仍需客户端自行编排，尚不是开箱即用的自动化功能。",
  "demo.imageAlt": "使用模拟数据展示的 MemoryWhale 终端与仪表盘",
  "data.eyebrow": "你的数据",
  "data.title": "数据留在本地，采集和传输由你决定。",
  "data.copy":
    "数据库位于你的机器上：Linux 通常是 <code>~/.local/share/MemoryWhale/</code>，macOS 通常是 <code>~/Library/Application Support/MemoryWhale/</code>。设置 <code>MEMORYWHALE_DATA_DIR</code> 可选择其他位置。",
  "data.captureLabel": "采集控制",
  "data.captureValue": "<code>.mwignore</code>、路径规则、仅记录命令模式",
  "data.redactionLabel": "脱敏",
  "data.redactionValue": "可遮蔽常见敏感信息，但不能作为安全边界",
  "data.sizeLabel": "大小限制",
  "data.sizeValue": "采集的文本字段默认限制为 1 MiB，超出部分会截断",
  "data.inspectLabel": "检查 / 删除",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "传输",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code>，或由你主动发起 SSH 传输",
  "data.stewardshipLabel": "维护",
  "data.stewardshipValue": "<code>mw memory compact</code>——先预览操作（dry-run），保留数据库记录",
  "security.eyebrow": "安全模型",
  "security.title": "默认本地，共享时明确授权。",
  "security.copy":
    "CLI、TUI、MCP 服务器、Web 仪表盘和桌面外壳都使用本地存储。<code>mw-mcp</code> 是通过标准输入输出（stdio）通信的本地进程，使用前需要确认其可信。仪表盘默认只监听本机回环地址（loopback）；监听非回环地址时必须设置访问令牌，且只应向可信网络开放。受保护的 HTTP MCP 需要 Bearer 身份验证；主动开启的 JSON API 沿用仪表盘的访问控制。HTTP 本身不加密连接。客户端接入这两种接口，都不等于开启自动采集。详见<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">本地数据威胁模型</a>。",
  "run.eyebrow": "安装",
  "run.title": "一行命令。不需要 Rust。",
  "run.copy":
    "提供适用于 Linux x86_64/aarch64 和 macOS 的预编译二进制文件。如果发布版本附带 SHA256 校验文件，安装程序会据此校验下载内容；旧版本可能没有校验和。建议先手动采集一次并检查结果，再考虑开启 <code>mw global on</code>。目前不提供 Windows 原生版本；在 WSL 中可使用 Linux 版本。",
  "run.tryLabel": "先试试",
  "run.tryValue": "<code>mw demo</code>——将示例数据写入所选存储",
  "run.prebuiltLabel": "预编译安装",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md#install-or-upgrade\">指定版本并校验 SHA256 的安装步骤</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.10.0 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": "在发布页面下载 .deb 安装包",
  "run.securityLabel": "安全",
  "run.securityValue": "<a href=\"#security\">了解安全模型</a>",
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
    title: "MemoryWhale — 你與 AI 程式助理的終端機記憶",
    description:
      "MemoryWhale 將開發過程中的指令、輸出等記錄保存在本機 SQLite，方便你與可信任的工具查找過往的錯誤與除錯經驗。資料預設留在本機，由你主動匯出或傳輸。",
    jsonLdDescription:
      "為開發者與 AI 程式助理長期保存除錯經驗。終端機記錄存入本機 SQLite，可透過 MCP 檢索。"
  },
  "nav.label": "頁面導覽",
  "brand.home": "MemoryWhale 首頁",
  "nav.terminal": "功能",
  "nav.how": "運作方式",
  "nav.agents": "AI 程式助理",
  "nav.who": "適合誰",
  "nav.install": "安裝",
  "nav.docs": "文件",
  "nav.releases": "發行版本",
  "nav.github": "GitHub ↗",
  "nav.delphin": "Delphin ↗",
  "nav.contextgc": "ContextGC ↗",
  "language.label": "語言",
  "language.en": "English",
  "language.ar": "العربية",
  "language.de": "Deutsch",
  "language.fr": "Français",
  "language.zh-CN": "简体中文",
  "language.zh-TW": "繁體中文",
  "language.ko": "한국어",
  "language.ja": "日本語",
  "search.openLabel": "搜尋本頁",
  "search.title": "在本頁尋找",
  "search.label": "搜尋首頁",
  "search.placeholder": "試試 MCP、安裝、隱私…",
  "search.close": "關閉搜尋",
  "search.hint": "搜尋本頁的標題與內容。",
  "search.empty": "找不到相關內容。",
  "search.results": "相關內容",
  "hero.releaseBadge": "v0.10.0 現已推出",
  "hero.title": "讓終端機<span class=\"hero-accent\">記住</span>除錯經驗。",
  "hero.lead":
    "在本機保存有用的指令、輸出與除錯背景，讓 AI 程式助理在新的工作階段也能延續上次的經驗。",
  "hero.demoCta": "觀看示範",
  "hero.installCta": "快速上手",
  "hero.localTitle": "本機優先",
  "hero.localBody": "記憶留在你的裝置上",
  "hero.privateTitle": "隱私保障",
  "hero.privateBody": "無需雲端服務",
  "hero.agentTitle": "支援 AI 程式助理",
  "hero.agentBody": "換個工作階段，經驗不丟失",
  "hero.terminalTitle": "接上你的 AI 程式助理",
  "hero.terminalNote": "使用 Cargo 與 Claude Code 的設定範例；輸出已精簡。",
  "hero.whaleAlt": "MemoryWhale 鯨魚吉祥物",
  "integrations.label": "支援搭配",
  "integrations.via": "透過 MCP",
  "integrations.more": "所有整合",
  "release.eyebrow": "0.10.0 新功能",
  "release.title": "記憶可共用，來源可追溯。",
  "release.copy":
    "CLI、Web UI 與桌面應用程式皆已更新至 0.10.0，可重複使用的 Rust 核心函式庫版本為 0.5.0。在 Rust 中以結構體字面值建立 <code>Memory</code> 時，現在必須提供 <code>agent: Option&lt;String&gt;</code> 欄位；serde 會為舊 JSON 中缺少的欄位填入預設值，因此舊資料仍可讀取。",
  "release.connectTitle": "連接 Claude Code 與 Rho",
  "release.connectBody":
    "<code>mw integrate claude</code> 與 <code>mw integrate rho</code> 會設定 MCP 連線，並安裝擷取掛鉤與技能檔案。<code>mw doctor</code> 會逐項檢查這些元件。",
  "release.provenanceTitle": "了解證據來自何處",
  "release.provenanceBody":
    "資料庫結構版本 10 用獨立欄位記錄指令由哪個程式助理執行，值為 <code>claude</code>、<code>rho</code> 或 <code>NULL</code>；只有空值顯示為 <code>terminal</code>。這個身分欄位與記錄的來源類型分開儲存。統一的儲存庫 ID 將關聯工作樹歸到同一儲存庫下，同時保留各工作樹的路徑。",
  "release.interfaceTitle": "選擇本機介面",
  "release.interfaceBody":
    "<code>mw-serve</code> 透過 <code>POST /mcp</code> 提供 HTTP MCP 介面。唯讀 JSON API 須以 <code>--api</code> 主動啟用。執行 <code>mw github context &lt;pr&gt;</code> 時，才會使用你的 <code>gh</code> 登入身分讀取 PR 中繼資料、檢查結果、提交狀態與審查資訊；不會簽出程式碼、自動儲存或在背景同步。",
  "who.eyebrow": "適合誰",
  "who.title": "三種開發日常，都用得上。",
  "who.copy":
    "除錯線索散落在終端機輸出、Shell 歷史、不同機器與臨時的 AI 對話裡？MemoryWhale 幫你把它們保存下來。查看附有真實指令記錄的<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">完整使用情境</a>。",
  "who.shellTitle": "🔍 經常在終端機裡除錯",
  "who.shellBody":
    "又遇到相同的建置、連結器或相依套件錯誤？Shell 歷史只有指令，沒有當時的輸出、最後幾行錯誤訊息與解決方法。<code>mw search</code> 能找回上次失敗的執行記錄，<em>以及</em>相關的除錯經驗。",
  "who.multiTitle": "🛰️ 在多台機器之間切換",
  "who.multiBody":
    "Jetson、實驗室伺服器、筆電各留著一部分歷史，一旦斷線，線索也容易遺失。<code>mw --live</code> 持續自動儲存工作階段，減少斷線造成的記錄遺失；需要跨機器使用時，由你執行 <code>mw push</code> / <code>mw pull</code> 傳輸記憶。",
  "who.agentTitle": "🤖 使用 AI 程式助理",
  "who.agentBody":
    "使用 Claude Code、Codex、Cursor 時，不必每次都從頭說明環境。接上 <code>mw-mcp</code> 後，程式助理可以查詢過往記錄，並主動呼叫 <code>remember</code> 保存經驗。修正是否有效，仍需要你實際驗證。",
  "features.label": "為終端機工作保留記憶",
  "features.terminalTitle": "終端機記憶",
  "features.terminalBody": "在開發過程中記錄有用的指令、輸出與除錯背景。",
  "features.retrievalTitle": "智慧檢索",
  "features.retrievalBody": "需要時找回過往的指令、失敗記錄與修正方法。",
  "features.localTitle": "本機儲存",
  "features.localBody": "將 MemoryWhale 資料保存在自己的裝置上。",
  "how.eyebrow": "運作方式",
  "how.title": "記錄下來、存好，需要時再找回。",
  "how.captureTitle": "擷取",
  "how.captureBody": "貼上一段終端機執行記錄、呼叫 Rust 輔助程式，或啟動持續自動儲存的 Shell 工作階段。",
  "how.storeTitle": "儲存",
  "how.storeBody": "指令執行記錄與引數都存入本機的 SQLite 資料庫。",
  "how.extractTitle": "擷取關鍵字",
  "how.extractBody": "Rust 程式從指令、備註與錯誤訊息中擷取關鍵字。",
  "how.exploreTitle": "探索",
  "how.exploreBody": "搜尋歷史記錄，或在圖譜中點選指令節點查看詳細內容。",
  "agents.eyebrow": "AI 程式助理",
  "agents.title": "踩過的坑，讓程式助理也記住。",
  "agents.copy":
    "AI 程式助理換個工作階段就可能失去先前的脈絡，把你做過的除錯再做一遍。<code>mw-mcp</code> 透過 Model Context Protocol 提供本機記憶存取；設定後，Claude Code、Rho、Codex 或 Cursor 就能查詢過往的失敗記錄。接入前，請確認你信任用戶端處理這些記錄；如果用戶端會把上下文傳給模型供應商，也需要信任該供應商處理相關資料。本機儲存不代表用戶端取出的資料不會離開本機。",
  "agents.clientsLabel": "提供整合指南的用戶端",
  "agents.matrix": "查看完整支援對照表",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">用戶端與工具設定指南</a>中的支援對照表逐項列出 MCP 支援、自動擷取能力與驗證狀態，也涵蓋 OpenRouter 與 CLIProxyAPI 等模型閘道。",
  "agents.setupLabel": "設定",
  "agents.setupValue": "一行指令",
  "agents.toolsLabel": "工具",
  "agents.toolsValue": "6 個本機 MCP 工具：recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "不用 AI 程式助理？",
  "agents.noAgentValue": "mw context 輸出可直接貼上的摘要",
  "demo.eyebrow": "擷取 → 記憶 → 檢索",
  "demo.title": "用模擬資料走一遍記錄與檢索流程。",
  "demo.copy":
    "記錄一次指令執行，再保存解決問題的方法。下次遇到相同錯誤，就能從本機記錄中查找。MCP 支援檢索與主動寫入，但僅接上 MCP 不會自動擷取日常終端機操作。",
  "demo.handoff":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">Claude 向 Rho 移交記錄的離線示範</a>會匯入預設測試資料（fixtures），用模擬的 Rho 用戶端連接實際運作的 MCP 服務。示範不執行真實的程式助理，也不執行或驗證測試資料中的 Cargo 修正方法。目前，Rho 掛鉤遇到缺少指令文字的失敗呼叫時會保留失敗中繼資料；同樣缺少指令文字的成功呼叫則會略過。任務開始時自動檢索歷史、出錯時查找相關記錄，以及上下文壓縮前保存記憶，都仍須由用戶端自行編排，尚不是內建的自動化功能。",
  "demo.imageAlt": "使用模擬資料展示的 MemoryWhale 終端機與儀表板",
  "data.eyebrow": "你的資料",
  "data.title": "資料留在本機，擷取與傳輸由你決定。",
  "data.copy":
    "資料庫位於你的機器上：Linux 通常是 <code>~/.local/share/MemoryWhale/</code>，macOS 通常是 <code>~/Library/Application Support/MemoryWhale/</code>。設定 <code>MEMORYWHALE_DATA_DIR</code> 可選擇其他位置。",
  "data.captureLabel": "擷取控制",
  "data.captureValue": "<code>.mwignore</code>、路徑規則、僅記錄指令模式",
  "data.redactionLabel": "敏感資訊遮蔽",
  "data.redactionValue": "可遮蔽常見敏感資訊，但不能作為安全邊界",
  "data.sizeLabel": "大小限制",
  "data.sizeValue": "擷取的文字欄位預設上限為 1 MiB，超出部分會截斷",
  "data.inspectLabel": "檢查 / 刪除",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "傳輸",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code>，或由你主動發起 SSH 傳輸",
  "data.stewardshipLabel": "維護",
  "data.stewardshipValue": "<code>mw memory compact</code>——先預覽操作（dry-run），保留資料列",
  "security.eyebrow": "安全模型",
  "security.title": "預設本機，共用時明確授權。",
  "security.copy":
    "CLI、TUI、MCP 伺服器、Web 儀表板與桌面外殼都使用本機儲存。<code>mw-mcp</code> 是透過標準輸入輸出（stdio）通訊的本機程序，使用前需要確認其可信。儀表板預設只監聽本機回環位址（loopback）；監聽非回環位址時必須設定存取權杖，且只應向可信任的網路開放。受保護的 HTTP MCP 需要 Bearer 驗證；主動啟用的 JSON API 沿用儀表板的存取控制。HTTP 本身不加密連線。用戶端接上這兩種介面，都不等於啟用自動擷取。詳見<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">本機資料威脅模型</a>。",
  "run.eyebrow": "安裝",
  "run.title": "一行指令。不需要 Rust。",
  "run.copy":
    "提供適用於 Linux x86_64/aarch64 與 macOS 的預先編譯二進位檔。如果發行版本附有 SHA256 校驗檔案，安裝程式會據此核對下載內容；舊版本可能沒有校驗碼。建議先手動擷取一次並檢查結果，再考慮啟用 <code>mw global on</code>。目前不提供 Windows 原生版本；在 WSL 中可使用 Linux 版本。",
  "run.tryLabel": "先試試",
  "run.tryValue": "<code>mw demo</code>——將範例資料寫入選定的儲存區",
  "run.prebuiltLabel": "預先編譯安裝",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md#install-or-upgrade\">指定版本並核對 SHA256 的安裝步驟</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.10.0 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": "在發行頁面下載 .deb 安裝套件",
  "run.securityLabel": "安全性",
  "run.securityValue": "<a href=\"#security\">了解安全模型</a>",
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
  "nav.terminal": "기능",
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
  "language.ar": "العربية",
  "language.de": "Deutsch",
  "language.fr": "프랑스어",
  "language.zh-CN": "중국어 간체",
  "language.zh-TW": "중국어 번체",
  "language.ko": "한국어",
  "language.ja": "일본어",
  "search.openLabel": "이 페이지 검색",
  "search.title": "이 페이지에서 찾기",
  "search.label": "홈페이지 검색",
  "search.placeholder": "MCP, 설치, 개인정보 보호 등을 검색하세요…",
  "search.close": "검색 닫기",
  "search.hint": "이 페이지의 제목과 내용을 검색하세요.",
  "search.empty": "일치하는 섹션이 없습니다.",
  "search.results": "일치하는 섹션",
  "hero.releaseBadge": "v0.10.0 출시",
  "hero.title": "터미널이 <span class=\"hero-accent\">기억하게</span> 하세요.",
  "hero.lead":
    "유용한 터미널 맥락을 로컬에 보관하세요. 코딩 에이전트가 이전 세션의 디버깅 지식을 되찾을 수 있습니다.",
  "hero.demoCta": "데모 보기",
  "hero.installCta": "빠른 시작",
  "hero.localTitle": "로컬 우선",
  "hero.localBody": "메모리는 내 기기에 보관",
  "hero.privateTitle": "개인정보 보호",
  "hero.privateBody": "클라우드 없이 사용",
  "hero.agentTitle": "코딩 에이전트를 위한 설계",
  "hero.agentBody": "세션이 끝나도 남는 디버깅 지식",
  "hero.terminalTitle": "코딩 에이전트 연결",
  "hero.terminalNote": "Cargo와 Claude Code를 사용한 설정 예시이며, 출력은 축약했습니다.",
  "hero.whaleAlt": "MemoryWhale 고래 마스코트",
  "integrations.label": "함께 쓰는 도구",
  "integrations.via": "MCP를 통해 연결",
  "integrations.more": "모든 통합",
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
  "features.label": "터미널 작업을 위한 메모리 계층",
  "features.terminalTitle": "터미널 메모리",
  "features.terminalBody": "작업 중 유용한 디버깅 맥락을 캡처하세요.",
  "features.retrievalTitle": "지능형 검색",
  "features.retrievalBody": "필요할 때 이전 명령, 실패 기록, 해결책을 되찾으세요.",
  "features.localTitle": "완전한 로컬 저장",
  "features.localBody": "MemoryWhale 데이터를 내 기기에 보관하세요.",
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
  "nav.terminal": "機能",
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
  "language.ar": "العربية",
  "language.de": "Deutsch",
  "language.fr": "フランス語",
  "language.zh-CN": "簡体字中国語",
  "language.zh-TW": "繁体字中国語",
  "language.ko": "韓国語",
  "language.ja": "日本語",
  "search.openLabel": "このページを検索",
  "search.title": "このページ内で探す",
  "search.label": "ホームページを検索",
  "search.placeholder": "MCP、インストール、プライバシーなど…",
  "search.close": "検索を閉じる",
  "search.hint": "このページの見出しと内容を検索します。",
  "search.empty": "一致するセクションはありません。",
  "search.results": "一致するセクション",
  "hero.releaseBadge": "v0.10.0 登場",
  "hero.title": "ターミナルに<span class=\"hero-accent\">記憶</span>を。",
  "hero.lead":
    "役立つターミナルのコンテキストをローカルに保存。コーディングエージェントが過去のセッションのデバッグ知識を取り戻せます。",
  "hero.demoCta": "デモを見る",
  "hero.installCta": "クイックスタート",
  "hero.localTitle": "ローカル優先",
  "hero.localBody": "記憶は自分のマシンに保存",
  "hero.privateTitle": "プライバシー保護",
  "hero.privateBody": "クラウド不要",
  "hero.agentTitle": "コーディングエージェント向け",
  "hero.agentBody": "セッションを越えて残るデバッグ知識",
  "hero.terminalTitle": "コーディングエージェントを接続",
  "hero.terminalNote": "Cargo と Claude Code による設定例。出力は省略しています。",
  "hero.whaleAlt": "MemoryWhale のクジラのマスコット",
  "integrations.label": "連携できるツール",
  "integrations.via": "MCP 経由",
  "integrations.more": "すべての連携",
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
  "features.label": "ターミナル作業のための記憶層",
  "features.terminalTitle": "ターミナルメモリ",
  "features.terminalBody": "作業中に役立つデバッグのコンテキストを記録。",
  "features.retrievalTitle": "スマートな検索",
  "features.retrievalBody": "必要なときに、過去のコマンド、失敗、解決策を取り戻せます。",
  "features.localTitle": "完全ローカル",
  "features.localBody": "MemoryWhale のデータは自分のマシンに保存。",
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
  ar: AR,
  de: DE,
  fr: FR,
  "zh-CN": ZH_CN,
  "zh-TW": ZH_TW,
  ko: KO,
  ja: JA
};
const supportedLanguages = Object.freeze(["en", "ar", "de", "fr", "zh-CN", "zh-TW", "ko", "ja"]);

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
    return ["en", "ar", "de", "fr", "ko", "ja"].includes(primary) ? primary : null;
  };

  const readStoredLanguage = () => {
    try {
      return normalizeLanguage(window.localStorage.getItem(storageKey));
    } catch {
      return null;
    }
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
    document.documentElement.dir = selectedLanguage === "ar" ? "rtl" : "ltr";
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
    document.querySelectorAll("[data-i18n-placeholder]").forEach((element) => {
      const value = dictionary[element.dataset.i18nPlaceholder];
      if (typeof value === "string") element.setAttribute("placeholder", value);
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
    document.dispatchEvent(new CustomEvent("memorywhale:languagechange", {
      detail: { language: selectedLanguage }
    }));
  };

  const initialLanguage = queryLanguage() || readStoredLanguage() || 'en';
  applyLanguage(initialLanguage);
  languageSelect?.addEventListener("change", (event) => {
    const selectedLanguage = normalizeLanguage(event.target.value) || "en";
    applyLanguage(selectedLanguage, { persist: true, updateUrl: true });
  });
})();
