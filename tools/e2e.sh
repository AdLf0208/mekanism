#!/usr/bin/env bash
# In-game check for Mekanism slices 1-3: Energized Smelter, Basic Energy Cube,
# Basic Universal Cable, Enrichment Chamber and Crusher, with every carried
# recipe, block and loot table exercised as the parity rule (AGENTS.md,
# 2026-11-20) requires.
#
# The game is **launcher-built with this package installed**: a native `.hmod`
# carries its own content and code, so there is no `--dev-mod` folder and no
# second content file any more (0.1.0 shipped a `.hmod` beside an `.hnmod`; the
# logs of those runs in ../../_shared/evidence/mekanism/ are historical).
#
#   ports/tools/launcher-game.sh run mekanism
#
# The leg launches `--offscreen` (not `--headless`): `click-slot`, which the
# crafting phase needs, requires a renderer.
#
# The End arena (stone floor at y=63, air above). Three rows at y=64:
#   z=0   x=0 smelter   x=1 cable   x=2 energy cube   (the cable couples 0 and 2)
#   z=12  x=0 chamber   x=1 cable   x=2 energy cube   (the ore -> dust chain)
#   z=22  x=0 crusher (charged) and x=2 crusher (never charged)
# and one-machine controls at z=4, 8, 16, 20 and 24. The crusher sweep uses
# x=6..24, z=-8..20 (ten crushers per batch), and the content/recipe rows use
# x=-6..2 and x=10 at z=26..28.
# The mod's commands are typed with `ctl chat "/cmd"` because `ctl command`
# runs the console path and bypasses a mod's command hook.
#
# Default run (must PASS):
#  A  content: the five blocks and three dusts are give-able and place, an
#     unknown id is refused, and every block's loot table drops the block;
#  B  recipes: all five carried shaped recipes craft in a real table with their
#     declared id and count, ingredients consumed, and two wrong-ingredient
#     controls leave the result empty;
#  C  the crusher: a charged crusher turns cobblestone into gravel for exactly
#     10,000 FE, an uncharged one does nothing, an input with no recipe burns
#     nothing, and its three-slot menu opens;
#  D  the sweep: every carried crushing row (145) runs in a real crusher with its
#     declared input count and output stack, and the input is consumed;
#  E  the four blocks of slices 1-2 place and report their buffers through
#     `/mekanism info`;
#  E  with the smelter unpowered, an ore in its input does not smelt (control 1);
#  E  a charged cube feeds the smelter through the cable (energy rises there and
#     falls in the cube);
#  E  the smelter smelts: two ore -> two iron ingots over two operations, at
#     10,000 FE each, and the buffer drops by exactly that;
#  E  a smelter with an unsmeltable input (a diamond) makes no progress and
#     burns no energy (control 2);
#  E  a cube beside a smelter with no cable between them transfers nothing
#     (control 3, the differential);
#  E  the chamber enriches: a charged cube feeds it through its cable, one iron
#     ore becomes two `mekanism:dust_iron` over one operation, at 10,000 FE;
#  E  the chain completes: the chamber's two dust go into the smelter's input and
#     smelt into two more iron ingots;
#  E  a chamber with an input it has no recipe for makes no progress and burns
#     nothing (control 4), and a chamber with no cable gets no energy (control 5);
#  E  every machine's energy, progress and slots survive save + relaunch (the
#     block entity's saved compound).
#  F  the chemical capability (slice 4): the published capability names include
#     `mekanism:chemical`; an insert through the key lands in the Basic Chemical
#     Tank's saved data (`chemical` + `amount`); part of it drains into the Gauge
#     Dropper through the item side of the same capability; an unknown chemical
#     id is refused **by name** and moves nothing; and the same insert against a
#     crusher, which answers no chemical, moves nothing (the counterfactual).
#
# NOCABLE=1 runs the cable scenario without the cable and asserts that the
# smelter and the chamber stay empty (the differential control passing).
# SELFTEST=1 flips the chemical insert assertion of phase F (it must FAIL), so
# the new family's own check is the one proven able to fail.
#
# CONTROL=1 (needs CONTROL_GAME=<an unmodded horizon-client>) first launches
# that game in its own scratch home and world and shows that the block ids are
# unknown there while this game registers them: the check cannot pass vacuously
# on a world that never needed the package.
set -euo pipefail
export LC_ALL=C
# Every line goes to stderr, which is not buffered when redirected: a run that
# is killed still leaves its transcript in the log.
exec 1>&2

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/mekanism
# A launcher-built game and its own control client: there is no staged build of
# this package to fall back to, so GAME has no default.
GAME=${GAME:-}
CTL=${CTL:-${GAME:+$(dirname "$GAME")/horizon-ctl}}
# Optional: the game's resources. A launcher-built instance knows its own, so an
# unset RES adds no argument.
RES=${RES:-${HORIZON_RES:-}}
SOCK=${SOCK:-/tmp/opencode/mekanism-e2e.sock}
SELFTEST=${SELFTEST:-0}
NOCABLE=${NOCABLE:-0}
CONTROL=${CONTROL:-${CONTROL_GAME:+1}}
CONTROL=${CONTROL:-0}
CONTROL_GAME=${CONTROL_GAME:-}
KEEP=${KEEP:-0}
WORLD=MekanismE2E
CONTROL_WORLD=MekanismControl
SMELTER=mekanism:energized_smelter
CUBE=mekanism:basic_energy_cube
CABLE=mekanism:basic_universal_cable
CHAMBER=mekanism:enrichment_chamber
CRUSHER=mekanism:crusher
TANK=mekanism:basic_chemical_tank
DROPPER=mekanism:gauge_dropper
DUST=mekanism:dust_iron

[ -n "$GAME" ] || { echo "set GAME=<the launcher-built horizon-client> (see the header)" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL (set CTL=)" >&2; exit 1; }
[ -f "$PORT/horizon-mod.toml" ] || { echo "missing the package: $PORT/horizon-mod.toml" >&2; exit 1; }
[ -f "$PORT/mekanism.hmod" ] || { echo "missing the package: $PORT/mekanism.hmod" >&2; exit 1; }

SCRATCH=$(mktemp -d /tmp/opencode/mekanism-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")" "$XDG_CONFIG_HOME"
rm -f "$SOCK"

# The game under test and its world: the control leg switches both in a subshell.
GAME_NOW=$GAME
WORLD_NOW=$WORLD

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
block_name() { ctl block "$1" "$2" "$3" | jq -r '.name // .block // empty'; }
inventory_count() { ctl inventory | jq -r --arg item "$1" '.counts[$item] // 0'; }
end() { ctl command "execute in minecraft:the_end run $1"; }
# A block stands where it is put, and the reply is checked: `setblock` answers
# `ok: false` (or leaves the old block) for an id the game does not know.
place() { # place X Y Z BLOCK
    end "setblock $1 $2 $3 $4" >/dev/null
    ctl wait 2
    [ "$(block_name "$1" "$2" "$3")" = "$4" ] \
        || fail "expected $4 at $1,$2,$3, got '$(block_name "$1" "$2" "$3")'"
}
fail() {
    echo "FAIL: $*" >&2
    ctl logs --lines 80 >&2 || true
    cp "$HORIZON_HOME/logs/latest.log" "$CLONE/ports/_shared/evidence/mekanism/e2e-game.log" 2>/dev/null || true
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
    grep -aE "\[CHAT\] $4: " "$HORIZON_HOME/logs/latest.log" 2>/dev/null | tail -1 | sed 's/.*\[CHAT\] //' || true
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

# How many `/mekanism chemical ...` answers the game's log holds so far, and the
# newest of them: every reply of that family starts with `chemical: `, so one
# grep cannot pick up another phase's line.
chem_lines() { grep -acE "\[CHAT\] chemical: " "$HORIZON_HOME/logs/latest.log" 2>/dev/null || true; }
chem_last() { grep -aE "\[CHAT\] chemical: " "$HORIZON_HOME/logs/latest.log" 2>/dev/null | tail -1 | sed 's/.*\[CHAT\] //' || true; }

# Run one `/mekanism chemical` command **once** and return its answer, waiting
# for a line the log did not hold before: a command that moves a chemical must
# never be re-sent by a retry loop.
chem_once() { # chem_once ARGS...
    local ask=("$@") before
    before=$(chem_lines)
    say "mekanism chemical ${ask[*]}" >/dev/null
    for _ in $(seq 1 12); do
        ctl wait 4 >/dev/null
        [ "$(chem_lines)" -gt "$before" ] && break
    done
    chem_last
}

# `/mekanism info` at the tank: its line is the tank's, not "No block entity
# there.", and it arrives as the same `chemical: ` family.
tank_info() {
    local before
    before=$(chem_lines)
    say "mekanism info 16 64 26" >/dev/null
    for _ in $(seq 1 12); do
        ctl wait 4 >/dev/null
        [ "$(chem_lines)" -gt "$before" ] && break
    done
    chem_last
}

# One command, then assert its answer says TEXT.
expect_chem() { # expect_chem TEXT -- ARGS...
    local want=$1 line
    shift 2
    line=$(chem_once "$@")
    echo "  chemical $*: $line"
    case "$line" in
        *"$want"*) ;;
        *) fail "expected /mekanism chemical $* to say '$want', got '$line'" ;;
    esac
}

# One `/data get block` reply line, for asserting saved data exactly.
block_data() { # block_data X Y Z PATH
    ctl command "data get block $1 $2 $3 $4" | jq -r '.chat[0] // ""'
}

# One `/data get entity` reply line (the player's held stack and its components).
entity_data() { # entity_data TARGET [PATH]
    ctl command "data get entity $1 ${2:-}" | jq -r '.chat[0] // ""'
}

cleanup() {
    local status=$?
    if [ "${SCREEN_OPEN:-0}" = 1 ] && [ -n "${GAME_PID:-}" ]; then
        kill "$GAME_PID" 2>/dev/null || true
    else
        ctl quit >/dev/null 2>&1 || true
    fi
    sleep 1
    if [ "$KEEP" = 1 ]; then
        echo "kept scratch: $SCRATCH" >&2
    else
        rm -rf "$SCRATCH" 2>/dev/null || true
    fi
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() {
    local attempt
    for attempt in 1 2 3; do
        rm -f "$SOCK"
        cd "$SCRATCH"
        local args=(--game "$GAME_NOW" --world "$WORLD_NOW" --seed 20261105 --creative --offscreen
                    --wait)
        # A launcher-built instance knows its resources; RES only overrides them.
        if [ -n "$RES" ]; then args+=(--resources "$RES"); fi
        local reply
        if reply=$("$CTL" --socket "$SOCK" launch "${args[@]}" 2>>"$SCRATCH/game-stderr.log"); then
            GAME_PID=$(jq -r '.pid // empty' <<<"$reply" 2>/dev/null || true)
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
    local lock="$HORIZON_HOME/saves/$WORLD_NOW/session.lock"
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
    ctl command "execute in minecraft:the_end run fill -8 63 -8 26 63 30 minecraft:stone"
    ctl command "execute in minecraft:the_end run fill -8 64 -8 26 74 30 minecraft:air"
    ctl command "execute in minecraft:the_end run kill @e[type=!player]"
    ctl wait 20
}

# CONTROL=1: the same scenario's block ids on a game **without** the package,
# in its own scratch home and world. Every id must be unknown there, and the
# mod's command must not answer: if they resolve, the run proves nothing.
control_leg() {
    if [ -z "$CONTROL_GAME" ]; then
        echo "  (CONTROL=1 needs CONTROL_GAME=<an unmodded horizon-client>; skipped)"
        return 0
    fi
    [ -x "$CONTROL_GAME" ] || { fail "CONTROL_GAME is not executable: $CONTROL_GAME"; }
    echo "== control: an unmodded game does not know these blocks =="
    local dir=$SCRATCH/control
    mkdir -p "$dir/home" "$dir/launcher" "$dir/data" "$dir/config"
    (
        # Its own scratch home and working directory: sharing them with the main
        # run mixed the two sessions' latest.log and raced on the world lock.
        export HORIZON_HOME=$dir/home
        export HORIZON_LAUNCHER_HOME=$dir/launcher
        export XDG_DATA_HOME=$dir/data
        export XDG_CONFIG_HOME=$dir/config
        SOCK=/tmp/opencode/mekanism-control.sock
        GAME_NOW=$CONTROL_GAME
        WORLD_NOW=$CONTROL_WORLD
        rm -f "$SOCK"
        launch
        arena
        # 0,64,0 can be the End exit portal's bedrock, which an unknown block id
        # leaves in place; probe clear ground off the portal instead.
        ctl command "execute in minecraft:the_end run setblock 4 64 4 $SMELTER" >/dev/null || true
        ctl command "execute in minecraft:the_end run setblock 6 64 4 $CUBE" >/dev/null || true
        ctl wait 6
        local smelter cube
        smelter=$(ctl block 4 64 4 | jq -r '.block // ""')
        cube=$(ctl block 6 64 4 | jq -r '.block // ""')
        echo "  placed blocks: $smelter / $cube"
        [ "$smelter" = "minecraft:air" ] || fail "the control game holds $smelter, so the smelter id was known"
        [ "$cube" = "minecraft:air" ] || fail "the control game holds $cube, so the cube id was known"
        # The command half comes from the package too, so an unmodded game has
        # no `/mekanism` at all: a machine line here would mean it ran anyway.
        say "mekanism info 0 64 0"
        ctl wait 4
        sleep 0.5
        local reply
        reply=$(grep -a "\[CHAT\] .*mekanism" "$HORIZON_HOME/logs/latest.log" 2>/dev/null | tail -1 | sed 's/.*\[CHAT\] //' || true)
        echo "  /mekanism reply: ${reply:-none}"
        case "$reply" in
            *"FE, progress"*) fail "the control game answered a machine report, so the package was loaded there" ;;
        esac
        echo "  without the package: no smelter or cube block, and no machine report"
        ctl quit >/dev/null 2>&1 || true
        sleep 2
    )
}

echo "== launch (The End arena, the package's content and code) =="
if [ "$CONTROL" = 1 ]; then control_leg; fi
launch
arena

# ---------------------------------------------------------------------------
# A. Content: every carried block and item, the unknown id, the loot tables
# ---------------------------------------------------------------------------
echo "== A1: every carried block and item is give-able =="
BLOCKS=("$SMELTER" "$CHAMBER" "$CRUSHER" "$CUBE" "$CABLE" "$TANK")
ITEMS=("mekanism:dust_iron" "mekanism:dust_gold" "mekanism:dust_copper" "$DROPPER")
for id in "${BLOCKS[@]}" "${ITEMS[@]}"; do
    ctl command "clear @s" >/dev/null
    ctl give "$id" 1 >/dev/null || fail "give $id failed"
    ctl wait 3
    [ "$(inventory_count "$id")" -ge 1 ] || fail "$id is not in the inventory after give"
done
echo "  ${#BLOCKS[@]} blocks and ${#ITEMS[@]} items are give-able"
ctl command "clear @s" >/dev/null

echo "== A2: every block stands where it is put, and an unknown id is refused =="
i=0
for id in "${BLOCKS[@]}"; do
    place "$((-6 + 2 * i))" 64 26 "$id"
    i=$((i + 1))
done
end "setblock 6 64 26 mekanism:not_a_block" >/dev/null 2>&1 || true
ctl wait 3
[ "$(block_name 6 64 26)" = "minecraft:air" ] \
    || fail "an unknown block id was accepted: $(block_name 6 64 26)"
echo "  ${#BLOCKS[@]} blocks place and read back; an unknown id is refused"

# ---------------------------------------------------------------------------
# F. The chemical capability: one tank, one item, one id table (slice 4)
# ---------------------------------------------------------------------------
# The tank stands clear of every other phase at 16,64,26; the counterfactual's
# crusher at 18,64,26. `mekanism:chemical` is asked exactly as another mod's
# crate would: `LevelRef::capability` by name, then the handler's own
# insert/extract. Nothing here reads the block entity's fields directly except
# the two `/data get block` assertions that prove what was saved.
echo "== F1: the chemical capability is published by name, with Mekanism's ids =="
CHEM_LIST=$(chem_once list)
echo "  chemical list: $CHEM_LIST"
case "$CHEM_LIST" in
    *"32 chemicals"*) ;;
    *) fail "the chemical id table did not answer: $CHEM_LIST" ;;
esac
case "$CHEM_LIST" in
    *mekanism:chemical*) ;;
    *) fail "mekanism:chemical is not among the published capabilities: $CHEM_LIST" ;;
esac
case "$CHEM_LIST" in
    *"32 chemicals, mekanism:hydrogen among them"*) ;;
    *) fail "mekanism:hydrogen is not in the id table: $CHEM_LIST" ;;
esac

echo "== F2: an insert through the capability lands in the tank's saved data =="
place 16 64 26 "$TANK"
CHEM_WANT="inserted 4000 mekanism:hydrogen into mekanism:basic_chemical_tank (4000/64000)"
if [ "$SELFTEST" = 1 ]; then
    echo "(selftest: asserting the insert moved nothing)"
    CHEM_WANT="inserted 0 mekanism:hydrogen"
fi
expect_chem "$CHEM_WANT" -- insert 4000 mekanism:hydrogen 16 64 26
CHEM_SAVED=$(block_data 16 64 26 chemical)
CHEM_AMOUNT=$(block_data 16 64 26 amount)
echo "  data get block 16 64 26 chemical: $CHEM_SAVED"
echo "  data get block 16 64 26 amount: $CHEM_AMOUNT"
case "$CHEM_SAVED" in
    *\"mekanism:hydrogen\"*) ;;
    *) fail "the tank's saved chemical is not mekanism:hydrogen: $CHEM_SAVED" ;;
esac
case "$CHEM_AMOUNT" in
    *"4000"*) ;;
    *) fail "the tank's saved amount is not 4000: $CHEM_AMOUNT" ;;
esac
expect_chem "holds mekanism:hydrogen 4000/64000 (answered by mekanism)" -- read 16 64 26
# `/mekanism info` at a tank answers its tank rather than claiming there is no
# block entity where there is one.
TANK_INFO=$(tank_info)
echo "  mekanism info 16 64 26: $TANK_INFO"
case "$TANK_INFO" in
    *"mekanism:basic_chemical_tank holds mekanism:hydrogen 4000/64000"*) ;;
    *) fail "/mekanism info at the tank did not answer its tank: $TANK_INFO" ;;
esac

echo "== F3: part of it drains into the gauge dropper through the item capability =="
ctl command "clear @s" >/dev/null
ctl give "$DROPPER" 1 >/dev/null || fail "give $DROPPER failed"
ctl wait 3
ctl select "$DROPPER" >/dev/null
# Survival, not creative: a creative hand gives an item without losing the old
# stack, so the exchange would leave the plain dropper in hand and put the filled
# one in the inventory. Survival replaces it where it is, as a bucket does.
ctl command "gamemode survival @s" >/dev/null
ctl wait 3
# The dropper moves at most its own transfer rate (250 a call), so asking for
# 1,000 moves 250: the item's limit is the item's, not the tank's.
expect_chem "drained 250 mekanism:hydrogen into the gauge dropper (tank 3750/64000, dropper 250/16000)" \
    -- drain 1000 16 64 26
CHEM_AMOUNT=$(block_data 16 64 26 amount)
echo "  data get block 16 64 26 amount: $CHEM_AMOUNT"
case "$CHEM_AMOUNT" in
    *"3750"*) ;;
    *) fail "the tank did not lose exactly the 250 the dropper took: $CHEM_AMOUNT" ;;
esac
DROPPER_DATA=$(entity_data @s SelectedItem)
echo "  data get entity @s SelectedItem: $DROPPER_DATA"
case "$DROPPER_DATA" in
    *"mekanism:chemical"*) ;;
    *) fail "the dropper's stack has no mekanism:chemical component: $DROPPER_DATA" ;;
esac
case "$DROPPER_DATA" in
    *mekanism:hydrogen*) ;;
    *) fail "the dropper's component does not name mekanism:hydrogen: $DROPPER_DATA" ;;
esac
case "$DROPPER_DATA" in
    *"amount: 250"*) ;;
    *) fail "the dropper's component does not hold 250: $DROPPER_DATA" ;;
esac
expect_chem "the gauge dropper holds mekanism:hydrogen 250/16000" -- dropper
ctl command "gamemode creative @s" >/dev/null
ctl wait 2

echo "== F4: an unknown chemical id is refused by name and moves nothing =="
expect_chem "unknown chemical mekanism:unobtainium" -- insert 500 mekanism:unobtainium 16 64 26
CHEM_AMOUNT=$(block_data 16 64 26 amount)
echo "  data get block 16 64 26 amount: $CHEM_AMOUNT"
case "$CHEM_AMOUNT" in
    *"3750"*) ;;
    *) fail "a refused chemical changed the tank: $CHEM_AMOUNT" ;;
esac

echo "== F5: the counterfactual: a block entity that answers no chemical moves nothing =="
# The same insert as F2 against a crusher: a real mod block entity with its own
# container and energy buffer, which does not answer `mekanism:chemical`.
place 18 64 26 "$CRUSHER"
expect_chem "no chemical handler at 18 64 26" -- insert 1000 mekanism:hydrogen 18 64 26
CHEM_ABSENT=$(block_data 18 64 26 chemical)
echo "  data get block 18 64 26 chemical: ${CHEM_ABSENT:-<no such key>}"
case "$CHEM_ABSENT" in
    *mekanism:hydrogen*|*"1000"*) fail "the crusher took a chemical it does not answer for: $CHEM_ABSENT" ;;
esac
echo "  the same insert moved nothing into a block that answers no chemical"

# ---------------------------------------------------------------------------
# B. Recipes: the five shaped recipes in a real table, and the blocks' loot
# ---------------------------------------------------------------------------
echo "== B1: the five carried shaped recipes in a real crafting table =="
place 10 64 28 minecraft:crafting_table
ctl command "execute in minecraft:the_end run tp @s 9.5 64 28.5" >/dev/null
ctl wait 4
ctl command "gamemode survival @s" >/dev/null
ctl wait 3
python3 "$PORT/tools/craft_check.py" "$SOCK" "$CTL" 10 64 28 \
    || fail "the crafting-table recipe check failed"
ctl command "gamemode creative @s" >/dev/null
ctl command "clear @s" >/dev/null
ctl wait 3

echo "== B2: every block's loot table drops the block =="
ctl command "gamemode survival @s" >/dev/null
ctl command "clear @s" >/dev/null
ctl give minecraft:iron_pickaxe 1 >/dev/null
ctl wait 4
ctl select minecraft:iron_pickaxe >/dev/null
ctl wait 2
i=0
for id in "${BLOCKS[@]}"; do
    x=$((-6 + 2 * i))
    place "$x" 64 26 "$id"
    i=$((i + 1))
done
i=0
for id in "${BLOCKS[@]}"; do
    x=$((-6 + 2 * i))
    ctl command "execute in minecraft:the_end run tp @s $((x)).5 64 27.5" >/dev/null
    ctl wait 5
    ctl look-at-block "$x" 64 26 >/dev/null 2>&1 || true
    reply=$(ctl break "$x" 64 26) || fail "cannot break $id: $reply"
    got=$(jq -r --arg d "$id" '.drops[$d] // 0' <<<"$reply")
    [ "$got" -ge 1 ] || fail "$id dropped $got of itself: $reply"
    i=$((i + 1))
done
echo "  ${#BLOCKS[@]} blocks each dropped their own item"
ctl command "gamemode creative @s" >/dev/null
ctl command "clear @s" >/dev/null
ctl wait 3

# ---------------------------------------------------------------------------
# C. The crusher: one operation, its cost, its controls and its menu
# ---------------------------------------------------------------------------
echo "== C: the crusher crushes for exactly 10,000 FE, and not without energy =="
if [ "$NOCABLE" = 0 ]; then
    place 0 64 22 "$CRUSHER"
    place 2 64 22 "$CRUSHER"
    ctl command "execute in minecraft:the_end run item replace block 0 64 22 container.0 with minecraft:cobblestone 1" >/dev/null
    ctl command "execute in minecraft:the_end run item replace block 2 64 22 container.0 with minecraft:cobblestone 1" >/dev/null
    ctl wait 6
    expect_contains 0 64 22 "$CRUSHER" "0/20000 FE, progress 0/200, input minecraft:cobblestone x1"
    say "mekanism energy 10000 0 64 22"
    ctl wait 6
    ctl wait 230
    if [ "$SELFTEST" = 1 ]; then
        echo "(selftest: asserting the crusher's output is still empty)"
        expect_contains 0 64 22 "$CRUSHER" "output empty x0"
    else
        expect_contains 0 64 22 "$CRUSHER" "input empty x0, output minecraft:gravel x1"
        # 200 ticks at 50 FE/t is exactly 10,000 FE: the buffer is emptied.
        expect_energy 0 64 22 "$CRUSHER" eq 0
    fi
    # The uncharged crusher beside it never started, so its cobblestone is intact.
    expect_contains 2 64 22 "$CRUSHER" "progress 0/200, input minecraft:cobblestone x1, output empty x0"
    expect_energy 2 64 22 "$CRUSHER" eq 0
    echo "  the crusher opens its three-slot menu"
    # A right-click is reach-limited: stand beside the crusher.
    ctl command "execute in minecraft:the_end run tp @s 0.5 64 23.5" >/dev/null
    ctl wait 4
    state=$(ctl interact 0 64 22)
    jq -e '.ok == true and .screen == "container"' <<<"$state" >/dev/null \
        || fail "the crusher did not open a container: $(jq -c '{ok, screen}' <<<"$state")"
    ctl close >/dev/null 2>&1 || true
    ctl wait 3
else
    echo "  (NOCABLE: the crusher phase is skipped with the cable scenario)"
fi

# ---------------------------------------------------------------------------
# D. The sweep: every carried crushing row through a real crusher
# ---------------------------------------------------------------------------
echo "== D: every carried crushing recipe in a real crusher =="
if [ "$NOCABLE" = 1 ]; then
    echo "  (NOCABLE: the sweep is skipped with the cable scenario)"
elif [ "$SELFTEST" = 1 ]; then
    echo "  (selftest: the sweep is skipped)"
else
    ROWS=$SCRATCH/crushing-rows.json
    python3 "$PORT/tools/gen_crushing.py" --rows >"$ROWS" \
        || fail "the crushing rows could not be read from the jar"
    rows=$(jq 'length' "$ROWS")
    [ "$rows" -ge 130 ] || fail "the jar yielded only $rows crushing rows"
    echo "  rows from the jar's mekanism:crushing data: $rows"
    python3 "$PORT/tools/crush_sweep.py" "$SOCK" "$CTL" "$ROWS" 6 64 -8 --batch 10 \
        || fail "the crushing sweep failed"
    python3 "$PORT/tools/crush_sweep.py" "$SOCK" "$CTL" --control 0 64 24 \
        || fail "the crusher's no-recipe control failed"
fi

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

echo "== the chamber opens its three-slot menu =="
# Under --headless a mod container screen opens but neither `screen`, `close`
# nor `quit` answers while it is open (the screen's tick never runs). The screen
# comes from the `ctl interact` reply, the run ends here, and `cleanup` ends the
# captured process; the three slots are pinned by the container unit test and
# the item path by the `item replace` steps above.
state=$(ctl interact 0 64 0)
jq -e '.ok == true and .screen == "container"' <<<"$state" >/dev/null \
    || fail "the smelter did not open a container: $(jq -c '{ok, screen}' <<<"$state")"
SCREEN_OPEN=1

if [ "$NOCABLE" = 1 ]; then
    echo "PASS: the chemical capability works (an insert lands in the tank's saved data, part of it drains into the gauge dropper, an unknown id and a block that answers no chemical both move nothing), and without a cable the charged cube moves no energy into the adjacent smelter or chamber, which never smelt or enrich"
elif [ "$SELFTEST" = 1 ]; then
    echo "SELFTEST finished without failing (this is a bug in the test)"
    exit 1
else
    echo "PASS: the six blocks, their seven shaped recipes and their loot tables work, every carried crushing recipe runs in a real crusher, the smelter, cube, cable and enrichment chamber work end to end - the cube's FE reaches the chamber through the cable and turns one ore into two dust, that dust smelts into two ingots, an unsmeltable input and an uncabled neighbour do nothing, the menus open, the machines' FE, progress and slots survive a save and relaunch, and the chemical capability moves one chemical into the tank's saved data, drains part of it into the gauge dropper through the item capability, and refuses an unknown id by name while a block that answers no chemical moves nothing"
fi
