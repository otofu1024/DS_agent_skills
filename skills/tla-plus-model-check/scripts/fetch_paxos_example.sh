#!/usr/bin/env bash
# tlaplus/Examples の Paxos の仕様と、TLC で動かすのに要るモジュールを1つのディレクトリに集める。
# 使い方: bash fetch_paxos_example.sh <保存先のディレクトリ>
set -euo pipefail

dest="${1:?保存先のディレクトリを指定してください}"
mkdir -p "$dest"

examples=https://raw.githubusercontent.com/tlaplus/Examples/master/specifications/Paxos
tlapm=https://raw.githubusercontent.com/tlaplus/tlapm/main/library
community=https://raw.githubusercontent.com/tlaplus/CommunityModules/master/modules

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
