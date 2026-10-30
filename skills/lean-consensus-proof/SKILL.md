---
name: lean-consensus-proof
description: 分散合意プロトコルの safety（agreement、quorum の交差、帰納的な不変条件）を Lean 4 と Mathlib で機械的に証明するスキル。Lean を使ったことがない人を前提に、環境の用意、プロジェクトの作成、プロトコルの状態と遷移の書き方、証明、エラーの読み方、sorry や余計な公理が残っていないかの確認までを進め、何を仮定して何を示したかを説明する。「Lean で証明したい」「形式証明」「定理証明」「機械検証」「quorum の交差を証明」「不変条件を証明」「Lean 4」「Mathlib」などの依頼で使う。
license: MIT
compatibility: Lean 4 のツールチェーン管理 elan と Mathlib が必要。Mathlib のビルド済みキャッシュを取るためにネットワーク接続が要り、プロジェクトごとに約 7GB のディスクを使う。Lean v4.35.0 と Mathlib v4.35.0（2026-10-31）で確認した。
metadata:
  version: "1.0"
  skill-author: otofu1024
---

# Lean による合意プロトコルの証明

ユーザーは Lean を使ったことがない前提で進める。証明を書き、エラーを読んで直すのはエージェントの仕事にする。
ユーザーには、何を仮定して何を示したか、どこが未証明かを、Lean を知らなくても分かる言葉で説明する。

## いつ使うか

- 任意の n と f について成り立つ性質（quorum の交差、agreement、帰納的な不変条件）を、機械的に確かめたいとき
- 論文の証明の一部（補題）を確かめたいとき。consensus-paper-critique の手順5で証明に疑いが残ったときに使う
- 自分のテーマの safety の主張を、書き下して確かめたいとき

向かないもの:

- 小さな構成で反例を探すこと。まずは tla-plus-model-check スキルでモデル検査する方が速い
- liveness のような、時間や公平性に関わる性質。扱えるが重いので、最初の対象にしない

## 手順

### 1. 示すことを決める

- system-model-spec スキルのモデルカードから、証明に使う仮定（ノードの集合、quorum の大きさ、故障の数、投票の規則）を取り出す
- 定理を日本語で1文にし、仮定を全部並べる。Lean の定理の仮定と1対1に対応させる
- 証明に使わなかった仮定は外す。仮定が少ないほど強い定理になる。使っていない仮定は Lean の linter が指摘することがある

### 2. 環境を用意する

```bash
# elan（Lean のツールチェーンを管理する道具）。Ubuntu なら apt で入る
sudo apt install elan
# ほかの環境では公式のインストーラを使う（https://lean-lang.org/install/manual/）
curl https://elan.lean-lang.org/elan-init.sh -sSf | sh

# Mathlib を使うプロジェクトを作る
lake +leanprover/lean4:v4.35.0 new <名前> math
cd <名前>
```

- `lake new … math` は、Mathlib の版をツールチェーンに合わせて固定し（`lakefile.toml` の `rev`）、ビルド済みのキャッシュも取ってくる。初回はキャッシュの取得に数分かかる
- 作ったら、`lakefile.toml` の `weak.linter.mathlibStandardSet = true` を `false` にする。Mathlib に貢献するときの書式の規則（ファイル先頭の著作権表示など）で、研究用には要らない
- 使った版（`lean-toolchain` と `lakefile.toml` の `rev`）を記録する。Mathlib の補題の名前は版によって変わる

### 3. 雛形から始める

assets/QuorumExample.lean に、証明済みの定理が5つある。

| 定理 | 内容 |
| --- | --- |
| `Quorum.majority_quorums_intersect` | 全体の過半数を占める2つの quorum は交わる |
| `Quorum.bft_quorums_share_correct` | n = 3f+1 のとき、大きさ 2f+1 以上の2つの quorum は正しいノードを少なくとも1つ共有する |
| `Quorum.decided_values_agree` | 各ノードが1回しか投票しなければ、過半数の quorum の全員が投票した値は1つに決まる |
| `Voting.agreement` | 上を状態機械として書いたプロトコルで、決まった値は1つ |
| `Voting.decided_stable` | 同じプロトコルで、決まった値は遷移のあとも決まったまま |

プロジェクトに入れる手順（`<Name>` は `lake new` が作ったライブラリの名前で、先頭が大文字）:

1. このファイルを `<Name>/QuorumExample.lean` にコピーする
2. ルートの `<Name>.lean` の中身を次にする。テンプレートが作った `<Name>/Basic.lean` は使わないなら消す

```lean
module

public import <Name>.QuorumExample
```

3. `lake build` を実行し、`Build completed successfully` が出ることを確かめてから、自分の定理を書き足す

### 4. プロトコルを書く

- ノードの集合は `Finset α` で表す。quorum はその部分集合で、大きさの条件（`U.card < 2 * Q.card` など）で書く
- 状態は `structure` で、初期状態は `Init : State → Prop`、遷移は `Next : State → State → Prop` で書く
- safety の性質は、どの状態でも成り立つ性質として示すか、帰納的な不変条件として示す。帰納的な不変条件では、`Init s → Inv s` と、`Inv s → Next s s' → Inv s'` の2つを示す
- 帰納法が通らないときは、不変条件を強める（ほかの性質を足す）
- 雛形の `Voting.decided_stable` が、遷移で性質が保たれることを示す例になっている

### 5. 証明を書いて直す

- まず各証明を `sorry` にして全体の形を作り、`lake build` が通る状態にする。それから1つずつ埋める
- よく使う tactic: `omega`（自然数と整数の線形な不等式）、`simp`、`rw`、`obtain`、`refine`、`exact`、`by_contra`、`push Not`（否定を内側へ移す。`push_neg` は非推奨）、`classical`
- 補題の名前が分からないときは、`exact?` を書いてビルドする。候補が `Try this:` として出る（例: 和集合と共通部分の大きさの関係から `Finset.card_union_add_card_inter` が見つかる）
- 1つ直すたびに `lake build` で確かめる。エラーが残ったまま先へ進まない

エラーの読み方:

| メッセージ | 意味と対処 |
| --- | --- |
| `omega could not prove the goal` | 反例の候補が表示される。仮定が足りないか、主張が誤っている |
| `unsolved goals` | 示し残したゴールが表示される。そのゴールに対して tactic を足す |
| `Unknown identifier`、`Unknown constant` | 名前の誤りか、版によって名前が変わった。`exact?` で探し直す |
| `Tactic introN failed` | ゴールが定義のままで、中に入れない。`unfold <定義名>` で展開する |
| `Type mismatch` | 期待された型と、渡した項の型が違う。表示された2つの型を比べる |

module 形式（ファイルの先頭が `module`）では、別のファイルの定義の中身は既定では展開できない。展開が要る定義には `@[expose]` を付ける。

### 6. 何を示したかを確かめる

`Check.lean` を作り、`lake env lean Check.lean` で実行する。

```lean
import <Name>
#print axioms <定理名>
```

- `propext`、`Classical.choice`、`Quot.sound` は Lean の標準の公理で、問題ない
- `sorryAx` が出たら、証明が終わっていない（どこかに `sorry` が残っている）
- 自分で `axiom` を宣言した場合も、ここに出る。仮定を axiom として置いていないかを確かめる

### 7. 報告する

1. 示した定理（日本語の1文と、Lean の定理名）
2. 仮定の一覧と、モデルカードの項目との対応。外した仮定があればそれも
3. 確かめた結果（`lake build` が通ったこと、`#print axioms` の出力）
4. 未証明の部分と、モデル化で省いたこと（時間、メッセージの損失、故障の種類など）
5. ファイルの場所と、再現の手順（使った版を含む）

ユーザーが直接読む最終稿は、yomiyasu スキルが使えるなら推敲する。定理名、コード、数式は変えない。

## 注意

- Lean で証明できたのは、Lean に書いたモデルについてだけ。モデルが論文やプロトコルと合っていなければ意味がない。手順7の4は必ず書く
- 「証明できた」と言うのは、`lake build` が通り、`#print axioms` に `sorryAx` がないときだけにする
- 分散プロトコル向けの枠組みに Veil（https://github.com/verse-lab/veil）がある。2026-10-31 時点では 2.0 の先行版で、Lean v4.32.0 に固定されている。このスキルは Mathlib だけで書く方法を扱う。Veil を使うなら、README と版を確かめてから使う
