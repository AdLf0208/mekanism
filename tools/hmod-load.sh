#!/usr/bin/env bash
# Proves the built content package (mekanism.hmod) loads on a real game: the
# shared, unmodded 1d63e0a5 game with `--dev-mod <the .hmod>` must register the
# three blocks and their items. A second launch WITHOUT `--dev-mod` is the
# negative control, so the check cannot pass vacuously.
set -euo pipefail
export LC_ALL=C
# Line-buffered evidence: every line goes to stderr (unbuffered when redirected).
exec 1>&2

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
PIN_BIN=$CLONE/ports/_shared/bin/horizon-1d63e0a5
CTL=${CTL:-$PIN_BIN/horizon-ctl}
GAME=${GAME:-$PIN_BIN/horizon-client}
HMOD=${HMOD:-$CLONE/ports/mekanism/mekanism.hmod}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/mekanism-hmod.sock}

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -f "$HMOD" ] || { echo "missing package: $HMOD" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/mekanism-hmod-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }

cleanup() {
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "${KEEP:-0}" = 1 ]; then echo "kept scratch: $SCRATCH" >&2; else rm -rf "$SCRATCH" 2>/dev/null || true; fi
}
trap cleanup EXIT

launch() { # launch WORLD [--dev-mod PATH]
    local world=$1; shift
    rm -f "$SOCK"
    cd "$SCRATCH"
    "$CTL" --socket "$SOCK" launch \
        --game "$GAME" --world "$world" --seed 20261105 --creative --headless --wait \
        --resources "$RES" "$@" >/dev/null 2>>"$SCRATCH/game.log"
    ctl wait-ready 120
    ctl wait 20
}

echo "== launch WITH the package (--dev-mod $HMOD) =="
launch MekanismHmod --dev-mod "$HMOD"
for pair in "0 mekanism:energized_smelter" "2 mekanism:basic_energy_cube" "4 mekanism:basic_universal_cable" "6 mekanism:enrichment_chamber"; do
    set -- $pair
    ctl command "setblock $1 64 0 $2" >/dev/null
    ctl wait 4
    block=$(ctl block "$1" 64 0 | jq -r '.name // .block // empty')
    echo "block $1,64,0 = $block"
    [ "$block" = "$2" ] || fail "the package did not register $2"
done
ctl give mekanism:energized_smelter 1 >/dev/null
ctl give mekanism:dust_iron 1 >/dev/null
ctl wait 6
ctl inventory | jq -e '[.hotbar[] | select(.item == "mekanism:energized_smelter")] | length >= 1' >/dev/null \
    || fail "the package did not register the energized smelter item: $(ctl inventory | jq -c '{hotbar,main}')"
ctl inventory | jq -e '[.hotbar[] | select(.item == "mekanism:dust_iron")] | length >= 1' >/dev/null \
    || fail "the package did not register the iron dust item: $(ctl inventory | jq -c '{hotbar,main}')"
echo "items: $(ctl inventory | jq -c '[.hotbar[] | select(.item == "mekanism:energized_smelter" or .item == "mekanism:dust_iron") | .item]')"
ctl quit >/dev/null 2>&1 || true
sleep 2

echo "== negative control: the SAME ids without --dev-mod =="
launch MekanismHmodControl
out=$(ctl command "setblock 0 64 0 mekanism:energized_smelter")
echo "setblock reply: $out"
if grep -q 'Changed the block' <<<"$out"; then
    fail "the unmodded game accepted mekanism:energized_smelter (the check is vacuous)"
fi
block=$(ctl block 0 64 0 | jq -r '.name // .block // empty')
echo "control block 0,64,0 = $block"
[ "$block" != "mekanism:energized_smelter" ] || fail "the control block is the smelter"
out=$(ctl command "setblock 2 64 0 mekanism:enrichment_chamber")
echo "setblock reply: $out"
if grep -q 'Changed the block' <<<"$out"; then
    fail "the unmodded game accepted mekanism:enrichment_chamber (the check is vacuous)"
fi
ctl give mekanism:energized_smelter 1 >/dev/null 2>&1 || true
ctl give mekanism:dust_iron 1 >/dev/null 2>&1 || true
ctl wait 6
ctl inventory | jq -e '[.hotbar[] | select(.item == "mekanism:energized_smelter")] | length == 0' >/dev/null \
    || fail "the unmodded game gave a mekanism smelter item"
ctl inventory | jq -e '[.hotbar[] | select(.item == "mekanism:dust_iron")] | length == 0' >/dev/null \
    || fail "the unmodded game gave a mekanism dust item"

echo "PASS: mekanism.hmod registered mekanism:energized_smelter, mekanism:basic_energy_cube, mekanism:basic_universal_cable and mekanism:enrichment_chamber with the energized smelter and iron dust items on a real game, and the same ids are unknown without the package"
echo "EXIT:0"
