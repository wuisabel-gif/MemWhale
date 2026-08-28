<!-- README-SOURCE-SHA256: 652ad87f7ae8e947eb1cdf6f8e255f55eee82b30ad81c77e5b756c5680c6a67a -->

# MemoryWhale

개발자와 코딩 에이전트를 위한 지속적이고 로컬 우선인 디버깅 메모리입니다.

[English README](README.md)

MemoryWhale은 디버깅 과정에서 실제로 일어난 일을 기록합니다. 실행한 명령어, 출력 결과, 오류와 실패, 그리고 실제로 문제를 해결한 방법까지 저장합니다.

이 기록은 로컬 SQLite에 저장됩니다. 터미널을 닫거나 SSH 연결이 끊기거나 에이전트 세션이 종료된 후에도 개발자와 코딩 에이전트가 이전 디버깅 기록을 다시 찾아 활용할 수 있습니다.

## 왜 MemoryWhale인가요?

- **실제로 일어난 일을 기억합니다.** 단순히 Shell History의 명령어 한 줄만 남기는 것이 아니라 명령어, 실행 환경, 출력, 실패 원인, 그리고 그 과정에서 얻은 해결 방법과 교훈까지 보존합니다.
- **여러 코딩 에이전트가 하나의 메모리를 공유할 수 있습니다.** stdio MCP를 지원하는 모든 클라이언트는 `mw-mcp`를 통해 동일한 로컬 메모리를 읽을 수 있으며, `remember`를 통해 단일 노트를 저장할 수 있습니다.
- **개발 기록을 로컬에 보관합니다.** MemoryWhale은 계정이나 호스팅 서비스 없이 사용할 수 있으며, 메모리를 위해 별도의 토큰 비용을 지불할 필요도 없습니다.

MemoryWhale은 모든 것을 기록하는 시스템이 아니라 개발과 디버깅 경험을 기록하는 시스템입니다. 자율 코딩 에이전트도, 범용 개인 메모리 시스템도 아니며, 프로젝트 문서를 대체하기 위한 도구도 아닙니다.

## 설치

Linux x86_64/aarch64 및 macOS용 사전 빌드 바이너리가 제공됩니다.

```bash
curl -fsSL https://raw.githubusercontent.com/wuisabel-gif/MemWhale/main/install.sh | sh
```

Cargo 또는 Homebrew를 통해 설치할 수도 있습니다.

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

Windows 사용자는 [WSL](https://learn.microsoft.com/windows/wsl/)에서 MemoryWhale을 실행할 수 있습니다. 패키지 설치, PATH 설정 및 플랫폼별 참고 사항은 [Getting Started 가이드](docs/guides/getting-started.md)를 확인하세요.

## 60초 만에 시작하기

```bash
mw global on                         # 이후의 대화형 Shell 명령어 기록
mw-run -- cargo check                # 명령어 하나와 해당 출력 기록
mw remember "the linker needed libssl-dev"
mw search "linker error"             # 이전 오류와 해결 방법 검색
mw context --last-error              # 에이전트나 채팅을 위한 압축된 컨텍스트 생성
mw pet                               # 메모리 저장소의 현재 상태 확인
mw pet --watch                       # 메모리 고래의 상태를 애니메이션으로 표시
```

더 긴 작업에서는 `mw --live`를 사용해 충돌에도 안전한 Shell 세션을 기록할 수 있습니다.

`mw tui`는 대화형 터미널 브라우저를 열고, `mw-serve`는 로컬 웹 대시보드를 실행합니다.

## 작동 방식

```text
CAPTURE                 MEMORY                    RETRIEVAL
수집                     메모리                    검색
shell / mw-run ──────► 로컬 SQLite ─────────────► search / context
agent hooks ─────────► 증거 + 해결 경험 ─────────► 유사한 과거 오류
                                                    │
                                                INTERFACES
                                                  인터페이스
                                       CLI / MCP / TUI / Web / Desktop
```

수집(Capture)과 검색(Retrieval)은 서로 독립적으로 작동합니다.

MCP를 사용하면 에이전트가 기존 메모리에 접근할 수 있지만, 일반적인 터미널 작업을 자동으로 기록하는 것은 아닙니다. 전체 구조는 [Architecture](docs/architecture.md) 및 [Capture Concept](docs/concepts/capture.md) 문서를 참고하세요.

## 코딩 에이전트와 함께 사용하기

`mw-mcp`는 MemoryWhale과 다양한 AI 코딩 도구를 연결하는 공통 통합 인터페이스입니다. 로컬에서 실행되는 stdio MCP 서버로, 여섯 개의 메모리 도구를 제공합니다.

- `recent_errors`
- `search_memory`
- `get_context`
- `remember`
- `similar_failures`
- `stats`

현재 Claude Code, Claude Desktop, Cursor, VS Code / GitHub Copilot, Windsurf, Zed, Codex CLI, Cline, Continue, Gemini CLI, Goose, OpenClaw, CrowClaw, Hermes Agent 및 기타 호환 클라이언트를 위한 가이드가 제공됩니다. 통합 매트릭스에는 24개의 클라이언트와 도구 항목이 있으며, 각 항목별로 저장소에서 제공하는 기능을 표시합니다.

예를 들어 Claude Code에서는 다음과 같이 등록합니다.

```bash
claude mcp add memorywhale -- mw-mcp
```

모든 클라이언트가 동일한 기능을 제공하는 것은 아닙니다. MCP는 메모리 접근을 지원하지만, 명령 실행 과정을 자동으로 기록하려면 클라이언트별 Hook이 필요합니다.

[Integration Matrix](integrations/README.md)는 다음 기능을 구분하고 검증된 각 클라이언트의 설정 가이드를 제공합니다.

- **Memory Access** — 메모리 접근
- **Automatic Capture** — 자동 실행 기록
- **Memory-use Guidance** — 에이전트의 메모리 활용 가이드

## MemoryWhale은 누구를 위한 도구인가요?

MemoryWhale은 디버깅 컨텍스트가 터미널 스크롤 기록, Shell History, 여러 개발 머신, 일시적인 에이전트 세션 등에 흩어져 있는 개발자를 위한 도구입니다.

특히 다음과 같은 경우에 유용합니다.

- 빌드, 의존성, Git, 개발 환경 또는 배포 문제를 자주 디버깅할 때;
- 여러 세션에 걸쳐 코딩 에이전트를 사용하거나 서로 다른 도구를 오갈 때;
- SSH 또는 여러 개발 머신에서 작업할 때;
- 반복되는 오류와 그 해결 방법을 나중에도 검색하고 싶을 때;
- 호스팅된 메모리 서비스보다 로컬 저장 방식을 선호할 때.

각 상황에 대한 실제 명령어와 전체 워크플로는 [Use Cases](docs/concepts/use-cases.md)에서 확인할 수 있습니다.

## 문서

- [Documentation Map](docs/README.md)
- [Getting Started](docs/guides/getting-started.md)
- [`mw pet` Reference](docs/reference/pet.md)
- [Terminal Capture](docs/guides/terminal-capture.md)
- [Agent Memory](docs/guides/agent-memory.md)
- [CLI Reference](docs/reference/cli.md)
- [MCP Reference](docs/reference/mcp.md)
- [Memory Compaction](docs/reference/compaction.md)
- [Security and Local Threat Model](docs/SECURITY.md)
- [Integration Guides and Capability Matrix](integrations/README.md)

## 기여하기

MemoryWhale은 개발 경험을 수집하고, 보존하고, 검색하고, 공유하는 방식을 개선하는 변경 사항을 환영합니다.

기여 범위, 개발 명령어 및 Pull Request 체크리스트는 [CONTRIBUTING.md](CONTRIBUTING.md)를 확인하세요.

MemoryWhale은 [MIT License](LICENSE)로 배포됩니다.
