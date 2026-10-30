# 照合に使う既知の結果

テーマや論文の主張を照らし合わせるための、古典的な可能性・不可能性・下界の結果。
書誌情報は OpenAlex で確認済み（2026-10-31）。主張は OpenAlex から取得した要旨で確かめた範囲で書き、要旨で確かめられなかった部分は「要確認」と記す。DLS だけは本文（著者の研究室が公開している JACM 版）で確かめた。
これは出発点にすぎない。結論に使うときは、原典で条件を確かめる。

## 不可能性と必要なノード数

| 結果 | モデル | 主張 | 出典 |
| --- | --- | --- | --- |
| FLP | 非同期、メッセージ通信、信頼できる通信路、crash 故障が最大1個、決定的 | 合意を常に達成する（全ての正しいプロセスが必ず終了する）プロトコルは存在しない | Fischer, Lynch, Paterson. JACM 32(2):374–382, 1985. DOI: 10.1145/3149.214121 |
| 部分同期（DLS） | 部分同期の2つの版（上限が未知 / 上限は既知だが未知の時刻 GST 以降に成り立つ）。validity は strong unanimity（全ての初期値が v なら v に決まる。故障したプロセッサの初期値も含む） | 通信が部分同期のとき、t 個の故障に耐える合意に必要な最小のプロセッサ数は、fail-stop と omission で 2t+1、署名ありの Byzantine と署名なしの Byzantine で 3t+1（本文 Table I の第3・4列）。プロセッサが部分同期で通信が同期の場合（第5列）は値が違う | Dwork, Lynch, Stockmeyer. JACM 35(2):288–323, 1988. DOI: 10.1145/42282.42283。本文: https://groups.csail.mit.edu/tds/papers/Lynch/jacm88.pdf |
| Byzantine Generals | 同期、署名なし（oral messages） | 裏切り者が m 人のとき、将軍が 3m+1 人以上いれば解ける。署名付きメッセージなら裏切り者が何人でも解ける。解いているのは broadcast（1人の司令官の値に合意） | Lamport, Shostak, Pease. TOPLAS 4(3):382–401, 1982. DOI: 10.1145/357172.357176 |
| Interactive consistency | 同期、署名なし | 全員の値のベクトルに合意するには n ≥ 3f+1 が必要十分。故障したプロセスが情報を偽って中継できない（暗号で近似できる）なら、任意の n ≥ f で解ける | Pease, Shostak, Lamport. JACM 27(2):228–234, 1980. DOI: 10.1145/322186.322188 |
| CAP | 非同期ネットワーク、分断が起こりうる | 原子的な（linearizable な）一貫性と、可用性（正しいノードへの全ての要求に応答）を同時には保証できない | Gilbert, Lynch. SIGACT News 33(2):51–59, 2002. DOI: 10.1145/564585.564601 |

### broadcast と agreement の違い（自分で確かめられる議論）

署名（偽造できない書面のメッセージ）があると、Byzantine Generals（broadcast）は裏切り者が何人いても解ける（Lamport–Shostak–Pease）。しかし、全員が入力を持つ agreement で strong validity を求めると、署名があっても n ≥ 2f+1 が要る。理由は次の通り。

n ≤ 2f とし、プロセスを大きさ f 以下の2つの組 A、B に分ける。

1. A が正しく入力 0、B が乗っ取られて「入力 1 の正しいプロセス」を演じる。A から見ると正しいプロセスの入力は全て 0 なので、strong validity により A は 0 に決める
2. B が正しく入力 1、A が乗っ取られて「入力 0 の正しいプロセス」を演じる。同様に B は 1 に決める
3. 全員が正しく、A は入力 0、B は入力 1。A には 1 と、B には 2 と区別がつかないので、A は 0、B は 1 に決め、agreement が破れる

このように、論文が broadcast と agreement のどちらを解いているかで、必要なノード数が変わる。validity の定義でも変わる。DLS の Table I の同期の列では、署名ありの Byzantine の値は t で、本文はこれを「故障がいくつあっても耐えられる」と説明している。しかしこれは、validity が「全ての初期値（故障したプロセッサの分も含む）が同じなら」で、初期値が特定の送信者によって署名されている前提での値で、上の議論の validity とは違う（DLS 本文の1節と2.4節）。

## 時間と通信量の下界

| 結果 | モデル | 主張 | 出典 |
| --- | --- | --- | --- |
| f+1 ラウンド（Byzantine） | 同期、Byzantine 故障が最大 f 個、決定的 | interactive consistency には最悪 f+1 ラウンドかかる（**要確認**: 要旨を取得できなかった。条件は本文で確かめる） | Fischer, Lynch. IPL 14(4):183–186, 1982. DOI: 10.1016/0020-0190(82)90033-3 |
| f+1 ラウンド（crash） | 同期、crash 故障が最大 f 個 | 合意には少なくとも f+1 ラウンド必要（bivalency を使った簡潔な証明） | Aguilera, Toueg. IPL 71(3–4):155–158, 1999. DOI: 10.1016/S0020-0190(99)00100-3 |
| 署名を使う Byzantine agreement | 同期、署名あり、故障が最大 t 個 | 署名で故障したプロセスの振る舞いを「中継しない」程度に抑えても、t+1 ラウンドは必要。t+1 ラウンド・O(nt) メッセージで達成するアルゴリズムを示した | Dolev, Strong. SIAM J. Comput. 12(4):656–666, 1983. DOI: 10.1137/0212045 |
| 通信量（Dolev–Reischuk） | 同期、Byzantine 故障が最大 t 個 | 署名を使うアルゴリズムでは、署名の数に Ω(nt) の下界がある。署名を使わないアルゴリズムでは、メッセージ数に Ω(nt) の下界がある。署名を使う場合のメッセージ数の下界は Ω(n + t²) | Dolev, Reischuk. JACM 32(1):191–204, 1985. DOI: 10.1145/2455.214112 |
| 通信量（適応的な敵対者） | 適応的な敵対者 | 二次未満の通信量で Byzantine agreement を達成するには、敵対者が送信済みのメッセージを取り消せない（after-the-fact removal ができない）という仮定が必要 | Abraham, Chan, Dolev, Nayak ほか. PODC 2019, pp. 317–326. DOI: 10.1145/3293611.3331629 |

通信量の主張（「線形」「二次未満」）を見たら、上の2行と、何を数えているか（メッセージ数、ビット数、署名の数）、敵対者の強さを照らし合わせる。

## 非同期での回避策

| 結果 | モデル | 主張 | 出典 |
| --- | --- | --- | --- |
| 乱択（Ben-Or） | 非同期、crash 故障、乱数あり | 過半数のプロセスが動き続ける限り、確率1で決定に至る。配送順を決める敵対者がいても成り立つ。Byzantine 故障向けの版もある（**要確認**: 耐えられる故障数は要旨にない） | Ben-Or. PODC 1983, pp. 27–30. DOI: 10.1145/800221.806707 |
| 非同期 Byzantine（Bracha） | 非同期、Byzantine 故障、乱数あり | n ≥ 3f+1 で動く乱択の Byzantine agreement と、その部品として reliable broadcast を示した（**要確認**: 要旨を取得できなかった。条件は本文で確かめる） | Bracha. Information and Computation 75(2):130–143, 1987. DOI: 10.1016/0890-5401(87)90054-X |
| 故障検出器（Chandra–Toueg） | 非同期、crash 故障、信頼できない故障検出器 | 間違いを無限回する故障検出器でも合意を解ける。どの検出器なら故障数によらず解けるか、どれなら過半数の正しいプロセスが要るかを示した。非同期・crash 故障では consensus と atomic broadcast は互いに帰着できる | Chandra, Toueg. JACM 43(2):225–267, 1996. DOI: 10.1145/226643.226647 |
| 最弱の故障検出器 | 非同期、crash 故障、過半数が正しい | 合意を解くには、どの故障検出器も少なくとも ◇W と同じだけの情報を出す必要がある。◇W が最弱 | Chandra, Hadzilacos, Toueg. JACM 43(4):685–722, 1996. DOI: 10.1145/234533.234549 |

## 合意より弱い・違う問題

| 結果 | モデル | 主張 | 出典 |
| --- | --- | --- | --- |
| レジスタの模倣（ABD） | 非同期、メッセージ通信、プロセッサの故障 | 過半数のプロセッサが故障せず連結していれば、wait-free で原子的な single-writer multi-reader レジスタを実装できる。これで、そのレジスタを使う共有メモリのアルゴリズムを、合意なしにメッセージ通信で模倣できる | Attiya, Bar-Noy, Dolev. JACM 42(1):124–142, 1995. DOI: 10.1145/200836.200869 |
| wait-free の階層 | 共有メモリ、wait-free | 合意への帰着で「X は Y から wait-free に実装できない」を示す方法を導入し、オブジェクトの階層を作った。原子的な read/write レジスタは最下層で、test&set や fetch&add も弱い。一方、任意の逐次オブジェクトを wait-free に実装できる万能なオブジェクトが存在する | Herlihy. TOPLAS 13(1):124–149, 1991. DOI: 10.1145/114005.102808 |
| atomic commit（Paxos Commit） | 非同期、crash 故障 | 2PC は coordinator が故障すると止まる。Paxos Commit は各参加者のコミット・中止の決定に Paxos を使い、2F+1 個の coordinator のうち F+1 個が動いていれば進む。故障がないときのメッセージ遅延は 2PC と同じにできるが、メッセージ数は多い。2PC は F = 0 の特別な場合 | Gray, Lamport. TODS 31(1):133–160, 2006. DOI: 10.1145/1132863.1132867 |
| Flexible Paxos | Paxos と同じ | Paxos の2つの段階の quorum は、段階をまたいで交われば十分で、同じ段階の quorum どうしが交わる必要はない。過半数の quorum は必須ではない | Howard, Malkhi, Spiegelman. OPODIS 2016（LIPIcs 70、2017 年刊）. DOI: 10.4230/LIPIcs.OPODIS.2016.25 |

## 代表的なプロトコル

比較の基準としてよく使われる。書誌情報は確認済み。

| プロトコル | 出典 |
| --- | --- |
| Paxos | Lamport. "The Part-Time Parliament". TOCS 16(2):133–169, 1998. DOI: 10.1145/279227.279229 |
| PBFT | Castro, Liskov. "Practical Byzantine Fault Tolerance". OSDI 1999, pp. 173–186. https://dl.acm.org/doi/10.5555/296806.296824 |
| Raft | Ongaro, Ousterhout. "In Search of an Understandable Consensus Algorithm". USENIX ATC 2014, pp. 305–320. https://www.usenix.org/conference/atc14/technical-sessions/presentation/ongaro |
| HotStuff | Yin, Malkhi, Reiter, Gueta, Abraham. "HotStuff: BFT Consensus with Linearity and Responsiveness". PODC 2019, pp. 347–356. DOI: 10.1145/3293611.3331591 |
