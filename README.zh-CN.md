<!-- README-SOURCE-SHA256: 31efb45a9e5acb20dcd2d289542d1245d9235c6a7b47ce4721a63eb35a43e809 -->

<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="MemoryWhale 标志" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>为开发者和编程智能体提供持久化的本地调试记忆。</strong></p>

<p align="center" dir="ltr"><a href="README.md">English README</a> · <a href="README.ar.md" lang="ar">العربية</a> · <a href="README.de.md" lang="de">Deutsch</a> · <a href="README.fr.md">README français</a> · <a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

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

**MemoryWhale 0.12.0 — Retrieval You Can Trust · 2026 年 9 月 25 日。**
CLI、Web 界面和桌面应用共用产品版本号 0.12.0；可复用的 Rust 核心版本为 0.6.0。
升级指南见[发布说明](https://github.com/wuisabel-gif/MemWhale/blob/v0.12.0/docs/releases/0.12.0.md)。打开存储库时，其 schema 会从 10 迁移到 13。

**想参与贡献？** 可以先看看 [Start here issue](https://github.com/wuisabel-gif/MemWhale/issues/317)。
很多任务不需要会 Rust，比如审校翻译、修正文档。

## 为什么选择 MemoryWhale

- **记住真正发生过的事情。** 把命令、环境、输出、失败和经验完整保存下来，而不只是 Shell 历史里的一行记录。
- **多个编程智能体共用一份记忆。** 任何兼容 stdio MCP 的客户端都能通过 `mw-mcp` 读写同一份本地记忆。
- **开发历史留在本地。** MemoryWhale 不需要账号，不依赖托管服务，也没有按 Token 计费的记忆费用。

MemoryWhale 只记录开发经验，而不是事无巨细什么都记。它是一层调试记忆，不是自主编程智能体，
不是通用的个人记忆系统，也不能替代项目文档。

## Agent-Native Memory 新特性

- **连接并检查智能体。** 使用 `mw integrate` 安装 Claude Code 或 Rho 的 MCP 访问、
  采集钩子和记忆使用指引；`mw doctor` 分别检查 MCP、钩子和技能。
- **来源清晰可查。** Schema 10 将命令的智能体字段存为 `claude`、`rho` 或 `NULL`。
  用于显示和筛选的标签 `terminal` 表示来源为终端/手动录入或旧版记录，并不能证明命令是人执行的。
  智能体身份与 `command`、`session`、`note` 等来源类型相互独立。
- **同一仓库，区分工作树。** 规范化的仓库 ID 会把关联的工作树归到一起，同时保留每个工作树的根目录和已有的项目标签。
  仓库识别只读取本地 Git 元数据，不访问远程服务。
- **使用本地接口。** `mw-serve` 在 `POST /mcp` 提供 HTTP MCP；`mw-serve --api` 显式启用只读 JSON API。
  两者复用仪表盘的监听地址；从非回环地址访问需要令牌。
- **按需获取 GitHub 上下文。** `mw github context <pr>` 借助你已登录的 `gh` 读取 PR 元数据、检查结果和评审意见。
  输出的上下文有长度限制并经过脱敏，不会检出代码，也不会自动存入记忆。不存在后台 GitHub 同步。

## 安装

提供 Linux x86_64/aarch64 和 macOS 的预编译二进制文件：

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

安装或升级后，检查一下版本和本地配置：

```bash
mw --version
mw doctor
```

Windows 用户可以在 [WSL](https://learn.microsoft.com/windows/wsl/) 中运行 MemoryWhale。
包管理器安装、PATH 设置和各平台注意事项见[入门指南](docs/guides/getting-started.md)。

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

时间较长的工作可以用 `mw --live` 录制 Shell 会话，即使进程崩溃也不易丢失。
`mw tui` 会打开交互式终端浏览界面，`mw-serve` 则启动本地 Web 仪表盘。

## 工作原理

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

采集和检索相互独立。MCP 让智能体能够访问已有记忆，但不会自动记录日常的终端操作。
完整模型见[架构](docs/architecture.md)和[采集概念](docs/concepts/capture.md)。

## 与编程智能体配合使用

`mw-mcp` 是各类集成的共同接入点：它是一个本地 stdio MCP 服务器，提供六个记忆工具，
也可以通过 `mw-serve` 以 HTTP 方式访问。现有指南涵盖 Claude Code、Rho、Claude Desktop、
Cursor、VS Code / GitHub Copilot、Windsurf、Zed、Codex CLI、Cline、Continue、
Gemini CLI、Goose、OpenClaw、CrowClaw、Hermes Agent 以及其他兼容客户端。

```bash
mw integrate claude
mw integrate rho
mw doctor
```

各客户端的能力并不相同。MCP 负责记忆访问；要自动采集执行记录，还需要针对该客户端的钩子。
[集成矩阵](integrations/README.md)分别列出访问、采集和记忆使用指引三项能力，并附有每份已验证配置指南的链接。

Rho 目前的钩子载荷不含命令文本和 stdout：失败可以用占位命令、以元数据形式记录；
不含命令文本的成功调用则直接跳过。[跨智能体交接演示](docs/guides/cross-agent-handoff.md)
使用测试数据和模拟的 Rho 客户端对接真实的 MCP，既没有运行真实的智能体，也不代表 Cargo 修复已经过验证。

内置技能只是指导如何使用记忆，并没有实现任务开始时自动回忆、出错时自动查找或上下文压缩前自动保存。
这些生命周期上的决策仍由客户端负责。通过 MCP 写入的经验默认处于待审核状态。

## MemoryWhale 适合谁？

如果你的调试线索散落在终端回滚记录、Shell 历史、多台机器和临时的智能体会话里，MemoryWhale 就是为你准备的。
以下场景尤其适用：

- 调试构建、依赖、Git、环境或部署；
- 跨会话使用编程智能体，或在不同工具之间切换；
- 通过 SSH 工作，或在多台开发机之间来回切换；
- 希望反复出现的失败和对应的修复始终能搜到；
- 更愿意把数据存在本地，而不是交给托管的记忆服务。

[使用场景](docs/concepts/use-cases.md)用真实命令把上面每种情况完整演示了一遍。

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

凡是能改进开发经验的采集、保存、检索或共享的改动，MemoryWhale 都欢迎。
范围约定、开发命令和 PR 检查清单见 [CONTRIBUTING.md](CONTRIBUTING.md)。
新贡献者可以从 [Start here issue](https://github.com/wuisabel-gif/MemWhale/issues/317) 中挑选任务。

每个 PR 还会收到 [Second-Opinion](https://github.com/wuisabel-gif/second-opinion) 的自动评审。它是同一位维护者的姊妹项目，一个 GitHub Action。
评审意见仅供参考，是否采纳由人来决定。配置方法和局限见 [Second-Opinion 指南](integrations/second-opinion/README.md)。

基于 [MIT 许可证](LICENSE) 开源。
