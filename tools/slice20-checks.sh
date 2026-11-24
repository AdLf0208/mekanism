#!/usr/bin/env bash
# The generated recipe/art/tag checks for slice 20, one lease.
set -euo pipefail
PORT=$(cd "$(dirname "$0")/.." && pwd)
cd "$PORT"
for gen in smelting enriching crushing injecting purifying washing crystallizing dissolving oxidizing infusing separating condensentrating; do
    echo "--- gen_$gen ---"
    python3 "tools/gen_${gen}.py" --check
done
for carry in acid crystallizer dissolution ore_chain separator shards storage condensentrator; do
    echo "--- carry_$carry ---"
    python3 "tools/carry_${carry}.py" --check
done
echo "--- block tags ---"
python3 tools/check-block-tags.py
echo "--- recipe types ---"
python3 /home/adlf/Desktop/all_the_mods_10_rust_rewrite/ports/tools/check-recipe-types.py mekanism
