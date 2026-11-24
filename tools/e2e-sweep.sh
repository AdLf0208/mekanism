#!/usr/bin/env bash
# Bounded in-game leg for the slice-16 12-row dissolution sweep.
#
# The full `tools/e2e.sh` re-proves every family from slices 1-15 on every run;
# under a loaded slot queue its older timing assertions can trip before phase
# V3 is reached. This leg launches the same launcher-built game and runs only
# the new family's control + 12 rows, with the same fail-capable deepslate flip.
# It uses the same env block as `launcher-game.sh env mekanism` (GAME, CTL,
# SOCK, E2E_TMP, RES) and prints the last `EXIT:<status>` line the log gate
# reads.
set -euo pipefail
exec 1>&2

PORT=$(cd "$(dirname "$0")/.." && pwd)
GAME=${GAME:?set GAME to the launcher-built horizon-client}
CTL=${CTL:-$(dirname "$GAME")/horizon-ctl}
SOCK=${SOCK:-/tmp/opencode/mekanism-sweep.sock}
RES=${RES:-${HORIZON_RES:-}}
SELFTEST=${SELFTEST:-0}
TMP_ROOT=${E2E_TMP:-/tmp/opencode}
mkdir -p "$TMP_ROOT"
SCRATCH=$(mktemp -d "$TMP_ROOT"/mekanism-sweep.XXXXXX)
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
LAUNCH=(--game "$GAME" --world MekanismSweep --seed 20261105 --creative --offscreen --wait)
if [ -n "$RES" ]; then LAUNCH+=(--resources "$RES"); fi
"$CTL" --socket "$SOCK" launch "${LAUNCH[@]}" >/dev/null
ctl wait-ready 120 >/dev/null
ctl wait 40 >/dev/null

# The End arena: the sweep's pairs stand at x=8..25, y=74, z=4.
ctl command "execute in minecraft:the_end run tp @s 0.5 64 3.5"
for _ in $(seq 1 120); do
    ctl wait 5 >/dev/null
    [ "$(ctl block 0 63 0 | jq -r '.loaded // false')" = true ] && break
done
ctl command "execute in minecraft:the_end run fill -8 63 -8 26 63 30 minecraft:stone" >/dev/null
ctl command "execute in minecraft:the_end run fill -8 64 -8 26 74 30 minecraft:air" >/dev/null
ctl command "execute in minecraft:the_end run kill @e[type=!player]" >/dev/null
ctl command "difficulty peaceful" >/dev/null
ctl wait 20 >/dev/null

ROWS=$SCRATCH/dissolving-rows.json
python3 -u "$PORT/tools/gen_dissolving.py" --rows >"$ROWS" \
    || fail "the dissolution rows could not be read from the jar"
echo "== V3 (bounded): all twelve carried dissolution rows in real chambers =="
HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
    python3 -u "$PORT/tools/dissolve_sweep.py" "$SOCK" "$CTL" --control 8 74 4 \
    || fail "the dissolution no-row control failed"
FLAGS=()
if [ "$SELFTEST" = 1 ]; then FLAGS+=(--selftest); fi
HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
    python3 -u "$PORT/tools/dissolve_sweep.py" "$SOCK" "$CTL" "$ROWS" 8 74 4 \
        --batch 6 "${FLAGS[@]}" \
    || fail "the 12-row dissolution sweep failed"
echo "PASS: all twelve carried dissolution rows and the dirt control ran in real chambers with their declared item count, per-tick acid, slurry and amount, the input consumed, 40,000 FE spent and the source tank short exactly its acid"
