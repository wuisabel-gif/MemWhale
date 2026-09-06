<!-- README-SOURCE-SHA256: c177f5ebba1899016da1a16ac5f7382f2cb7c39b0d2b0a8cfa73aa0fccf46aec -->

<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="MemoryWhale 标志" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>为开发者和编程智能体提供持久化的本地调试记忆。</strong></p>

<p align="center"><a href="README.md">English README</a> · <a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="发布版本"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="MIT 许可证"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="本地优先，不上传数据"/>
</p>

MemoryWhale 记录调试时真正发生过的事情：命令、输出、失败，以及最终奏效的修复。
这些证据保存在本地 SQLite 中，即使终端已关闭、SSH 已断开或智能体会话已结束，
你和编程智能体仍然可以找回它们。

**MemoryWhale 0.10.0 — Agent-Native Memory · 2026 年 9 月 6 日。**
CLI、Web 界面和桌面应用统一使用产品版本 0.10.0；可复用的 Rust 核心版本为 0.5.0。
升级指南和 Rust API 的不兼容变更请参阅[发布说明](https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md)。

## 为什么选择 MemoryWhale

- **记住真正发生过的事情。** 保存命令、环境、输出、失败与经验，而不只是一行 Shell 历史。
- **让不同编程智能体共享记忆。** 任何兼容 stdio MCP 的客户端都可以通过 `mw-mcp` 读写同一份本地记忆。
- **让开发历史留在本地。** MemoryWhale 无需账号、托管服务或按 Token 计费的记忆服务。

MemoryWhale 记录开发经验，而不是所有信息。它是调试记忆层，不是自主编程智能体、
通用个人记忆系统，也不能替代项目文档。

## Agent-Native Memory 的新功能

- **连接并检查智能体。** 使用 `mw integrate` 安装 Claude Code 或 Rho 的 MCP 访问、
  采集钩子和记忆使用指引；`mw doctor` 分别检查 MCP、钩子和技能。
- **明确标注来源。** 数据库结构版本 10 将命令的智能体字段存为 `claude`、`rho` 或 `NULL`。
  显示和筛选标签 `terminal` 表示终端、手动或旧记录来源，并不能证明命令由人类执行。
  智能体身份独立于 `command`、`session` 或 `note` 等数据来源类型。
- **共享仓库身份，区分工作树。** 规范化仓库 ID 将关联工作树归为一组，同时保留各工作树根目录和已有项目标签。
  发现过程读取本地 Git 元数据，不访问远程服务。
- **使用本地接口。** `mw-serve` 在 `POST /mcp` 提供 HTTP MCP；`mw-serve --api` 显式启用只读 JSON API。
  两者共用仪表盘的监听器；非回环访问需要令牌。
- **显式获取 GitHub 上下文。** `mw github context <pr>` 通过现有 `gh` 登录读取 PR 元数据、检查结果和评审。
  它输出经过限量和脱敏处理的上下文，不检出代码，也不自动保存到记忆中。没有后台 GitHub 同步。

## 安装

Linux x86_64/aarch64 和 macOS 提供预编译二进制文件：

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

也可以通过 Cargo 或 Homebrew 安装：

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

安装或升级后，检查版本和本地配置：

```bash
mw --version
mw doctor
```

Windows 用户可以在 [WSL](https://learn.microsoft.com/windows/wsl/) 中运行 MemoryWhale。
软件包安装、PATH 设置和平台说明请参阅[入门指南](docs/guides/getting-started.md)。

## 60 秒示例

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

![mw pet 状态演示](assets/pet-demo.gif)

对于较长的工作，`mw --live` 可以记录具备崩溃恢复能力的 Shell 会话。
`mw tui` 打开交互式终端浏览器，`mw-serve` 启动本地 Web 仪表盘。

## 工作原理

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

采集与检索相互独立。MCP 让智能体访问已有记忆，并不会自动记录普通终端活动。
完整模型请参阅[架构](docs/architecture.md)和[采集概念](docs/concepts/capture.md)。

## 与编程智能体配合使用

`mw-mcp` 是统一的集成接口：一个本地 stdio MCP 服务器，提供六个记忆工具，
也可通过 `mw-serve` 以 HTTP 方式访问。现有指南涵盖 Claude Code、Rho、Claude Desktop、
Cursor、VS Code / GitHub Copilot、Windsurf、Zed、Codex CLI、Cline、Continue、
Gemini CLI、Goose、OpenClaw、CrowClaw、Hermes Agent 以及其他兼容客户端。

```bash
mw integrate claude
mw integrate rho
mw doctor
```

并非所有客户端都具备相同能力。MCP 提供记忆访问；自动执行采集需要客户端专用钩子。
[集成矩阵](integrations/README.md)区分访问、采集和记忆使用指引，并链接到各个已验证的设置指南。

Rho 当前的钩子载荷缺少命令文本和 stdout：失败可作为元数据配合占位命令记录；
没有命令文本的成功调用会被跳过。[跨智能体交接演示](docs/guides/cross-agent-handoff.md)
使用测试样例和模拟的 Rho 客户端连接真实 MCP，并非运行真实智能体或验证 Cargo 修复。

内置技能指导记忆使用，但没有实现任务开始时自动回忆、失败时自动查找或压缩前自动保存。
这些生命周期决策仍由客户端负责。通过 MCP 编写的经验默认处于待审核状态。

## MemoryWhale 适合谁？

MemoryWhale 面向调试上下文分散在终端滚动记录、Shell 历史、不同机器和临时智能体会话中的开发者。
如果你经常遇到以下情况，它会特别有用：

- 调试构建、依赖、Git、环境或部署；
- 跨会话使用编程智能体，或切换工具；
- 通过 SSH 或在多台开发机器之间工作；
- 希望反复出现的失败及其修复一直可搜索；
- 更倾向本地存储，而不是托管记忆服务。

[使用场景](docs/concepts/use-cases.md)展示了这些场景的端到端流程和实际命令。

## 文档

- [文档地图](docs/README.md)
- [入门指南](docs/guides/getting-started.md)
- [`mw pet` 参考](docs/reference/pet.md)
- [终端采集](docs/guides/terminal-capture.md)
- [智能体记忆](docs/guides/agent-memory.md)
- [CLI 参考](docs/reference/cli.md)
- [本地 JSON API](docs/reference/api.md)
- [MCP 参考](docs/reference/mcp.md)
- [安全与本地威胁模型](docs/SECURITY.md)
- [生态系统](ECOSYSTEM.md) — Delphin、ContextGC 与 MemoryWhale 协作
- [集成指南与能力矩阵](integrations/README.md)

## 参与贡献

MemoryWhale 接受能改善开发经验采集、保存、检索或共享的变更。
请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)，了解范围规则、开发命令和拉取请求检查清单。

基于 [MIT 许可证](LICENSE) 开源。
