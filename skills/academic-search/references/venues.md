# 分散システムの主な発表先と言い換え

スクリーニング（Step 3）で論文の位置づけを判断するときと、クエリの言い換え（Step 1）に使う。
会議名は Semantic Scholar の `venue` で略称と正式名の両方が出る。`venue` での絞り込みより、結果を見て判断する方が確実。

## 発表先

| 系統 | 主な会議・論文誌 | 特徴 |
| --- | --- | --- |
| 分散計算の理論 | PODC、DISC、OPODIS、SIROCCO、JACM、Distributed Computing | モデルを厳密に定義し、可能性・不可能性・下界・計算量を示す |
| システム | OSDI、SOSP、NSDI、EuroSys、USENIX ATC、Middleware | 実装と評価（スループット、レイテンシ、障害時の挙動） |
| 信頼性 | DSN、SRDS | 故障モデル、耐故障性の評価 |
| ブロックチェーン・暗号 | CCS、IEEE S&P、USENIX Security、NDSS、FC、AFT、CRYPTO | ビザンチン故障、署名や閾値署名、適応的な敵対者、通信量 |
| データベース | SIGMOD、VLDB（PVLDB）、ICDE | 複製、トランザクション、atomic commit、地理分散 |
| 形式検証 | CAV、TACAS、FM、POPL、PLDI、OOPSLA、ITP、CPP | 分散プロトコルの検証手法（TLA+、Ivy、Dafny、Coq、Lean） |
| プレプリント | arXiv の cs.DC（分散）、cs.CR（暗号・セキュリティ）、cs.LO（論理） | 最新の研究。査読前であることに注意 |

## 言い換えの例

同じ対象でも分野によって呼び名が違う。1つのクエリで見つからなくても、別名で見つかることが多い。

| 対象 | 言い換え |
| --- | --- |
| 複製された状態の合意 | state machine replication、SMR、atomic broadcast、total order broadcast、replicated log |
| ビザンチン耐性の合意 | Byzantine fault tolerance、BFT consensus、Byzantine agreement、blockchain consensus、BFT SMR |
| ビザンチン放送 | Byzantine broadcast、reliable broadcast、Byzantine reliable broadcast、consistent broadcast |
| DAG 型の合意 | DAG-based consensus、DAG BFT、mempool、Narwhal、Bullshark |
| リーダー交代 | view change、leader rotation、pacemaker、view synchronization |
| 部分同期 | partial synchrony、eventual synchrony、GST |
| 故障の種類 | crash fault、crash-recovery、omission fault、Byzantine fault、hybrid fault model |
| 複製の再構成 | reconfiguration、membership change、dynamic membership |
| 地理分散 | geo-replication、geo-distributed consensus、WAN consensus |
| トランザクションの合意 | atomic commit、two-phase commit、Paxos Commit |
| 一貫性 | linearizability、sequential consistency、serializability、strict serializability |
