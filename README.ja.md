<!-- README-SOURCE-SHA256: 693301b6a0450efa434e88f08ce48396310ee90b85790c7269a5f1abaffcb523 -->

<p align="center">
  <img src="assets/memorywhale-logo-sm.png" alt="MemoryWhale ロゴ" width="160" />
</p>

<h1 align="center">MemoryWhale</h1>

<p align="center"><strong>開発者とコーディングエージェントのための、永続的なローカルデバッグメモリ。</strong></p>

<p align="center" dir="ltr"><a href="README.md">English README</a> · <a href="README.ar.md" lang="ar">العربية</a> · <a href="README.de.md" lang="de">Deutsch</a> · <a href="README.fr.md">README français</a> · <a href="README.zh-CN.md">简体中文 README</a> · <a href="README.zh-TW.md">繁體中文 README</a> · <a href="README.ko.md">한국어 README</a> · <a href="README.ja.md">日本語 README</a></p>

<p align="center">
  <a href="https://github.com/wuisabel-gif/MemWhale/actions/workflows/ci.yml"><img src="https://img.shields.io/github/actions/workflow/status/wuisabel-gif/MemWhale/ci.yml?branch=main&label=CI&logo=github" alt="CI"/></a>
  <a href="https://github.com/wuisabel-gif/MemWhale/releases"><img src="https://img.shields.io/github/v/release/wuisabel-gif/MemWhale?color=2b43dd&label=release" alt="リリース"/></a>
  <a href="https://crates.io/crates/memorywhale-cli"><img src="https://img.shields.io/crates/v/memorywhale-cli?color=2b43dd&label=crates.io" alt="crates.io"/></a>
  <img src="https://img.shields.io/badge/license-MIT-2b43dd" alt="MIT ライセンス"/>
  <img src="https://img.shields.io/badge/local--first-no%20upload-168a69" alt="ローカルファースト、アップロードなし"/>
</p>

MemoryWhale は、デバッグ中に実際に起きたこと（実行したコマンド、出力、失敗、そして実際に効いた修正）を記録します。
記録はローカルの SQLite に保存されるので、ターミナルや SSH 接続、エージェントのセッションがなくなった後でも、
あなた自身やコーディングエージェントがあとから探し出せます。

**MemoryWhale 0.11.0 — Explainable Debugging Evidence · 2026 年 9 月 20 日**
CLI、Web UI、デスクトップアプリの製品バージョンはいずれも 0.11.0 で、再利用可能な Rust コアのバージョンは 0.5.0 です。
アップグレード手順は[リリースノート](https://github.com/wuisabel-gif/MemWhale/blob/v0.11.0/docs/releases/0.11.0.md)をご覧ください。スキーマは引き続き 10 です。

**コントリビュートしてみませんか？** まずは[「Start here」Issue](https://github.com/wuisabel-gif/MemWhale/issues/317) をご覧ください。
翻訳のレビューやドキュメントの修正など、Rust の知識がなくても取り組めるタスクがたくさんあります。

## MemoryWhale を使う理由

- **実際に起きたことを残す。** シェル履歴の 1 行だけでなく、コマンド、環境、出力、失敗、そこから得た教訓までを保存します。
- **複数のコーディングエージェントで 1 つのメモリを共有する。** 互換性のある stdio MCP クライアントであれば、`mw-mcp` を通じて同じローカルメモリを読み書きできます。
- **開発履歴を手元に置いておく。** アカウントもホスティング型サービスも不要で、トークン従量制のメモリ料金もかかりません。

MemoryWhale が記録するのは開発上の経験であって、何もかもを記録するわけではありません。あくまでデバッグ用のメモリレイヤーであり、
自律型のコーディングエージェントでも、汎用の個人向けメモリシステムでも、プロジェクトのドキュメントの代わりでもありません。

## Agent-Native Memory の新機能

- **エージェントを接続し、状態を確認する。** `mw integrate` で Claude Code または Rho 向けの MCP アクセス、
  キャプチャ用フック、メモリの使い方のガイダンスをインストールできます。`mw doctor` は MCP、フック、スキルをそれぞれ個別にチェックします。
- **記録の出所を明示する。** スキーマ 10 では、コマンドを実行したエージェントを `claude`、`rho`、`NULL` のいずれかで保存します。
  表示・絞り込み用のラベル `terminal` は、ターミナル／手動、またはレガシーな出所を表すもので、人間が実行したことを保証するものではありません。
  エージェントの識別情報は、`command`、`session`、`note` といったソース種別とは別に管理されます。
- **リポジトリは共有し、ワークツリーは区別する。** 正規のリポジトリ ID によってリンクされたワークツリーをひとまとめにしつつ、各ワークツリーのルートと既存のプロジェクトタグはそのまま保持します。
  検出にはリモートサービスではなく、ローカルの Git メタデータを使います。
- **ローカルのインターフェースを使う。** `mw-serve` は `POST /mcp` で HTTP MCP を提供します。`mw-serve --api` を指定すると、読み取り専用の JSON API も有効になります（オプトイン）。
  どちらもダッシュボードと同じリスナーを使い、ループバック以外からアクセスするにはトークンが必要です。
- **GitHub のコンテキストは明示的に取得する。** `mw github context <pr>` は、既存の `gh` のログイン情報を使って PR のメタデータ、チェック、レビューを読み取ります。
  出力するのは分量を制限し、機密情報を伏せたコンテキストだけで、コードをチェックアウトしたり、自動でメモリに保存したりはしません。GitHub とのバックグラウンド同期もありません。

## インストール

Linux x86_64/aarch64 と macOS 向けにビルド済みバイナリを提供しています。

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

Windows をお使いの場合は、[WSL](https://learn.microsoft.com/windows/wsl/) 上で MemoryWhale を実行できます。
パッケージでのインストール、PATH の設定、プラットフォームごとの注意点については[入門ガイド](docs/guides/getting-started.md)をご覧ください。

## 60 秒でわかる使い方

```bash
mw global on                         # capture future interactive shell commands
mw-run -- cargo check                # capture one command and its output
mw remember "the linker needed libssl-dev"
mw search "linker error"             # recover the failure and its fix
mw context --last-error              # compact context for any agent or chat
mw pet                               # check your memory store's mood
```

![mw pet の気分デモ](assets/pet-demo.gif)

長めの作業では、`mw --live` を使うとクラッシュに強いシェルセッションとして記録できます。
`mw tui` はターミナル上で対話的に閲覧できるブラウザーを開き、`mw-serve` はローカルの Web ダッシュボードを起動します。

## 仕組み

```text
CAPTURE                 MEMORY                 RETRIEVAL
shell / mw-run ──────► local SQLite ────────► search / context
agent hooks ─────────► evidence + lessons ──► similar failures
                                                   │
                                              INTERFACES
                                      CLI / MCP / TUI / Web / Desktop
```

キャプチャと検索はそれぞれ独立しています。MCP を使うとエージェントは既存のメモリにアクセスできますが、通常のターミナル操作が自動で記録されるわけではありません。
全体像については[アーキテクチャ](docs/architecture.md)と[キャプチャの考え方](docs/concepts/capture.md)をご覧ください。

## 使っているコーディングエージェントと連携できます

`mw-mcp` は各種連携に共通する接点です。6 つのメモリツールを公開するローカルの stdio MCP サーバーで、
`mw-serve` を通じて HTTP でも利用できます。Claude Code、Rho、Claude Desktop、
Cursor、VS Code / GitHub Copilot、Windsurf、Zed、Codex CLI、Cline、Continue、
Gemini CLI、Goose、OpenClaw、CrowClaw、Hermes Agent、その他の互換クライアント向けのガイドを用意しています。

```bash
mw integrate claude
mw integrate rho
mw doctor
```

クライアントによって提供される機能は異なります。MCP で可能なのはメモリへのアクセスで、実行内容を自動でキャプチャするにはクライアントごとのフックが必要です。
[連携マトリクス](integrations/README.md)では、アクセス、キャプチャ、メモリの使い方のガイダンスを区別して示し、検証済みのセットアップガイドすべてにリンクしています。

現在の Rho のフックペイロードには、コマンド文字列と stdout が含まれていません。そのため、失敗は目印用のダミーコマンドを付けたメタデータとして記録でき、
コマンド文字列のない成功した呼び出しはスキップされます。[エージェント間の引き継ぎデモ](docs/guides/cross-agent-handoff.md)は、
フィクスチャと模擬的な Rho クライアントを実際の MCP に接続して動かすもので、実際に稼働しているエージェントを使ったものでも、Cargo の修正を検証したものでもありません。

同梱のスキルはメモリの使い方を案内するもので、タスク開始時の自動想起、失敗時の検索、コンテキスト圧縮前の保存といった処理を実装しているわけではありません。
こうしたライフサイクル上の判断はクライアント側に委ねられます。MCP 経由で書き込まれた教訓は、デフォルトではレビュー待ちの状態になります。

## MemoryWhale はどんな人向け？

MemoryWhale は、デバッグのコンテキストがターミナルのスクロールバック、シェル履歴、複数のマシン、使い捨てのエージェントセッションに散らばっている開発者に向けたツールです。
特に次のような場合に役立ちます。

- ビルド、依存関係、Git、環境、デプロイまわりの問題をデバッグするとき
- 複数のセッションにまたがってコーディングエージェントを使うときや、ツールを切り替えるとき
- SSH 経由で、または複数の開発マシンにまたがって作業するとき
- 繰り返し起きる失敗とその修正を、あとから検索できるようにしておきたいとき
- ホスティング型のメモリサービスよりもローカル保存を好むとき

[ユースケース](docs/concepts/use-cases.md)では、これらの場面をそれぞれ実際のコマンドを使ったエンドツーエンドのシナリオとして紹介しています。

## ドキュメント

- [ドキュメントマップ](docs/README.md)
- [入門ガイド](docs/guides/getting-started.md)
- [`mw pet` リファレンス](docs/reference/pet.md)
- [ターミナルキャプチャ](docs/guides/terminal-capture.md)
- [エージェントメモリ](docs/guides/agent-memory.md)
- [CLI リファレンス](docs/reference/cli.md)
- [ローカル JSON API](docs/reference/api.md)
- [MCP リファレンス](docs/reference/mcp.md)
- [セキュリティとローカル脅威モデル](docs/SECURITY.md)
- [エコシステム](ECOSYSTEM.md) — Delphin、ContextGC、MemoryWhale の組み合わせ
- [連携ガイドと機能マトリクス](integrations/README.md)

## コントリビュート

MemoryWhale では、開発経験のキャプチャ、保存、検索、共有を改善する変更を受け付けています。
対象範囲のルール、開発用コマンド、プルリクエストのチェックリストについては [CONTRIBUTING.md](CONTRIBUTING.md) をお読みください。
初めてコントリビュートする方は、[「Start here」Issue](https://github.com/wuisabel-gif/MemWhale/issues/317) からタスクを選べます。

すべてのプルリクエストには、同じメンテナーによる姉妹プロジェクトの GitHub Action、[Second-Opinion](https://github.com/wuisabel-gif/second-opinion) による自動レビューも付きます。
コメントはあくまで提案で、対応するかどうかは人が判断します。
設定方法と制限については [Second-Opinion ガイド](integrations/second-opinion/README.md) を参照してください。

[MIT ライセンス](LICENSE)の下で公開しています。
