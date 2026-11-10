#!/usr/bin/env bash
# Persistence repro for a Mekanism machine's saved compound.
#
# Sets a cube's FE and leaves a smelter mid-operation through `/mekanism`, reads
# the live compound with `data get block`, saves, quits (waiting for the world
# lock), relaunches the same world and reads everything again. A mod block
# entity's compound must survive (`docs/native-mods.md`: the amount lives in the
# entity's saved compound, so it survives restarts).
#
# Knobs:
#   SAVE=save|save-all|none   the save before the quit (default `save`)
#   TPAWAY=1                  teleport 3,000 blocks away and back first, so the
#                             machines' chunk unloads and reloads (the unload
#                             save path)
#   KEEP=1                    keep the scratch home
#
# Run it through the slot registry with the port's fast build:
#   build-slot.sh e2e --job <job> --port ports/mekanism -- tools/persist-repro.sh
set -euo pipefail
export LC_ALL=C
exec 1>&2

CLONE=$(cd "$(dirname "$0")/../.." && pwd)
PORT=$CLONE/ports/mekanism
CTL=${CTL:-$CLONE/ports/_shared/bin/horizon-1d63e0a5/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/mekanism-persist.sock}
EVIDENCE=$CLONE/ports/_shared/evidence/mekanism
WORLD=MekanismPersist
CUBE=mekanism:basic_energy_cube
SMELTER=mekanism:energized_smelter
SAVE=${SAVE:-save}
TPAWAY=${TPAWAY:-0}
KEEP=${KEEP:-0}

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/mekanism-persist-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
fail() {
    echo "FAIL: $*" >&2
    ctl logs --lines 60 >&2 || true
    cp "$SCRATCH/game/logs/latest.log" "$EVIDENCE/persist-game.log" 2>/dev/null || true
    exit 1
}

info() { # info X Y Z BLOCK
    say "mekanism info $1 $2 $3" >/dev/null
    ctl wait 4 >/dev/null
    sleep 0.5
    grep -aE "\[CHAT\] $4: " "$SCRATCH/game/logs/latest.log" 2>/dev/null | tail -1 | sed 's/.*\[CHAT\] //' || true
}

nbt() { # nbt X Y Z : the live block entity compound, last reply line
    ctl command "execute in minecraft:the_end run data get block $1 $2 $3" >/dev/null
    sleep 0.5
    grep -aE "\[CHAT\] .*\{.*(energy|progress|id).*\}" "$SCRATCH/game/logs/latest.log" 2>/dev/null | tail -1 | sed 's/.*\[CHAT\] //' || true
}

cleanup() {
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
}
trap cleanup EXIT

launch() {
    local attempt
    for attempt in 1 2 3; do
        rm -f "$SOCK"
        cd "$SCRATCH"
        if "$CTL" --socket "$SOCK" launch \
            --game "$GAME" --world "$WORLD" --seed 20261105 --creative --headless --wait \
            --resources "$RES" --dev-mod "$PORT/content" >/dev/null 2>>"$SCRATCH/game-stderr.log"; then
            ctl wait-ready 120 >/dev/null
            ctl wait 40 >/dev/null
            return 0
        fi
        echo "  (launch attempt $attempt did not come up; waiting for the world lock)"
        sleep 10
    done
    fail "the game did not launch"
}

stop_game() {
    ctl quit >/dev/null 2>&1 || true
    local lock="$SCRATCH/game/saves/$WORLD/session.lock"
    for _ in $(seq 1 40); do
        [ -e "$lock" ] || break
        sleep 1
    done
    sleep 2
}

arena() {
    ctl command "execute in minecraft:the_end run tp @s 0.5 64 3.5"
    local loaded=false
    for _ in $(seq 1 120); do
        ctl wait 5
        if [ "$(ctl block 0 63 0 | jq -r '.loaded // false')" = true ]; then loaded=true; break; fi
    done
    [ "$loaded" = true ] || fail "the End arena chunk never loaded"
    ctl command "execute in minecraft:the_end run fill -8 63 -8 24 63 24 minecraft:stone"
    ctl command "execute in minecraft:the_end run fill -8 64 -8 24 74 24 minecraft:air"
    ctl command "execute in minecraft:the_end run kill @e[type=!player]"
    ctl wait 20
}

echo "== launch, place a cube and a smelter =="
launch
arena
ctl command "execute in minecraft:the_end run setblock 2 64 0 $CUBE"
ctl command "execute in minecraft:the_end run setblock 0 64 0 $SMELTER"
ctl wait 20
echo "  info cube:    $(info 2 64 0 "$CUBE")"
echo "  info smelter: $(info 0 64 0 "$SMELTER")"

echo "== set the values and read them back live =="
say "mekanism energy 6666 2 64 0"
say "mekanism energy 20000 0 64 0"
ctl command "execute in minecraft:the_end run item replace block 0 64 0 container.0 with minecraft:iron_ore 4"
ctl wait 4
# 20,000 FE is two operations; the machine then stalls with two ore left.
ctl wait 460
if [ "$TPAWAY" = 1 ]; then
    echo "== teleport far away so the machines' chunk unloads, then back =="
    ctl command "execute in minecraft:the_end run tp @s 3000.5 64 3000.5"
    ctl wait 200
    ctl command "execute in minecraft:the_end run tp @s 0.5 64 3.5"
    ctl wait 200
fi
BEFORE_CUBE=$(info 2 64 0 "$CUBE")
BEFORE_SMELTER=$(info 0 64 0 "$SMELTER")
echo "  before save, cube:    $BEFORE_CUBE"
echo "  before save, smelter: $BEFORE_SMELTER"
echo "  data cube:    $(nbt 2 64 0)"
echo "  data smelter: $(nbt 0 64 0)"
case "$BEFORE_CUBE" in *" 6666/"*) ;; *) fail "the cube did not take 6666 FE: $BEFORE_CUBE" ;; esac
case "$BEFORE_SMELTER" in
    *"output minecraft:iron_ingot x2"*) ;;
    *) fail "the smelter did not smelt its four ore into two ingots first: $BEFORE_SMELTER" ;;
esac

echo "== save: $SAVE =="
if [ "$SAVE" != none ]; then
    ctl "$SAVE" | sed 's/^/  /'
fi

echo "== quit and relaunch =="
stop_game
launch
ctl wait 40 >/dev/null
AFTER_CUBE=$(info 2 64 0 "$CUBE")
AFTER_SMELTER=$(info 0 64 0 "$SMELTER")
echo "  after relaunch, cube:    $AFTER_CUBE"
echo "  after relaunch, smelter: $AFTER_SMELTER"
echo "  data cube:    $(nbt 2 64 0)"
echo "  data smelter: $(nbt 0 64 0)"

case "$AFTER_CUBE" in
    *" 6666/"*) echo "PERSIST: the cube kept 6666 FE" ;;
    *) echo "LOST: the cube did not keep 6666 FE"; exit 1 ;;
esac
case "$AFTER_SMELTER" in
    *"input minecraft:iron_ore x2"*) echo "PERSIST: the smelter kept its two ore and its output" ;;
    *) echo "LOST: the smelter's slots did not survive: $AFTER_SMELTER"; exit 1 ;;
esac
echo "EXIT:0"
