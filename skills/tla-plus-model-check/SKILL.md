---
name: tla-plus-model-check
description: 分散合意プロトコル（Paxos、Raft、BFT など）を TLA+ で仕様に書き、モデル検査器 TLC で小さな構成を網羅的に調べて、safety の反例を探すスキル。TLA+ を使ったことがない人を前提に、環境の用意、仕様と設定ファイルの書き方、TLC の実行、反例（エラーの trace）の読み方、状態数を抑える方法までを進め、何を確かめて何を確かめていないかを説明する。「TLA+」「TLC」「モデル検査」「仕様を書いて確かめたい」「反例を探したい」「Paxos の仕様」「安全性を検査」などの依頼で使う。
license: MIT
compatibility: Java 11 以上と、TLA+ の公式リリースの tla2tools.jar（v1.7.4）が必要。jar と例の取得にネットワーク接続が要る。TLC 2.19（tla2tools.jar v1.7.4）と OpenJDK 21 で確認した（2026-10-31）。
metadata:
  version: "1.0"
  skill-author: otofu1024
---

# TLA+ による合意プロトコルのモデル検査

ユーザーは TLA+ を使ったことがない前提で進める。仕様を書き、TLC を動かし、反例を読むのはエージェントの仕事にする。
ユーザーには、どの構成（ノード数、値の数）で何を確かめたか、反例が何を意味するかを、TLA+ を知らなくても分かる言葉で説明する。

## いつ使うか

- プロトコルの案や論文のプロトコルに、safety の反例がないかを、小さな構成で素早く確かめたいとき
- Lean で証明する前に、主張が少なくとも小さな構成で成り立つかを確かめたいとき（lean-consensus-proof スキルの前段）
- 反例が見つかったとき、どの順で何が起きると壊れるのかを具体的に見たいとき

TLC が確かめるのは、設定ファイルで決めた有限の構成だけ。任意の n と f について成り立つとは言えない。一般の場合は lean-consensus-proof スキルで証明する。

## 手順

### 1. 確かめることを決める

- system-model-spec スキルのモデルカードから、仕様に入れる仮定を選ぶ（ノードの集合、quorum、故障、メッセージの扱い）
- 確かめる性質を、不変条件（どの状態でも成り立つ式）として書く。合意なら「決まった値は1つまで」
- 構成は小さく始める。ノード3つ、値2つ程度から始め、状態数を見ながら増やす

### 2. 環境を用意する

```bash
java -version   # 11 以上
mkdir -p ~/.local/share/tlaplus
curl -fL -o ~/.local/share/tlaplus/tla2tools.jar https://github.com/tlaplus/tlaplus/releases/download/v1.7.4/tla2tools.jar
```

- v1.7.4 は 2024-08-05 の安定版で、中の TLC は 2.19。開発中の v1.8.0 は先行版として別に配られている
- エディタで書くなら、VS Code の拡張機能 `tlaplus.vscode-ide` がある
- Claude Code の auto mode では、ダウンロードした jar の実行が止められることがある。そのときは利用者に、`Bash(java -XX:+UseParallelGC -jar <jar の絶対パス> *)` の許可ルールを settings.json に足してもらう。TLC のコマンドは `cd` や `&&` でつながず、単独で実行する

### 3. 雛形から始める

assets/Voting.tla に、各ノードが1回だけ投票するプロトコルの仕様がある。lean-consensus-proof スキルの雛形と同じプロトコルなので、TLC で確かめてから Lean で一般の場合を証明する、という流れを試せる。

| ファイル | 内容 |
| --- | --- |
| assets/Voting.tla | 仕様。正しい規則の `Spec` と、投票し直せる誤った規則の `SpecBuggy` |
| assets/Voting.cfg | `Spec` を、ノード3つ・値2つで調べる設定。エラーなし（41状態） |
| assets/VotingBuggy.cfg | `SpecBuggy` を同じ構成で調べる設定。7状態の反例が見つかる |

仕様の書き方の要点:

- 定数（`CONSTANTS`）でノードと値の集合を受け取り、設定ファイルで具体的な値を与える。`Node = {n1, n2, n3}` のように書くと、`n1` などはモデル値（ほかと区別できるだけの値）になる
- 状態は変数（`VARIABLES`）で表し、`Init` で初期状態を、`Next` で1歩の遷移を書く
- 決まった値の記録（`chosen`）のように、性質を書くための変数を足してよい
- 大小の比較や足し算を使うなら `EXTENDS Naturals` が要る。`FiniteSets`（`Cardinality`）は Naturals を外に見せないので、`EXTENDS Naturals, FiniteSets` と並べて書く

### 4. TLC を動かす

```bash
java -XX:+UseParallelGC -jar ~/.local/share/tlaplus/tla2tools.jar -workers auto -config Voting.cfg Voting.tla
```

- `-config` を省くと、仕様と同じ名前の `.cfg` を読む
- `-workers auto` で CPU のコアをすべて使う
- TLC は作業用のディレクトリ（`states/`）を作る。別の場所に置くなら `-metadir <ディレクトリ>` を付ける
- 設定ファイルの `INVARIANT` に、確かめる不変条件を並べる。型の不変条件（`TypeOK`）も一緒に確かめると、仕様の書き間違いが早く見つかる

### 5. 結果を読む

終了コードと最後の数行で、3つのどれかが分かる。

| 結果 | 終了コード | 出力 |
| --- | --- | --- |
| エラーなし | 0 | `Model checking completed. No error has been found.` と、`107 states generated, 41 distinct states found, 0 states left on queue.` のような状態数 |
| 不変条件の違反 | 12 | `Error: Invariant Agreement is violated.` のあとに、`Error: The behavior up to this point is:` と、初期状態から違反までの状態の列（trace） |
| 構文・意味のエラー | 150 | `Semantic errors:` や `Could not find declaration or definition of symbol '>'.`、最後に `Error: Parsing or semantic analysis failed.` |

trace の読み方:

- `State 1: <Initial predicate>` から始まり、`State 2: <Revote line 38, ...>` のように、各状態にたどり着いた動作の名前と、仕様の中の行番号が付く
- `votes = (n1 :> v1 @@ n2 :> None @@ n3 :> None)` は関数の値で、「n1 は v1、n2 と n3 は None」を表す
- ユーザーには、trace を「誰が何をした結果、どの順で性質が破れたか」の文章にして伝える。雛形の反例なら、「n1 と n3 が v1 に投票して v1 が決まったあと、n1 が v2 に投票し直し、n2 と合わせて v2 も決まった」となる

「エラーなし」は、その構成で到達できる全ての状態で不変条件が成り立ったという意味で、それ以上ではない。構成（定数の値）と状態数を必ず一緒に報告する。

### 6. 状態数を抑える

- まず構成を小さくする（ノード数、値の数、ballot の上限）
- 入れ替えても振る舞いが変わらない定数の集合は、設定ファイルの `SYMMETRY` で同一視できる（tlaplus/Examples の MCPaxos が例）。要素が1つしかない集合を symmetry にすると、TLC が警告を出す
- それでも多いときは、確かめた範囲を書き残したうえで、状態の制約で探索を打ち切る

### 7. 報告する

1. 確かめた性質（日本語の1文と、不変条件の名前）
2. 構成（定数の値）と、仕様に入れなかったこと（メッセージの損失、故障の種類、時間など）
3. 結果（エラーなしなら状態数、反例なら trace を日本語で説明したもの）
4. 次にやること（構成を広げる、Lean で一般の場合を証明する、仕様を直す）

ユーザーが直接読む最終稿は、yomiyasu スキルが使えるなら推敲する。仕様、コマンド、定数の名前は変えない。

## 参考になる仕様: Lamport の Paxos

tlaplus/Examples（https://github.com/tlaplus/Examples）の specifications/Paxos に、Lamport の Paxos の仕様（Paxos.tla、Voting.tla、Consensus.tla）と、TLC で動かすための MCPaxos.tla・MCPaxos.cfg がある。
これらは TLAPS の library と CommunityModules のモジュールを import するので、tla2tools.jar だけでは動かない。scripts/fetch_paxos_example.sh が、必要なファイルを1つのディレクトリに集める。

```bash
bash <このスキル>/scripts/fetch_paxos_example.sh paxos
cd paxos
java -XX:+UseParallelGC -jar ~/.local/share/tlaplus/tla2tools.jar -workers auto -config MCPaxos.cfg MCPaxos.tla
```

- MCPaxos.tla の既定の構成は、acceptor が1つ（`MCAcceptor == {a1}`）、値が1つの最小のもので、25状態で終わる
- 合意として意味のある構成にするには、ファイル内のコメントにある値に変える（`MCAcceptor == {a1, a2, a3}`、`MCValue == {v1, v2}`、`MCQuorum == {{a1, a2}, {a1, a3}, {a2, a3}}`）。この構成でも数秒以内に終わる（443状態）
- MCPaxos.cfg は、不変条件 `Inv1`〜`Inv4` と、Voting の仕様への refinement（`PROPERTY VotingSpecBar`）を確かめる
