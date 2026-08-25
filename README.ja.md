# MemoryWhale

開発者とコーディングエージェントのための、永続的でローカル優先のデバッグメモリです。

[English README](README.md) · [简体中文 README](README.zh-CN.md) · [한국어 README](README.ko.md)

MemoryWhale は、デバッグ中に実際に起きたことを記録します。実行したコマンド、出力、エラーや失敗、そして実際に問題を解決した方法まで保存します。

これらの記録はローカルの SQLite に保存されます。ターミナルを閉じた後、SSH 接続が切れた後、あるいは AI エージェントのセッションが終了した後でも、あなたやコーディングエージェントは過去のデバッグ記録を検索して再利用できます。

## なぜ MemoryWhale なのか

- **実際に起きたことを記憶します。** Shell History のコマンド一行だけではなく、コマンド、実行環境、出力、失敗、そしてそこから得られた解決策や知見まで保存します。
- **複数のコーディングエージェントで同じメモリを共有できます。** stdio MCP に対応したクライアントであれば、`mw-mcp` を通じて同じローカルメモリを読み取り、`remember` で1件のノートを保存できます。
- **開発履歴をローカルに保ちます。** MemoryWhale はアカウントやホスティングサービスを必要とせず、メモリのためにトークン単位の料金を支払う必要もありません。

MemoryWhale が記録するのは、あらゆる情報ではなく開発・デバッグの経験です。自律型コーディングエージェントでも、汎用的な個人向けメモリシステムでもなく、プロジェクトドキュメントの代替でもありません。

## インストール

Linux x86_64/aarch64 および macOS 向けのビルド済みバイナリを利用できます。

```bash
curl -fsSL https://raw.githubusercontent.com/wuisabel-gif/MemWhale/v0.8.0/install.sh | sh
```

この URL はインストールスクリプト自体を v0.8.0 のタグに固定しています。スクリプトは最新の安定リリースを探し、リリースアセットに SHA256 ファイルがある場合はダウンロード内容を検証します。特定のリリースを固定して使う場合は、[Releases](https://github.com/wuisabel-gif/MemWhale/releases) から対応するアセットを直接選択してください。

Cargo または Homebrew からインストールすることもできます。

```bash
cargo install memorywhale-cli

brew tap wuisabel-gif/memorywhale https://github.com/wuisabel-gif/MemWhale
brew install memorywhale
```

Windows ユーザーは [WSL](https://learn.microsoft.com/windows/wsl/) 上で MemoryWhale を実行できます。パッケージのインストール、PATH の設定、プラットフォームごとの注意事項については [Getting Started ガイド](docs/guides/getting-started.md)を参照してください。

## 60秒で試す

```bash
mw global on                         # 以降の対話型 Shell コマンドを記録
mw-run -- cargo check                # 1つのコマンドとその出力を記録
mw remember "the linker needed libssl-dev"
mw search "linker error"             # 過去のエラーと解決策を検索
mw context --last-error              # エージェントやチャット向けの簡潔なコンテキストを生成
mw pet                               # メモリストアの現在の状態を確認
mw pet --watch                       # メモリの状態をアニメーション表示
```

長時間の作業では、`mw --live` を使ってクラッシュに強い Shell セッションを記録できます。

`mw tui` はインタラクティブなターミナルブラウザを開き、`mw-serve` はローカル Web ダッシュボードを起動します。

## 仕組み

```text
CAPTURE                 MEMORY                     RETRIEVAL
収集                     メモリ                      検索
shell / mw-run ──────► ローカル SQLite ──────────► search / context
agent hooks ─────────► 証拠 + 学習した知見 ──────► 類似した過去のエラー
                                                    │
                                                INTERFACES
                                                 インターフェース
                                       CLI / MCP / TUI / Web / Desktop
```

収集（Capture）と検索（Retrieval）は独立しています。

MCP を使うことでエージェントは既存のメモリへアクセスできますが、通常のターミナル操作が自動的に記録されるわけではありません。全体のモデルについては、[Architecture](docs/architecture.md) および [Capture Concept](docs/concepts/capture.md) のドキュメントを参照してください。

## コーディングエージェントと連携する

`mw-mcp` は、MemoryWhale とさまざまな AI コーディングツールを接続する共通の統合インターフェースです。ローカルで動作する stdio MCP サーバーとして、6つのメモリツールを提供します。

- `recent_errors`
- `search_memory`
- `get_context`
- `remember`
- `similar_failures`
- `stats`

現在、Claude Code、Claude Desktop、Cursor、VS Code / GitHub Copilot、Windsurf、Zed、Codex CLI、Cline、Continue、Gemini CLI、Goose、OpenClaw、CrowClaw、Hermes Agent、およびその他の互換クライアント向けのガイドがあります。Integration Matrix には24のクライアントとツールの項目があり、それぞれの対応機能を示しています。

たとえば Claude Code では次のように登録します。

```bash
claude mcp add memorywhale -- mw-mcp
```

すべてのクライアントが同じ機能を提供しているわけではありません。MCP はメモリへのアクセスを提供しますが、コマンド実行を自動的に記録するには、クライアント固有の Hook が必要です。

[Integration Matrix](integrations/README.md) では、次の機能を区別し、各クライアントのセットアップガイドを提供しています。

- **Memory Access** — メモリへのアクセス
- **Automatic Capture** — 実行履歴の自動記録
- **Memory-use Guidance** — エージェントによるメモリ活用のガイド

## MemoryWhale は誰のためのツールですか？

MemoryWhale は、デバッグに必要なコンテキストがターミナルのスクロールバック、Shell History、複数の開発マシン、一時的なエージェントセッションなどに分散してしまう開発者のためのツールです。

特に、次のような場合に役立ちます。

- ビルド、依存関係、Git、開発環境、デプロイを頻繁にデバッグする;
- 複数のセッションでコーディングエージェントを使ったり、複数のツールを切り替えたりする;
- SSH や複数の開発マシンを使って作業する;
- 繰り返し発生するエラーとその解決策を後から検索できるようにしたい;
- ホスティング型のメモリサービスではなく、ローカルストレージを使いたい。

それぞれのケースについて、実際のコマンドを使ったエンドツーエンドの例は [Use Cases](docs/concepts/use-cases.md) を参照してください。

## ドキュメント

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

## コントリビューション

MemoryWhale では、開発経験の収集・保存・検索・共有を改善する変更を歓迎しています。

コントリビューションの対象範囲、開発用コマンド、Pull Request のチェックリストについては [CONTRIBUTING.md](CONTRIBUTING.md) を参照してください。

MemoryWhale は [MIT License](LICENSE) のもとで公開されています。
