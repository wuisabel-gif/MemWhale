<!-- README-SOURCE-SHA256: 31efb45a9e5acb20dcd2d289542d1245d9235c6a7b47ce4721a63eb35a43e809 -->

<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="MemoryWhale 로고" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>개발자와 코딩 에이전트를 위한 영구 로컬 디버깅 메모리</strong></p>

<p align="center" dir="ltr"><a href="README.md">English README</a> · <a href="README.ar.md" lang="ar">العربية</a> · <a href="README.de.md" lang="de">Deutsch</a> · <a href="README.fr.md">README français</a> · <a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="릴리스"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="MIT 라이선스"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="로컬 우선, 업로드 없음"/>
</p>

MemoryWhale은 디버깅하면서 실제로 일어난 일, 즉 실행한 명령과 출력, 실패, 그리고 효과가 있었던 해결책을 기록합니다.
이 기록을 로컬 SQLite에 저장하기 때문에 터미널을 닫거나 SSH 연결이 끊기거나 에이전트 세션이 끝난 뒤에도
여러분과 코딩 에이전트가 다시 찾아볼 수 있습니다.

**MemoryWhale 0.12.0 — Retrieval You Can Trust · 2026년 9월 25일**
CLI, 웹 UI, 데스크톱 앱은 제품 버전 0.12.0을 함께 사용하며, 재사용 가능한 Rust 코어는 0.6.0입니다.
업그레이드 방법은 [릴리스 노트](https://github.com/wuisabel-gif/MemWhale/blob/v0.12.0/docs/releases/0.12.0.md)를 참고하세요.
스토어를 열면 스키마가 10에서 13으로 마이그레이션됩니다.

**기여하고 싶으신가요?** [Start here 이슈](https://github.com/wuisabel-gif/MemWhale/issues/317)부터 시작해 보세요.
번역 검토나 문서 수정처럼 Rust를 몰라도 할 수 있는 작업이 많습니다.

## 왜 MemoryWhale인가

- **실제로 일어난 일을 기억합니다.** 셸 히스토리 한 줄이 아니라 명령, 환경, 출력, 실패, 그리고 거기서 얻은 교훈까지 남깁니다.
- **여러 코딩 에이전트가 하나의 메모리를 씁니다.** 호환되는 stdio MCP 클라이언트라면 어느 것이든 `mw-mcp`를 통해 같은 로컬 메모리를 읽고 쓸 수 있습니다.
- **개발 이력은 로컬에 남습니다.** 계정도, 호스팅 서비스도, 토큰당 메모리 요금도 필요 없습니다.

MemoryWhale은 모든 것을 기록하지 않고 개발 경험만 기록합니다. 디버깅용 메모리 계층일 뿐,
자율 코딩 에이전트나 범용 개인 메모리 시스템이 아니며 프로젝트 문서를 대신하지도 않습니다.

## Agent-Native Memory 새 기능

- **에이전트 연결과 점검.** `mw integrate`로 Claude Code나 Rho에 MCP 접근,
  캡처 훅, 메모리 사용 가이드를 설치합니다. `mw doctor`는 MCP, 훅, 스킬을 각각 따로 점검합니다.
- **출처를 명시적으로 관리.** 스키마 10은 명령을 실행한 에이전트를 `claude`, `rho`, `NULL` 중 하나로 저장합니다.
  표시·필터용 레이블 `terminal`은 터미널/수동 입력이거나 이전 버전에서 넘어온 기록이라는 뜻이지, 사람이 직접 실행했다는 증거는 아닙니다.
  에이전트 정보는 `command`, `session`, `note` 같은 소스 유형과는 별개입니다.
- **저장소는 공유하고 워크트리는 구분.** 정규 저장소 ID로 연결된 워크트리를 하나로 묶되, 각 워크트리의 루트와 기존 프로젝트 태그는 그대로 유지합니다.
  저장소 탐지는 원격 서비스가 아니라 로컬 Git 메타데이터만 읽습니다.
- **로컬 인터페이스 사용.** `mw-serve`는 `POST /mcp`로 HTTP MCP를 제공하고, `mw-serve --api`를 주면 읽기 전용 JSON API도 켜집니다.
  둘 다 대시보드와 같은 리스너를 쓰며, 루프백이 아닌 곳에서 접근하려면 토큰이 필요합니다.
- **GitHub 컨텍스트는 명시적으로 가져오기.** `mw github context <pr>`는 기존 `gh` 로그인으로 PR 메타데이터, 체크, 리뷰를 읽습니다.
  길이를 제한하고 민감 정보를 가린 컨텍스트를 출력할 뿐, 코드를 체크아웃하거나 메모리에 자동 저장하지 않습니다. 백그라운드 GitHub 동기화도 없습니다.

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

Cargo나 Homebrew로도 설치할 수 있습니다.

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

설치하거나 업그레이드한 뒤에는 버전과 로컬 설정을 확인하세요.

```bash
mw --version
mw doctor
```

Windows에서는 [WSL](https://learn.microsoft.com/windows/wsl/)에서 MemoryWhale을 실행하면 됩니다.
패키지 설치, PATH 설정, 플랫폼별 참고 사항은 [시작 가이드](docs/guides/getting-started.md)에 정리되어 있습니다.

## 60초 만에 살펴보기

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

![mw pet 기분 데모](assets/pet-demo.gif)

오래 걸리는 작업이라면 `mw --live`로 비정상 종료에도 살아남는 셸 세션을 기록할 수 있습니다.
`mw tui`는 대화형 터미널 브라우저를 열고, `mw-serve`는 로컬 웹 대시보드를 띄웁니다.

## 동작 방식

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

캡처와 검색은 서로 독립적입니다. MCP는 에이전트가 이미 저장된 메모리에 접근하게 해 줄 뿐, 평소의 터미널 활동을 자동으로 기록하지는 않습니다.
전체 구조는 [아키텍처](docs/architecture.md)와 [캡처 개념](docs/concepts/capture.md) 문서를 참고하세요.

## 코딩 에이전트와 함께 쓰기

`mw-mcp`는 모든 통합의 공통 연결 지점입니다. 메모리 도구 6개를 제공하는 로컬 stdio MCP 서버이며,
`mw-serve`를 통해 HTTP로도 쓸 수 있습니다. Claude Code, Rho, Claude Desktop,
Cursor, VS Code / GitHub Copilot, Windsurf, Zed, Codex CLI, Cline, Continue,
Gemini CLI, Goose, OpenClaw, CrowClaw, Hermes Agent 및 기타 호환 클라이언트용 가이드가 준비되어 있습니다.

```bash
mw integrate claude
mw integrate rho
mw doctor
```

클라이언트마다 지원 범위가 다릅니다. MCP로는 메모리에 접근만 할 수 있고, 실행 내용을 자동으로 캡처하려면 클라이언트별 훅이 있어야 합니다.
[통합 매트릭스](integrations/README.md)에서 접근, 캡처, 메모리 사용 가이드 지원 여부를 구분해 보여 주며, 검증된 설정 가이드도 모두 링크해 두었습니다.

현재 Rho 훅 페이로드에는 명령 텍스트와 stdout이 들어 있지 않습니다. 그래서 실패는 센티널 명령과 함께 메타데이터로 기록되고,
명령 텍스트가 없는 성공 호출은 기록하지 않고 건너뜁니다. [에이전트 간 인계 데모](docs/guides/cross-agent-handoff.md)는
실제 에이전트가 아니라 픽스처와 시뮬레이션한 Rho 클라이언트로 실제 MCP를 호출하며, 여기 나오는 Cargo 수정도 검증된 것은 아닙니다.

번들 스킬은 메모리를 어떻게 쓸지 안내할 뿐, 작업 시작 시 자동 회상, 실패 조회, 컨텍스트 압축 전 저장을 직접 구현하지는 않습니다.
이런 생명주기 관련 판단은 클라이언트의 몫입니다. MCP로 작성된 교훈은 기본적으로 검토 대기 상태로 저장됩니다.

## 누구를 위한 도구인가

MemoryWhale은 디버깅 맥락이 터미널 스크롤백, 셸 히스토리, 여러 머신, 일회성 에이전트 세션에 흩어져 있는 개발자를 위한 도구입니다.
특히 이런 경우에 유용합니다.

- 빌드, 의존성, Git, 개발 환경, 배포 문제를 디버깅할 때
- 여러 세션에 걸쳐 코딩 에이전트를 쓰거나 도구를 바꿔 가며 쓸 때
- SSH로 작업하거나 여러 개발 머신을 오갈 때
- 반복되는 실패와 해결책을 나중에도 검색할 수 있게 남겨 두고 싶을 때
- 호스팅 메모리 서비스보다 로컬 저장소를 선호할 때

[사용 사례](docs/concepts/use-cases.md)에서 각 경우를 실제 명령과 함께 처음부터 끝까지 따라가 볼 수 있습니다.

## 문서

- [문서 안내](docs/README.md)
- [시작하기](docs/guides/getting-started.md)
- [`mw pet` 레퍼런스](docs/reference/pet.md)
- [터미널 캡처](docs/guides/terminal-capture.md)
- [에이전트 메모리](docs/guides/agent-memory.md)
- [CLI 레퍼런스](docs/reference/cli.md)
- [로컬 JSON API](docs/reference/api.md)
- [MCP 레퍼런스](docs/reference/mcp.md)
- [보안 및 로컬 위협 모델](docs/SECURITY.md)
- [에코시스템](ECOSYSTEM.md) — Delphin, ContextGC, MemoryWhale을 함께 쓰는 방법
- [통합 가이드와 기능 매트릭스](integrations/README.md)

## 기여하기

개발 경험을 캡처하고, 보존하고, 검색하고, 공유하는 기능을 개선하는 변경을 받습니다.
기여 범위 규칙, 개발용 명령, 풀 리퀘스트 체크리스트는 [CONTRIBUTING.md](CONTRIBUTING.md)를 읽어 주세요.
처음 기여하신다면 [Start here 이슈](https://github.com/wuisabel-gif/MemWhale/issues/317)에서 작업을 골라 보세요.

모든 풀 리퀘스트에는 같은 메인테이너의 자매 프로젝트인 GitHub Action [Second-Opinion](https://github.com/wuisabel-gif/second-opinion)의 자동 리뷰도 달립니다.
리뷰 코멘트는 제안일 뿐이며, 반영 여부는 사람이 결정합니다.
설정 방법과 한계는 [Second-Opinion 가이드](integrations/second-opinion/README.md)를 참고하세요.

[MIT 라이선스](LICENSE)로 배포됩니다.
