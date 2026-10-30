#!/usr/bin/env bash
# tlaplus/Examples の Paxos の仕様と、TLC で動かすのに要るモジュールを1つのディレクトリに集める。
# 使い方: bash fetch_paxos_example.sh <保存先のディレクトリ>
set -euo pipefail

dest="${1:?保存先のディレクトリを指定してください}"
mkdir -p "$dest"

# 取得元はコミットで固定する（2026-10-31 に TLC 2.19 で動作を確かめた版）。上げるときは動作を確かめ直す。
examples=https://raw.githubusercontent.com/tlaplus/Examples/0e32a039a4286cc3e1b0f70c00eaff3178d49f85/specifications/Paxos
tlapm=https://raw.githubusercontent.com/tlaplus/tlapm/4744c0f30eb8705f52092230bfc6ec8e8956ba6c/library
community=https://raw.githubusercontent.com/tlaplus/CommunityModules/1805e9d5e1cef05d74806309fb87788e69b48f26/modules

for f in Paxos.tla Voting.tla Consensus.tla MCPaxos.tla MCPaxos.cfg; do
  curl -fsSL -o "$dest/$f" "$examples/$f"
done

# Voting.tla と Consensus.tla が import する TLAPS の library。
# FiniteSets.tla と Bags.tla は標準モジュールと同じ名前なので取らない。
for f in TLAPS.tla FiniteSetTheorems.tla WellFoundedInduction.tla NaturalsInduction.tla; do
  curl -fsSL -o "$dest/$f" "$tlapm/$f"
done

# FiniteSetTheorems.tla が使う CommunityModules のモジュール
for f in Functions.tla Folds.tla; do
  curl -fsSL -o "$dest/$f" "$community/$f"
done

echo "[OK] $dest に集めました。TLC で MCPaxos.tla を動かせます。"
