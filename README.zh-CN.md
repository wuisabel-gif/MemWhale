# MemoryWhale

为开发者和编程智能体提供持久化、本地优先的调试记忆。

[English README](README.md)

MemoryWhale 会记录你在调试过程中真正发生过的事情：执行过的命令、输出结果、失败信息，以及最终奏效的解决方法。

这些调试证据会保存在本地 SQLite 数据库中。即使终端窗口已经关闭、SSH 连接已经断开，或者 AI Agent 的会话已经结束，你和你的编程智能体仍然可以重新找到这些信息。

## 为什么使用 MemoryWhale？

- **记住真正发生过的事情。** 保存命令、运行环境、输出、错误和经验，而不只是一行 Shell History。
- **让不同的编程智能体共享同一份记忆。** 任何兼容 stdio MCP 的客户端，都可以通过 `mw-mcp` 读取和写入同一个本地记忆库。
- **让开发历史留在本地。** MemoryWhale 不需要账号、托管服务，也不需要为“记忆”额外支付按 Token 计费的费用。

MemoryWhale 记录的是开发与调试经验，而不是所有信息。它是一个调试记忆层（debugging memory layer），不是自主编程 Agent、通用个人记忆系统，也不能替代项目文档。

## 安装

Linux x86_64/aarch64 和 macOS 提供预编译二进制文件：

```bash
curl -fsSL https://raw.githubusercontent.com/wuisabel-gif/MemWhale/main/install.sh | sh
```

也可以通过 Cargo 或 Homebrew 安装：

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

Windows 用户可以通过 [WSL](https://learn.microsoft.com/windows/wsl/) 运行 MemoryWhale。关于软件包安装、PATH 配置以及不同平台的注意事项，请参阅[快速开始指南](docs/guides/getting-started.md)。

## 60 秒上手

```bash
mw global on                         # 开始记录之后执行的交互式 Shell 命令
mw-run -- cargo check                # 记录单条命令及其输出
mw remember "the linker needed libssl-dev"
mw search "linker error"             # 找回之前的错误以及对应解决方法
mw context --last-error              # 为任意 Agent 或聊天生成精简上下文
mw pet                               # 查看记忆库当前的状态
mw pet --watch                       # 让记忆鲸持续展示状态动画
```

对于持续时间较长的工作，可以使用 `mw --live` 记录具有崩溃恢复能力的 Shell 会话。

`mw tui` 可以打开交互式终端浏览器，`mw-serve` 会启动本地 Web Dashboard。

## 工作原理

```text
采集 CAPTURE              记忆 MEMORY                检索 RETRIEVAL
shell / mw-run ──────► 本地 SQLite ─────────────► search / context
agent hooks ─────────► 调试证据 + 经验 ─────────► 相似历史错误
                                                     │
                                                  接口
                                        CLI / MCP / TUI / Web / Desktop
```

采集（Capture）和检索（Retrieval）是两个相互独立的过程。

MCP 可以让 Agent 访问已有记忆，但它不会自动记录普通终端中的操作。完整的工作模型请参阅 [Architecture](docs/architecture.md) 和 [Capture Concept](docs/concepts/capture.md)。

## 与编程智能体一起使用

`mw-mcp` 是 MemoryWhale 与各种 AI 编程工具之间的统一集成接口。它是一个运行在本地的 stdio MCP Server，对外提供六个记忆工具：

- `recent_errors`
- `search_memory`
- `get_context`
- `remember`
- `similar_failures`
- `stats`

目前已有集成指南覆盖 Claude Code、Claude Desktop、Cursor、VS Code / GitHub Copilot、Windsurf、Zed、Codex CLI、Cline、Continue、Gemini CLI、Goose、OpenClaw、CrowClaw、Hermes Agent，以及其他兼容 MCP 的客户端。集成矩阵目前包含 24 个客户端和工具条目，并会标明每个条目的验证状态。

例如 Claude Code：

```bash
claude mcp add memorywhale -- mw-mcp
```

不同客户端提供的能力并不完全相同。MCP 负责访问记忆，而自动记录命令执行过程通常需要针对具体客户端配置 Hook。

[Integration Matrix](integrations/README.md) 会区分：

- **Memory Access**：是否可以访问 MemoryWhale 记忆；
- **Automatic Capture**：是否可以自动捕获执行过程；
- **Memory-use Guidance**：是否能够指导 Agent 主动使用记忆。

## MemoryWhale 适合谁？

MemoryWhale 面向那些调试上下文经常散落在终端滚动记录、Shell History、不同开发机器，以及临时 AI Agent 会话中的开发者。

如果你经常遇到下面这些情况，它会尤其有用：

- 调试 Build、依赖、Git、开发环境或 Deployment 问题；
- 跨多个会话使用 Coding Agent，或者经常切换不同 AI 工具；
- 通过 SSH 工作，或者需要在多台开发机器之间切换；
- 希望重复出现的错误以及对应解决方案能够被长期搜索；
- 更愿意将开发记忆保存在本地，而不是上传到托管式 Memory Service。

在 [Use Cases](docs/concepts/use-cases.md) 文档中，可以看到这些场景对应的完整工作流程和实际命令。

## 文档

- [Documentation Map](docs/README.md)
- [快速开始](docs/guides/getting-started.md)
- [`mw pet` 参考](docs/reference/pet.md)
- [终端采集](docs/guides/terminal-capture.md)
- [Agent Memory](docs/guides/agent-memory.md)
- [CLI 参考](docs/reference/cli.md)
- [MCP 参考](docs/reference/mcp.md)
- [记忆压缩](docs/reference/compaction.md)
- [安全与本地威胁模型](docs/SECURITY.md)
- [集成指南与能力矩阵](integrations/README.md)

## 参与贡献

MemoryWhale 欢迎任何能够改善开发经验采集、保存、检索或共享方式的贡献。

请阅读 [CONTRIBUTING.md](CONTRIBUTING.md)，了解项目的贡献范围、开发命令以及 Pull Request Checklist。

基于 [MIT License](LICENSE) 开源。
