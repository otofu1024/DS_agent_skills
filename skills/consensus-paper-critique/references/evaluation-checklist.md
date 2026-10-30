# 評価（実験）を読むときのチェックリスト

論文の評価の節を読むときに、項目ごとに「書かれている / 書かれていない / 主張の範囲と合わない / 対象外」を判定する。
論文の評価の種類に当てはまらない項目（性能の測定がない論文での負荷のかけ方など）は「対象外」とし、欠点に数えない。
出典は 2026-10-31 に取得して確認した一次資料。番号は末尾の一覧に対応する。

## 負荷とレイテンシ

- クライアントが open-loop（一定の到着率で送る）か closed-loop（応答を待ってから次を送る）かが書かれているか。同じ公称の負荷でも結果が大きく変わる [1][2]
- 一定の到着率を前提にするなら、coordinated omission（停滞中に送られるはずだった要求を数え損なう）を扱っているか [2][3]
- レイテンシを平均だけで示していないか。分布（CDF、p50・p99・p99.9）と、サンプル数・測定区間があるか [3][4]
- ウォームアップと測定区間を分けて書いているか [5]
- 繰り返しの回数と、ばらつき（信頼区間や標準偏差）が示されているか。1回の実行を結果としていないか [6][7]

## 設定

- バッチの大きさ、ペイロードの大きさ、要求レート、レプリカ数が書かれているか。どれも結果を変える [7][8]
- 命令の衝突で性能が変わるプロトコル（EPaxos など）では、衝突率が書かれているか [8]
- ベースラインと条件（言語、ライブラリ、ストレージ、ネットワーク、ハードウェア、チューニング）が揃っているか。揃えられない理由が書かれているか [9]

## 環境

- WAN での振る舞いを主張するなら、実際の地理分散か、遅延の注入があるか [7][8][9]
- netem は送信側のパケットにしか効かない。往復で対称な遅延を主張するなら、両方向に設定しているか [10]

## 故障

- 故障なしの定常状態だけでなく、故障の検知、リーダー交代や view change、回復時間を測っているか [7][9][11]
- 回復時間の終点（新しいリーダーが決まった時点か、最初のクライアント操作が成功した時点か）が明記されているか（[11] を参考にした推奨）
- crash 故障の実験を、Byzantine 故障の検証として扱っていないか [9][12]

## 正しさ

- 性能実験の操作履歴を、主張する一貫性（linearizability など）で検査しているか [13][14]
- 1つの履歴の検査や、選んだ攻撃シナリオのテストを、証明のように扱っていないか [14][15]

## 再現性

- アーティファクトが公開されているか。DOI つきのアーカイブか。短い動作確認の手順と、全体を再現する手順があるか [16][17]

## 人を対象にした実験

「理解しやすい」「使いやすい」といった主張を、人を対象にした実験で示している場合に使う問い。

- 何を測ったか。理解度のテストの点数か、実装の正しさか、作業時間か。測ったものと主張（「理解しやすい」）の間にどんな隔たりがあるか
- 誰を対象にしたか。人数、選び方、事前の知識。結論をどの範囲の人に広げてよいか
- 比べる条件はそろっているか。教材の量と質、講義の長さ、説明した人
- 順序の効果を除いているか。両方の条件を受けるなら、順序を入れ替えた群を作っているか
- 差の大きさと、ばらつきや信頼区間を示しているか。どの統計検定を使ったか
- 測った時点はいつか。直後のテストだけか、時間をおいた保持も測ったか

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
