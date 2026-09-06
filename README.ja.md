<!-- README-SOURCE-SHA256: 2f6582d6f7bc44c7565242f9d5f5baba95e2e7ccdea75c18230bad6b144369a5 -->

<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="MemoryWhale ロゴ" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>開発者とコーディングエージェントのための、永続的なローカルデバッグメモリ。</strong></p>

<p align="center"><a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="リリース"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="MIT ライセンス"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="ローカルファースト、アップロードなし"/>
</p>

MemoryWhale はデバッグ中に実際に起きたことを記録します。コマンド、出力、失敗、そして効果のあった修正です。
その証拠をローカルの SQLite に保存するため、ターミナルを閉じたり、SSH 接続が切れたり、
エージェントのセッションが終了したりしても、開発者とコーディングエージェントが再び見つけられます。

**MemoryWhale 0.10.0 — Agent-Native Memory · 2026 年 9 月 6 日。**
CLI、Web UI、デスクトップアプリの製品バージョンは共通で 0.10.0、再利用可能な Rust コアは 0.5.0 です。
アップグレード手順と Rust API の破壊的変更は[リリースノート](docs/releases/0.10.0.md)をご覧ください。

## MemoryWhale を使う理由

- **実際に起きたことを覚えておく。** シェル履歴の 1 行だけでなく、コマンド、環境、出力、失敗、教訓を残します。
- **複数のコーディングエージェントで 1 つのメモリを使う。** 互換性のある stdio MCP クライアントは、`mw-mcp` を通じて同じローカルメモリを読み書きできます。
- **開発履歴をローカルに保つ。** MemoryWhale はアカウント、ホスティングサービス、トークン単位のメモリ料金なしで動作します。

MemoryWhale が記録するのは開発経験であり、あらゆる情報ではありません。デバッグのためのメモリ層であって、
自律型コーディングエージェントや汎用の個人メモリシステムではなく、プロジェクト文書の代わりでもありません。

## Agent-Native Memory の新機能

- **エージェントを接続して点検する。** `mw integrate` で Claude Code または Rho の MCP アクセス、
  キャプチャフック、メモリ利用ガイダンスをインストールします。`mw doctor` は MCP、フック、スキルを個別に確認します。
- **由来を明確に保つ。** スキーマ 10 はコマンドのエージェントを `claude`、`rho`、または `NULL` として保存します。
  表示・フィルター用ラベルの `terminal` はターミナル、手動、または旧レコードの由来を意味し、人間が実行した証拠ではありません。
  エージェントの識別情報は `command`、`session`、`note` などのソース種別とは別です。
- **リポジトリを共有し、ワークツリーを区別する。** 正規化されたリポジトリ ID でリンクされたワークツリーをまとめつつ、各ルートと既存のプロジェクトタグを保持します。
  検出ではリモートサービスではなくローカルの Git メタデータを読み取ります。
- **ローカルインターフェースを使う。** `mw-serve` は `POST /mcp` で HTTP MCP を提供します。`mw-serve --api` で読み取り専用 JSON API を明示的に有効にできます。
  どちらもダッシュボードのリスナーを使い、ループバック以外からのアクセスにはトークンが必要です。
- **GitHub のコンテキストを明示的に取得する。** `mw github context <pr>` は既存の `gh` ログインを使って PR のメタデータ、チェック、レビューを読み取ります。
  サイズ制限と機密情報のマスキングを施したコンテキストを出力するだけで、コードのチェックアウトやメモリへの自動保存は行いません。バックグラウンドの GitHub 同期もありません。

## インストール

Linux x86_64/aarch64 と macOS 向けにビルド済みバイナリを提供しています。

```bash
curl -fsSL https://raw.githubusercontent.com/wuisabel-gif/MemWhale/main/install.sh | sh
```

Cargo または Homebrew でもインストールできます。

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

インストールまたはアップグレード後、バージョンとローカル設定を確認してください。

```bash
mw --version
mw doctor
```

Windows では [WSL](https://learn.microsoft.com/windows/wsl/) 内で MemoryWhale を実行できます。
パッケージのインストール、PATH 設定、プラットフォーム別の説明は[入門ガイド](docs/guides/getting-started.md)をご覧ください。

## 60 秒の使用例

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

![mw pet の気分デモ](assets/pet-demo.gif)

長めの作業では、`mw --live` で突然の終了に備えたシェルセッションを記録できます。
`mw tui` は対話型ターミナルブラウザーを開き、`mw-serve` はローカル Web ダッシュボードを起動します。

## 仕組み

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

キャプチャと検索は独立しています。MCP はエージェントに既存メモリへのアクセスを提供しますが、通常のターミナル操作を自動記録しません。
全体のモデルは[アーキテクチャ](docs/architecture.md)と[キャプチャの概念](docs/concepts/capture.md)をご覧ください。

## コーディングエージェントと連携する

`mw-mcp` は共通の統合インターフェースです。6 つのメモリツールを公開するローカル stdio MCP サーバーで、
`mw-serve` 経由の HTTP でも利用できます。既存のガイドは Claude Code、Rho、Claude Desktop、
Cursor、VS Code / GitHub Copilot、Windsurf、Zed、Codex CLI、Cline、Continue、
Gemini CLI、Goose、OpenClaw、CrowClaw、Hermes Agent、およびその他の互換クライアントを扱っています。

```bash
mw integrate claude
mw integrate rho
mw doctor
```

すべてのクライアントが同じ機能を提供するわけではありません。MCP はメモリアクセスを提供し、実行の自動キャプチャにはクライアント専用のフックが必要です。
[統合マトリクス](integrations/README.md)ではアクセス、キャプチャ、メモリ利用ガイダンスを区別し、検証済みの設定ガイドにリンクしています。

現在の Rho フックのペイロードにはコマンド文字列と stdout がありません。失敗はプレースホルダーのコマンドとともにメタデータとして記録でき、
コマンド文字列のない成功呼び出しはスキップされます。[エージェント間の引き継ぎデモ](docs/guides/cross-agent-handoff.md)は
フィクスチャと模擬 Rho クライアントを使って実際の MCP に接続しますが、実際のエージェントの実行や Cargo の修正効果の検証は行いません。

同梱スキルはメモリ利用を案内しますが、タスク開始時の自動想起、失敗時の自動検索、圧縮前の自動保存は実装していません。
こうしたライフサイクルの判断はクライアントが担います。MCP 経由で書かれた教訓は、デフォルトではレビュー待ちになります。

## MemoryWhale は誰のためのもの？

MemoryWhale は、デバッグのコンテキストがターミナルのスクロールバック、シェル履歴、複数のマシン、一時的なエージェントセッションに散らばっている開発者のためのものです。
特に次のような場合に役立ちます。

- ビルド、依存関係、Git、環境、デプロイの問題をデバッグする
- セッションをまたいでコーディングエージェントを使う、またはツールを切り替える
- SSH 経由、または複数の開発マシンで作業する
- 繰り返す失敗とその修正を後から検索できるようにしたい
- ホスティングされたメモリサービスよりローカル保存を好む

[ユースケース](docs/concepts/use-cases.md)では、それぞれの場面を実際のコマンドとともに一連の流れとして紹介しています。

## ドキュメント

- [ドキュメントマップ](docs/README.md)
- [入門ガイド](docs/guides/getting-started.md)
- [`mw pet` リファレンス](docs/reference/pet.md)
- [ターミナルのキャプチャ](docs/guides/terminal-capture.md)
- [エージェントメモリ](docs/guides/agent-memory.md)
- [CLI リファレンス](docs/reference/cli.md)
- [ローカル JSON API](docs/reference/api.md)
- [MCP リファレンス](docs/reference/mcp.md)
- [セキュリティとローカル脅威モデル](docs/SECURITY.md)
- [エコシステム](ECOSYSTEM.md) — Delphin、ContextGC、MemoryWhale の連携
- [統合ガイドと機能マトリクス](integrations/README.md)

## コントリビュート

MemoryWhale は、開発経験のキャプチャ、保存、検索、共有を改善する変更を受け付けています。
対象範囲、開発コマンド、プルリクエストのチェックリストは [CONTRIBUTING.md](CONTRIBUTING.md) をお読みください。

[MIT ライセンス](LICENSE)で公開しています。
