# リポジトリの指針

分散合意の研究を支える Agent Skills 集。テーマ作り、批判的な文献調査、システムモデルの明確化を中心に、形式検証と実験を扱う。各スキルは `skills/<name>/` に置き、[Agent Skills 仕様](https://agentskills.io/specification) に従う。

スキルを作る・直す前に、このファイルを読む。

## 構成

```text
skills/<name>/
├── SKILL.md        # 必須
├── references/     # 長い資料。必要なときだけ読まれる
├── scripts/        # 実行する補助プログラム
└── assets/         # テンプレートなど
mcp/semantic-scholar/   # 文献検索用の MCP サーバー（Rust。academic-search が使う）
tools/latstat/          # 実験結果の解析ツール（Rust。consensus-experiment が使う）
```

- スキルのディレクトリには、エージェントが読み込むものだけを置く。テスト、生成物、作業用のデータは置かない。
- スキル内の別ファイルは、スキルのルートからの相対パスで参照する。深さは1階層まで。
- 作業用のメモや調査結果は `.work/`（git 管理外）に置く。

## frontmatter

使えるフィールドは次の6つだけ。ほかのキーを最上位に置くと検証で落ちる。

| フィールド | 必須 | 制約 |
| --- | --- | --- |
| `name` | 必須 | 1〜64文字。英小文字・数字・ハイフンのみ。先頭・末尾・連続のハイフンは不可。ディレクトリ名と一致させる |
| `description` | 必須 | 1〜1024文字。何をするスキルか、いつ使うかを書く |
| `license` | 任意 | このリポジトリでは `MIT` |
| `compatibility` | 任意 | 500文字以内。必要な環境だけを書く |
| `allowed-tools` | 任意 | スペース区切りの文字列 |
| `metadata` | 任意 | 文字列の値を持つマッピング。`version` は必須扱い |

- YAML はブロック形式で書く。`metadata: {"version": "1.0"}` のような flow 形式は、検証器（strictyaml）が frontmatter 全体を読めなくなる。
- `metadata` の値は引用符で囲んだ文字列にする（`version: "1.0"`）。
- `description` は日本語で書く。発動の手がかりになる語を入れ、英語の専門用語（Paxos、Raft、BFT、TLA+ など）も含める。
- 作者は `metadata.skill-author` に書く。

## 本文

- 日本語で書く。専門用語は原語を残す（linearizability、GST、quorum など）。無理に訳語を作らない。
- `SKILL.md` は500行以内に収める。長い資料は `references/` に分ける。
- 手順、判断基準、具体例を書く。背景の説明は必要最小限にする。
- スキルは狭く作る。広いスキルはどの依頼でも発動候補になり、他のスキルの邪魔をする。
- 他のスキルを使わせるときは、スキル名で指す（例: 「academic-search スキルで探す」）。

## 文献と定理の扱い

- 論文を挙げるときは、確認済みの ID（DOI、arXiv ID、Semantic Scholar の URL のいずれか）を付ける。記憶だけで書かない。
- 確認には academic-search スキル（semantic-scholar MCP）を使う。使えないときは OpenAlex や DBLP の API で確かめる。
- 定理や下界は、モデルの条件（タイミング、故障の種類、認証の有無、決定的か乱択か）と一緒に書く。条件を省いた主張は誤りのもとになる。
- 要旨や本文で確かめられなかった主張には「要確認」と書く。

## yomiyasu での推敲

- ユーザーが直接読む文章（スキルが返す最終回答、README）は、yomiyasu スキルで推敲する。
- エージェントが読む文章（`SKILL.md`、`references/`）、他のスキルに渡す途中結果、コード、TLA+ や Lean の仕様は推敲しない。
- 推敲で引用、数値、専門用語、判定を変えない。

## プログラム

- 新しく書くプログラムは Rust を第一候補にする（作者の好み）。
- スキルに同梱するスクリプトは、利用者の環境で動くことが前提になる。Rust で書く場合は、ビルド方法と必要なツールチェーンを `compatibility` に書く。

## 検証

```bash
# 仕様への適合（CI でも実行する）
skills-ref validate skills/<name>

# npx skills がスキルを見つけられるか
npx skills add . --list

# Rust のプログラム（MCP サーバーと latstat）
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

lean-consensus-proof の雛形（assets/QuorumExample.lean）を変えたら、`.github/workflows/lean-examples.yml` と同じ手順（`lake new … math`、雛形のコピー、`lake build`、`#print axioms`）で、ビルドが通り `sorryAx` が出ないことを確かめる。Lean や Mathlib の版を上げるときも同じ。

## 更新

- スキルを直したら `metadata.version` を上げる。通常の改善はマイナー（`"1.2"` → `"1.3"`）、作り直しはメジャー。
- 例や手順を変えたら、実際に動かして確かめてから書く。
