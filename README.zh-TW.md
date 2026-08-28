<!-- README-SOURCE-SHA256: 549ef1e456cdc737af888a321cab2241e5e3cc032eea3f5a4ffe033f129ed530 -->

# MemoryWhale

為開發者與程式設計 Agent 提供持久化、本地優先的除錯記憶。

[English README](README.md) · [简体中文 README](README.zh-CN.md) · [한국어 README](README.ko.md)

MemoryWhale 會記錄你在除錯過程中真正發生過的事情：執行過的指令、輸出結果、錯誤與失敗，以及最後真正有效的解決方法。

這些資訊會儲存在本地 SQLite 資料庫中。即使終端機已經關閉、SSH 連線已經中斷，或 AI Agent 的工作階段已經結束，你和你的程式設計 Agent 仍然可以重新找到這些資訊。

## 為什麼使用 MemoryWhale？

- **記住真正發生過的事情。** 保留當時的指令、執行環境、輸出、錯誤與最後得到的經驗，而不只是一行 Shell History。
- **讓不同的程式設計 Agent 共用同一份記憶。** 任何相容 stdio MCP 的客戶端，都可以透過 `mw-mcp` 讀取同一個本地記憶庫，並透過 `remember` 儲存單一筆記。
- **讓開發歷史留在本地。** MemoryWhale 不需要帳號、不依賴託管服務，也不需要為「記憶」額外支付按 Token 計費的費用。

MemoryWhale 記錄的是開發與除錯經驗，而不是所有資訊。它是一個除錯記憶層（debugging memory layer），不是自主程式設計 Agent、通用型個人記憶系統，也不能取代專案文件。

## 安裝

Linux x86_64/aarch64 與 macOS 提供預先編譯的二進位檔：

```bash
curl -fsSL https://raw.githubusercontent.com/wuisabel-gif/MemWhale/v0.8.0/install.sh | sh
```

這個版本化的安裝腳本會尋找最新的穩定版本，並在發布資產提供 SHA256 檔案時驗證下載內容。若要使用其他版本，請將 URL 中的標籤替換成對應的 release tag。

也可以透過 Cargo 或 Homebrew 安裝：

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

Windows 使用者可以透過 [WSL](https://learn.microsoft.com/windows/wsl/) 執行 MemoryWhale。關於套件安裝、PATH 設定與各平台注意事項，請參閱[快速開始指南](docs/guides/getting-started.md)。

## 60 秒快速上手

```bash
mw global on                         # 開始記錄之後執行的互動式 Shell 指令
mw-run -- cargo check                # 記錄單一指令及其輸出
mw remember "the linker needed libssl-dev"
mw search "linker error"             # 找回之前的錯誤與解決方法
mw context --last-error              # 為任何 Agent 或聊天產生精簡上下文
mw pet                               # 看看你的記憶庫現在是什麼心情
mw pet --watch                       # 讓記憶鯨持續展示狀態動畫
```

對於時間較長的工作，可以使用 `mw --live` 記錄具備崩潰恢復能力的 Shell 工作階段。

`mw tui` 可以開啟互動式終端瀏覽器，而 `mw-serve` 則會啟動本地 Web Dashboard。

## 運作方式

```text
擷取 CAPTURE              記憶 MEMORY                檢索 RETRIEVAL
shell / mw-run ──────► 本地 SQLite ─────────────► search / context
agent hooks ─────────► 除錯證據 + 經驗 ─────────► 相似的歷史錯誤
                                                     │
                                                   介面
                                        CLI / MCP / TUI / Web / Desktop
```

擷取（Capture）與檢索（Retrieval）彼此獨立。

MCP 可以讓 Agent 存取現有記憶，但它不會自動記錄一般終端機中的操作。完整運作模型請參閱 [Architecture](docs/architecture.md) 與 [Capture Concept](docs/concepts/capture.md) 文件。

## 與你的程式設計 Agent 一起使用

`mw-mcp` 是 MemoryWhale 與各種 AI 程式設計工具之間的共用整合介面：一個在本地執行的 stdio MCP Server，提供六個記憶工具：

- `recent_errors`
- `search_memory`
- `get_context`
- `remember`
- `similar_failures`
- `stats`

目前已有整合指南涵蓋 Claude Code、Claude Desktop、Cursor、VS Code / GitHub Copilot、Windsurf、Zed、Codex CLI、Cline、Continue、Gemini CLI、Goose、OpenClaw、CrowClaw、Hermes Agent，以及其他相容的客戶端。整合矩陣目前包含 24 個客戶端與工具條目，並會標示各條目提供的功能。

例如 Claude Code：

```bash
claude mcp add memorywhale -- mw-mcp
```

不同客戶端提供的能力並不完全相同。MCP 負責存取記憶；若要自動擷取指令執行過程，則需要針對特定客戶端設定 Hook。

[Integration Matrix](integrations/README.md) 會區分 Memory Access、Automatic Capture 與 Memory-use Guidance，並提供每個客戶端的設定指南。

## MemoryWhale 適合誰？

MemoryWhale 適合那些除錯上下文經常散落在終端機捲動紀錄、Shell History、不同開發機器或暫時性 Agent 工作階段的開發者。

如果你經常：

- 除錯建置、相依套件、Git、開發環境或部署問題；
- 跨多個工作階段使用程式設計 Agent，或在不同工具之間切換；
- 透過 SSH 工作，或在多台開發機器之間切換；
- 希望重複出現的錯誤及其解決方法能夠持續被搜尋；
- 比起託管式記憶服務，更偏好將資料儲存在本地；

那麼 MemoryWhale 會特別適合你。

請參閱 [Use Cases](docs/concepts/use-cases.md)，查看這些情境的完整端到端流程與實際指令。

## 文件

- [Documentation Map](docs/README.md)
- [快速開始](docs/guides/getting-started.md)
- [`mw pet` Reference](docs/reference/pet.md)
- [Terminal Capture](docs/guides/terminal-capture.md)
- [Agent Memory](docs/guides/agent-memory.md)
- [CLI Reference](docs/reference/cli.md)
- [MCP Reference](docs/reference/mcp.md)
- [Memory Compaction](docs/reference/compaction.md)
- [Security and Local Threat Model](docs/SECURITY.md)
- [Integration Guides and Capability Matrix](integrations/README.md)

## 參與貢獻

MemoryWhale 歡迎任何能改善開發經驗擷取、保存、檢索或共享方式的貢獻。

請閱讀 [CONTRIBUTING.md](CONTRIBUTING.md)，了解專案的貢獻範圍、開發指令以及 Pull Request Checklist。

MemoryWhale 採用 [MIT License](LICENSE) 授權。
