# 分野ごとの特徴と用語の対応

## 分野ごとの特徴

### 分散計算の理論

- 主な発表先: PODC、DISC、OPODIS、SIROCCO、JACM、Distributed Computing
- 呼び名: consensus、Byzantine agreement、Byzantine broadcast、reliable broadcast、atomic broadcast
- 普通の前提: モデルを厳密に定義する。同期・部分同期・非同期を明示する。最悪の場合を考える
- 新規性の基準: 新しい可能性・不可能性・下界。既知の上界と下界の隙間を埋めること。新しいモデルを導入するなら、その理由
- 証拠の形: 定理と証明
- 他分野から見落とされやすい点: 定数倍や実装の重さ、配備の条件

### システム

- 主な発表先: OSDI、SOSP、NSDI、EuroSys、USENIX ATC、Middleware
- 呼び名: state machine replication、replication、Raft、Paxos、consensus protocol
- 普通の前提: 部分同期と crash 故障が中心。実装を伴う。永続化やネットワークの現実を扱う
- 新規性の基準: 実際の問題を解くこと。性能・可用性・運用性の改善と、その理由の説明（洞察）
- 証拠の形: 実装と評価。障害時の挙動の評価。アーティファクトの公開
- 他分野から見落とされやすい点: 下界や不可能性、モデルの曖昧さ

### 信頼性

- 主な発表先: DSN、SRDS
- 呼び名: fault tolerance、dependability、replication
- 普通の前提: 故障モデルを測定やデータから考える
- 証拠の形: 評価、故障注入、解析

### ブロックチェーン・暗号

- 主な発表先: CCS、IEEE S&P、USENIX Security、NDSS、FC、AFT、CRYPTO
- 呼び名: BFT consensus、blockchain consensus、finality、DAG-based consensus、longest-chain、proof-of-stake
- 普通の前提: Byzantine 故障。署名・閾値署名・VRF などの暗号。適応的な敵対者。permissionless な設定もある。参加者が多い
- 新規性の基準: 通信量やラウンド数の改善、より強い敵対者への耐性、準備の仮定を弱めること、大規模な実装
- 証拠の形: 証明（しばしば暗号学的な帰着）と実装評価の両方
- 他分野から見落とされやすい点: 古典的な分散計算の結果（再発見になりやすい）、crash-recovery や運用の現実

### データベース

- 主な発表先: SIGMOD、VLDB（PVLDB）、ICDE
- 呼び名: replication、atomic commit、two-phase commit、geo-distributed transaction、consensus（複製の意味で使うことがある）
- 普通の前提: トランザクションと分離レベル。crash 故障が中心
- 新規性の基準: トランザクションの性能、地理分散でのレイテンシ、一貫性と性能の両立
- 証拠の形: 実装と評価（スループット、レイテンシ、abort 率）
- 他分野から見落とされやすい点: consensus と atomic commit の違い、Byzantine 故障

### 形式検証

- 主な発表先: CAV、TACAS、FM、POPL、PLDI、OOPSLA、ITP、CPP
- 呼び名: protocol verification、inductive invariant、refinement
- 普通の前提: 仕様と実装の対応。safety を中心に、liveness は扱いが難しい
- 新規性の基準: 検証の自動化、扱えるプロトコルの広さ、証明の負担の削減
- 証拠の形: 機械的に検証された証明、検証できたプロトコルの数と規模
- 他分野から見落とされやすい点: モデル化で省いた部分（時間、故障の種類）が結論に効くこと

## 用語の対応

| 概念 | 理論 | システム（Raft / Paxos） | BFT・ブロックチェーン | 注意 |
| --- | --- | --- | --- | --- |
| 指導者の任期 | round、phase | Raft の term、Paxos の ballot | view（PBFT、HotStuff）、round（Tendermint） | 番号の意味と単調性の扱いが違う |
| ログの位置 | instance、slot | log index | height、block、slot | 1つの位置に複数のコマンドを入れるか |
| 決定 | decide | commit、apply | commit、finalize、finality | ブロックチェーンの probabilistic finality は確定ではない |
| 合意の証拠 | quorum | majority、quorum | quorum certificate（QC）、certificate | 署名の集まりか、受け取った票の数か |
| 正しい参加者 | correct process | non-faulty replica | honest node | crash 故障では「止まっていない」、Byzantine では「規則に従う」 |
| 一貫性 | agreement | linearizability、strong consistency | safety、consistency | linearizability（単一オブジェクト）と serializability（トランザクション）は別 |
| 可用性 | termination、wait-freedom | availability | liveness | CAP の availability は「故障していないノードが受け取った全ての要求が、いずれ応答を得る」で、日常の意味より強い |
| 複製の変更 | — | membership change、joint consensus | reconfiguration、validator set change | 理論では扱わないことが多い |
