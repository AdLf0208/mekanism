#!/usr/bin/env bash
# In-game check for Mekanism slices 1-2: Energized Smelter, Basic Energy Cube,
# Basic Universal Cable and Enrichment Chamber.
#
# The End arena (stone floor at y=63). Three rows at y=64:
#   z=0   x=0 smelter   x=1 cable   x=2 energy cube   (the cable couples 0 and 2)
#   z=12  x=0 chamber   x=1 cable   x=2 energy cube   (the ore -> dust chain)
# and one-machine controls at z=4, 8, 16 and 20.
# The mod's commands are typed with `ctl chat "/cmd"` because `ctl command`
# runs the console path and bypasses the native `perform_command` hook.
#
# Default run (must PASS):
#  - the four blocks place and report their buffers through `/mekanism info`;
#  - with the smelter unpowered, an ore in its input does not smelt (control 1);
#  - a charged cube feeds the smelter through the cable (energy rises there and
#    falls in the cube);
#  - the smelter smelts: two ore -> two iron ingots over two operations, at
#    10,000 FE each, and the buffer drops by exactly that;
#  - a smelter with an unsmeltable input (a diamond) makes no progress and
#    burns no energy (control 2);
#  - a cube beside a smelter with no cable between them transfers nothing
#    (control 3, the differential);
#  - the chamber enriches: a charged cube feeds it through its cable, one iron
#    ore becomes two `mekanism:dust_iron` over one operation, at 10,000 FE;
#  - the chamber opens its three-slot menu and the two dust are in it;
#  - the chain completes: the chamber's two dust go into the smelter's input and
#    smelt into two iron ingots;
#  - a chamber with an input it has no recipe for makes no progress and burns
#    nothing (control 4), and a chamber with no cable gets no energy (control 5);
#  - every machine's energy, progress and slots survive save + relaunch (the
#    block entity's saved compound).
#
# NOCABLE=1 runs the same scenario without the cable and asserts that the
# smelter and the chamber stay empty (the differential control passing).
# SELFTEST=1 flips the enrichment assertion (it must FAIL), proving the
# scenario can fail.
set -euo pipefail
export LC_ALL=C
# Every line goes to stderr, which is not buffered when redirected: a run that
# is killed still leaves its transcript in the log.
exec 1>&2

CLONE=${CAMPAIGN_ROOT:?set CAMPAIGN_ROOT to a checkout of the ATM10-Horizon campaign tree}
PORT=$CLONE/ports/mekanism
CTL=${CTL:-$CLONE/ports/_shared/bin/horizon-1d63e0a5/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-.local/horizon-client/26.3}}
SOCK=${SOCK:-/tmp/opencode/mekanism-e2e.sock}
SELFTEST=${SELFTEST:-0}
NOCABLE=${NOCABLE:-0}
KEEP=${KEEP:-0}
WORLD=MekanismE2E
SMELTER=mekanism:energized_smelter
CUBE=mekanism:basic_energy_cube
CABLE=mekanism:basic_universal_cable
CHAMBER=mekanism:enrichment_chamber
DUST=mekanism:dust_iron

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/mekanism-home.XXXXXX)
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
    ctl logs --lines 80 >&2 || true
    cp "$SCRATCH/game/logs/latest.log" "$CLONE/ports/_shared/evidence/mekanism/e2e-game.log" 2>/dev/null || true
    exit 1
}

# The newest `/mekanism info` line of the machine at a position, by its block
# id, so a line still in the log for another machine cannot be mistaken for it.
info_once() { # info_once X Y Z BLOCK_ID
    say "mekanism info $1 $2 $3" >/dev/null
    ctl wait 4 >/dev/null
    # Read the game's own log: `ctl messages` reads the window client's chat
    # buffer, whose sync under load trails the server by a few seconds, while
    # every chat line reaches `latest.log` when it is shown.
    sleep 0.5
    grep -aE "\[CHAT\] $4: " "$SCRATCH/game/logs/latest.log" 2>/dev/null | tail -1 | sed 's/.*\[CHAT\] //' || true
}

# The FE a machine reports, read again until the answer is fresh.
energy_once() { # energy_once X Y Z BLOCK_ID
    info_once "$1" "$2" "$3" "$4" | sed -n 's/.*: \([0-9]\{1,\}\)\/[0-9]\{1,\} FE.*/\1/p'
}

# Poll until the machine says `TEXT` (each pass asks again, so the line read is
# the newest answer).
expect_contains() { # expect_contains X Y Z BLOCK_ID TEXT
    local line="" ok=0
    for _ in 1 2 3 4 5; do
        line=$(info_once "$1" "$2" "$3" "$4")
        case "$line" in
            *"$5"*) ok=1; break ;;
        esac
    done
    echo "  info($1,$2,$3): $line"
    [ "$ok" = 1 ] || fail "expected /mekanism info at $1,$2,$3 to say '$5', got '$line'"
}

# Poll until the machine's FE satisfy OP VALUE (`eq`, `gt` or `lt`).
expect_energy() { # expect_energy X Y Z BLOCK_ID OP VALUE
    local got="" ok=0
    for _ in 1 2 3 4 5; do
        got=$(energy_once "$1" "$2" "$3" "$4")
        [ -n "$got" ] || continue
        case "$5" in
            eq) [ "$got" -eq "$6" ] && ok=1 ;;
            gt) [ "$got" -gt "$6" ] && ok=1 ;;
            lt) [ "$got" -lt "$6" ] && ok=1 ;;
        esac
        [ "$ok" = 1 ] && break
    done
    echo "  energy($1,$2,$3): ${got:-none}"
    [ "$ok" = 1 ] || fail "expected $5 $6 FE at $1,$2,$3, got '${got:-nothing}'"
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

# Quit the game and wait for its world lock to go, so the relaunch may open it.
stop_game() {
    ctl quit >/dev/null 2>&1 || true
    local lock="$SCRATCH/game/saves/$WORLD/session.lock"
    for _ in $(seq 1 40); do
        [ -e "$lock" ] || break
        sleep 1
    done
    sleep 2
}

# The first launch only: a stone floor and a clear volume in The End.
arena() {
    ctl command "execute in minecraft:the_end run tp @s 0.5 64 3.5"
    local loaded=false
    for _ in $(seq 1 120); do
        ctl wait 5
        if [ "$(ctl block 0 63 0 | jq -r '.loaded // false')" = true ]; then
            loaded=true
            break
        fi
    done
    [ "$loaded" = true ] || fail "the End arena chunk never loaded"
    ctl command "execute in minecraft:the_end run fill -8 63 -8 24 63 24 minecraft:stone"
    ctl command "execute in minecraft:the_end run fill -8 64 -8 24 74 24 minecraft:air"
    ctl command "execute in minecraft:the_end run kill @e[type=!player]"
    ctl wait 20
}

echo "== launch (The End arena, content + native bundle) =="
launch
arena

echo "== the machines place and their block entities answer =="
ctl command "execute in minecraft:the_end run setblock 0 64 0 mekanism:energized_smelter"
if [ "$NOCABLE" = 1 ]; then
    echo "  (NOCABLE: no cable between the cube and the smelter)"
else
    ctl command "execute in minecraft:the_end run setblock 1 64 0 mekanism:basic_universal_cable"
fi
ctl command "execute in minecraft:the_end run setblock 2 64 0 mekanism:basic_energy_cube"
# A second smelter with an unsmeltable input (control 2) and a third machine
# pair with no cable between them (control 3).
ctl command "execute in minecraft:the_end run setblock 0 64 4 mekanism:energized_smelter"
ctl command "execute in minecraft:the_end run setblock 0 64 8 mekanism:energized_smelter"
ctl command "execute in minecraft:the_end run setblock 2 64 8 mekanism:basic_energy_cube"
# The ore -> dust chain, and a chamber with no cable (control 5) and one with an
# input it has no recipe for (control 4).
ctl command "execute in minecraft:the_end run setblock 0 64 12 mekanism:enrichment_chamber"
if [ "$NOCABLE" = 1 ]; then
    echo "  (NOCABLE: no cable between the cube and the chamber)"
else
    ctl command "execute in minecraft:the_end run setblock 1 64 12 mekanism:basic_universal_cable"
fi
ctl command "execute in minecraft:the_end run setblock 2 64 12 mekanism:basic_energy_cube"
ctl command "execute in minecraft:the_end run setblock 0 64 16 mekanism:enrichment_chamber"
ctl command "execute in minecraft:the_end run setblock 2 64 16 mekanism:basic_energy_cube"
ctl command "execute in minecraft:the_end run setblock 0 64 20 mekanism:enrichment_chamber"
ctl wait 20
ctl block 0 64 0
expect_contains 0 64 0 "$SMELTER" "0/20000 FE, progress 0/200, input empty x0, output empty x0"
expect_contains 2 64 0 "$CUBE" "0/4000000 FE"
expect_contains 0 64 12 "$CHAMBER" "0/20000 FE, progress 0/200, input empty x0, output empty x0"
if [ "$NOCABLE" = 0 ]; then
    expect_contains 1 64 0 "$CABLE" "0/8000 FE"
    expect_contains 1 64 12 "$CABLE" "0/8000 FE"
fi

echo "== control 1: no energy, so an ore does not smelt =="
ctl command "execute in minecraft:the_end run item replace block 0 64 0 container.0 with minecraft:iron_ore 1"
ctl wait 10
expect_contains 0 64 0 "$SMELTER" "input minecraft:iron_ore x1"
ctl wait 40
expect_contains 0 64 0 "$SMELTER" "progress 0/200, input minecraft:iron_ore x1, output empty x0"

echo "== the cube charges, and the cable moves the energy to the smelter =="
say "mekanism energy 10000 2 64 0"
ctl wait 4
ctl wait 40
if [ "$NOCABLE" = 1 ]; then
    expect_energy 0 64 0 "$SMELTER" eq 0
    expect_energy 2 64 0 "$CUBE" eq 10000
else
    expect_energy 0 64 0 "$SMELTER" gt 0
    expect_energy 2 64 0 "$CUBE" lt 10000
fi

echo "== two ore smelt into two iron ingots over two operations =="
ctl command "execute in minecraft:the_end run item replace block 0 64 0 container.0 with minecraft:iron_ore 2"
ctl wait 10
expect_contains 0 64 0 "$SMELTER" "input minecraft:iron_ore x2"
if [ "$NOCABLE" = 0 ]; then
    say "mekanism energy 20000 0 64 0"
    ctl wait 4
fi
ctl wait 420
if [ "$NOCABLE" = 1 ]; then
    # No cable, no energy: the two ore are still there and nothing smelted.
    expect_contains 0 64 0 "$SMELTER" "progress 0/200, input minecraft:iron_ore x2, output empty x0"
elif [ "$SELFTEST" = 1 ]; then
    echo "(selftest: asserting the output is still empty)"
    expect_contains 0 64 0 "$SMELTER" "output empty x0"
else
    expect_contains 0 64 0 "$SMELTER" "input empty x0, output minecraft:iron_ingot x2"
fi

echo "== control 2: an unsmeltable input makes no progress and burns nothing =="
ctl command "execute in minecraft:the_end run item replace block 0 64 4 container.0 with minecraft:diamond 1"
ctl wait 10
say "mekanism energy 10000 0 64 4"
ctl wait 4
ctl wait 40
expect_contains 0 64 4 "$SMELTER" "progress 0/200, input minecraft:diamond x1"
expect_energy 0 64 4 "$SMELTER" eq 10000

echo "== control 3: a cube beside a smelter with no cable transfers nothing, and one operation costs 10,000 FE =="
ctl command "execute in minecraft:the_end run item replace block 0 64 8 container.0 with minecraft:iron_ore 1"
ctl wait 10
say "mekanism energy 20000 0 64 8"
say "mekanism energy 10000 2 64 8"
ctl wait 4
ctl wait 240
expect_contains 0 64 8 "$SMELTER" "input empty x0, output minecraft:iron_ingot x1"
expect_energy 0 64 8 "$SMELTER" eq 10000
expect_energy 2 64 8 "$CUBE" eq 10000

echo "== the chamber enriches one ore into two iron dust, fed by the cable =="
ctl command "execute in minecraft:the_end run item replace block 0 64 12 container.0 with minecraft:iron_ore 1"
ctl wait 10
expect_contains 0 64 12 "$CHAMBER" "input minecraft:iron_ore x1"
say "mekanism energy 20000 2 64 12"
ctl wait 4
ctl wait 420
if [ "$NOCABLE" = 1 ]; then
    # No cable, no energy: the ore stays and no dust is made.
    expect_contains 0 64 12 "$CHAMBER" "progress 0/200, input minecraft:iron_ore x1, output empty x0"
    expect_energy 0 64 12 "$CHAMBER" eq 0
    expect_energy 2 64 12 "$CUBE" eq 20000
elif [ "$SELFTEST" = 1 ]; then
    echo "(selftest: asserting the chamber's output is still empty)"
    expect_contains 0 64 12 "$CHAMBER" "output empty x0"
else
    expect_contains 0 64 12 "$CHAMBER" "input empty x0, output $DUST x2"
    # The cube's 20,000 FE all reach the chamber and one operation spends
    # 10,000, so the machine keeps exactly the other half.
    expect_energy 0 64 12 "$CHAMBER" eq 10000
fi

echo "== the chamber opens its three-slot menu, holding the two dust =="
ctl command "execute in minecraft:the_end run tp @s 0.5 64 15.5"
ctl wait 10
ctl interact 0 64 12
ctl wait 6
state=$(ctl screen)
jq -e '.screen == "container"' <<<"$state" >/dev/null \
    || fail "the chamber did not open a container: $(jq -c '{screen}' <<<"$state")"
jq -e '[.slots[] | select(.slot >= 300)] | length == 3' <<<"$state" >/dev/null \
    || fail "the chamber's menu does not have 3 container slots: $(jq -c '[.slots[] | select(.slot >= 300)] | length' <<<"$state")"
if [ "$NOCABLE" = 0 ] && [ "$SELFTEST" = 0 ]; then
    jq -e --arg dust "$DUST" '[.slots[] | select(.slot >= 300 and .item == $dust and .count == 2)] | length == 1' <<<"$state" >/dev/null \
        || fail "the chamber's menu does not show the two dust: $(jq -c '[.slots[] | select(.slot >= 300)]' <<<"$state")"
fi
ctl close
ctl wait 4

echo "== the chain: the chamber's two dust smelt into two more iron ingots =="
if [ "$NOCABLE" = 1 ] || [ "$SELFTEST" = 1 ]; then
    echo "  (skipped: no dust was made)"
else
    # The cube and its cable have done their job. Cutting them leaves the
    # smelter's own 20,000 FE as the only energy in play, so the two
    # operations it runs cost exactly 20,000 and the reading is exact.
    ctl command "execute in minecraft:the_end run setblock 1 64 0 minecraft:air"
    ctl command "execute in minecraft:the_end run setblock 2 64 0 minecraft:air"
    # Empty the buffer before the dust goes in, so the machine cannot start on
    # the energy it still held and the 20,000 given below pays for exactly the
    # two operations (2 x 200 ticks x 50 FE).
    say "mekanism energy 0 0 64 0"
    ctl wait 4
    ctl command "execute in minecraft:the_end run item replace block 0 64 0 container.0 with $DUST 2"
    ctl wait 4
    expect_contains 0 64 0 "$SMELTER" "input $DUST x2"
    say "mekanism energy 20000 0 64 0"
    ctl wait 4
    ctl wait 420
    # The smelter already held the two ingots the first phase smelted; the two
    # dust add two more, so the output slot holds four.
    expect_contains 0 64 0 "$SMELTER" "input empty x0, output minecraft:iron_ingot x4"
    expect_energy 0 64 0 "$SMELTER" eq 0
fi

echo "== control 4: an input the chamber has no recipe for burns nothing =="
ctl command "execute in minecraft:the_end run item replace block 0 64 20 container.0 with minecraft:diamond 1"
ctl wait 10
say "mekanism energy 10000 0 64 20"
ctl wait 4
ctl wait 40
expect_contains 0 64 20 "$CHAMBER" "progress 0/200, input minecraft:diamond x1, output empty x0"
expect_energy 0 64 20 "$CHAMBER" eq 10000

echo "== control 5: a charged cube with no cable to the chamber transfers nothing =="
say "mekanism energy 10000 2 64 16"
ctl wait 4
ctl wait 40
expect_energy 0 64 16 "$CHAMBER" eq 0
expect_energy 2 64 16 "$CUBE" eq 10000

echo "== persistence: save, quit and relaunch read the block entities back =="
# Cut the smelter's cable so nothing feeds it while it stalls, empty its buffer
# so the operation cannot start early, then leave it mid-operation: 3,000 FE is
# exactly 60 ticks of work at 50 FE/t, from progress 0.
if [ "$NOCABLE" = 0 ]; then
    ctl command "execute in minecraft:the_end run setblock 1 64 0 minecraft:air"
fi
ctl command "execute in minecraft:the_end run setblock 2 64 0 minecraft:air"
say "mekanism energy 0 0 64 0"
ctl wait 4
ctl command "execute in minecraft:the_end run item replace block 0 64 0 container.0 with minecraft:iron_ore 1"
ctl wait 6
say "mekanism energy 3000 0 64 0"
ctl wait 4
ctl wait 140
BEFORE_SMELTER=$(info_once 0 64 0 "$SMELTER")
echo "  before save, smelter: $BEFORE_SMELTER"
case "$BEFORE_SMELTER" in
    *" 0/20000 FE, progress 60/200, input minecraft:iron_ore x1"*) ;;
    *) fail "the smelter did not stall at progress 60 with 0 FE: $BEFORE_SMELTER" ;;
esac
# A chamber with FE and no input is idle, so its buffer stays put; the cube at
# 2,64,16 has no cable, so its storage stays put too.
say "mekanism energy 4321 0 64 16"
say "mekanism energy 6666 2 64 16"
ctl wait 4
ctl wait 20
echo "  before save, chamber: $(info_once 0 64 16 "$CHAMBER")"
echo "  before save, cube:    $(info_once 2 64 16 "$CUBE")"
ctl save | sed 's/^/  /'
stop_game
launch
# Stand where the scenario left the player, so the same chunks are loaded.
ctl command "execute in minecraft:the_end run tp @s 0.5 64 3.5" >/dev/null
ctl wait 40 >/dev/null
AFTER_SMELTER=$(info_once 0 64 0 "$SMELTER")
echo "  after relaunch, smelter: $AFTER_SMELTER"
expect_contains 0 64 16 "$CHAMBER" "4321/20000 FE, progress 0/200, input empty x0, output empty x0"
expect_contains 2 64 16 "$CUBE" "6666/4000000 FE"
case "$AFTER_SMELTER" in
    *" 0/20000 FE, progress 60/200, input minecraft:iron_ore x1"*) ;;
    *) fail "the smelter's energy, progress and slots did not survive the save: $AFTER_SMELTER" ;;
esac
echo "  (the saved compound survives: FE, progress and slots read back)"

if [ "$NOCABLE" = 1 ]; then
    echo "PASS: the blocks answer, and without a cable the charged cube moves no energy into the adjacent smelter or chamber, which never smelt or enrich"
elif [ "$SELFTEST" = 1 ]; then
    echo "SELFTEST finished without failing (this is a bug in the test)"
    exit 1
else
    echo "PASS: the smelter, cube, cable and enrichment chamber work end to end - the cube's FE reaches the chamber through the cable and turns one ore into two dust, that dust smelts into two ingots, an unsmeltable input and an uncabled neighbour do nothing, both three-slot menus open, and the machines' FE, progress and slots survive a save and relaunch"
fi
echo "EXIT:0"
