<!-- README-SOURCE-SHA256: 693301b6a0450efa434e88f08ce48396310ee90b85790c7269a5f1abaffcb523 -->

<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="MemoryWhale 標誌" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>給開發者與程式開發代理使用、可長期保存的本機除錯記憶。</strong></p>

<p align="center" dir="ltr"><a href="README.md">English README</a> · <a href="README.ar.md" lang="ar">العربية</a> · <a href="README.de.md" lang="de">Deutsch</a> · <a href="README.fr.md">README français</a> · <a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="發行版本"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="MIT 授權"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="本機優先，不上傳任何資料"/>
</p>

MemoryWhale 會記下你除錯時實際發生的事：執行過的指令、輸出、失敗，以及最後真正有效的修正。
這些證據存在本機的 SQLite 裡，就算終端機關了、SSH 斷了，或代理的工作階段已經結束，
你和你的程式開發代理之後都還找得到。

**MemoryWhale 0.11.0 — Explainable Debugging Evidence · 2026 年 9 月 20 日。**
CLI、Web 介面與桌面應用程式共用同一個產品版本 0.11.0；可重複使用的 Rust 核心則是 0.5.0。
升級指南請見[版本說明](https://github.com/wuisabel-gif/MemWhale/blob/v0.11.0/docs/releases/0.11.0.md)。
資料庫 schema 版本維持 10。

**想參與貢獻嗎？** 可以先從 [Start here issue](https://github.com/wuisabel-gif/MemWhale/issues/317) 開始。
很多工作完全不需要會 Rust，例如審閱翻譯、修正文件。

## 為什麼選擇 MemoryWhale

- **記住實際發生過的事。** 保留指令、環境、輸出、失敗與學到的經驗，而不只是 Shell 歷史裡的一行指令。
- **多個程式開發代理共用同一份記憶。** 任何相容的 stdio MCP 用戶端，都能透過 `mw-mcp` 讀寫同一份本機記憶。
- **開發歷史留在本機。** 使用 MemoryWhale 不需要帳號、不需要託管服務，也不會有按 token 計費的記憶費用。

MemoryWhale 記錄的是開發經驗，而不是所有東西。它是一層除錯記憶，不是自主運作的程式開發代理、
通用的個人記憶系統，也不能取代專案文件。

## Agent-Native Memory 新功能

- **連接並檢查代理。** 用 `mw integrate` 為 Claude Code 或 Rho 安裝 MCP 存取、
  擷取 hook 與記憶使用指引；`mw doctor` 會分別檢查 MCP、hook 與 skill。
- **來源資訊一清二楚。** Schema 10 將指令的代理欄位存為 `claude`、`rho` 或 `NULL`。
  顯示與篩選用的標籤 `terminal` 代表來源是終端機／手動操作或舊版資料，並不能證明指令是由人執行的。
  代理身分和 `command`、`session`、`note` 這類來源類型是分開記錄的。
- **同一個儲存庫，分得出不同 worktree。** 標準儲存庫 ID 會把連結的 worktree 歸在一起，同時保留各 worktree 的根目錄與既有的專案標籤。
  偵測時只讀取本機的 Git 中繼資料，不會連到遠端服務。
- **使用本機介面。** `mw-serve` 在 `POST /mcp` 提供 HTTP MCP；`mw-serve --api` 則可選擇啟用唯讀 JSON API。
  兩者都沿用儀表板的監聽位址；從非 loopback 位址存取時需要權杖。
- **明確地取得 GitHub 脈絡。** `mw github context <pr>` 會透過你現有的 `gh` 登入，讀取 PR 中繼資料、檢查結果與審查意見。
  它輸出有長度上限且已遮蔽敏感資訊的脈絡，不會 checkout 程式碼，也不會自動存進記憶。不會在背景同步 GitHub。

## 安裝

提供 Linux x86_64/aarch64 與 macOS 的預先編譯二進位檔：

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

也可以用 Cargo 或 Homebrew 安裝：

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

安裝或升級完成後，確認版本與本機設定：

```bash
mw --version
mw doctor
```

Windows 使用者可以在 [WSL](https://learn.microsoft.com/windows/wsl/) 裡執行 MemoryWhale。
套件安裝、PATH 設定與各平台注意事項，請見[入門指南](docs/guides/getting-started.md)。

## 60 秒上手範例

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

![mw pet 心情示範](assets/pet-demo.gif)

需要長時間工作時，`mw --live` 會錄下不怕當機中斷的 Shell 工作階段。`mw tui`
會開啟互動式的終端機瀏覽介面，`mw-serve` 則會啟動本機 Web 儀表板。

## 運作方式

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

擷取與檢索是各自獨立的。MCP 讓代理能存取既有的記憶，但不會自動記錄一般的終端機操作。
完整架構請見[架構說明](docs/architecture.md)與[擷取概念](docs/concepts/capture.md)。

## 搭配你的程式開發代理

`mw-mcp` 是共通的整合接口：一個提供六個記憶工具的本機 stdio MCP 伺服器，
也可以透過 `mw-serve` 以 HTTP 存取。目前已有的指南涵蓋 Claude Code、Rho、Claude Desktop、
Cursor、VS Code / GitHub Copilot、Windsurf、Zed、Codex CLI、Cline、Continue、
Gemini CLI、Goose、OpenClaw、CrowClaw、Hermes Agent，以及其他相容的用戶端。

```bash
mw integrate claude
mw integrate rho
mw doctor
```

每個用戶端能做到的事不盡相同。MCP 提供的是記憶存取；要自動擷取指令執行，需要該用戶端專屬的 hook。
[整合對照表](integrations/README.md)分別列出存取、擷取與記憶使用指引的支援情況，並附上每份已驗證設定指南的連結。

Rho 目前的 hook payload 不含指令文字與 stdout：失敗會以中繼資料加上一個佔位指令的方式記錄；
沒有指令文字的成功呼叫則會略過。[跨代理交接示範](docs/guides/cross-agent-handoff.md)
使用 fixture 與模擬的 Rho 用戶端連接真實的 MCP，並不是實際運作中的代理，也不是經過驗證的 Cargo 修正。

內附的 skill 會引導代理如何使用記憶，但並未實作任務開始時自動回想、失敗時自動查詢，或在壓縮前自動儲存。
這些生命週期上的決定仍由用戶端負責。透過 MCP 寫入的經驗預設為待審核狀態。

## MemoryWhale 適合誰？

如果你的除錯脈絡散落在終端機捲動紀錄、Shell 歷史、不同機器，以及用完即丟的代理工作階段裡，
MemoryWhale 正是為你這樣的開發者而做。以下情況特別適用：

- 需要排查建置、相依套件、Git、環境或部署問題；
- 在多個工作階段中使用程式開發代理，或在不同工具之間切換；
- 透過 SSH 工作，或在多台開發機器之間來回；
- 希望反覆出現的失敗和對應的修正都能搜尋得到；
- 比起託管的記憶服務，更偏好存在本機。

[使用情境](docs/concepts/use-cases.md)把上面每一種情況都寫成完整的端對端情境，並附上實際指令。

## 文件

- [文件總覽](docs/README.md)
- [入門指南](docs/guides/getting-started.md)
- [`mw pet` 參考文件](docs/reference/pet.md)
- [終端機擷取](docs/guides/terminal-capture.md)
- [代理記憶](docs/guides/agent-memory.md)
- [CLI 參考文件](docs/reference/cli.md)
- [本機 JSON API](docs/reference/api.md)
- [MCP 參考文件](docs/reference/mcp.md)
- [安全性與本機威脅模型](docs/SECURITY.md)
- [生態系](ECOSYSTEM.md) — Delphin、ContextGC 與 MemoryWhale 如何搭配使用
- [整合指南與功能對照表](integrations/README.md)

## 參與貢獻

MemoryWhale 接受能改善開發經驗的擷取、保存、檢索或分享的變更。範圍規則、開發指令與 pull request 檢查清單，
請見 [CONTRIBUTING.md](CONTRIBUTING.md)。新的貢獻者可以從 [Start here issue](https://github.com/wuisabel-gif/MemWhale/issues/317) 挑一個任務開始。

每個 pull request 也會收到 [Second-Opinion](https://github.com/wuisabel-gif/second-opinion) 的自動審查。它是同一位維護者的姊妹專案，一個 GitHub Action。
審查意見僅供參考，是否採納由人決定。設定方式與限制請見 [Second-Opinion 指南](integrations/second-opinion/README.md)。

以 [MIT 授權](LICENSE)釋出。
