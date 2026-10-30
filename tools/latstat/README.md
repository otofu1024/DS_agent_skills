# latstat

分散合意・state machine replication のベンチマーク CSV を、独立した実行ごとに要約する小さな CLI です。latency の分位点、スループット、失敗数を出力し、複数 run があれば run 間の信頼区間も計算します。

## 入力形式

CSV はヘッダー行が必須です。各ファイルを独立した 1 run として扱います。

```csv
latency_us,start_us,ok
12000,0,true
15000,1000000,true
22000,2000000,false
```

latency 列は必須です。start 列と ok 列は任意で、start がない場合は warm-up/cool-down を指定できず、window と throughput は出力しません。ok 列がなければ全操作を成功として扱います。失敗操作は window 内の失敗数に数え、latency 統計から除外します。`kept` は latency 統計に含めた成功操作数です。

warm-up は `最小 start + 秒数` より前の操作を除外し、cool-down は `最大 start - 秒数` より後の操作を除外します。境界時刻の操作は残します。window はこの二つの境界間の秒数、throughput は成功操作数を window で割った値です。

## フラグ

| フラグ | 既定値 | 説明 |
| --- | --- | --- |
| `CSV...` | なし | 1 ファイルにつき独立した 1 run。1 個以上必要です。 |
| `--latency-column NAME` | `latency_us` | latency 列名。 |
| `--start-column NAME` | `start_us` | 任意の開始時刻列名。latency と同じ `--unit`。 |
| `--ok-column NAME` | `ok` | 任意の成功フラグ列名。値は `true`/`false`/`1`/`0`。 |
| `--unit us / ms / ns` | `us` | 入力の latency と start 時刻の単位。 |
| `--warmup SECONDS` | なし | run 冒頭から指定秒数を除外します。start 列が必要です。 |
| `--cooldown SECONDS` | なし | run 末尾から指定秒数を除外します。start 列が必要です。 |
| `--json` | off | 表の代わりに JSON を出力します。latency 値は常に µs で、各単位も出力します。 |
| `--cdf PATH` | なし | 各 run の empirical CDF を CSV に書きます。列は `run,latency_us,fraction` です。 |
| `--cdf-points POINTS` | `1000` | CDF の run ごとの最大点数。2 以上。min と max は残します。 |
| `--help` (`-h`) | — | ヘルプを表示します。nearest-rank 定義も表示します。 |
| `--version` (`-V`) | — | バージョンを表示します。 |

`--json` の出力では、run ごとの latency のフィールドは `min_us`, `p50_us`, `p90_us`, `p99_us`, `p99_9_us`, `max_us`, `mean_us`, `stddev_us` です。start 列がなければ `window_seconds` と `throughput_ops_per_second` は `null` になります。

## 統計の定義

分位点には **nearest rank** を使います。値を昇順に並べ、確率 `p` と件数 `n` から 1-indexed の rank `max(1, ceil(p × n))` を選びます。run 内の標準偏差は母標準偏差（分母 `n`）です。

2 run 以上では各数値指標について run 間の平均、標本標準偏差 `s`（分母 `n−1`）、両側 95% Student-t 区間 `mean ± t(0.975, n−1) × s / √n` を出します。t の分位点は Student-t 分布の逆 CDF で求めます。テストでは `t(0.975,4)=2.776` と `t(0.975,9)=2.262` になることを確かめています。5 run 未満では信頼区間が不安定になり得る旨を stderr に警告します。

CDF は同じ latency のサンプルをまとめ、各点に `latency 以下のサンプル数 / 成功サンプル数` を記録します。点数上限を超える場合は等間隔に downsample し、最小値と最大値を必ず残します。

## 実行例

```sh
cargo run --release -- tests/fixtures/run1.csv tests/fixtures/run2.csv --warmup 1
```

```text
warning: 2 independent run(s) provided; fewer than 5 runs may give imprecise across-run confidence intervals
Per-run summary (latency in ms):
run | kept | warm-up | cool-down | failed | window(s) | successful ops/s | min | p50 | p90 | p99 | p99.9 | max | mean | sd
tests/fixtures/run1.csv | 4 | 1 | 0 | 0 | 3.000 | 1.333 | 12.000 | 14.000 | 20.000 | 20.000 | 20.000 | 20.000 | 15.500 | 2.958
tests/fixtures/run2.csv | 4 | 1 | 0 | 0 | 3.000 | 1.333 | 22.000 | 24.000 | 30.000 | 30.000 | 30.000 | 30.000 | 26.000 | 3.162

Across-run summaries (mean, sample SD, two-sided 95% Student t CI):
metric | n | unit | mean | sample SD | CI low | CI high
...
latency_p50 | 2 | ms | 19.000 | 7.071 | -44.531 | 82.531
...
```
