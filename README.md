# DS_agent_skills

分散合意の研究のための Agent Skills 集です。研究テーマを批判的に検討する、論文を批判的に読む、システムモデルを明確にする、隣接分野との違いを整理する、実験を回す、といった作業を、Claude Code などのコーディングエージェントに任せられるようにします。

文献は Semantic Scholar で確かめ、確認できない論文は根拠に使いません。テーマ案にはすぐに同意せず、調べたうえで判定します。

## スキル

| スキル | 内容 |
| --- | --- |
| research-theme-review | テーマ案をすぐには肯定せず、先行研究を調べたうえで、新規性、モデルの妥当性、既知の結果との整合、検証の方法などで評価し、採用・改訂・保留・棄却のどれかに判定する。判定はテーマごとにファイルへ記録する |
| consensus-paper-critique | 論文の主張と根拠を対応づけ、隠れた仮定、証明や評価の弱点、先行研究との関係を整理する |
| system-model-spec | 論文やテーマのシステムモデルを、タイミング、故障、閾値、暗号、敵対者、解く問題などの項目ごとに洗い出し、FLP などの既知の不可能性や下界と照らし合わせる |
| field-positioning | 論文やテーマを、分散計算の理論、システム、ブロックチェーン・暗号、データベース、形式検証の各分野と比べて、何が違うのかを整理する |
| consensus-experiment | 既存の実装を探して使い、スループット、レイテンシ、故障からの回復を測って、パーセンタイルと信頼区間で集計する |
| tla-plus-model-check | プロトコルを TLA+ で仕様に書き、TLC で小さな構成を網羅的に調べて safety の反例を探す。TLA+ を使ったことがない前提で進め、反例の trace を文章にして説明する |
| lean-consensus-proof | quorum の交差、agreement、帰納的な不変条件などの safety を、Lean 4 と Mathlib で証明する。Lean を使ったことがない前提で進め、何を仮定して何を示したかを説明する |
| academic-search | Semantic Scholar で論文を探し、確認済みの文献を根拠に答える |

スキルは手順の中でほかのスキルを名前で指して使います。たとえば research-theme-review は、先行研究の検索に academic-search を、モデルの整理に system-model-spec を使います。全部まとめて入れることをおすすめします。

## 入れ方

```bash
# 全部入れる
npx skills add otofu1024/DS_agent_skills

# 一覧を見てから選ぶ
npx skills add otofu1024/DS_agent_skills --list
npx skills add otofu1024/DS_agent_skills --skill research-theme-review
```

人が読む文章の推敲には yomiyasu スキルを使います。入っていなければ、推敲を省きます。

```bash
npx skills add nanaism/yomiyasu
```

### 文献検索の MCP サーバー

academic-search は Semantic Scholar の MCP サーバーを使います。サーバーのビルドには Rust のツールチェーン（cargo）が必要です。詳しくは [mcp/semantic-scholar/README.md](mcp/semantic-scholar/README.md) を見てください。

```bash
cargo install --git https://github.com/otofu1024/DS_agent_skills ds-semantic-scholar-mcp
claude mcp add -s user semantic-scholar -e SEMANTIC_SCHOLAR_API_KEY=<キー> -- "$HOME/.cargo/bin/ds-semantic-scholar-mcp"
```

### 実験の解析ツール

consensus-experiment は実験結果の集計に latstat を使います。詳しくは [tools/latstat/README.md](tools/latstat/README.md) を見てください。

```bash
cargo install --git https://github.com/otofu1024/DS_agent_skills latstat
```

## 使い方の例

- 「部分同期で、通信量が線形の BFT 合意を、閾値署名なしで作れないかな」→ research-theme-review が先行研究を調べて判定する
- 「この論文を批判的に読んで: arXiv:1803.05069」→ consensus-paper-critique
- 「この2本の論文、システムモデルはどう違う？」→ system-model-spec
- 「ブロックチェーンの BFT と、古典的な BFT は何が違うの？」→ field-positioning
- 「Raft と Multi-Paxos のレイテンシを、50ms の遅延を入れて比べたい」→ consensus-experiment
- 「このリーダー交代の手順に反例がないか、3ノードで調べて」→ tla-plus-model-check
- 「過半数の quorum どうしが必ず交わることを、Lean で証明して」→ lean-consensus-proof

## 構成

```text
skills/                 # スキル（npx skills が読む）
mcp/semantic-scholar/   # 文献検索の MCP サーバー（Rust）
tools/latstat/          # 実験結果の解析ツール（Rust）
AGENTS.md               # スキルを作る・直すときの規約
```

## 謝辞

リポジトリの構成と規約は [K-Dense-AI/scientific-agent-skills](https://github.com/K-Dense-AI/scientific-agent-skills)（MIT）を参考にしました。academic-search の検索の流れは、作者が使っていたスキルをもとにしています。

## ライセンス

MIT
