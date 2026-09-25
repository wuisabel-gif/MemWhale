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
  "hero.releaseBadge": "v0.12.0 is here",
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
  "release.eyebrow": "New in 0.12.0",
  "release.title": "Retrieval you can trust.",
  "release.copy":
    "Product 0.12.0 spans the CLI, web UI, and desktop app. The reusable Rust core is 0.6.0, and opening a store migrates it to SQLite schema 13. Choose what a search returns, review memories that disagree, and keep debugging evidence as case files and recipes.",
  "release.connectTitle": "Search the way you mean",
  "release.connectBody":
    "<code>mw search --mode evidence|lessons|recipes|failures</code> narrows results to one kind of memory. Opt-in <code>--ranking bayesian</code> combines the existing signals as log-odds; the default ranking is unchanged.",
  "release.provenanceTitle": "Review before you trust",
  "release.provenanceBody":
    "<code>mw feedback</code> records local helpful, irrelevant, outdated, or contradicted marks. <code>mw contradictions</code> flags memories that may disagree for you to confirm or reject. Neither changes a memory, and feedback does not affect ranking yet.",
  "release.interfaceTitle": "Keep the whole story",
  "release.interfaceBody":
    "<code>mw case</code> groups ordered command runs with observations and a conclusion, exportable as JSON or Markdown. <code>mw recipe</code> saves a reusable command from verified runs. Both stay local and redacted, and nothing runs automatically.",
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
    "Coding-agent sessions can lose context and repeat debugging you already did. <code>mw-mcp</code> is a Model Context Protocol server over your local memory — register it once and Claude Code, Rho, Codex, or Cursor can query past failures directly. Anything the client retrieves can reach the model provider it sends context to, so connect only clients you trust.",
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
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.12.1/docs/releases/0.12.1.md#install-or-upgrade\">Pinned, checksum-verified installer instructions</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.12.1 --locked</code>",
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
  "language.de": "الألمانية",
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
  "hero.releaseBadge": "الإصدار v0.12.0 متاح الآن",
  "hero.title": "اجعل طرفيتك <span class=\"hero-accent\">تتذكّر</span>.",
  "hero.lead":
    "احفظ سياق الطرفية المفيد على جهازك، واسمح لوكلاء البرمجة باسترجاع خبرة تصحيح الأخطاء من الجلسات السابقة.",
  "hero.demoCta": "شاهد العرض التجريبي",
  "hero.installCta": "البدء السريع",
  "hero.localTitle": "محلي أولًا",
  "hero.localBody": "تبقى الذاكرة على جهازك",
  "hero.privateTitle": "خصوصية",
  "hero.privateBody": "لا حاجة إلى السحابة",
  "hero.agentTitle": "مصمّم لوكلاء البرمجة",
  "hero.agentBody": "خبرة تصحيح الأخطاء لا تضيع بانتهاء الجلسة",
  "hero.terminalTitle": "اربط وكيل البرمجة لديك",
  "hero.terminalNote": "مثال إعداد باستخدام Cargo وClaude Code؛ المخرجات مختصرة.",
  "hero.whaleAlt": "حوت MemoryWhale، تميمة المشروع",
  "integrations.label": "يعمل مع",
  "integrations.via": "عبر MCP",
  "integrations.more": "جميع التكاملات",
  "release.eyebrow": "الجديد في 0.12.0",
  "release.title": "استرجاع يمكنك الوثوق به.",
  "release.copy":
    "يشمل إصدار المنتج 0.12.0 واجهة سطر الأوامر (CLI)، وواجهة الويب، وتطبيق سطح المكتب؛ أما نواة Rust القابلة لإعادة الاستخدام فإصدارها 0.6.0، وعند فتح مخزن البيانات يُرحَّل إلى مخطط SQLite رقم 13. اختر ما يعيده البحث، وراجع الذكريات المتعارضة، واحفظ شواهد تصحيح الأخطاء في ملفات حالات ووصفات.",
  "release.connectTitle": "ابحث بما تقصده",
  "release.connectBody":
    "يحصر <code>mw search --mode evidence|lessons|recipes|failures</code> النتائج في نوع واحد من الذكريات. ويجمع <code>--ranking bayesian</code>، عند تفعيله صراحةً، الإشارات الحالية على هيئة لوغاريتم الأرجحية (log-odds)؛ ويبقى الترتيب الافتراضي دون تغيير.",
  "release.provenanceTitle": "راجِع قبل أن تثق",
  "release.provenanceBody":
    "يسجّل <code>mw feedback</code> محليًا تقييماتك للذكريات: مفيدة، أو غير ذات صلة، أو قديمة، أو متناقضة. ويُبرز <code>mw contradictions</code> الذكريات التي قد تتعارض لتؤكد التعارض أو ترفضه بنفسك. لا يغيّر أيٌّ منهما الذكريات نفسها، ولا تؤثر التقييمات في الترتيب حتى الآن.",
  "release.interfaceTitle": "احتفظ بالقصة كاملة",
  "release.interfaceBody":
    "يجمع <code>mw case</code> عمليات تنفيذ الأوامر بترتيبها مع الملاحظات والاستنتاج، ويمكن تصديرها بصيغة JSON أو Markdown. ويحفظ <code>mw recipe</code> أمرًا قابلًا لإعادة الاستخدام من عمليات تنفيذ جرى التحقق منها. يبقى كلاهما محليًا مع حجب البيانات الحساسة، ولا يُنفَّذ أي شيء تلقائيًا.",
  "who.eyebrow": "لمن صُمّم",
  "who.title": "صُمّم لثلاثة أنماط من العمل.",
  "who.copy":
    "يخدم MemoryWhale المطورين الذين يتوزّع سياق تصحيح الأخطاء لديهم بين سجل مخرجات الطرفية، وسجل الصدفة، والأجهزة، وجلسات الوكلاء المؤقتة. اطّلع على <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">الشروحات التفصيلية لحالات الاستخدام</a> مع سجلات أوامر حقيقية.",
  "who.shellTitle": "🔍 من يصحّح الأخطاء عبر سطر الأوامر",
  "who.shellBody":
    "واجهت خطأ البناء أو الربط أو الاعتماديات نفسه مرتين. يحتفظ سجل الصدفة بالأمر — لا بالمخرجات، أو نهاية رسالة الخطأ، أو الحل. أما <code>mw search</code> فيعيد التنفيذ السابق الذي أخفق، <em>ومعه</em> الدرس المرتبط به.",
  "who.multiTitle": "🛰️ من يعمل على أجهزة متعددة",
  "who.multiBody":
    "Jetson، وخادم المختبر، والحاسوب المحمول — تنقطع الجلسات ويحتفظ كل جهاز بسجل خاص وغير مكتمل. يواصل <code>mw --live</code> الحفظ التلقائي رغم انقطاع الاتصال؛ وينقل <code>mw push</code> / <code>mw pull</code> الذاكرة بين الأجهزة بطلب صريح.",
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
  "how.captureBody": "الصق سجل تنفيذ من الطرفية، أو استدعِ أداة Rust المساعدة، أو ابدأ جلسة صدفة تُحفظ تلقائيًا باستمرار.",
  "how.storeTitle": "تخزين",
  "how.storeBody": "يحفظ SQLite عمليات تنفيذ الأوامر ووسيطاتها محليًا على جهازك.",
  "how.extractTitle": "استخراج",
  "how.extractBody": "يستخرج Rust الكلمات المفتاحية من الأوامر والملاحظات ونصوص الأخطاء.",
  "how.exploreTitle": "استكشاف",
  "how.exploreBody": "ابحث، أو انقر على عُقد الأوامر في واجهة الرسم البياني المضيئة.",
  "agents.eyebrow": "وكلاء الذكاء الاصطناعي",
  "agents.title": "امنح وكيلك ذاكرة لما أخفق من قبل.",
  "agents.copy":
    "قد تفقد جلسات وكلاء البرمجة السياق وتكرّر تصحيح أخطاء سبق أن عالجتها. <code>mw-mcp</code> خادم Model Context Protocol يتيح الوصول إلى ذاكرتك المحلية — سجّله مرة واحدة ليتمكن <bdi dir=\"ltr\">Claude Code, Rho, Codex, Cursor</bdi> من الاستعلام مباشرةً عن الإخفاقات السابقة. وتذكّر أنك تأتمن العميل على الشواهد التي يسترجعها، وكذلك أي مزوّد نماذج يرسل إليه هذا العميل السياق.",
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
    "التقط أمرًا واحدًا، واحفظ الشرح الذي حلّ المشكلة، ثم ابحث في المخزن المحلي عندما يتكرر الإخفاق نفسه. يوفّر MCP الاسترجاع والكتابة الصريحة؛ ولا يلتقط نشاط الطرفية العادي تلقائيًا.",
  "demo.handoff":
    "يستورد <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">العرض التجريبي دون اتصال لتسليم السياق من Claude إلى Rho</a> بيانات اختبار جاهزة ويحاكي عميل Rho باستخدام MCP حقيقي. لا يشغّل وكلاء فعليين، ولا ينفّذ إصلاح Cargo الوارد في بيانات الاختبار أو يتحقق منه. تحتفظ خطافات Rho حاليًا بالبيانات الوصفية للإخفاق عند غياب نص الأمر؛ وتتجاوز الاستدعاءات الناجحة التي لا تحتوي على نص الأمر. أما الاسترجاع التلقائي عند بدء المهمة، والبحث عن الإخفاقات، والحفظ قبل ضغط السياق، فتبقى من مسؤولية العميل في تنسيق عمله، وليست أتمتة مضمّنة في المنتج.",
  "demo.imageAlt": "عرض تجريبي لطرفية MemoryWhale ولوحة التحكم ببيانات اصطناعية",
  "data.eyebrow": "بياناتك",
  "data.title": "محلي أولًا: كل خيار واضح أمامك.",
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
  "data.stewardshipValue": "<code>mw memory compact</code> — تشغيل تجريبي (dry-run) أولًا، دون حذف أي صفوف",
  "security.eyebrow": "نموذج الأمان",
  "security.title": "محلي افتراضيًا، وبقرار صريح عند المشاركة.",
  "security.copy":
    "تستخدم واجهة سطر الأوامر (CLI)، والواجهة النصية (TUI)، وخادم MCP، ولوحة الويب، وتطبيق سطح المكتب المخزنَ المحلي. يعمل <code>mw-mcp</code> كعملية محلية موثوقة عبر stdio؛ وترتبط لوحة الويب افتراضيًا بعنوان loopback. تتطلب إتاحة اللوحة على عنوان غير loopback رمز وصول، وينبغي قصر إتاحتها على شبكة موثوقة. يتطلب HTTP MCP المحمي مصادقة Bearer؛ وتشترك واجهة JSON API، التي تُفعّل باختيار صريح، في ضوابط الوصول الخاصة بلوحة الويب. لا يشفّر HTTP الاتصال. ووصول العميل عبر أيٍّ من الواجهتين لا يعني التقاطًا تلقائيًا. راجع <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">نموذج تهديدات البيانات المحلية</a>.",
  "run.eyebrow": "التثبيت",
  "run.title": "سطر واحد. لا حاجة إلى Rust.",
  "run.copy":
    "تتوفر ملفات تنفيذية جاهزة لأنظمة Linux x86_64/aarch64 وmacOS. يتحقق برنامج التثبيت من ملفات SHA256 المنشورة عندما يوفّرها الإصدار؛ وقد لا يتوفر مجموع اختباري (checksum) للإصدارات الأقدم. ابدأ بالتقاط أمر واحد صراحةً وافحص نتيجته، ولا تفكّر في <code>mw global on</code> إلا بعد ذلك. لا يُدعم Windows دعمًا أصليًا، لكن يمكن تشغيل نسخة Linux داخل WSL.",
  "run.tryLabel": "جرّب أولًا",
  "run.tryValue": "<code>mw demo</code> — يكتب بيانات نموذجية في المخزن المحدد",
  "run.prebuiltLabel": "تثبيت ملفات تنفيذية جاهزة",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.12.1/docs/releases/0.12.1.md#install-or-upgrade\">تعليمات تثبيت لإصدار محدد مع التحقق من المجموع الاختباري</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.12.1 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": "حزمة .deb من صفحة الإصدارات",
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
      "MemoryWhale speichert Belege aus der Entwicklungsarbeit in einer lokalen SQLite-Datenbank, damit Menschen und vertrauenswürdige Tools frühere Fehlschläge und Erkenntnisse wiederfinden. Local-first – Export und Übertragung nur auf ausdrücklichen Wunsch.",
    jsonLdDescription:
      "Dauerhaftes, lokales Debugging-Gedächtnis für Entwickler und Coding-Agenten. Speichert Belege aus dem Terminal in einer lokalen SQLite-Datenbank und stellt sie über MCP bereit."
  },
  "nav.label": "Seitennavigation",
  "brand.home": "MemoryWhale-Startseite",
  "nav.terminal": "Funktionen",
  "nav.how": "Funktionsweise",
  "nav.agents": "KI-Agenten",
  "nav.who": "Für wen?",
  "nav.install": "Installieren",
  "nav.docs": "Dokumentation",
  "nav.releases": "Releases",
  "nav.github": "GitHub ↗",
  "nav.delphin": "Delphin ↗",
  "nav.contextgc": "ContextGC ↗",
  "language.label": "Sprache",
  "language.en": "English",
  "language.ar": "العربية",
  "language.de": "Deutsch",
  "language.fr": "Français",
  "language.zh-CN": "简体中文",
  "language.zh-TW": "繁體中文",
  "language.ko": "한국어",
  "language.ja": "日本語",
  "search.openLabel": "Diese Seite durchsuchen",
  "search.title": "Auf dieser Seite suchen",
  "search.label": "Startseite durchsuchen",
  "search.placeholder": "z. B. MCP, Installation, Datenschutz…",
  "search.close": "Suche schließen",
  "search.hint": "Durchsuche Überschriften und Inhalte dieser Seite.",
  "search.empty": "Keine passenden Abschnitte.",
  "search.results": "Passende Abschnitte",
  "hero.releaseBadge": "v0.12.0 ist da",
  "hero.title": "Gib deinem Terminal ein <span class=\"hero-accent\">Gedächtnis</span>.",
  "hero.lead":
    "Bewahre nützlichen Terminal-Kontext lokal auf. Lass Coding-Agenten auf Debugging-Wissen aus früheren Sitzungen zurückgreifen.",
  "hero.demoCta": "Demo ansehen",
  "hero.installCta": "Schnellstart",
  "hero.localTitle": "Local-first",
  "hero.localBody": "Das Gedächtnis bleibt auf deinem Rechner",
  "hero.privateTitle": "Privat",
  "hero.privateBody": "Keine Cloud nötig",
  "hero.agentTitle": "Für Coding-Agenten gebaut",
  "hero.agentBody": "Debugging-Wissen überdauert Sitzungen",
  "hero.terminalTitle": "Coding-Agenten anbinden",
  "hero.terminalNote": "Beispiel-Setup mit Cargo und Claude Code; Ausgabe gekürzt.",
  "hero.whaleAlt": "Wal-Maskottchen von MemoryWhale",
  "integrations.label": "Funktioniert mit",
  "integrations.via": "über MCP",
  "integrations.more": "Alle Integrationen",
  "release.eyebrow": "Neu in 0.12.0",
  "release.title": "Treffer, denen du vertrauen kannst.",
  "release.copy":
    "Die Produktversion 0.12.0 umfasst CLI, Weboberfläche und Desktop-App. Der wiederverwendbare Rust-Kern steht bei 0.6.0; beim Öffnen wird ein Speicher auf SQLite-Schema 13 migriert. Leg fest, was eine Suche liefert, prüfe widersprüchliche Erinnerungen und bewahre Debugging-Belege als Fallakten und Rezepte auf.",
  "release.connectTitle": "Suchen, wie du es meinst",
  "release.connectBody":
    "<code>mw search --mode evidence|lessons|recipes|failures</code> beschränkt die Ergebnisse auf eine Art von Erinnerung. Das optionale <code>--ranking bayesian</code> verrechnet die vorhandenen Signale als Log-Odds; das Standard-Ranking bleibt unverändert.",
  "release.provenanceTitle": "Erst prüfen, dann vertrauen",
  "release.provenanceBody":
    "<code>mw feedback</code> speichert lokale Markierungen: hilfreich, irrelevant, veraltet oder widersprüchlich. <code>mw contradictions</code> kennzeichnet Erinnerungen, die sich möglicherweise widersprechen, damit du sie bestätigen oder verwerfen kannst. Keins von beiden verändert eine Erinnerung, und Feedback wirkt sich noch nicht auf das Ranking aus.",
  "release.interfaceTitle": "Die ganze Geschichte festhalten",
  "release.interfaceBody":
    "<code>mw case</code> bündelt Befehlsläufe in ihrer Reihenfolge mit Beobachtungen und einem Fazit und lässt sich als JSON oder Markdown exportieren. <code>mw recipe</code> speichert einen wiederverwendbaren Befehl aus verifizierten Läufen. Beides bleibt lokal und maskiert, und nichts wird automatisch ausgeführt.",
  "who.eyebrow": "Für wen?",
  "who.title": "Gemacht für drei Arbeitsweisen.",
  "who.copy":
    "MemoryWhale richtet sich an Entwickler, deren Debugging-Kontext über Terminal-Scrollback, Shell-History, verschiedene Rechner und kurzlebige Agentensitzungen verstreut ist. Die ausführlichen <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">Anwendungsfälle</a> spielen das mit echten Befehlsprotokollen durch.",
  "who.shellTitle": "🔍 Debuggen vor allem in der Shell",
  "who.shellBody":
    "Derselbe Build-, Linker- oder Abhängigkeitsfehler taucht zum zweiten Mal auf. Die Shell-History kennt den Befehl – aber nicht die Ausgabe, das Ende der Fehlermeldung oder den Fix. <code>mw search</code> liefert den alten fehlgeschlagenen Lauf <em>und</em> die damit verknüpfte Erkenntnis.",
  "who.multiTitle": "🛰️ Arbeit auf mehreren Rechnern",
  "who.multiBody":
    "Jetson, Laborserver, Laptop – Sitzungen brechen ab, und jeder Rechner hat seinen eigenen, lückenhaften Verlauf. <code>mw --live</code> speichert auch über Verbindungsabbrüche hinweg automatisch; mit <code>mw push</code> / <code>mw pull</code> überträgst du das Gedächtnis gezielt zwischen Rechnern.",
  "who.agentTitle": "🤖 Arbeit mit Coding-Agenten",
  "who.agentBody":
    "Claude Code, Codex, Cursor – in jeder Sitzung erklärst du deine Umgebung aufs Neue. Mit <code>mw-mcp</code> kann der Agent frühere Belege abfragen und mit <code>remember</code> gezielt eine Erkenntnis speichern. Ob ein Fix wirklich funktioniert, musst du trotzdem selbst prüfen.",
  "features.label": "Eine Gedächtnisschicht für die Arbeit im Terminal",
  "features.terminalTitle": "Terminal-Gedächtnis",
  "features.terminalBody": "Halte nützlichen Debugging-Kontext fest, während du arbeitest.",
  "features.retrievalTitle": "Intelligente Suche",
  "features.retrievalBody": "Finde frühere Befehle, Fehlschläge und Fixes genau dann, wenn du sie brauchst.",
  "features.localTitle": "Vollständig lokal",
  "features.localBody": "Deine MemoryWhale-Daten bleiben auf deinem eigenen Rechner.",
  "how.eyebrow": "Funktionsweise",
  "how.title": "Erfassen, speichern, extrahieren, erkunden.",
  "how.captureTitle": "Erfassen",
  "how.captureBody": "Füge die Ausgabe eines Terminal-Laufs ein, ruf das Rust-Hilfsprogramm auf oder starte eine Shell mit Live-Autosave.",
  "how.storeTitle": "Speichern",
  "how.storeBody": "SQLite speichert Befehlsläufe samt Argumenten lokal auf deinem Rechner.",
  "how.extractTitle": "Extrahieren",
  "how.extractBody": "Rust extrahiert Schlüsselwörter aus Befehlen, Notizen und Fehlermeldungen.",
  "how.exploreTitle": "Erkunden",
  "how.exploreBody": "Suche Befehlsknoten in der leuchtenden Graph-Ansicht oder klick sie direkt an.",
  "agents.eyebrow": "KI-Agenten",
  "agents.title": "Gib deinem Agenten ein Gedächtnis für das, was schon schiefging.",
  "agents.copy":
    "Sitzungen mit Coding-Agenten können Kontext verlieren und Debugging wiederholen, das du längst erledigt hast. <code>mw-mcp</code> ist ein Model-Context-Protocol-Server für dein lokales Gedächtnis – einmal registriert, können Claude Code, Rho, Codex oder Cursor frühere Fehlschläge direkt abfragen. Du musst dem Client die abgerufenen Belege anvertrauen können – ebenso jedem Modellanbieter, an den dieser Client Kontext sendet.",
  "agents.clientsLabel": "Clients mit Integrationsanleitungen",
  "agents.matrix": "Mehr in der Matrix",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">Einrichtungsanleitungen für Clients und Tools</a> – die Funktionsmatrix dokumentiert für jeden Client MCP-Unterstützung, automatische Aufzeichnung und Prüfstatus, auch für Modell-Gateways wie OpenRouter und CLIProxyAPI.",
  "agents.setupLabel": "Einrichtung",
  "agents.setupValue": "Ein Befehl",
  "agents.toolsLabel": "Tools",
  "agents.toolsValue": "6 lokale MCP-Tools: recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "Kein Agent?",
  "agents.noAgentValue": "mw context gibt eine kopierfertige Zusammenfassung aus",
  "demo.eyebrow": "Aufzeichnung → Gedächtnis → Abruf",
  "demo.title": "Der Kernablauf, gezeigt mit synthetischen Daten.",
  "demo.copy":
    "Zeichne einen Befehl auf, speichere die Erklärung, mit der sich das Problem lösen ließ, und durchsuche den lokalen Speicher, wenn derselbe Fehlschlag wieder auftaucht. MCP ermöglicht Abruf und explizites Schreiben; normale Terminalaktivität zeichnet es nicht automatisch auf.",
  "demo.handoff":
    "Die <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">Offline-Demo zur Übergabe von Claude an Rho</a> importiert Fixtures und simuliert einen Rho-Client gegen echtes MCP. Sie startet keine echten Agenten und führt den Cargo-Fix aus den Fixtures weder aus noch verifiziert sie ihn. Rho-Hooks behalten derzeit Metadaten zu Fehlschlägen, wenn der Befehlstext fehlt; erfolgreiche Aufrufe ohne Befehlstext werden übersprungen. Automatischer Abruf beim Start einer Aufgabe, das Nachschlagen von Fehlschlägen und das Speichern vor der Kontextkomprimierung bleiben Sache der Client-Orchestrierung und sind keine mitgelieferte Automatik.",
  "demo.imageAlt": "MemoryWhale-Demo von Terminal und Dashboard mit synthetischen Daten",
  "data.eyebrow": "Deine Daten",
  "data.title": "Local-first heißt: nachvollziehbare Entscheidungen.",
  "data.copy":
    "Die Datenbank liegt auf deinem Rechner: unter Linux meist in <code>~/.local/share/MemoryWhale/</code>, unter macOS in <code>~/Library/Application Support/MemoryWhale/</code>. Mit <code>MEMORYWHALE_DATA_DIR</code> wählst du einen anderen Speicherort.",
  "data.captureLabel": "Aufzeichnung steuern",
  "data.captureValue": "<code>.mwignore</code>, Pfadrichtlinie, nur Befehle",
  "data.redactionLabel": "Maskierung",
  "data.redactionValue": "Hilft bei gängigen Secrets, ist aber keine Sicherheitsgrenze",
  "data.sizeLabel": "Größenlimit",
  "data.sizeValue": "Aufgezeichnete Textfelder sind standardmäßig auf 1 MiB begrenzt, der Rest wird abgeschnitten",
  "data.inspectLabel": "Prüfen / löschen",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "Übertragung",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code> oder gezielte SSH-Übertragung",
  "data.stewardshipLabel": "Gedächtnispflege",
  "data.stewardshipValue": "<code>mw memory compact</code> – erst ein Probelauf, alle Zeilen bleiben erhalten",
  "security.eyebrow": "Sicherheitsmodell",
  "security.title": "Standardmäßig lokal. Geteilt wird nur ausdrücklich.",
  "security.copy":
    "CLI, TUI, MCP-Server, Web-Dashboard und Desktop-Shell greifen auf den lokalen Speicher zu. <code>mw-mcp</code> ist ein vertrauenswürdiger lokaler stdio-Prozess; das Dashboard lauscht standardmäßig nur auf Loopback. Ist das Dashboard über Loopback hinaus erreichbar, braucht es ein Token und sollte nur in einem vertrauenswürdigen Netzwerk freigegeben werden. Geschütztes HTTP-MCP erfordert Bearer-Authentifizierung; die optionale JSON-API nutzt dieselben Zugriffskontrollen wie das Dashboard. HTTP verschlüsselt die Verbindung nicht. Clientzugriff über eine der beiden Schnittstellen bedeutet keine automatische Aufzeichnung. Mehr dazu im <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">Bedrohungsmodell für lokale Daten</a>.",
  "run.eyebrow": "Installation",
  "run.title": "Eine Zeile. Kein Rust nötig.",
  "run.copy":
    "Vorkompilierte Binaries gibt es für Linux x86_64/aarch64 und macOS. Der Installer prüft die veröffentlichten SHA256-Dateien, sofern ein Release sie mitliefert; bei älteren Releases fehlt unter Umständen eine Prüfsumme. Starte mit einer einzelnen, gezielten Aufzeichnung, sieh sie dir an und zieh erst dann <code>mw global on</code> in Betracht. Windows wird nicht nativ unterstützt; unter WSL lässt sich der Linux-Build verwenden.",
  "run.tryLabel": "Zuerst ausprobieren",
  "run.tryValue": "<code>mw demo</code> — schreibt Beispieldaten in den ausgewählten Speicher",
  "run.prebuiltLabel": "Vorkompilierte Binaries",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.12.1/docs/releases/0.12.1.md#install-or-upgrade\">Anleitung für den fest versionierten Installer mit Prüfsummencheck</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.12.1 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": ".deb auf der Releases-Seite",
  "run.securityLabel": "Sicherheit",
  "run.securityValue": "<a href=\"#security\">Modell lesen</a>",
  "run.verifyLabel": "Prüfen",
  "run.verifyValue": "<code>mw --version</code> · <code>mw doctor</code>",
  "footer.copyright": "Copyright (c) 2026 wuisabel-gif. MemoryWhale - Terminal-Gedächtnis und Wissensgraph auf Basis von Rust/Tauri.",
  "footer.docs": "Dokumentation",
  "footer.useCases": "Anwendungsfälle",
  "footer.cli": "CLI-Referenz",
  "footer.security": "Sicherheitsrichtlinie",
  "footer.integrations": "Integrationen"
};

const FR = {
  meta: {
    title: "MemoryWhale — la mémoire du terminal, pour vous et votre agent IA",
    description:
      "MemoryWhale enregistre les traces de votre travail de développement dans une base SQLite locale, pour que vous et les outils de confiance puissiez retrouver les échecs passés et les leçons qui en ont été tirées. Local par défaut, avec export et transfert explicites.",
    jsonLdDescription:
      "Mémoire de débogage locale et persistante, pour les développeurs et les agents de code. Enregistre les traces du terminal dans une base SQLite locale et les met à disposition via MCP."
  },
  "nav.label": "Navigation de la page",
  "brand.home": "Accueil MemoryWhale",
  "nav.terminal": "Fonctionnalités",
  "nav.how": "Fonctionnement",
  "nav.agents": "Agents IA",
  "nav.who": "Pour qui ?",
  "nav.install": "Installation",
  "nav.docs": "Documentation",
  "nav.releases": "Versions",
  "nav.github": "GitHub ↗",
  "nav.delphin": "Delphin ↗",
  "nav.contextgc": "ContextGC ↗",
  "language.label": "Langue",
  "language.en": "English",
  "language.ar": "العربية",
  "language.de": "Deutsch",
  "language.fr": "Français",
  "language.zh-CN": "简体中文",
  "language.zh-TW": "繁體中文",
  "language.ko": "한국어",
  "language.ja": "日本語",
  "search.openLabel": "Rechercher dans cette page",
  "search.title": "Rechercher dans la page",
  "search.label": "Rechercher sur la page d’accueil",
  "search.placeholder": "Par exemple : MCP, installation, confidentialité…",
  "search.close": "Fermer la recherche",
  "search.hint": "Recherche dans les titres et le contenu de cette page.",
  "search.empty": "Aucune section ne correspond.",
  "search.results": "Sections correspondantes",
  "hero.releaseBadge": "La v0.12.0 est disponible",
  "hero.title": "Donnez de la <span class=\"hero-accent\">mémoire</span> à votre terminal.",
  "hero.lead":
    "Conservez en local le contexte utile du terminal. Vos agents de code retrouvent ce qui a été appris en déboguant lors des sessions précédentes.",
  "hero.demoCta": "Voir la démo",
  "hero.installCta": "Démarrage rapide",
  "hero.localTitle": "Local d’abord",
  "hero.localBody": "La mémoire reste sur votre machine",
  "hero.privateTitle": "Privé",
  "hero.privateBody": "Aucun cloud nécessaire",
  "hero.agentTitle": "Pensé pour les agents de code",
  "hero.agentBody": "Les acquis du débogage survivent aux sessions",
  "hero.terminalTitle": "Connectez votre agent de code",
  "hero.terminalNote": "Exemple de configuration avec Cargo et Claude Code ; sortie abrégée.",
  "hero.whaleAlt": "La baleine, mascotte de MemoryWhale",
  "integrations.label": "Compatible avec",
  "integrations.via": "via MCP",
  "integrations.more": "Toutes les intégrations",
  "release.eyebrow": "Nouveautés de la 0.12.0",
  "release.title": "Une recherche digne de confiance.",
  "release.copy":
    "La version produit 0.12.0 couvre le CLI, l’interface web et l’application de bureau. Le cœur Rust réutilisable passe en 0.6.0, et l’ouverture d’une base la migre vers le schéma SQLite 13. Choisissez ce que renvoie une recherche, examinez les mémoires qui se contredisent et conservez vos preuves de débogage sous forme de dossiers et de recettes.",
  "release.connectTitle": "Cherchez ce que vous voulez dire",
  "release.connectBody":
    "<code>mw search --mode evidence|lessons|recipes|failures</code> limite les résultats à un seul type de mémoire. L’option facultative <code>--ranking bayesian</code> combine les signaux existants en log-odds ; le classement par défaut reste inchangé.",
  "release.provenanceTitle": "Vérifiez avant de vous fier",
  "release.provenanceBody":
    "<code>mw feedback</code> enregistre en local vos avis : utile, hors sujet, obsolète ou contredit. <code>mw contradictions</code> signale les mémoires susceptibles de se contredire, pour que vous les confirmiez ou les rejetiez. Aucune des deux commandes ne modifie une mémoire, et les avis n’influencent pas encore le classement.",
  "release.interfaceTitle": "Gardez toute l’histoire",
  "release.interfaceBody":
    "<code>mw case</code> regroupe des exécutions de commandes dans l’ordre, avec des observations et une conclusion, exportables en JSON ou en Markdown. <code>mw recipe</code> enregistre une commande réutilisable à partir d’exécutions vérifiées. Tout reste en local, avec masquage des secrets, et rien ne s’exécute automatiquement.",
  "who.eyebrow": "Pour qui ?",
  "who.title": "Pensé pour trois façons de travailler.",
  "who.copy":
    "MemoryWhale s’adresse aux développeurs dont le contexte de débogage est éparpillé entre le scrollback du terminal, l’historique du shell, différentes machines et des sessions d’agent éphémères. Consultez les <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">cas d’usage détaillés</a>, avec de vraies transcriptions de commandes.",
  "who.shellTitle": "🔍 Le débogueur qui vit dans le shell",
  "who.shellBody":
    "Vous tombez deux fois sur la même erreur de build, d’édition de liens ou de dépendance. L’historique du shell garde la commande, mais pas la sortie, la fin du message d’erreur ni le correctif. <code>mw search</code> renvoie l’ancienne exécution en échec <em>et</em> la leçon qui y est associée.",
  "who.multiTitle": "🛰️ Le développeur multi-machines",
  "who.multiBody":
    "Jetson, serveur du labo, portable : les sessions se coupent et chaque machine garde un historique cloisonné et incomplet. <code>mw --live</code> sauvegarde en continu malgré les déconnexions ; <code>mw push</code> / <code>mw pull</code> transfèrent explicitement la mémoire d’une machine à l’autre.",
  "who.agentTitle": "🤖 L’utilisateur d’agents de code",
  "who.agentBody":
    "Claude Code, Codex, Cursor : chaque session commence par réexpliquer votre environnement. Avec <code>mw-mcp</code>, l’agent peut interroger les traces existantes et enregistrer explicitement une leçon avec <code>remember</code>. C’est toujours à vous de vérifier qu’un correctif fonctionne.",
  "features.label": "Une couche de mémoire pour le travail dans le terminal",
  "features.terminalTitle": "Mémoire du terminal",
  "features.terminalBody": "Capturez le contexte de débogage utile pendant que vous travaillez.",
  "features.retrievalTitle": "Recherche intelligente",
  "features.retrievalBody": "Retrouvez les commandes, les échecs et les correctifs passés au moment où vous en avez besoin.",
  "features.localTitle": "Entièrement local",
  "features.localBody": "Les données de MemoryWhale restent sur votre machine.",
  "how.eyebrow": "Fonctionnement",
  "how.title": "Capturer, stocker, extraire, explorer.",
  "how.captureTitle": "Capturer",
  "how.captureBody": "Collez une exécution du terminal, appelez l’utilitaire Rust ou lancez un shell sauvegardé en continu.",
  "how.storeTitle": "Stocker",
  "how.storeBody": "SQLite enregistre les exécutions et les arguments des commandes, en local sur votre machine.",
  "how.extractTitle": "Extraire",
  "how.extractBody": "Rust extrait des mots-clés des commandes, des notes et des messages d’erreur.",
  "how.exploreTitle": "Explorer",
  "how.exploreBody": "Lancez une recherche ou cliquez sur les nœuds de commandes dans le graphe lumineux.",
  "agents.eyebrow": "Agents IA",
  "agents.title": "Donnez à votre agent la mémoire de ce qui a déjà échoué.",
  "agents.copy":
    "Les sessions d’agents de code peuvent perdre leur contexte et refaire un débogage que vous avez déjà mené. <code>mw-mcp</code> est un serveur Model Context Protocol branché sur votre mémoire locale : enregistrez-le une fois, et Claude Code, Rho, Codex ou Cursor peuvent interroger directement les échecs passés. N’accordez au client l’accès aux traces qu’il récupère que si vous lui faites confiance, ainsi qu’au fournisseur de modèles auquel il envoie ce contexte.",
  "agents.clientsLabel": "Clients disposant d’un guide d’intégration",
  "agents.matrix": "Plus dans la matrice",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">Guides de configuration des clients et outils</a> — la matrice des capacités indique, pour chaque client, la prise en charge de MCP, la capture automatique et l’état de vérification, y compris pour les passerelles de modèles comme OpenRouter et CLIProxyAPI.",
  "agents.setupLabel": "Configuration",
  "agents.setupValue": "Une seule commande",
  "agents.toolsLabel": "Outils",
  "agents.toolsValue": "6 outils MCP locaux : recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "Pas d’agent ?",
  "agents.noAgentValue": "mw context affiche un résumé prêt à coller",
  "demo.eyebrow": "Capture → mémoire → recherche",
  "demo.title": "La boucle principale, illustrée avec des données fictives.",
  "demo.copy":
    "Capturez une commande, enregistrez l’explication de ce qui l’a corrigée, puis interrogez le stockage local quand le même échec se reproduit. MCP permet la recherche et l’écriture explicite ; il ne capture pas automatiquement l’activité courante du terminal.",
  "demo.handoff":
    "La <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">démo hors ligne de passage de relais de Claude à Rho</a> importe des fixtures et simule un client Rho face à un vrai serveur MCP. Elle ne fait tourner aucun agent réel, et n’exécute ni ne vérifie le correctif Cargo contenu dans la fixture. Pour l’instant, les hooks Rho conservent les métadonnées d’échec quand le texte de la commande est absent ; les appels réussis sans texte de commande sont ignorés. Le rappel automatique au démarrage d’une tâche, la recherche des échecs et la sauvegarde avant compaction relèvent de l’orchestration côté client : ils ne sont pas automatisés par MemoryWhale.",
  "demo.imageAlt": "Démo fictive du terminal et du dashboard MemoryWhale",
  "data.eyebrow": "Vos données",
  "data.title": "Local par défaut, des choix visibles.",
  "data.copy":
    "La base de données se trouve sur votre machine, généralement dans <code>~/.local/share/MemoryWhale/</code> sous Linux ou <code>~/Library/Application Support/MemoryWhale/</code> sous macOS. Définissez <code>MEMORYWHALE_DATA_DIR</code> pour choisir un autre emplacement.",
  "data.captureLabel": "Contrôle de la capture",
  "data.captureValue": "<code>.mwignore</code>, règles par chemin, commandes seules",
  "data.redactionLabel": "Masquage",
  "data.redactionValue": "Masque les secrets courants ; ce n’est pas une barrière de sécurité",
  "data.sizeLabel": "Taille maximale",
  "data.sizeValue": "Par défaut, les champs de texte capturés sont tronqués à 1 Mio",
  "data.inspectLabel": "Consulter / supprimer",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "Transfert",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code> ou transfert SSH explicite",
  "data.stewardshipLabel": "Maintenance",
  "data.stewardshipValue": "<code>mw memory compact</code> — dry-run d’abord, lignes conservées",
  "security.eyebrow": "Modèle de sécurité",
  "security.title": "Local par défaut, explicite dès qu’il y a partage.",
  "security.copy":
    "Le CLI, le TUI, le serveur MCP, le dashboard web et l’application de bureau utilisent le stockage local. <code>mw-mcp</code> est un processus stdio local de confiance ; par défaut, le dashboard n’écoute que sur l’interface loopback. Un dashboard exposé hors loopback exige un token et ne doit l’être que sur un réseau de confiance. MCP en HTTP protégé exige une authentification Bearer ; l’API JSON, activée sur demande, partage les contrôles d’accès du dashboard. HTTP ne chiffre pas la connexion. Aucune de ces deux interfaces ne transforme l’accès d’un client en capture automatique. Consultez le <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">modèle de menace des données locales</a>.",
  "run.eyebrow": "Installation",
  "run.title": "Une ligne. Pas besoin de Rust.",
  "run.copy":
    "Des binaires précompilés sont disponibles pour Linux x86_64/aarch64 et macOS. L’installateur vérifie les fichiers SHA256 publiés lorsque la version en fournit ; les versions plus anciennes n’ont pas forcément de somme de contrôle. Commencez par une capture explicite, examinez-la, et seulement ensuite envisagez <code>mw global on</code>. Windows n’est pas pris en charge nativement ; WSL permet d’utiliser la version Linux.",
  "run.tryLabel": "Pour commencer",
  "run.tryValue": "<code>mw demo</code> — écrit des données d’exemple dans le stockage sélectionné",
  "run.prebuiltLabel": "Binaires précompilés",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.12.1/docs/releases/0.12.1.md#install-or-upgrade\">Instructions d’installation épinglées, avec vérification des sommes de contrôle</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.12.1 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": "Paquet .deb sur la page des versions",
  "run.securityLabel": "Sécurité",
  "run.securityValue": "<a href=\"#security\">Lire le modèle</a>",
  "run.verifyLabel": "Vérifier",
  "run.verifyValue": "<code>mw --version</code> · <code>mw doctor</code>",
  "footer.copyright": "Copyright (c) 2026 wuisabel-gif. MemoryWhale — mémoire du terminal et graphe de connaissances, en Rust/Tauri.",
  "footer.docs": "Documentation",
  "footer.useCases": "Cas d’usage",
  "footer.cli": "Référence du CLI",
  "footer.security": "Politique de sécurité",
  "footer.integrations": "Intégrations"
};

const ZH_CN = {
  meta: {
    title: "MemoryWhale — 你和 AI 智能体共用的终端记忆",
    description:
      "MemoryWhale 把开发过程中的证据采集到本地 SQLite，方便你和受信任的工具找回以往的失败记录与经验。本地优先，导出和传输都需要你主动操作。",
    jsonLdDescription:
      "面向开发者和编程智能体的持久化本地调试记忆。将终端证据采集到本地 SQLite，并通过 MCP 提供访问。"
  },
  "nav.label": "页面导航",
  "brand.home": "MemoryWhale 首页",
  "nav.terminal": "功能",
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
  "hero.releaseBadge": "v0.12.0 现已发布",
  "hero.title": "让终端<span class=\"hero-accent\">记住</span>调试经验。",
  "hero.lead":
    "在本地保留有用的终端上下文，让编程智能体从以往会话中找回调试经验。",
  "hero.demoCta": "查看演示",
  "hero.installCta": "快速上手",
  "hero.localTitle": "本地优先",
  "hero.localBody": "记忆留在你的设备上",
  "hero.privateTitle": "隐私保护",
  "hero.privateBody": "无需云服务",
  "hero.agentTitle": "为编程智能体打造",
  "hero.agentBody": "换个会话，经验不丢",
  "hero.terminalTitle": "接入你的编程智能体",
  "hero.terminalNote": "使用 Cargo 和 Claude Code 的配置示例；输出已精简。",
  "hero.whaleAlt": "MemoryWhale 鲸鱼吉祥物",
  "integrations.label": "已支持",
  "integrations.via": "通过 MCP",
  "integrations.more": "所有集成",
  "release.eyebrow": "0.12.0 新功能",
  "release.title": "检索结果，值得信赖。",
  "release.copy":
    "产品版本 0.12.0 覆盖 CLI、Web UI 和桌面应用；可复用的 Rust 核心为 0.6.0，打开存储库时会自动迁移到 SQLite schema 13。现在可以指定搜索返回哪类结果、审查相互矛盾的记忆，并把调试证据保存为案例档案和命令配方。",
  "release.connectTitle": "按你的意图搜索",
  "release.connectBody":
    "<code>mw search --mode evidence|lessons|recipes|failures</code> 可把结果限定为某一类记忆。<code>--ranking bayesian</code> 需手动开启，它会把现有信号按对数几率（log-odds）合并；默认排序保持不变。",
  "release.provenanceTitle": "先审查，再信任",
  "release.provenanceBody":
    "<code>mw feedback</code> 在本地记录“有用”“无关”“过时”或“有矛盾”等标记。<code>mw contradictions</code> 会标出可能相互矛盾的记忆，由你确认或驳回。两者都不会修改记忆，反馈目前也还不影响排序。",
  "release.interfaceTitle": "保留完整来龙去脉",
  "release.interfaceBody":
    "<code>mw case</code> 把按顺序执行的命令连同观察记录和结论整理在一起，可导出为 JSON 或 Markdown。<code>mw recipe</code> 从已验证的运行中保存可复用的命令。两者都只保存在本地并经过脱敏，也不会自动执行任何操作。",
  "who.eyebrow": "适合谁",
  "who.title": "三种开发日常，都用得上。",
  "who.copy":
    "调试线索散落在终端输出、Shell 历史、不同机器和临时的 AI 会话里？MemoryWhale 帮你把它们留存下来。查看附有真实命令记录的<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">完整使用场景</a>。",
  "who.shellTitle": "🔍 经常在终端里调试",
  "who.shellBody":
    "又遇到相同的构建、链接器或依赖错误？Shell 历史只有命令，没有当时的输出、末尾报错和解决方法。<code>mw search</code> 能找回上次失败的执行记录，<em>以及</em>关联的调试经验。",
  "who.multiTitle": "🛰️ 在多台机器之间切换",
  "who.multiBody":
    "Jetson、实验室服务器、笔记本——会话说断就断，每台机器上只留着一份不完整的私有历史。<code>mw --live</code> 在断连时也会持续自动保存；<code>mw push</code> / <code>mw pull</code> 由你显式地在机器之间迁移记忆。",
  "who.agentTitle": "🤖 编程智能体用户",
  "who.agentBody":
    "Claude Code、Codex、Cursor——每开一个会话都得重新介绍一遍环境。接入 <code>mw-mcp</code> 后，智能体可以查询以往的证据，并通过 <code>remember</code> 显式保存经验。修复是否真的有效，仍然需要你自己验证。",
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
  "agents.eyebrow": "AI 智能体",
  "agents.title": "踩过的坑，让智能体也记住。",
  "agents.copy":
    "编程智能体换个会话就可能丢失上下文，把你调过的问题再调一遍。<code>mw-mcp</code> 是基于本地记忆的 Model Context Protocol 服务器——注册一次，Claude Code、Rho、Codex 或 Cursor 就能直接查询以往的失败记录。客户端取到的证据由它自行处理，因此你需要信任该客户端，以及它会把上下文发送给的任何模型提供商。",
  "agents.clientsLabel": "提供集成指南的客户端",
  "agents.matrix": "查看完整支持对照表",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">客户端与工具配置指南</a>中的支持对照表逐项列出 MCP 支持、自动采集能力及验证状态，也涵盖 OpenRouter 和 CLIProxyAPI 等模型网关。",
  "agents.setupLabel": "设置",
  "agents.setupValue": "一条命令",
  "agents.toolsLabel": "工具",
  "agents.toolsValue": "6 个本地 MCP 工具：recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "没用智能体？",
  "agents.noAgentValue": "mw context 输出可直接粘贴的摘要",
  "demo.eyebrow": "采集 → 记忆 → 检索",
  "demo.title": "用模拟数据体验核心流程。",
  "demo.copy":
    "记录一次命令执行，再保存解决问题的方法。下次遇到相同错误，就能从本地记录中查找。MCP 支持检索和主动写入，但仅接入 MCP 不会自动采集日常终端操作。",
  "demo.handoff":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">Claude 到 Rho 的离线交接演示</a>会导入测试数据（fixtures），用模拟的 Rho 客户端对接真实的 MCP。演示不运行真实的智能体，也不执行或验证测试数据中的 Cargo 修复方法。目前，Rho 钩子遇到缺少命令文本的失败调用时会保留失败元数据；同样缺少命令文本的成功调用则会跳过。任务开始时自动检索历史、出错时查找相关记录，以及上下文压缩前保存记忆，都仍需客户端自行编排，并非已经发布的自动化功能。",
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
  "data.stewardshipValue": "<code>mw memory compact</code>——先 dry-run 预览，原有记录保留",
  "security.eyebrow": "安全模型",
  "security.title": "默认本地，共享需显式操作。",
  "security.copy":
    "CLI、TUI、MCP 服务器、Web 仪表盘和桌面外壳都使用本地存储。<code>mw-mcp</code> 是受信任的本地 stdio 进程；仪表盘默认只监听本机回环地址（loopback）；监听非回环地址时必须设置访问令牌，且只应向可信网络开放。受保护的 HTTP MCP 需要 Bearer 身份验证；主动开启的 JSON API 沿用仪表盘的访问控制。HTTP 本身不加密连接。客户端接入这两种接口，都不等于开启自动采集。详见<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">本地数据威胁模型</a>。",
  "run.eyebrow": "安装",
  "run.title": "一行命令，无需 Rust。",
  "run.copy":
    "提供适用于 Linux x86_64/aarch64 和 macOS 的预编译二进制文件。如果发布版本附带 SHA256 校验文件，安装程序会据此校验下载内容；旧版本可能没有校验和。建议先手动采集一次并检查结果，再考虑开启 <code>mw global on</code>。目前不提供 Windows 原生版本；在 WSL 中可使用 Linux 版本。",
  "run.tryLabel": "先试试",
  "run.tryValue": "<code>mw demo</code>——向当前选定的存储写入示例数据",
  "run.prebuiltLabel": "预编译安装",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.12.1/docs/releases/0.12.1.md#install-or-upgrade\">指定版本并校验 SHA256 的安装步骤</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.12.1 --locked</code>",
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
      "MemoryWhale 將開發過程中的指令、輸出等記錄保存在本機 SQLite，讓你與你信任的工具找回過去的失敗與除錯經驗。資料以本機為主，匯出與傳輸都由你明確操作。",
    jsonLdDescription:
      "給開發者與 AI 程式助理使用、可長期保存的本機除錯記憶。將終端機證據存進本機 SQLite，並透過 MCP 提供查詢。"
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
  "hero.releaseBadge": "v0.12.0 現已推出",
  "hero.title": "讓終端機<span class=\"hero-accent\">記住</span>除錯經驗。",
  "hero.lead":
    "把有用的終端機脈絡留在本機，讓 AI 程式助理能找回先前工作階段累積的除錯知識。",
  "hero.demoCta": "觀看示範",
  "hero.installCta": "快速上手",
  "hero.localTitle": "本機優先",
  "hero.localBody": "記憶留在你的裝置上",
  "hero.privateTitle": "注重隱私",
  "hero.privateBody": "不需要雲端服務",
  "hero.agentTitle": "為 AI 程式助理打造",
  "hero.agentBody": "除錯知識不隨工作階段消失",
  "hero.terminalTitle": "接上你的 AI 程式助理",
  "hero.terminalNote": "使用 Cargo 與 Claude Code 的設定範例；輸出已精簡。",
  "hero.whaleAlt": "MemoryWhale 鯨魚吉祥物",
  "integrations.label": "可搭配使用",
  "integrations.via": "透過 MCP",
  "integrations.more": "所有整合",
  "release.eyebrow": "0.12.0 新功能",
  "release.title": "值得信賴的檢索結果。",
  "release.copy":
    "產品版本 0.12.0 涵蓋 CLI、Web UI 與桌面應用程式；可重複使用的 Rust 核心為 0.6.0，開啟資料儲存區時會自動遷移到 SQLite schema 13。你可以指定搜尋要回傳哪類結果、檢視彼此矛盾的記憶，並將除錯證據保存為案例檔與指令配方。",
  "release.connectTitle": "照你的意思搜尋",
  "release.connectBody":
    "<code>mw search --mode evidence|lessons|recipes|failures</code> 可將結果限縮為單一類型的記憶。<code>--ranking bayesian</code> 需自行啟用，會以對數勝算（log-odds）結合現有訊號；預設排序維持不變。",
  "release.provenanceTitle": "先檢視，再信任",
  "release.provenanceBody":
    "<code>mw feedback</code> 會在本機記錄「有幫助」、「不相關」、「已過時」或「有矛盾」等標記。<code>mw contradictions</code> 會標出可能彼此矛盾的記憶，交由你確認或駁回。兩者都不會修改記憶，回饋目前也還不會影響排序。",
  "release.interfaceTitle": "保留完整脈絡",
  "release.interfaceBody":
    "<code>mw case</code> 會將依序執行的指令連同觀察紀錄與結論整理在一起，可匯出為 JSON 或 Markdown。<code>mw recipe</code> 會從已驗證的執行中儲存可重複使用的指令。兩者都只保存在本機，敏感資訊會經過遮蔽，也不會自動執行任何東西。",
  "who.eyebrow": "適合誰",
  "who.title": "為三種工作方式而設計。",
  "who.copy":
    "MemoryWhale 適合除錯脈絡散落在終端機捲動紀錄、Shell 歷史、不同機器與臨時代理工作階段裡的開發者。完整的<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">使用情境逐步說明</a>附有真實的指令紀錄。",
  "who.shellTitle": "🔍 以 Shell 為主的除錯者",
  "who.shellBody":
    "同一個建置、連結器或相依套件錯誤，你已經遇過兩次。Shell 歷史只記得指令，記不得輸出、錯誤訊息的最後幾行和修法。<code>mw search</code> 會找回當時失敗的執行紀錄，<em>以及</em>與它連結的經驗。",
  "who.multiTitle": "🛰️ 在多台機器間工作的人",
  "who.multiBody":
    "Jetson、實驗室伺服器、筆電——工作階段常常斷線，每台機器各自保有一份不完整的歷史。<code>mw --live</code> 在斷線時也會持續自動儲存；<code>mw push</code> / <code>mw pull</code> 則由你明確地在機器之間搬移記憶。",
  "who.agentTitle": "🤖 AI 程式助理的使用者",
  "who.agentBody":
    "Claude Code、Codex、Cursor——每開一個工作階段，都得重新解釋一次你的環境。接上 <code>mw-mcp</code> 後，程式助理可以查詢先前的證據，並用 <code>remember</code> 明確保存經驗。修正是否真的有效，仍需要你自己驗證。",
  "features.label": "終端機工作的記憶層",
  "features.terminalTitle": "終端機記憶",
  "features.terminalBody": "在工作時擷取有用的除錯脈絡。",
  "features.retrievalTitle": "智慧檢索",
  "features.retrievalBody": "在需要的時候找回先前的指令、失敗與修正。",
  "features.localTitle": "完全在本機",
  "features.localBody": "MemoryWhale 的資料都留在你自己的機器上。",
  "how.eyebrow": "運作方式",
  "how.title": "擷取、儲存、抽取、探索。",
  "how.captureTitle": "擷取",
  "how.captureBody": "貼上一段終端機執行紀錄、呼叫 Rust 輔助工具，或啟動會即時自動儲存的 Shell。",
  "how.storeTitle": "儲存",
  "how.storeBody": "SQLite 在你的機器上保存指令執行紀錄與參數。",
  "how.extractTitle": "抽取",
  "how.extractBody": "Rust 從指令、筆記與錯誤訊息中抽取關鍵字。",
  "how.exploreTitle": "探索",
  "how.exploreBody": "搜尋，或在發光的圖譜介面中點選指令節點。",
  "agents.eyebrow": "AI 程式助理",
  "agents.title": "讓程式助理記得哪些做法已經失敗過。",
  "agents.copy":
    "AI 程式助理的工作階段可能遺失脈絡，把你做過的除錯再做一遍。<code>mw-mcp</code> 是架在本機記憶上的 Model Context Protocol 伺服器——註冊一次，Claude Code、Rho、Codex 或 Cursor 就能直接查詢過去的失敗。用戶端取回的證據交由它處理，因此你必須信任這個用戶端，也包括它會把脈絡傳送過去的任何模型供應商。",
  "agents.clientsLabel": "提供整合指南的用戶端",
  "agents.matrix": "對照表中還有更多",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">用戶端與工具設定指南</a>——功能對照表逐一列出每個用戶端的 MCP 支援、自動擷取與驗證狀態，也包括 OpenRouter、CLIProxyAPI 等模型閘道。",
  "agents.setupLabel": "設定",
  "agents.setupValue": "一個指令",
  "agents.toolsLabel": "工具",
  "agents.toolsValue": "6 個本機 MCP 工具：recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "沒有 AI 程式助理？",
  "agents.noAgentValue": "mw context 會印出可直接貼上的摘要",
  "demo.eyebrow": "擷取 → 記憶 → 檢索",
  "demo.title": "用模擬資料看懂核心流程。",
  "demo.copy":
    "擷取一個指令，保存修好它的說明，等同樣的失敗再出現時，就到本機資料庫搜尋。MCP 提供檢索與明確寫入，但不會自動擷取一般的終端機操作。",
  "demo.handoff":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">Claude 到 Rho 的離線交接示範</a>會匯入 fixture，並用模擬的 Rho 用戶端連接真實的 MCP。它不會執行實際運作中的代理，也不會執行或驗證 fixture 裡的 Cargo 修正。目前缺少指令文字時，Rho hook 仍會保留失敗的中繼資料；沒有指令文字的成功呼叫則會略過。任務開始時自動回想、失敗時查詢，以及壓縮前儲存，仍屬於用戶端的編排工作，並不是已內建的自動化功能。",
  "demo.imageAlt": "MemoryWhale 終端機與儀表板示範（模擬資料）",
  "data.eyebrow": "你的資料",
  "data.title": "本機優先，每個選擇都看得見。",
  "data.copy":
    "資料庫位於你的機器上：Linux 通常是 <code>~/.local/share/MemoryWhale/</code>，macOS 通常是 <code>~/Library/Application Support/MemoryWhale/</code>。設定 <code>MEMORYWHALE_DATA_DIR</code> 可選擇其他位置。",
  "data.captureLabel": "擷取控制",
  "data.captureValue": "<code>.mwignore</code>、路徑政策、只記錄指令",
  "data.redactionLabel": "敏感資訊遮蔽",
  "data.redactionValue": "能處理常見的機密資訊，但不是安全邊界",
  "data.sizeLabel": "大小限制",
  "data.sizeValue": "擷取的文字欄位預設上限 1 MiB，超過會截斷",
  "data.inspectLabel": "檢查 / 刪除",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "傳輸",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code>，或明確透過 SSH 傳輸",
  "data.stewardshipLabel": "維護",
  "data.stewardshipValue": "<code>mw memory compact</code>——先用 dry-run 預覽，資料列都會保留",
  "security.eyebrow": "安全模型",
  "security.title": "預設在本機，分享時由你明確決定。",
  "security.copy":
    "CLI、TUI、MCP 伺服器、Web 儀表板與桌面版外殼都使用本機資料庫。<code>mw-mcp</code> 是受信任的本機 stdio 處理程序；儀表板預設只綁定 loopback 位址。儀表板若綁定非 loopback 位址，必須設定權杖，而且只應在可信任的網路上開放。受保護的 HTTP MCP 需要 Bearer 驗證；選擇啟用的 JSON API 則沿用儀表板的存取控制。HTTP 本身不會加密連線。用戶端能存取這兩種介面，並不代表會自動擷取。詳見<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">本機資料威脅模型</a>。",
  "run.eyebrow": "安裝",
  "run.title": "一行指令，不需要 Rust。",
  "run.copy":
    "提供 Linux x86_64/aarch64 與 macOS 的預先編譯二進位檔。若發行版本有提供 SHA256 檔案，安裝程式會加以驗證；較舊的版本可能沒有檢查碼。建議先明確擷取一次、檢查結果，之後再考慮 <code>mw global on</code>。Windows 並非原生支援的平台；可以在 WSL 中使用 Linux 版本。",
  "run.tryLabel": "先試試",
  "run.tryValue": "<code>mw demo</code>——將範例資料寫入目前選定的資料庫",
  "run.prebuiltLabel": "預編譯版安裝",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.12.1/docs/releases/0.12.1.md#install-or-upgrade\">固定版本、經檢查碼驗證的安裝說明</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.12.1 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": "發行頁面提供 .deb",
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
      "MemoryWhale은 개발 과정의 증거를 로컬 SQLite에 저장해, 사람과 신뢰하는 도구가 지난 실패와 교훈을 다시 찾을 수 있게 합니다. 로컬 우선이며, 내보내기와 전송은 명시적으로만 이루어집니다.",
    jsonLdDescription:
      "개발자와 코딩 에이전트를 위한 영구 로컬 디버깅 메모리. 터미널 증거를 로컬 SQLite에 저장하고 MCP로 제공합니다."
  },
  "nav.label": "페이지 탐색",
  "brand.home": "MemoryWhale 홈",
  "nav.terminal": "기능",
  "nav.how": "동작 방식",
  "nav.agents": "AI 에이전트",
  "nav.who": "사용 대상",
  "nav.install": "설치",
  "nav.docs": "문서",
  "nav.releases": "릴리스",
  "nav.github": "GitHub ↗",
  "nav.delphin": "Delphin ↗",
  "nav.contextgc": "ContextGC ↗",
  "language.label": "언어",
  "language.en": "영어",
  "language.ar": "아랍어",
  "language.de": "독일어",
  "language.fr": "프랑스어",
  "language.zh-CN": "중국어 간체",
  "language.zh-TW": "중국어 번체",
  "language.ko": "한국어",
  "language.ja": "일본어",
  "search.openLabel": "이 페이지 검색",
  "search.title": "페이지 내 검색",
  "search.label": "홈페이지 검색",
  "search.placeholder": "MCP, 설치, 개인정보…",
  "search.close": "검색 닫기",
  "search.hint": "이 페이지의 제목과 본문을 검색합니다.",
  "search.empty": "일치하는 섹션이 없습니다.",
  "search.results": "일치하는 섹션",
  "hero.releaseBadge": "v0.12.0 출시",
  "hero.title": "터미널이 <span class=\"hero-accent\">기억하게</span> 하세요.",
  "hero.lead":
    "유용한 터미널 맥락을 로컬에 남겨 두세요. 코딩 에이전트가 이전 세션의 디버깅 지식을 다시 찾아 쓸 수 있습니다.",
  "hero.demoCta": "데모 보기",
  "hero.installCta": "빠른 시작",
  "hero.localTitle": "로컬 우선",
  "hero.localBody": "메모리는 내 머신에 저장",
  "hero.privateTitle": "프라이버시",
  "hero.privateBody": "클라우드 불필요",
  "hero.agentTitle": "코딩 에이전트용 설계",
  "hero.agentBody": "세션이 끝나도 남는 디버깅 지식",
  "hero.terminalTitle": "코딩 에이전트 연결하기",
  "hero.terminalNote": "Cargo와 Claude Code 설정 예시입니다. 출력은 일부 생략했습니다.",
  "hero.whaleAlt": "MemoryWhale 고래 마스코트",
  "integrations.label": "함께 쓸 수 있는 도구",
  "integrations.via": "MCP 연동",
  "integrations.more": "전체 통합 보기",
  "release.eyebrow": "0.12.0 새 기능",
  "release.title": "믿고 쓰는 검색.",
  "release.copy":
    "제품 버전 0.12.0은 CLI, 웹 UI, 데스크톱 앱에 공통으로 적용됩니다. 재사용 가능한 Rust 코어는 0.6.0이며, 스토어를 열면 SQLite 스키마 13으로 마이그레이션됩니다. 검색 결과의 종류를 고르고, 서로 어긋나는 메모리를 검토하고, 디버깅 증거를 케이스 파일과 레시피로 남길 수 있습니다.",
  "release.connectTitle": "의도대로 검색",
  "release.connectBody":
    "<code>mw search --mode evidence|lessons|recipes|failures</code>는 결과를 한 종류의 메모리로 좁힙니다. 직접 켜야 하는 <code>--ranking bayesian</code>은 기존 신호를 로그 오즈로 결합하며, 기본 순위는 그대로입니다.",
  "release.provenanceTitle": "믿기 전에 검토",
  "release.provenanceBody":
    "<code>mw feedback</code>은 유용함, 무관함, 오래됨, 모순됨 표시를 로컬에 기록합니다. <code>mw contradictions</code>는 서로 어긋날 수 있는 메모리를 표시하고, 확인할지 기각할지는 사용자가 정합니다. 둘 다 메모리를 변경하지 않으며, 피드백은 아직 순위에 영향을 주지 않습니다.",
  "release.interfaceTitle": "전체 맥락 보존",
  "release.interfaceBody":
    "<code>mw case</code>는 순서대로 실행한 명령을 관찰 내용 및 결론과 함께 묶으며, JSON이나 Markdown으로 내보낼 수 있습니다. <code>mw recipe</code>는 검증된 실행에서 재사용 가능한 명령을 저장합니다. 둘 다 로컬에만 저장되고 민감한 정보는 가려지며, 자동으로 실행되는 것은 없습니다.",
  "who.eyebrow": "사용 대상",
  "who.title": "세 가지 작업 방식을 위해 만들었습니다.",
  "who.copy":
    "MemoryWhale은 디버깅 맥락이 터미널 스크롤백, 셸 히스토리, 여러 머신, 일회성 에이전트 세션에 흩어져 있는 개발자를 위한 도구입니다. 실제 명령 기록과 함께 <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">사용 사례를 처음부터 끝까지</a> 살펴보세요.",
  "who.shellTitle": "🔍 셸 중심으로 디버깅하는 개발자",
  "who.shellBody":
    "같은 빌드, 링커, 의존성 오류를 또 만납니다. 셸 히스토리에는 명령만 남을 뿐 출력도, 오류 마지막 부분도, 해결책도 남지 않습니다. <code>mw search</code>는 예전의 실패 실행<em>과</em> 거기에 연결된 교훈을 함께 찾아 줍니다.",
  "who.multiTitle": "🛰️ 여러 머신을 오가는 개발자",
  "who.multiBody":
    "Jetson, 연구실 서버, 노트북 — 세션은 수시로 끊기고, 머신마다 따로 떨어진 불완전한 기록만 남습니다. <code>mw --live</code>는 연결이 끊겨도 계속 자동 저장하고, <code>mw push</code> / <code>mw pull</code>로 머신 간에 메모리를 명시적으로 옮길 수 있습니다.",
  "who.agentTitle": "🤖 코딩 에이전트 사용자",
  "who.agentBody":
    "Claude Code, Codex, Cursor — 세션을 열 때마다 개발 환경부터 다시 설명해야 합니다. <code>mw-mcp</code>를 쓰면 에이전트가 이전 증거를 조회하고 <code>remember</code>로 교훈을 명시적으로 저장할 수 있습니다. 다만 수정이 실제로 통하는지는 여전히 직접 확인해야 합니다.",
  "features.label": "터미널 작업을 위한 메모리 계층",
  "features.terminalTitle": "터미널 메모리",
  "features.terminalBody": "작업하는 동안 유용한 디버깅 맥락을 캡처합니다.",
  "features.retrievalTitle": "똑똑한 검색",
  "features.retrievalBody": "필요할 때 이전 명령, 실패, 해결책을 다시 찾아 줍니다.",
  "features.localTitle": "완전한 로컬",
  "features.localBody": "MemoryWhale 데이터는 내 머신에만 저장됩니다.",
  "how.eyebrow": "동작 방식",
  "how.title": "캡처, 저장, 추출, 탐색.",
  "how.captureTitle": "캡처",
  "how.captureBody": "터미널 실행 기록을 붙여 넣거나, Rust 헬퍼를 호출하거나, 실시간 자동 저장 셸을 시작합니다.",
  "how.storeTitle": "저장",
  "how.storeBody": "SQLite가 명령 실행 기록과 인수를 내 머신에 로컬로 저장합니다.",
  "how.extractTitle": "추출",
  "how.extractBody": "Rust가 명령, 메모, 오류 텍스트에서 키워드를 뽑아냅니다.",
  "how.exploreTitle": "탐색",
  "how.exploreBody": "빛나는 그래프 화면에서 검색하거나 명령 노드를 클릭해 살펴봅니다.",
  "agents.eyebrow": "AI 에이전트",
  "agents.title": "이미 실패한 일을 에이전트가 기억하게 하세요.",
  "agents.copy":
    "코딩 에이전트는 세션이 바뀌면 컨텍스트를 잃고, 이미 해 본 디버깅을 되풀이하기도 합니다. <code>mw-mcp</code>는 로컬 메모리를 제공하는 Model Context Protocol 서버입니다. 한 번 등록해 두면 Claude Code, Rho, Codex, Cursor가 과거 실패를 직접 조회할 수 있습니다. 조회된 증거는 해당 클라이언트는 물론, 그 클라이언트가 컨텍스트를 보내는 모델 제공자에게도 맡기게 된다는 점을 염두에 두세요.",
  "agents.clientsLabel": "통합 가이드가 있는 클라이언트",
  "agents.matrix": "매트릭스에서 더 보기",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">클라이언트·도구별 설정 가이드</a> — 기능 매트릭스에 클라이언트별 MCP 지원, 자동 캡처, 검증 상태가 정리되어 있으며, OpenRouter나 CLIProxyAPI 같은 모델 게이트웨이도 포함됩니다.",
  "agents.setupLabel": "설정",
  "agents.setupValue": "명령 하나",
  "agents.toolsLabel": "도구",
  "agents.toolsValue": "로컬 MCP 도구 6개: recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "에이전트가 없다면?",
  "agents.noAgentValue": "mw context로 바로 붙여 넣을 수 있는 요약 출력",
  "demo.eyebrow": "캡처 → 메모리 → 검색",
  "demo.title": "합성 데이터로 핵심 흐름을 확인해 보세요.",
  "demo.copy":
    "명령 하나를 캡처하고, 문제를 해결한 설명을 저장한 뒤, 같은 실패가 다시 나타나면 로컬 저장소를 검색하세요. MCP는 조회와 명시적 기록을 지원할 뿐, 평소 터미널 활동을 자동으로 캡처하지는 않습니다.",
  "demo.handoff":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">오프라인 Claude→Rho 인계 데모</a>는 픽스처를 가져온 뒤 실제 MCP를 상대로 Rho 클라이언트를 시뮬레이션합니다. 실제 에이전트를 실행하지 않으며, 픽스처에 담긴 Cargo 수정을 실행하거나 검증하지도 않습니다. 현재 Rho 훅은 명령 텍스트가 없으면 실패 메타데이터를 남기고, 명령 텍스트가 없는 성공 호출은 건너뜁니다. 작업 시작 시 자동 회상, 실패 조회, 컨텍스트 압축 전 저장은 이미 제공되는 자동화가 아니라 클라이언트가 조율해야 할 부분으로 남아 있습니다.",
  "demo.imageAlt": "합성 데이터로 만든 MemoryWhale 터미널 및 대시보드 데모",
  "data.eyebrow": "내 데이터",
  "data.title": "로컬 우선이란, 선택이 눈에 보인다는 뜻입니다.",
  "data.copy":
    "데이터베이스는 내 머신에 있습니다. 보통 Linux에서는 <code>~/.local/share/MemoryWhale/</code>, macOS에서는 <code>~/Library/Application Support/MemoryWhale/</code>입니다. 다른 위치를 쓰려면 <code>MEMORYWHALE_DATA_DIR</code>를 설정하세요.",
  "data.captureLabel": "캡처 제어",
  "data.captureValue": "<code>.mwignore</code>, 경로 정책, 명령만 기록",
  "data.redactionLabel": "민감 정보 마스킹",
  "data.redactionValue": "흔한 시크릿을 가리는 데 도움이 되지만 보안 경계는 아닙니다",
  "data.sizeLabel": "크기 제한",
  "data.sizeValue": "캡처한 텍스트 필드는 기본 1 MiB까지 저장하고 초과분은 잘라냅니다",
  "data.inspectLabel": "조회 / 삭제",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "전송",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code> 또는 명시적인 SSH 전송",
  "data.stewardshipLabel": "관리",
  "data.stewardshipValue": "<code>mw memory compact</code> — 먼저 dry-run으로 확인, 행은 보존",
  "security.eyebrow": "보안 모델",
  "security.title": "기본은 로컬, 공유는 명시적으로.",
  "security.copy":
    "CLI, TUI, MCP 서버, 웹 대시보드, 데스크톱 셸은 모두 로컬 저장소를 사용합니다. <code>mw-mcp</code>는 신뢰하는 로컬 stdio 프로세스이고, 대시보드는 기본적으로 루프백에 바인딩됩니다. 루프백이 아닌 주소로 대시보드를 열려면 토큰이 필요하며, 신뢰할 수 있는 네트워크에서만 노출해야 합니다. 보호된 HTTP MCP는 Bearer 인증이 필요하고, 선택적으로 켜는 JSON API는 대시보드와 같은 접근 제어를 따릅니다. HTTP는 연결을 암호화하지 않습니다. 어느 인터페이스든 클라이언트가 접근한다고 해서 자동 캡처가 되지는 않습니다. 자세한 내용은 <a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">로컬 데이터 위협 모델</a>을 참고하세요.",
  "run.eyebrow": "설치",
  "run.title": "한 줄이면 끝. Rust는 필요 없습니다.",
  "run.copy":
    "Linux x86_64/aarch64와 macOS용 사전 빌드 바이너리를 제공합니다. 설치 스크립트는 릴리스에 게시된 SHA256 파일이 있으면 이를 검증하며, 이전 릴리스에는 체크섬이 없을 수 있습니다. 먼저 명시적으로 캡처를 하나 해 보고 결과를 확인한 다음에 <code>mw global on</code>을 고려하세요. Windows는 네이티브로 지원하지 않으며, WSL에서는 Linux 빌드를 쓸 수 있습니다.",
  "run.tryLabel": "먼저 체험",
  "run.tryValue": "<code>mw demo</code> — 선택한 저장소에 샘플 데이터를 기록합니다",
  "run.prebuiltLabel": "사전 빌드 설치",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.12.1/docs/releases/0.12.1.md#install-or-upgrade\">버전 고정·체크섬 검증 설치 방법</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.12.1 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": "릴리스 페이지의 .deb 패키지",
  "run.securityLabel": "보안",
  "run.securityValue": "<a href=\"#security\">보안 모델 보기</a>",
  "run.verifyLabel": "확인",
  "run.verifyValue": "<code>mw --version</code> · <code>mw doctor</code>",
  "footer.copyright": "Copyright (c) 2026 wuisabel-gif. MemoryWhale - Rust/Tauri 기반 터미널 메모리 및 지식 그래프.",
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
      "MemoryWhale は開発中の証拠をローカルの SQLite に記録し、あなたや信頼できるツールが過去の失敗や教訓を取り出せるようにします。ローカルファーストで、エクスポートや転送は明示的に行います。",
    jsonLdDescription:
      "開発者とコーディングエージェントのための、永続的なローカルデバッグメモリ。ターミナルでの証拠をローカルの SQLite に記録し、MCP 経由で提供します。"
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
  "language.en": "English",
  "language.ar": "العربية",
  "language.de": "Deutsch",
  "language.fr": "Français",
  "language.zh-CN": "简体中文",
  "language.zh-TW": "繁體中文",
  "language.ko": "한국어",
  "language.ja": "日本語",
  "search.openLabel": "このページを検索",
  "search.title": "このページ内を検索",
  "search.label": "ホームページ内を検索",
  "search.placeholder": "例: MCP、インストール、プライバシー…",
  "search.close": "検索を閉じる",
  "search.hint": "このページの見出しと本文を検索します。",
  "search.empty": "一致するセクションはありません。",
  "search.results": "一致したセクション",
  "hero.releaseBadge": "v0.12.0 リリース",
  "hero.title": "ターミナルに<span class=\"hero-accent\">記憶</span>を。",
  "hero.lead":
    "役立つターミナルのコンテキストを手元に残し、コーディングエージェントが過去のセッションのデバッグ知識を引き出せるようにします。",
  "hero.demoCta": "デモを見る",
  "hero.installCta": "クイックスタート",
  "hero.localTitle": "ローカルファースト",
  "hero.localBody": "メモリは手元のマシンに保存",
  "hero.privateTitle": "プライベート",
  "hero.privateBody": "クラウド不要",
  "hero.agentTitle": "コーディングエージェント向け",
  "hero.agentBody": "デバッグの知見がセッションをまたいで残る",
  "hero.terminalTitle": "コーディングエージェントを接続",
  "hero.terminalNote": "Cargo と Claude Code を使ったセットアップ例（出力は一部省略）",
  "hero.whaleAlt": "MemoryWhale のクジラのマスコット",
  "integrations.label": "対応ツール",
  "integrations.via": "MCP 経由",
  "integrations.more": "すべての連携を見る",
  "release.eyebrow": "0.12.0 の新機能",
  "release.title": "信頼できる検索を。",
  "release.copy":
    "製品バージョン 0.12.0 は CLI、Web UI、デスクトップアプリに共通です。再利用可能な Rust コアは 0.6.0 で、ストアを開くと SQLite スキーマ 13 に移行されます。検索で返す内容を選び、食い違うメモリを確認し、デバッグの証拠をケースファイルやレシピとして残せます。",
  "release.connectTitle": "意図どおりに検索",
  "release.connectBody":
    "<code>mw search --mode evidence|lessons|recipes|failures</code> は、結果を 1 種類のメモリに絞り込みます。オプトインの <code>--ranking bayesian</code> は、既存のシグナルを対数オッズとして組み合わせます。デフォルトのランキングは変わりません。",
  "release.provenanceTitle": "信頼する前に確認",
  "release.provenanceBody":
    "<code>mw feedback</code> は、「役に立った」「無関係」「古い」「矛盾している」といった評価をローカルに記録します。<code>mw contradictions</code> は食い違う可能性のあるメモリを示し、確定するか却下するかはあなたが判断します。どちらもメモリ自体は変更せず、フィードバックはまだランキングに影響しません。",
  "release.interfaceTitle": "経緯をまるごと残す",
  "release.interfaceBody":
    "<code>mw case</code> は、順序どおりのコマンド実行を観察内容と結論とともにまとめ、JSON または Markdown でエクスポートできます。<code>mw recipe</code> は、検証済みの実行から再利用可能なコマンドを保存します。どちらもローカルに保存され、機密情報はマスクされます。自動で実行されるものはありません。",
  "who.eyebrow": "対象ユーザー",
  "who.title": "3 つの働き方に合わせて。",
  "who.copy":
    "MemoryWhale は、デバッグのコンテキストがターミナルのスクロールバック、シェル履歴、複数のマシン、使い捨てのエージェントセッションに散らばっている開発者のためのツールです。実際のコマンドログ付きの<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/concepts/use-cases.md\" style=\"color:var(--azure);text-decoration:underline;\">ユースケース解説</a>もご覧ください。",
  "who.shellTitle": "🔍 シェル中心でデバッグする人",
  "who.shellBody":
    "同じビルドエラーやリンカーエラー、依存関係のエラーに二度ぶつかる。シェル履歴に残るのはコマンドだけで、出力もエラーの末尾も、直し方も残りません。<code>mw search</code> なら、以前失敗した実行<em>と</em>、それに紐づく教訓の両方が返ってきます。",
  "who.multiTitle": "🛰️ 複数のマシンを行き来する人",
  "who.multiBody":
    "Jetson、ラボのサーバー、ノート PC。セッションは途中で切れ、各マシンにはそのマシンだけの不完全な履歴が残ります。<code>mw --live</code> は接続が切れても自動保存を続け、<code>mw push</code> / <code>mw pull</code> でマシン間のメモリを明示的に移せます。",
  "who.agentTitle": "🤖 コーディングエージェントを使う人",
  "who.agentBody":
    "Claude Code、Codex、Cursor。セッションのたびに環境の説明からやり直しです。<code>mw-mcp</code> を使えば、エージェントは過去の証拠を参照し、<code>remember</code> で教訓を明示的に保存できます。ただし、修正が本当に効くかどうかは自分で確かめる必要があります。",
  "features.label": "ターミナル作業のためのメモリレイヤー",
  "features.terminalTitle": "ターミナルメモリ",
  "features.terminalBody": "作業しながら、役立つデバッグのコンテキストを記録。",
  "features.retrievalTitle": "インテリジェントな検索",
  "features.retrievalBody": "過去のコマンド、失敗、修正を、必要なときに取り出せます。",
  "features.localTitle": "完全ローカル",
  "features.localBody": "MemoryWhale のデータは自分のマシンに保存。",
  "how.eyebrow": "仕組み",
  "how.title": "キャプチャ、保存、抽出、探索。",
  "how.captureTitle": "キャプチャ",
  "how.captureBody": "ターミナルの実行結果を貼り付ける、Rust ヘルパーを呼び出す、または自動保存付きのライブシェルを起動します。",
  "how.storeTitle": "保存",
  "how.storeBody": "コマンドの実行記録と引数を、SQLite でマシン上にローカル保存します。",
  "how.extractTitle": "抽出",
  "how.extractBody": "コマンド、メモ、エラーテキストから Rust でキーワードを抽出します。",
  "how.exploreTitle": "探索",
  "how.exploreBody": "光るグラフ画面で検索したり、コマンドのノードをクリックしたりできます。",
  "agents.eyebrow": "AI エージェント",
  "agents.title": "すでに失敗したことを、エージェントが覚えておけるように。",
  "agents.copy":
    "コーディングエージェントのセッションはコンテキストを失い、あなたがすでに済ませたデバッグを繰り返すことがあります。<code>mw-mcp</code> は、ローカルメモリ上で動く Model Context Protocol サーバーです。一度登録すれば、Claude Code、Rho、Codex、Cursor から過去の失敗を直接照会できます。クライアントが取得した証拠を扱う以上、そのクライアント（およびクライアントがコンテキストを送信するモデルプロバイダー）を信頼できることが前提です。",
  "agents.clientsLabel": "連携ガイドがあるクライアント",
  "agents.matrix": "マトリクスで詳しく見る",
  "agents.guides":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/integrations/README.md\" style=\"color:var(--azure);text-decoration:underline;\">クライアント・ツール別のセットアップガイド</a> — 機能マトリクスには、クライアントごとの MCP 対応、自動キャプチャ、検証状況がまとめられています。OpenRouter や CLIProxyAPI などのモデルゲートウェイも対象です。",
  "agents.setupLabel": "セットアップ",
  "agents.setupValue": "コマンド 1 つ",
  "agents.toolsLabel": "ツール",
  "agents.toolsValue": "ローカル MCP ツール 6 種: recent_errors · search_memory · get_context · remember · similar_failures · stats",
  "agents.noAgentLabel": "エージェントを使わない場合",
  "agents.noAgentValue": "mw context で貼り付け用のダイジェストを出力",
  "demo.eyebrow": "キャプチャ → メモリ → 検索",
  "demo.title": "合成データでコアのループを体験。",
  "demo.copy":
    "コマンドを 1 つキャプチャし、それを解決した説明を保存しておけば、同じ失敗が再発したときにローカルストアを検索できます。MCP が提供するのは検索と明示的な書き込みであり、通常のターミナル操作を自動でキャプチャするものではありません。",
  "demo.handoff":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/guides/cross-agent-handoff.md\" style=\"color:var(--azure);text-decoration:underline;\">オフラインの Claude → Rho 引き継ぎデモ</a>は、フィクスチャをインポートし、実際の MCP に対して Rho クライアントをシミュレートします。実際のエージェントを動かすことも、フィクスチャに含まれる Cargo の修正を実行・検証することもありません。現在の Rho フックは、コマンド文字列がない場合でも失敗のメタデータを保持し、コマンド文字列のない成功した呼び出しはスキップします。タスク開始時の自動想起、失敗の検索、コンテキスト圧縮前の保存は、同梱の自動化機能ではなく、引き続きクライアント側でオーケストレーションする必要があります。",
  "demo.imageAlt": "MemoryWhale のターミナルとダッシュボードのデモ（合成データ）",
  "data.eyebrow": "あなたのデータ",
  "data.title": "ローカルファーストだから、選択が見える。",
  "data.copy":
    "データベースは手元のマシンにあります。通常、Linux では <code>~/.local/share/MemoryWhale/</code>、macOS では <code>~/Library/Application Support/MemoryWhale/</code> です。別の場所を使うには <code>MEMORYWHALE_DATA_DIR</code> を設定してください。",
  "data.captureLabel": "キャプチャの制御",
  "data.captureValue": "<code>.mwignore</code>、パスポリシー、コマンドのみの記録",
  "data.redactionLabel": "秘匿化",
  "data.redactionValue": "よくある秘密情報の保護に役立ちますが、セキュリティ境界ではありません",
  "data.sizeLabel": "サイズ制限",
  "data.sizeValue": "キャプチャするテキストフィールドは既定で 1 MiB まで（超過分は切り詰め）",
  "data.inspectLabel": "確認 / 削除",
  "data.inspectValue": "<code>mw audit</code> · <code>mw rm</code> · <code>mw prune</code>",
  "data.transferLabel": "転送",
  "data.transferValue": "<code>mw export</code> / <code>mw import</code>、または明示的な SSH 転送",
  "data.stewardshipLabel": "メンテナンス",
  "data.stewardshipValue": "<code>mw memory compact</code> — まず dry-run で確認、行は保持",
  "security.eyebrow": "セキュリティモデル",
  "security.title": "デフォルトはローカル、共有するときは明示的に。",
  "security.copy":
    "CLI、TUI、MCP サーバー、Web ダッシュボード、デスクトップシェルはいずれもローカルストアを使います。<code>mw-mcp</code> は信頼されたローカルの stdio プロセスで、ダッシュボードはデフォルトでループバックにバインドされます。ループバック以外で公開するダッシュボードにはトークンが必要で、公開先は信頼できるネットワークに限ってください。保護された HTTP MCP には Bearer 認証が必要で、オプトインの JSON API はダッシュボードと同じアクセス制御を使います。HTTP では通信は暗号化されません。どちらのインターフェースも、クライアントにアクセスを許可するだけで、自動キャプチャを有効にするものではありません。詳しくは<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/main/docs/SECURITY.md\" style=\"color:var(--azure);text-decoration:underline;\">ローカルデータの脅威モデル</a>をご覧ください。",
  "run.eyebrow": "インストール",
  "run.title": "1 行でインストール。Rust は不要です。",
  "run.copy":
    "Linux x86_64/aarch64 と macOS 向けにビルド済みバイナリを提供しています。リリースで SHA256 ファイルが公開されている場合、インストーラーはそれを使って検証します。古いリリースにはチェックサムがない場合があります。まずは明示的に 1 回キャプチャして中身を確認し、そのうえで <code>mw global on</code> を検討してください。Windows はネイティブではサポートしていませんが、WSL では Linux 版のビルドを使えます。",
  "run.tryLabel": "まずはお試し",
  "run.tryValue": "<code>mw demo</code> — 選択中のストアにサンプルデータを書き込みます",
  "run.prebuiltLabel": "ビルド済みバイナリ",
  "run.prebuiltValue":
    "<a href=\"https://github.com/wuisabel-gif/MemWhale/blob/v0.12.1/docs/releases/0.12.1.md#install-or-upgrade\">バージョン固定・チェックサム検証付きのインストール手順</a>",
  "run.cargoLabel": "Cargo",
  "run.cargoValue": "<code>cargo install memorywhale-cli --version 0.12.1 --locked</code>",
  "run.debianLabel": "Debian / Jetson",
  "run.debianValue": "リリースページの .deb パッケージ",
  "run.securityLabel": "セキュリティ",
  "run.securityValue": "<a href=\"#security\">セキュリティモデルを見る</a>",
  "run.verifyLabel": "動作確認",
  "run.verifyValue": "<code>mw --version</code> · <code>mw doctor</code>",
  "footer.copyright": "Copyright (c) 2026 wuisabel-gif. MemoryWhale - Rust/Tauri 製のターミナルメモリとナレッジグラフ。",
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
