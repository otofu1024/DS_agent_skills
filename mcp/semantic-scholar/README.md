# Semantic Scholar MCP サーバー

academic-search スキルが使う、Semantic Scholar Academic Graph API の MCP サーバーです。Rust で書かれていて、stdio で動きます。

## できること

| ツール | 内容 |
| --- | --- |
| `search_papers` | キーワードで論文を関連度順に検索する（年、会議、分野、被引用数で絞り込める） |
| `match_paper` | 題名から論文を1件特定する。記憶にある論文の実在確認に使う |
| `get_paper` | 1本の論文の書誌情報、要旨、TLDR、被引用数を返す |
| `get_citations` | その論文を引用している後続研究を返す |
| `get_references` | その論文が引用している先行研究を返す |
| `get_author` | 著者の情報と論文の一部を返す |

Semantic Scholar の API はキーがあっても1秒に1リクエストまでです。サーバーがリクエストの間隔を空け、429 や 5xx の応答は最大3回まで再試行します。検索は1回の呼び出しにつき1リクエストで、件数や要旨の長さには上限があるので、結果が大きくなりすぎません。

## 入れ方

Rust のツールチェーン（cargo）が必要です。

```bash
cargo install --git https://github.com/otofu1024/DS_agent_skills ds-semantic-scholar-mcp
```

手元にリポジトリがあるなら、`cargo install --path mcp/semantic-scholar` でも入ります。

## Claude Code への登録

API キーは https://www.semanticscholar.org/product/api#api-key-form から申請できます。キーがなくても動きますが、全ユーザー共有の枠はすぐ混んで 429 が返ります。

```bash
claude mcp add -s user -e SEMANTIC_SCHOLAR_API_KEY=<キー> semantic-scholar -- "$HOME/.cargo/bin/ds-semantic-scholar-mcp"
```

キーを含むコマンドは、会話の記録に残らないよう、自分のターミナルで実行してください。登録できたかは `claude mcp list` で確かめられます。

## 環境変数

| 変数 | 内容 | 既定値 |
| --- | --- | --- |
| `SEMANTIC_SCHOLAR_API_KEY` | API キー。`x-api-key` ヘッダーで送る | なし |
| `S2_MIN_INTERVAL` | リクエストの開始の間隔（秒） | 1.1 |
| `S2_MAX_RETRIES` | 429・5xx のときの再試行の回数（0〜3） | 3 |

## 開発

```bash
cd mcp/semantic-scholar
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

テストは HTTP と時計を差し替えて動かすので、ネットワークにはつながりません。
