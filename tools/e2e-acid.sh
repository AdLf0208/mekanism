#!/usr/bin/env bash
# Bounded in-game leg for the slice-17 acid chain.
#
# The full `tools/e2e.sh` re-proves every family from slices 1-16 on every run;
# under a loaded slot queue its older timing assertions can trip before the new
# family's phase. This leg launches the same launcher-built game and runs only
# the acid chain's control + four phases, with the same SELFTEST=acid flip.
# It uses the env block `launcher-game.sh env mekanism` exports (GAME, CTL,
# SOCK, E2E_TMP, RES) and ends with the `EXIT:<status>` line the log gate reads.
set -euo pipefail
exec 1>&2

PORT=$(cd "$(dirname "$0")/.." && pwd)
GAME=${GAME:?set GAME to the launcher-built horizon-client}
CTL=${CTL:-$(dirname "$GAME")/horizon-ctl}
SOCK=${SOCK:-/tmp/opencode/mekanism-acid.sock}
RES=${RES:-${HORIZON_RES:-}}
SELFTEST=${SELFTEST:-0}
TMP_ROOT=${E2E_TMP:-/tmp/opencode}
mkdir -p "$TMP_ROOT"
SCRATCH=$(mktemp -d "$TMP_ROOT"/mekanism-acid.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*"; exit 1; }
cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" 2>/dev/null || true
    echo "EXIT:$status"
}
trap cleanup EXIT

cd "$SCRATCH"
LAUNCH=(--game "$GAME" --world MekanismAcid --seed 20261105 --creative --offscreen --wait)
if [ -n "$RES" ]; then LAUNCH+=(--resources "$RES"); fi
"$CTL" --socket "$SOCK" launch "${LAUNCH[@]}" >/dev/null
ctl wait-ready 120 >/dev/null
ctl wait 40 >/dev/null

ctl command "execute in minecraft:the_end run tp @s 0.5 64 3.5"
for _ in $(seq 1 120); do
    ctl wait 5 >/dev/null
    [ "$(ctl block 0 63 8 | jq -r '.loaded // false')" = true ] && break
done
ctl command "execute in minecraft:the_end run fill -8 63 -8 26 84 30 minecraft:stone" >/dev/null
ctl command "execute in minecraft:the_end run fill -8 64 -8 26 84 30 minecraft:air" >/dev/null
ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
ctl command "difficulty peaceful" >/dev/null
ctl wait 20 >/dev/null

echo "== W (bounded): the locally made acid chain =="
HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
    python3 -u "$PORT/tools/acid_chain.py" "$SOCK" "$CTL" 12 76 8 --control \
    || fail "the sulfurless oxidizer control failed"
FLAGS=()
if [ "$SELFTEST" = 1 ] || [ "$SELFTEST" = acid ]; then FLAGS+=(--selftest); fi
HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
    python3 -u "$PORT/tools/acid_chain.py" "$SOCK" "$CTL" 12 76 8 "${FLAGS[@]}" \
    || fail "the acid chain did not run end to end"
echo "PASS: sulfur dust oxidized to SO2, SO2 and oxygen infused to SO3, SO3 and water vapour infused to locally made sulfuric acid, and that acid dissolved an iron ore into 1000 units of dirty iron; the sulfurless control made no gas and spent no FE"
