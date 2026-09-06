<!-- README-SOURCE-SHA256: 0c0dee340943c6650cc58749d96cb86c7728e6fc31cef1f4daba0be190a01965 -->

<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="MemoryWhale 標誌" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>為開發者與程式設計代理提供持久的本機除錯記憶。</strong></p>

<p align="center"><a href="README.md">English README</a> · <a href="README.fr.md">README français</a> · <a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="發行版本"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="MIT 授權"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="本機優先，不上傳資料"/>
</p>

MemoryWhale 記錄除錯時真正發生過的事：指令、輸出、失敗，以及最後有效的修正。
這些證據儲存在本機 SQLite 中，即使終端機已關閉、SSH 已中斷，或代理工作階段已結束，
你和程式設計代理仍能找回它們。

**MemoryWhale 0.10.0 — Agent-Native Memory · 2026 年 9 月 6 日。**
CLI、Web 介面與桌面應用程式統一採用產品版本 0.10.0；可重用的 Rust 核心版本為 0.5.0。
升級指南與 Rust API 的不相容變更請參閱[發行說明](https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md)。

## 為什麼選擇 MemoryWhale

- **記住真正發生過的事。** 保留指令、環境、輸出、失敗與經驗，而不只是一行 Shell 歷史。
- **讓不同程式設計代理共用記憶。** 任何相容的 stdio MCP 用戶端都能透過 `mw-mcp` 讀寫同一份本機記憶。
- **讓開發歷史留在本機。** MemoryWhale 不需要帳號、代管服務或按 Token 計費的記憶服務。

MemoryWhale 記錄開發經驗，而非所有資訊。它是除錯記憶層，不是自主程式設計代理、
通用個人記憶系統，也不能取代專案文件。

## Agent-Native Memory 的新功能

- **連接並檢查代理。** 透過 `mw integrate` 安裝 Claude Code 或 Rho 的 MCP 存取、
  擷取掛鉤與記憶使用指引；`mw doctor` 分別檢查 MCP、掛鉤和技能。
- **明確保留來源資訊。** 資料庫結構版本 10 將指令的代理欄位儲存為 `claude`、`rho` 或 `NULL`。
  顯示及篩選標籤 `terminal` 表示終端機、手動或舊記錄來源，並不能證明指令由人類執行。
  代理身分獨立於 `command`、`session` 或 `note` 等資料來源類型。
- **共用儲存庫身分，區分工作樹。** 標準化儲存庫 ID 將關聯工作樹歸為一組，同時保留各工作樹根目錄與既有專案標籤。
  探索過程讀取本機 Git 中繼資料，不存取遠端服務。
- **使用本機介面。** `mw-serve` 在 `POST /mcp` 提供 HTTP MCP；`mw-serve --api` 明確啟用唯讀 JSON API。
  兩者共用儀表板的接聽器；非回環存取需要權杖。
- **明確取得 GitHub 上下文。** `mw github context <pr>` 透過現有的 `gh` 登入讀取 PR 中繼資料、檢查結果與審查。
  它輸出經過大小限制與敏感資訊遮蔽的上下文，不簽出程式碼，也不自動儲存到記憶中。沒有背景 GitHub 同步。

## 安裝

Linux x86_64/aarch64 和 macOS 提供預先編譯的二進位檔：

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

也可以透過 Cargo 或 Homebrew 安裝：

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

安裝或升級後，檢查版本與本機設定：

```bash
mw --version
mw doctor
```

Windows 使用者可以在 [WSL](https://learn.microsoft.com/windows/wsl/) 中執行 MemoryWhale。
套件安裝、PATH 設定與平台說明請參閱[入門指南](docs/guides/getting-started.md)。

## 60 秒範例

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

![mw pet 心情示範](assets/pet-demo.gif)

較長的工作可以用 `mw --live` 記錄具備當機復原能力的 Shell 工作階段。
`mw tui` 開啟互動式終端機瀏覽器，`mw-serve` 啟動本機 Web 儀表板。

## 運作方式

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

擷取與檢索彼此獨立。MCP 讓代理存取既有記憶，不會自動記錄一般終端機活動。
完整模型請參閱[架構](docs/architecture.md)與[擷取概念](docs/concepts/capture.md)。

## 與程式設計代理搭配使用

`mw-mcp` 是共用的整合介面：一個提供六個記憶工具的本機 stdio MCP 伺服器，
也能透過 `mw-serve` 以 HTTP 存取。現有指南涵蓋 Claude Code、Rho、Claude Desktop、
Cursor、VS Code / GitHub Copilot、Windsurf、Zed、Codex CLI、Cline、Continue、
Gemini CLI、Goose、OpenClaw、CrowClaw、Hermes Agent 以及其他相容用戶端。

```bash
mw integrate claude
mw integrate rho
mw doctor
```

並非所有用戶端都具備相同能力。MCP 提供記憶存取；自動執行擷取需要用戶端專用掛鉤。
[整合矩陣](integrations/README.md)區分存取、擷取與記憶使用指引，並連結各個已驗證的設定指南。

Rho 目前的掛鉤載荷缺少指令文字與 stdout：失敗可用中繼資料配合佔位指令記錄；
沒有指令文字的成功呼叫會被略過。[跨代理交接示範](docs/guides/cross-agent-handoff.md)
使用測試樣本與模擬的 Rho 用戶端連接真正的 MCP，並非執行真正的代理或驗證 Cargo 修正。

內建技能提供記憶使用指引，但未實作任務開始時自動回憶、失敗時自動查找或壓縮前自動儲存。
這些生命週期決策仍由用戶端負責。透過 MCP 撰寫的經驗預設處於待審查狀態。

## MemoryWhale 適合誰？

MemoryWhale 適合除錯上下文散落在終端機捲動記錄、Shell 歷史、不同機器與臨時代理工作階段中的開發者。
如果你經常遇到下列情況，它會特別有用：

- 除錯建置、相依套件、Git、環境或部署問題；
- 跨工作階段使用程式設計代理，或切換工具；
- 透過 SSH 或在多台開發機器之間工作；
- 希望重複出現的失敗及其修正保持可搜尋；
- 偏好本機儲存，而不是代管記憶服務。

[使用情境](docs/concepts/use-cases.md)提供各情境的端到端流程與實際指令。

## 文件

- [文件地圖](docs/README.md)
- [入門指南](docs/guides/getting-started.md)
- [`mw pet` 參考](docs/reference/pet.md)
- [終端機擷取](docs/guides/terminal-capture.md)
- [代理記憶](docs/guides/agent-memory.md)
- [CLI 參考](docs/reference/cli.md)
- [本機 JSON API](docs/reference/api.md)
- [MCP 參考](docs/reference/mcp.md)
- [安全與本機威脅模型](docs/SECURITY.md)
- [生態系統](ECOSYSTEM.md) — Delphin、ContextGC 與 MemoryWhale 協作
- [整合指南與能力矩陣](integrations/README.md)

## 參與貢獻

MemoryWhale 接受能改善開發經驗擷取、保存、檢索或分享的變更。
請閱讀 [CONTRIBUTING.md](CONTRIBUTING.md)，了解範圍規則、開發指令與提取要求檢查清單。

採用 [MIT 授權](LICENSE)。
