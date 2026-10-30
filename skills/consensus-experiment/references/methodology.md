# 実験の方法とコマンド

出典は 2026-10-31 に取得して確認した一次資料。番号は末尾の一覧に対応する。数値の例は出典の例であって、どの実験にも当てはまる値ではない。

## 負荷のかけ方

- **open-loop**: 前の要求の完了を待たず、一定の到着率で送る。**closed-loop**: 応答を待ってから次を送る。同じ公称の負荷でも結果が大きく変わるので、どちらを使ったかを必ず書く [1][2]
- 一定の到着率で測るなら、coordinated omission に注意する。停滞中に送られるはずだった要求を数えないと、テイルレイテンシが小さく見える。wrk2 は「送るはずだった時刻」からレイテンシを測る [2][3]
- 飽和点を探すときは、要求レートを段階的に上げ、スループットとレイテンシの曲線を描く [7]

## 測定

- レイテンシは分布で報告する。CDF と p50・p95・p99・p99.9、サンプル数、測定区間を示す。平均だけにしない [3][4]
- ウォームアップと測定区間を分け、両方の長さを書く。適切な長さは実装と負荷によるので、一般的な値はない（ウォームアップを変えて結果が安定するかを確かめる）[5]
- 独立な実行を繰り返し、ばらつきを示す。必要な回数は、観測したばらつきから決める [6]
- 高いパーセンタイル（p99.9 など）は、十分なサンプル数がないと意味がない [2]

## 変える変数

結果を変える変数は、1つずつ変える [7][8]。

- レプリカ数
- バッチの大きさ
- ペイロードの大きさ
- 要求レート
- レプリカ間の遅延（WAN の模擬）
- 命令の衝突率（EPaxos 系）
- 故障（下記）

## 故障と回復

- リーダーを繰り返し止めて、故障の検知と新しいリーダーの選出にかかる時間を測る（Raft §9.3 の方法）[11]
- 回復時間の終点を定義する。新しいリーダーが決まった時点か、最初のクライアント操作が成功した時点か
- 回復中のサービスへの影響（スループットの時系列）も示す
- crash 故障の実験は Byzantine 故障の検証にはならない。どの故障モデルを試したかを明記する [9][12]

## Linux での遅延・損失・分断

netem は**送信側**のパケットにだけ効く。往復で対称な遅延にするには、両方向（両方のノードのインターフェース）に設定する [10]。

```sh
# 遅延 100ms
sudo tc qdisc add dev eth0 root netem delay 100ms
# 遅延 100ms ± 10ms（相関 25%）
sudo tc qdisc change dev eth0 root netem delay 100ms 10ms 25%
# 損失 0.1%
sudo tc qdisc change dev eth0 root netem loss 0.1%
# 片付け
sudo tc qdisc del dev eth0 root
```

1組のノード間の分断（相手が 10.0.0.2 の場合）。複数ノードの分断では、分断をまたぐ全ての組に同じ規則を入れる [18]。

```sh
sudo iptables -I OUTPUT -d 10.0.0.2 -j DROP
sudo iptables -I INPUT  -s 10.0.0.2 -j DROP
# 回復
sudo iptables -D OUTPUT -d 10.0.0.2 -j DROP
sudo iptables -D INPUT  -s 10.0.0.2 -j DROP
```

1台の上でノードを分けるには、ネットワーク名前空間を使う。名前空間ごとに別のネットワークスタック、経路、ファイアウォールの規則を持つ [19][20]。

```sh
sudo ip netns add n1
sudo ip netns add n2
sudo ip link add v1 type veth peer name v2
sudo ip link set v1 netns n1
sudo ip link set v2 netns n2
```

コンテナを使う場合は、ユーザー定義のブリッジネットワークを作り、各レプリカを1つのコンテナにする。遅延や分断は、該当するインターフェースに tc や iptables で入れる [21]。

```sh
docker network create --driver bridge consensus-net
```

注入した規則は、実験の後に必ず消す。消すコマンドを実験の記録に残す。

## シミュレーションとの使い分け

- Maelstrom は、ノードのプログラムをシミュレートしたネットワークでつなぎ、遅延、損失、分断、停止を入れて履歴を記録する。プロトコルの振る舞いを素早く繰り返し試すのに向く [22]
- 実ネットワークの性能を主張するなら、tc・名前空間・コンテナなどで本物のネットワークスタックを通した実験を行う。シミュレーションの数値を実ネットワークの性能として示さない

## 正しさの検査

性能実験でも、クライアントの操作と結果を記録し、主張する一貫性で検査する [14]。

- Porcupine: 逐次仕様と並行な履歴を渡すと、linearizable かを判定する [13]
- Knossos: 単一スレッドのモデルに対する履歴の検査。README 自身が、アルゴリズムの正しさは確かではなく、結果を手で確かめるよう勧めている [23]
- Elle: トランザクションの履歴の異常を検出する。合意の API の linearizability 検査にそのまま使うものではない [24]
- Twins: 同じ ID と鍵を持つ双子を動かして、Byzantine な振る舞い（リーダーの二枚舌、二重投票、ロックの忘却）を試す BFT 実装のテスト手法。選んだシナリオを試すもので、証明ではない [15]

履歴の検査は、その実行についての証拠であって、全てのスケジュールについての証明ではない。

## アーティファクトの形

ACM のバッジは Available、Evaluated—Functional、Evaluated—Reusable、Results Reproduced を区別する。Available には、DOI などの識別子とライセンスを持つ公開アーカイブが要る [16]。USENIX NSDI '26 は、短い動作確認（Getting Started）と全体の再現（Detailed Instructions）を README に分けて書くことを求めている [17]。

残すもの: ソースの版（コミット）かバイナリ・コンテナ、負荷生成器と検査器の版、設定、スクリプト、生の履歴と結果、グラフを作るスクリプト、依存関係、マシン・クラスタ・ネットワークの構成。

## 出典

1. Schroeder, Wierman, Harchol-Balter. "Open Versus Closed: A Cautionary Tale". NSDI 2006. https://www.usenix.org/legacy/event/nsdi06/tech/full_papers/schroeder/schroeder_html/
2. wrk2 README. https://raw.githubusercontent.com/giltene/wrk2/master/README.md
3. HdrHistogram `AbstractHistogram` documentation. https://hdrhistogram.github.io/HdrHistogram/JavaDoc/org/HdrHistogram/AbstractHistogram.html
4. HdrHistogram README. https://raw.githubusercontent.com/HdrHistogram/HdrHistogram/master/README.md
5. SPECweb2005 benchmark design. https://www.spec.org/osg/web2005/docs/designdocument.html
6. Kalibera, Jones. "Rigorous Benchmarking in Reasonable Time". ISMM 2013. https://kar.kent.ac.uk/33611/45/p63-kaliber.pdf
7. HotStuff（arXiv 版）§§8.2–8.4. https://arxiv.org/pdf/1803.05069
8. EPaxos（SOSP 2013）§§7.1、7.2、7.6. https://www.pdl.cmu.edu/PDL-FTP/associated/epaxos-sosp2013.pdf
9. Bullshark §§9–10. https://arxiv.org/pdf/2201.05677
10. tc-netem(8). https://man7.org/linux/man-pages/man8/tc-netem.8.html
11. Raft（USENIX ATC 2014）§9.3. https://raft.github.io/raft.pdf
12. Narwhal and Tusk §7. https://arxiv.org/pdf/2105.11827
13. Porcupine README. https://raw.githubusercontent.com/anishathalye/porcupine/master/README.md
14. Jepsen の履歴と検査の概要. https://raw.githubusercontent.com/jepsen-io/jepsen/main/doc/whats-here.md
15. Twins. https://arxiv.org/pdf/2004.10617
16. ACM Artifact Review and Badging（ACM SIGSIM PADS の案内）. https://sigsim.acm.org/conf/pads/2027/blog/artifact-evaluation/
17. USENIX NSDI '26 Call for Artifacts. https://www.usenix.org/conference/nsdi26/call-for-artifacts
18. iptables(8). https://man7.org/linux/man-pages/man8/iptables.8.html
19. ip-netns(8). https://man7.org/linux/man-pages/man8/ip-netns.8.html
20. ip-link(8). https://man7.org/linux/man-pages/man8/ip-link.8.html
21. Docker bridge network driver. https://docs.docker.com/engine/network/drivers/bridge/
22. Maelstrom README. https://raw.githubusercontent.com/jepsen-io/maelstrom/main/README.md
23. Knossos README. https://raw.githubusercontent.com/jepsen-io/knossos/master/README.md
24. Elle README. https://raw.githubusercontent.com/jepsen-io/elle/master/README.markdown
