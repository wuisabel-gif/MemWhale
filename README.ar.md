<!-- README-SOURCE-SHA256: 0a6b911696113a6221346fd4145b26a73f5a62493d8a8d34048d26a832c0c100 -->

<div dir="rtl">

<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="شعار MemoryWhale" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>ذاكرة محلية ودائمة لتصحيح الأخطاء، للمطورين ووكلاء البرمجة.</strong></p>

<p align="center" dir="ltr"><a href="README.md">English README</a> · <a href="README.ar.md" lang="ar">العربية</a> · <a href="README.de.md" lang="de">Deutsch</a> · <a href="README.fr.md">README français</a> · <a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="التكامل المستمر"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="الإصدار"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="رخصة MIT"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="محلي أولاً، دون رفع تلقائي"/>
</p>

يسجّل MemoryWhale ما حدث فعلاً أثناء تصحيح الأخطاء: الأوامر، ومخرجاتها،
والإخفاقات، والإصلاحات التي نجحت. ويحتفظ بهذه الأدلة في قاعدة SQLite محلية
لتتمكن أنت ووكلاء البرمجة من استرجاعها بعد إغلاق الطرفية أو انقطاع اتصال SSH
أو انتهاء جلسة الوكيل.

**MemoryWhale 0.10.0 — Agent-Native Memory · 6 سبتمبر 2026.**
تشترك واجهة سطر الأوامر وواجهة الويب وتطبيق سطح المكتب في إصدار المنتج 0.10.0،
أما نواة Rust القابلة لإعادة الاستخدام فإصدارها 0.5.0. راجع
[ملاحظات الإصدار](https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md)
للاطلاع على دليل الترقية والتغيير غير المتوافق في واجهة Rust البرمجية.

## لماذا MemoryWhale؟

- **تذكّر ما حدث فعلاً.** احتفظ بالأمر والبيئة والمخرجات والخطأ والدرس المستفاد،
  لا بمجرد سطر في سجل أوامر الصدفة.
- **استخدم ذاكرة مشتركة بين وكلاء البرمجة.** يستطيع أي عميل متوافق مع MCP عبر
  stdio القراءة من الذاكرة المحلية نفسها والكتابة فيها باستخدام `mw-mcp`.
- **احتفظ بسجل التطوير محلياً.** يعمل MemoryWhale دون حساب أو خدمة مستضافة
  أو رسوم على تخزين الذاكرة محسوبة بعدد الرموز.

يسجّل MemoryWhale خبرة التطوير، وليس كل شيء. إنه طبقة ذاكرة لتصحيح الأخطاء،
وليس وكيل برمجة مستقلاً أو نظاماً عاماً للذاكرة الشخصية أو بديلاً عن توثيق المشروع.

## الجديد في Agent-Native Memory

- **اربط الوكلاء وتحقق من إعدادهم.** ثبّت وصول MCP وخطافات التسجيل وإرشادات
  استخدام الذاكرة لـ Claude Code أو Rho باستخدام `mw integrate`؛ ويفحص
  `mw doctor` كلاً من MCP والخطافات والمهارات بصورة مستقلة.
- **احتفظ بمصدر واضح للأدلة.** يخزّن مخطط قاعدة البيانات 10 هوية وكيل الأمر
  بالقيم `claude` أو `rho` أو `NULL`. يشير وسم العرض والتصفية `terminal`
  إلى تسجيل طرفي أو يدوي أو قديم، ولا يثبت أن إنساناً نفّذ الأمر. وهوية الوكيل
  مستقلة عن نوع المصدر، مثل `command` أو `session` أو `note`.
- **شارك المستودع مع التمييز بين أشجار العمل.** تجمع معرّفات المستودعات
  الموحّدة أشجار العمل المرتبطة مع الحفاظ على جذر كل شجرة ووسوم المشروع الحالية.
  تعتمد عملية الاكتشاف على بيانات Git الوصفية المحلية، لا على خدمة بعيدة.
- **استخدم واجهات محلية.** يوفّر `mw-serve` واجهة MCP عبر HTTP عند
  `POST /mcp`؛ ويُفعّل `mw-serve --api` واجهة JSON للقراءة فقط بصورة صريحة.
  تستخدم الواجهتان مستمع لوحة التحكم نفسه، ويتطلب الوصول من خارج عنوان
  الحلقة المحلية رمز مصادقة.
- **استرجع سياق GitHub عند الطلب.** يقرأ `mw github context <pr>` بيانات
  طلب الدمج وفحوصاته ومراجعاته باستخدام تسجيل الدخول الحالي في `gh`. ويعرض
  سياقاً محدود الحجم مع تنقيح البيانات الحساسة التي يتعرف عليها، دون استخراج
  الشيفرة من المستودع أو حفظ السياق تلقائياً في الذاكرة. لا توجد مزامنة GitHub
  تعمل في الخلفية.

## التثبيت

تتوفر ملفات تنفيذية جاهزة لنظام Linux بمعماريتي x86_64 وaarch64 ولنظام macOS:

<div dir="ltr">

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

</div>

أو ثبّته باستخدام Cargo أو Homebrew:

<div dir="ltr">

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

</div>

بعد التثبيت أو الترقية، تحقّق من الإصدار والإعداد المحلي:

<div dir="ltr">

```bash
mw --version
mw doctor
```

</div>

يمكن لمستخدمي Windows تشغيل MemoryWhale داخل
[WSL](https://learn.microsoft.com/windows/wsl/). راجع
[دليل البدء](docs/guides/getting-started.md) لمعرفة كيفية تثبيت الحزم وإعداد
PATH والملاحظات الخاصة بكل منصة.

## مثال في ستين ثانية

<div dir="ltr">

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

</div>

![عرض توضيحي لحالات mw pet](assets/pet-demo.gif)

للعمل لفترات أطول، يسجّل `mw --live` جلسة الصدفة مع الحفظ المستمر لتقليل
فقدان البيانات عند التعطّل. ويفتح `mw tui` متصفحاً تفاعلياً داخل الطرفية،
بينما يشغّل `mw-serve` لوحة التحكم المحلية على الويب.

## كيف يعمل؟

<div dir="ltr">

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

</div>

التسجيل والاسترجاع وظيفتان مستقلتان. يمنح MCP الوكيل وصولاً إلى الذاكرة
الموجودة؛ لكنه لا يسجّل نشاط الطرفية المعتاد تلقائياً. راجع
[البنية المعمارية](docs/architecture.md) و[مفهوم التسجيل](docs/concepts/capture.md)
لفهم النموذج كاملاً.

## يعمل مع وكيل البرمجة الذي تستخدمه

يمثّل `mw-mcp` نقطة التكامل المشتركة: خادم MCP محلي عبر stdio يوفّر ست أدوات
للذاكرة، ويمكن الوصول إليها أيضاً عبر HTTP باستخدام `mw-serve`. تتوفر أدلة
لـ Claude Code وRho وClaude Desktop وCursor وVS Code / GitHub Copilot وWindsurf
وZed وCodex CLI وCline وContinue وGemini CLI وGoose وOpenClaw وCrowClaw
وHermes Agent وغيرها من العملاء المتوافقين.

<div dir="ltr">

```bash
mw integrate claude
mw integrate rho
mw doctor
```

</div>

لا يوفّر جميع العملاء الإمكانات نفسها. يتيح MCP الوصول إلى الذاكرة، أما
التسجيل التلقائي للأوامر المنفّذة فيحتاج إلى خطاف خاص بالعميل. تميّز
[مصفوفة التكامل](integrations/README.md) بين الوصول والتسجيل وإرشادات استخدام
الذاكرة، وتربط بكل دليل إعداد تم التحقق منه.

لا تتضمن حمولة خطافات Rho الحالية نص الأمر أو stdout: يمكن تسجيل الإخفاقات
كبيانات وصفية مع أمر بديل يدل على غياب النص، بينما تُتجاوز الاستدعاءات الناجحة
التي لا تحتوي على نص الأمر. يستخدم
[عرض تسليم الذاكرة بين الوكلاء](docs/guides/cross-agent-handoff.md)
بيانات اختبار وعميل Rho محاكياً مع خادم MCP فعلي، لا وكلاء يعملون فعلياً
ولا إصلاح Cargo تم تنفيذه والتحقق منه.

توجّه المهارة المرفقة استخدام الذاكرة؛ لكنها لا تنفّذ استرجاعاً تلقائياً عند
بدء المهمة أو بحثاً تلقائياً عن الإخفاقات أو حفظاً قبل ضغط السياق. تبقى قرارات
دورة الحياة هذه مسؤولية العميل. وتنتظر الدروس التي تُكتب عبر MCP المراجعة
افتراضياً.

## لمن صُمّم MemoryWhale؟

MemoryWhale مخصص للمطورين الذين يتوزع سياق تصحيح الأخطاء لديهم بين مخرجات
الطرفية وسجل الصدفة والأجهزة وجلسات الوكلاء المؤقتة. ويكون مفيداً خصوصاً إذا كنت:

- تصلح مشكلات البناء أو الاعتماديات أو Git أو البيئات أو النشر؛
- تستخدم وكلاء البرمجة عبر جلسات متعددة أو تنتقل بين أدوات مختلفة؛
- تعمل عبر SSH أو على عدة أجهزة تطوير؛
- تريد إبقاء الإخفاقات المتكررة وإصلاحاتها قابلة للبحث؛
- تفضّل التخزين المحلي على خدمة ذاكرة مستضافة.

تعرّف على [حالات الاستخدام](docs/concepts/use-cases.md) لرؤية كل سيناريو
من البداية إلى النهاية مع أوامر فعلية.

## التوثيق

- [خريطة التوثيق](docs/README.md)
- [البدء](docs/guides/getting-started.md)
- [مرجع `mw pet`](docs/reference/pet.md)
- [تسجيل الطرفية](docs/guides/terminal-capture.md)
- [ذاكرة الوكلاء](docs/guides/agent-memory.md)
- [مرجع واجهة سطر الأوامر](docs/reference/cli.md)
- [واجهة JSON المحلية](docs/reference/api.md)
- [مرجع MCP](docs/reference/mcp.md)
- [الأمان ونموذج التهديدات المحلي](docs/SECURITY.md)
- [منظومة الأدوات](ECOSYSTEM.md) — Delphin وContextGC وMemoryWhale معاً
- [أدلة التكامل ومصفوفة الإمكانات](integrations/README.md)

## المساهمة

يرحّب MemoryWhale بالتغييرات التي تحسّن تسجيل خبرات التطوير أو حفظها أو
استرجاعها أو مشاركتها. اقرأ [CONTRIBUTING.md](CONTRIBUTING.md) لمعرفة نطاق
المشروع وأوامر التطوير وقائمة التحقق الخاصة بطلبات الدمج.

متاح بموجب [رخصة MIT](LICENSE).

</div>
