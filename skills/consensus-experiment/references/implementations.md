# 既存の実装と道具

2026-10-31 に GitHub API と各リポジトリの README・LICENSE を取得して調べた結果。使う前に、最終更新、ライセンス、README を取り直して確かめる。
「README に記載なし」は「機能がない」とは限らない。必要ならコードやドキュメントを読む。

## 合意プロトコルの実装

| リポジトリ | プロトコル | 言語 | ライセンス | 最終更新 | ベンチマーク（README の記載） | 故障注入（README の記載） |
| --- | --- | --- | --- | --- | --- | --- |
| [etcd-io/raft](https://github.com/etcd-io/raft) | Raft（ライブラリ） | Go | Apache-2.0 | 2026-10-23 | 記載なし（決定的な状態機械としてテストしやすい設計、`raftexample` あり） | 記載なし |
| [hashicorp/raft](https://github.com/hashicorp/raft) | Raft（ライブラリ） | Go | MPL-2.0 | 2026-10-18 | 記載なし | 記載なし |
| [tikv/raft-rs](https://github.com/tikv/raft-rs) | Raft（ライブラリ） | Rust | Apache-2.0 | 2026-05-13 | `cargo bench`（Criterion。README いわく整備中） | 記載なし |
| [databendlabs/openraft](https://github.com/databendlabs/openraft) | Raft（ライブラリ） | Rust | Apache-2.0 | 2026-10-30 | `make bench_cluster_of_3` または benchmarks/minimal で `cargo run --release --bin bench`。メモリ上のストアとプロセス内のネットワークを使う、フレームワーク自体の測定 | 決定的シミュレーションのファザーと、Jepsen のテスト（分断、プロセスの停止、遅い・不安定なパケット、時計のずれ） |
| [apache/ratis](https://github.com/apache/ratis) | Raft（ライブラリ） | Java | Apache-2.0 | 2026-10-30 | README には記載なし（別ファイルを参照） | 記載なし |
| [efficient/epaxos](https://github.com/efficient/epaxos) | EPaxos、Paxos、Mencius、Generalized Paxos | Go | Apache-2.0（API は NOASSERTION、LICENSE は Apache 2.0） | 2021-08-15 | 記載なし | 記載なし |
| [ailidani/paxi](https://github.com/ailidani/paxi) | Multi-Paxos、Flexible Paxos、WPaxos、EPaxos、SDPaxos、Mencius ほか | Go | MIT | 2023-12-29 | `paxi/bin` で `./build.sh`、`./server -id 1.1 -algorithm=paxos &`、`./client -id 1.1 -config config.json`。単一プロセスのシミュレーションもある | 機能一覧に「Fault injection」 |
| [bft-smart/library](https://github.com/bft-smart/library) | BFT-SMaRt（BFT SMR） | Java | Apache-2.0 | 2025-11-19 | デモ（カウンタ）のみ。`./gradlew installDist`、`./smartrun.sh bftsmart.demo.counter.CounterServer 0`〜`3`、クライアント `CounterClient` | 記載なし（`system.bft=false` で crash 故障のみのモード） |
| [hot-stuff/libhotstuff](https://github.com/hot-stuff/libhotstuff) | HotStuff（Event-Driven、Basic、Chained） | C++ | Apache-2.0 | 2023-06-22 | `scripts/run_demo.sh`、`scripts/run_demo_client.sh`。論文の再現は deploy ディレクトリ | `scripts/faulty_leader_demo.sh`（リーダーの停止） |
| [relab/hotstuff](https://github.com/relab/hotstuff) | Chained HotStuff、Fast-HotStuff、Simple-HotStuff ほか | Go | MIT | 2026-07-15 | 実験の配備とグラフ作成の道具。`./hotstuff help run`、`./plot --help` | Twins による safety のテスト、Byzantine な振る舞いの差し込み |
| [MystenLabs/narwhal](https://github.com/MystenLabs/narwhal)（アーカイブ済み） | Narwhal、Tusk、Bullshark | Rust | Apache-2.0 | 2022-10-17 | `fab local`（ローカル4ノード、TPS とレイテンシを出力） | README の例は故障0。注入の方法は記載なし |
| [MystenLabs/mysticeti](https://github.com/MystenLabs/mysticeti)（アーカイブ済み） | Mysticeti | Rust | Apache-2.0 | 2026-06-09 | 再現用の試作。コマンドの記載なし | 記載なし |
| [cometbft/cometbft](https://github.com/cometbft/cometbft) | Tendermint | Go | Apache-2.0 | 2026-10-26 | 記載なし | 記載なし |

## 検査と実験の道具

| リポジトリ | 内容 | 言語 | ライセンス | 最終更新 | 使い方（README の記載） |
| --- | --- | --- | --- | --- | --- |
| [jepsen-io/jepsen](https://github.com/jepsen-io/jepsen) | 実クラスタでの検査。nemesis で故障を入れ、履歴を検査する | Clojure | 要確認（API は null、LICENSE ファイルが見つからない） | 2026-10-30 | `lein run test --nodes-file ~/container-nodes ...` |
| [jepsen-io/maelstrom](https://github.com/jepsen-io/maelstrom) | シミュレートしたネットワーク上でノードのプログラムを動かす。遅延、損失、分断、停止を入れられる | Clojure | EPL-1.0 | 2026-07-10 | `java -jar maelstrom.jar test --help` |
| [anishathalye/porcupine](https://github.com/anishathalye/porcupine) | 記録した履歴の linearizability の検査 | Go | MIT | 2026-09-21 | ライブラリとして使う。逐次仕様と履歴を渡す |

## 系列ごとの出発点

- **Raft**: 測定と故障シナリオの両方が要るなら openraft。ただしベンチマークはメモリ上のストアとプロセス内のネットワークなので、配備時の性能とは別物として扱う。永続化とネットワークを本物に差し替えてから比べる
- **Paxos 系（Multi-Paxos、EPaxos、Flexible Paxos）**: 複数のプロトコルを同じ枠組みで比べるなら Paxi。最終更新は 2023 年で、README いわく開発途中。EPaxos の論文の実装そのものを再現するなら efficient/epaxos
- **BFT SMR（PBFT 系）**: 動かせる BFT の基準として BFT-SMaRt。デモの負荷なので、測定はクライアント側で記録する
- **HotStuff**: 変種の比較なら relab/hotstuff（実験の道具と Twins がある）。元の実装は libhotstuff
- **DAG-based BFT**: `fab local` ですぐ測れる Narwhal。アーカイブ済みで、例は故障なし
- **検査**: 実クラスタなら Jepsen、ローカルで素早く試すなら Maelstrom、履歴の linearizability は Porcupine

## 注意

- アーカイブ済み・研究用の試作（Narwhal、Mysticeti、EPaxos、Paxi、libhotstuff）は、ビルドが通らないことがある。依存の版を固定して記録する
- ライセンスが確認できないもの（Jepsen）は、改変して再配布する前に確かめる
- 論文の数値を再現する前に、README のベンチマークがどの条件（ノード数、ネットワーク、ストア）で動くのかを読む
