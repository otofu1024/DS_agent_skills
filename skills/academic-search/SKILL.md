---
name: academic-search
description: 論文・学術文献を Semantic Scholar で検索し、一次情報に基づいて根拠のある回答を返すスキル。「論文」「文献」「研究」「研究動向」「学術的に」「文献ベースで」「サーベイ」「先行研究」「paper」「literature」「survey」「academic」「研究を調べて」「文献調査」などの言葉があるときは必ず使う。技術的な質問でも、ユーザーが論文や文献に基づく回答をはっきり求めたときは必ず使う。論文を明示的に求めていない一般的な技術的質問には使わない。記憶にある論文を引用する前に、実在と書誌情報を確かめるときにも使う。
license: MIT
compatibility: semantic-scholar MCP サーバー（DS_agent_skills リポジトリの mcp/semantic-scholar）とネットワーク接続が必要。API キー（環境変数 SEMANTIC_SCHOLAR_API_KEY）の設定を推奨。
metadata:
  version: "1.0"
  skill-author: otofu1024
---

# Academic Search

ハルシネーションを防ぎ、一次情報（論文・学術文献）に基づいて回答するためのスキル。
Semantic Scholar MCP（`semantic-scholar` サーバー）で検索し、根拠のある回答を返す。

## 使うツール

| ツール | 用途 |
| --- | --- |
| `search_papers` | キーワード検索（関連度順）。`limit`、`year`、`venue`、`fields_of_study`、`min_citation_count` で絞れる |
| `match_paper` | 題名から論文を1件特定する。記憶にある論文の実在確認に使う |
| `get_paper` | 1本の論文の書誌情報、要旨、TLDR、被引用数 |
| `get_citations` | その論文を引用している後続研究 |
| `get_references` | その論文が引用している先行研究 |
| `get_author` | 著者の情報と論文の一部 |

- Claude Code では `mcp__semantic-scholar__search_papers` のような名前で見える。
- 論文 ID には S2 の paperId のほか、`DOI:...`、`ARXIV:...`、`CorpusId:...` を使える。
- **レート制限**: Semantic Scholar は1秒に1リクエストまで。サーバーが間隔を空け、429 も数回までは再試行する。呼び出しは1つずつ順番に行い、並列にしない。`status: 429` が返り続けたら少し待って再試行し、それでも駄目ならユーザーに伝える。

---

## 検索フロー

### Step 1: クエリ生成
- 基本は **英語クエリのみ** で検索する
- 専門用語を使った正確な表現にする（例: "write amplification LSM Tree compaction"）
- 同じ概念の別名も試す。分散システムでは分野ごとに呼び名が違う（例: "state machine replication" と "atomic broadcast" と "total order broadcast"）。言い換えの例は references/venues.md にある
- **例外：日本固有のトピック**（日本国内の統計・調査・市場動向・政策など）の場合は日本語クエリも追加する
  - 例: 「日本 インターネット普及率」「国内 スーパーコンピュータ 市場」
  - 判断基準：日本国外の研究者が書く可能性が低い、日本語の一次資料が主体になりそうなトピック

### Step 2: 検索実行
`search_papers` を使って検索する。
- **通常**: 英語クエリのみ、`limit` は **5〜10件** を目安にする
- **日本固有トピック**: 英語・日本語を両方実行し、重複を除いてまとめる
- 年で絞るときは `year`（例: `"2020-"`、`"2016-2020"`）を使う
- 会議名の `venue` は表記揺れが多い。まず外して検索し、結果の `venue` と要旨を見て選ぶ

### Step 3: Abstract スクリーニング
取得した論文のタイトル・著者・年・abstract を確認し、**関連度を3段階で判定**する：

| 判定 | 基準 | 次のアクション |
|------|------|----------------|
| ◎ 高関連 | 質問のコアに直接関係する | 本文・引用ネットワークを深掘り |
| ○ 中関連 | 部分的に参考になる | Abstract レベルで参照 |
| △ 低関連 | ほぼ関係なし | スキップ |

### Step 4: 深掘り（◎ 高関連の論文のみ）
以下を必要に応じて実行する：
- `get_paper` でフルメタデータ・要旨・TLDR を取得
- `get_citations` で後続研究をたどる。並びは被引用数の順ではないので、`isInfluential`、`intents`、`citationCount` を見て重要なものを選ぶ
- `get_references` で先行研究をさかのぼる
- `get_author` で著者の他の研究を確認（著者が主要な研究者の場合）

深掘りの範囲は **質問の複雑さ** と **利用可能なトークン量** で判断する。
シンプルな質問なら Abstract スクリーニングだけで十分な場合が多い。

### Step 5: 回答の構成
- **一次情報（論文）を根拠に** 回答する。ブログ・Qiita 等は「ヒント」として使い、最終的には論文で確認する
- 各文・各主張に **参考文献（論文タイトル・著者・年・DOI か arXiv ID か Semantic Scholar URL）を付ける**
- 見つからなかった部分・不確かな部分は正直に「文献が見つからなかった」と伝える

---

## 引用前の実在確認

記憶から論文を挙げるとき（「○○の研究がある」と書くとき）は、書く前に `match_paper` か `search_papers` で実在と書誌情報を確かめる。

- 確かめられた論文だけを、ID つきで挙げる
- 確かめられなかったものは挙げないか、「未確認」と明記する
- 題名・著者・年・会議のどれかが記憶と食い違ったら、取得した情報を正とする

---

## 回答フォーマット

```
## [トピック名]

[回答本文。なるべく一文ごとに文献を引用する。]

### 参考文献
1. AuthorA et al. (2023) "Title of Paper" — DOI: 10.xxxx/... または arXiv:xxxx.xxxxx
2. AuthorB et al. (2021) "Another Paper" — [Semantic Scholar リンク]
...

### 検索クエリ（透明性のため記載）
- EN: "query used in English"
- JA: "使用した日本語クエリ"
```

ユーザーが直接読む回答の最終稿は、yomiyasu スキルが使えるなら推敲してから出す。引用、数値、専門用語、論文名は変えない。
他のスキル（research-theme-review など）の途中で検索結果を渡すだけのときは推敲しない。

---

## 注意事項

- **ハルシネーション厳禁**: 論文が見つからない場合、それを正直に伝える。曖昧な記憶で回答しない
- **孫引き禁止**: ブログや解説サイトに載っている主張は、元の論文まで辿って確認する
- **文献が少ない場合**: Semantic Scholar で見つからなければ、Web 検索で arXiv・ACM Digital Library・IEEE Xplore・USENIX・DBLP を直接当たることも検討する
- **日本語論文について**: J-STAGE など国内 DB は Semantic Scholar に収録されないことがある。日本語クエリで少ない場合は「国内文献が少ない可能性がある」と補足する

## MCP サーバーが使えないとき

semantic-scholar のツールが見つからないときは、Graph API を curl で直接呼ぶ。1秒に1回を守る。

```bash
H=(); [ -n "$SEMANTIC_SCHOLAR_API_KEY" ] && H=(-H "x-api-key: $SEMANTIC_SCHOLAR_API_KEY")
B=https://api.semanticscholar.org/graph/v1

# search_papers に当たる検索
curl -s "${H[@]}" "$B/paper/search?query=raft+consensus&limit=5&fields=title,year,venue,authors,externalIds,abstract"

# match_paper に当たる題名の照合
curl -s "${H[@]}" "$B/paper/search/match?query=In+Search+of+an+Understandable+Consensus+Algorithm&fields=title,year,venue,externalIds"

# get_paper に当たる1本の詳細（ID は DOI:...、ARXIV:...、S2 の paperId のいずれか）
curl -s "${H[@]}" "$B/paper/DOI:10.1145/3149.214121?fields=title,year,venue,authors,externalIds,abstract,tldr,citationCount"

# get_citations と get_references に当たる引用関係
curl -s "${H[@]}" "$B/paper/DOI:10.1145/3149.214121/citations?limit=20&fields=title,year,venue,externalIds,citationCount,intents,isInfluential"
curl -s "${H[@]}" "$B/paper/DOI:10.1145/3149.214121/references?limit=50&fields=title,year,venue,externalIds,citationCount"
```

Semantic Scholar が 429 を返し続けて使えないときは、検索と書誌情報の確認だけは OpenAlex（`https://api.openalex.org/works?search=...`）で代えられる。ただし引用の意図（`intents`）や `isInfluential` は取れないので、そのことを回答に書く。
OpenAlex の題名検索は、探している論文とは別の論文を1件目に返すことがある。返ってきた題名が探している論文と一致するかを確かめてから使う。一致しなければ、その論文は「未確認」として扱う。

サーバーの導入方法は、リポジトリの README と mcp/semantic-scholar/README.md にある。
