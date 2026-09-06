<!-- README-SOURCE-SHA256: c177f5ebba1899016da1a16ac5f7382f2cb7c39b0d2b0a8cfa73aa0fccf46aec -->

<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="MemoryWhale 로고" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>개발자와 코딩 에이전트를 위한 지속적인 로컬 디버깅 메모리.</strong></p>

<p align="center"><a href="README.md">English README</a> · <a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="릴리스"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="MIT 라이선스"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="로컬 우선, 업로드 없음"/>
</p>

MemoryWhale은 디버깅 중 실제로 일어난 일을 기록합니다. 명령, 출력, 실패, 그리고 효과가 있었던 수정까지 담습니다.
이 증거를 로컬 SQLite에 저장하므로 터미널이 닫히거나 SSH 연결이 끊기거나 에이전트 세션이 끝난 후에도
사용자와 코딩 에이전트가 다시 찾을 수 있습니다.

**MemoryWhale 0.10.0 — Agent-Native Memory · 2026년 9월 6일.**
CLI, 웹 UI, 데스크톱 앱의 제품 버전은 모두 0.10.0이며, 재사용 가능한 Rust 코어의 버전은 0.5.0입니다.
업그레이드 안내와 호환되지 않는 Rust API 변경은 [릴리스 노트](https://github.com/wuisabel-gif/MemWhale/blob/v0.10.0/docs/releases/0.10.0.md)를 참고하세요.

## 왜 MemoryWhale인가요?

- **실제로 일어난 일을 기억합니다.** 셸 기록 한 줄뿐 아니라 명령, 환경, 출력, 실패와 교훈을 보존합니다.
- **코딩 에이전트 간에 하나의 메모리를 공유합니다.** 호환되는 모든 stdio MCP 클라이언트가 `mw-mcp`를 통해 같은 로컬 메모리를 읽고 쓸 수 있습니다.
- **개발 이력을 로컬에 보관합니다.** MemoryWhale은 계정, 호스팅 서비스, 토큰당 메모리 요금 없이 작동합니다.

MemoryWhale은 모든 정보가 아니라 개발 경험을 기록합니다. 디버깅 메모리 계층이며,
자율 코딩 에이전트나 범용 개인 메모리 시스템이 아니고 프로젝트 문서를 대체하지도 않습니다.

## Agent-Native Memory의 새로운 기능

- **에이전트를 연결하고 점검합니다.** `mw integrate`로 Claude Code 또는 Rho의 MCP 접근,
  캡처 훅과 메모리 사용 지침을 설치합니다. `mw doctor`는 MCP, 훅, 스킬을 각각 검사합니다.
- **출처를 명확히 유지합니다.** 스키마 10은 명령을 생성한 에이전트를 `claude`, `rho` 또는 `NULL`로 저장합니다.
  표시·필터 레이블인 `terminal`은 터미널/수동 또는 기존 기록의 출처를 뜻하며, 사람이 실행했다는 증거는 아닙니다.
  에이전트 식별 정보는 `command`, `session`, `note` 같은 소스 유형과 별개입니다.
- **저장소를 공유하면서 작업 트리는 구분합니다.** 정규화된 저장소 ID로 연결된 작업 트리를 묶되, 각 작업 트리의 루트와 기존 프로젝트 태그를 보존합니다.
  탐색은 원격 서비스가 아닌 로컬 Git 메타데이터를 읽습니다.
- **로컬 인터페이스를 사용합니다.** `mw-serve`는 `POST /mcp`에서 HTTP MCP를 제공합니다. `mw-serve --api`는 읽기 전용 JSON API를 명시적으로 활성화합니다.
  두 인터페이스는 대시보드 리스너를 공유하며, 루프백이 아닌 접근에는 토큰이 필요합니다.
- **GitHub 컨텍스트를 명시적으로 가져옵니다.** `mw github context <pr>`는 기존 `gh` 로그인으로 PR 메타데이터, 검사 결과, 리뷰를 읽습니다.
  크기를 제한하고 민감 정보를 가린 컨텍스트를 출력할 뿐, 코드를 체크아웃하거나 메모리에 자동 저장하지 않습니다. 백그라운드 GitHub 동기화도 없습니다.

## 설치

Linux x86_64/aarch64와 macOS용 사전 빌드 바이너리를 제공합니다.

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

Cargo 또는 Homebrew로 설치할 수도 있습니다.

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

설치 또는 업그레이드 후 버전과 로컬 설정을 확인하세요.

```bash
mw --version
mw doctor
```

Windows에서는 [WSL](https://learn.microsoft.com/windows/wsl/) 안에서 MemoryWhale을 실행할 수 있습니다.
패키지 설치, PATH 설정과 플랫폼별 설명은 [시작 가이드](docs/guides/getting-started.md)를 참고하세요.

## 60초 예제

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

![mw pet 기분 데모](assets/pet-demo.gif)

긴 작업에는 `mw --live`로 갑작스러운 종료에 대비한 셸 세션을 기록할 수 있습니다.
`mw tui`는 대화형 터미널 브라우저를 열고, `mw-serve`는 로컬 웹 대시보드를 시작합니다.

## 작동 방식

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

캡처와 검색은 독립적입니다. MCP는 에이전트가 기존 메모리에 접근하게 하지만 일반 터미널 활동을 자동으로 기록하지는 않습니다.
전체 모델은 [아키텍처](docs/architecture.md)와 [캡처 개념](docs/concepts/capture.md)을 참고하세요.

## 코딩 에이전트와 함께 사용하기

`mw-mcp`는 공통 통합 인터페이스입니다. 여섯 가지 메모리 도구를 제공하는 로컬 stdio MCP 서버이며,
`mw-serve`를 통해 HTTP로도 접근할 수 있습니다. 기존 가이드는 Claude Code, Rho, Claude Desktop,
Cursor, VS Code / GitHub Copilot, Windsurf, Zed, Codex CLI, Cline, Continue,
Gemini CLI, Goose, OpenClaw, CrowClaw, Hermes Agent와 기타 호환 클라이언트를 다룹니다.

```bash
mw integrate claude
mw integrate rho
mw doctor
```

모든 클라이언트가 같은 기능을 제공하지는 않습니다. MCP는 메모리 접근을 지원하고, 실행 자동 캡처에는 클라이언트별 훅이 필요합니다.
[통합 매트릭스](integrations/README.md)는 접근, 캡처, 메모리 사용 지침을 구분하고 검증된 설정 가이드들을 연결합니다.

Rho의 현재 훅 페이로드에는 명령 텍스트와 stdout이 없습니다. 실패는 자리표시자 명령과 함께 메타데이터로 기록할 수 있으며,
명령 텍스트가 없는 성공 호출은 건너뜁니다. [에이전트 간 인계 데모](docs/guides/cross-agent-handoff.md)는
픽스처와 모의 Rho 클라이언트로 실제 MCP를 사용하지만, 실제 에이전트를 실행하거나 Cargo 수정의 효과를 검증하지는 않습니다.

번들 스킬은 메모리 사용을 안내하지만, 작업 시작 시 자동 회상, 실패 시 자동 조회, 압축 전 자동 저장을 구현하지는 않습니다.
이러한 수명 주기 결정은 클라이언트가 담당합니다. MCP로 작성한 교훈은 기본적으로 검토 대기 상태입니다.

## MemoryWhale은 누구를 위한 도구인가요?

MemoryWhale은 디버깅 컨텍스트가 터미널 스크롤백, 셸 기록, 여러 머신과 임시 에이전트 세션에 흩어진 개발자를 위한 도구입니다.
특히 다음과 같은 경우에 유용합니다.

- 빌드, 의존성, Git, 환경 또는 배포 문제를 디버깅할 때
- 여러 세션에서 코딩 에이전트를 사용하거나 도구를 전환할 때
- SSH로 작업하거나 여러 개발 머신을 오갈 때
- 반복되는 실패와 해결책을 계속 검색할 수 있게 보존하고 싶을 때
- 호스팅된 메모리 서비스보다 로컬 저장소를 선호할 때

[사용 사례](docs/concepts/use-cases.md)에서 각 상황에 맞는 전체 흐름과 실제 명령을 확인하세요.

## 문서

- [문서 지도](docs/README.md)
- [시작하기](docs/guides/getting-started.md)
- [`mw pet` 레퍼런스](docs/reference/pet.md)
- [터미널 캡처](docs/guides/terminal-capture.md)
- [에이전트 메모리](docs/guides/agent-memory.md)
- [CLI 레퍼런스](docs/reference/cli.md)
- [로컬 JSON API](docs/reference/api.md)
- [MCP 레퍼런스](docs/reference/mcp.md)
- [보안 및 로컬 위협 모델](docs/SECURITY.md)
- [생태계](ECOSYSTEM.md) — Delphin, ContextGC, MemoryWhale의 조합
- [통합 가이드 및 기능 매트릭스](integrations/README.md)

## 기여하기

MemoryWhale은 개발 경험의 캡처, 보존, 검색 또는 공유를 개선하는 변경을 환영합니다.
범위 규칙, 개발 명령과 풀 리퀘스트 체크리스트는 [CONTRIBUTING.md](CONTRIBUTING.md)를 읽어주세요.

[MIT 라이선스](LICENSE)로 배포됩니다.
