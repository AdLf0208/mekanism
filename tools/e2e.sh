#!/usr/bin/env bash
# In-game check for Mekanism slices 1-6: Energized Smelter, Enrichment Chamber,
# Crusher, Basic Energy Cube, Basic Universal Cable, Basic Chemical Tank, Gauge
# Dropper, Chemical Injection Chamber and Purification Chamber, with every
# carried recipe, block and loot table exercised as the parity rule
# (AGENTS.md, 2026-11-20) requires.
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
# and one-machine controls at z=4, 8, 16, 20 and 24. Slice 14's storage-block
# and nugget family crafts at the same table (x=10, z=28), builds its block to
# break at x=12, z=27, and its four blocks join A2/B2's row at z=26. The crusher sweep uses
# x=6..24, z=-8 (ten crushers per batch), the smelting sweeps x=-6..32 at z=-12
# and z=-14 (twenty and ten smelters per batch) with their headline and control
# at z=-10, and the content/recipe rows use
# x=-6..2 and x=10 at z=26..28. The injection chamber's phases stand above that
# floor at y=66: the headline at z=12 (a tank at x=0 and the chamber at x=1, the
# no-neighbour control at x=4, the wrong-gas pair at x=7/8) and the injecting
# sweep at x=6..22, z=-4 (six tank+chamber pairs per batch).
# The mod's commands are typed with `ctl chat "/cmd"` because `ctl command`
# runs the console path and bypasses a mod's command hook.
#
# Default run (must PASS):
#  A  content: the five blocks and three dusts are give-able and place, an
#     unknown id is refused, and every block's loot table drops the block;
#  B  recipes: all seventeen carried shaped recipes craft in a real table with
#     their declared id and count, ingredients consumed, and wrong-ingredient
#     controls leave the result empty;
#  C  the crusher: a charged crusher turns cobblestone into gravel for exactly
#     10,000 FE, an uncharged one does nothing, an input with no recipe burns
#     nothing, and its three-slot menu opens;
#  D  the sweep: every carried crushing row (156, slice 13's four ingot ->
#     dust rows included) runs in a real crusher with its declared input count
#     and output stack, and the input is consumed;
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
#  G  the capability's first consumer (slice 5): a Chemical Injection Chamber
#     beside a tank of hydrogen chloride pulls its gas **through**
#     `mekanism:chemical`, turns three raw iron into the jar's eight
#     `mekanism:shard_iron` for exactly 80,000 FE (200 x 400), and the neighbour
#     tank reads its 4,000 minus the 200 units the operation spent while the
#     machine's own tank is empty; with no tank beside it, and with a tank of
#     oxygen a raw-iron recipe cannot use, it makes no progress, spends no FE and
#     draws nothing;
#  H  the injecting sweep: every carried injecting row (79, the seven
#     crystal->shard rows slice 11 adds included) runs in a real chamber with its
#     declared input count and output stack, the input consumed and the neighbour
#     tank short exactly the gas the operation spent;
#  I  the purification chamber (slice 6): a neighbour tank's 4,000 oxygen turns
#     one iron ore into three clumps for exactly 40,000 FE (200 x 200), leaving
#     the tank 3,800 and the machine's own 210-unit tank empty; with no tank, and
#     with a tank of the injection chamber's hydrogen chloride, it makes no
#     progress and spends no FE; the jar's chain then runs end to end -- a clump
#     crushes into a dirty dust, that dirty dust enriches into a dust (its own
#     recipe is `mekanism:enriching`, not purifying), and a dirty dust in a
#     purification chamber makes nothing;
#  J  the purifying sweep: every carried purifying row (15) runs in a real
#     chamber, the input consumed and the tank short 200 x its per-tick gas.
#  L  the Chemical Washer (slice 8), the port's first **fluid** machine: water
#     arrives through the game's own `horizon:fluids` capability and lands in the
#     block entity's `fluids` list, a dirty slurry arrives through
#     `mekanism:chemical`, and one operation (200 ticks) spends exactly 40,000 FE
#     and 5 mB of water to turn 1 unit of dirty slurry into 1 unit of clean
#     slurry; a clean slurry in the input makes nothing and draws no water, dirty
#     slurry with an empty water tank spends no FE, water is refused by the
#     chemical capability and a chemical tank answers no fluid handler, and the
#     output tank drains through the chemical capability (tank 1);
#  M  the wash sweep: every carried washing row (7) runs in a real washer with
#     its water, its dirty slurry consumed and its clean slurry made.
# N  the Chemical Crystallizer (slice 9), the washer's consumer: N1 charges the
#     160,000 FE buffer once (exactly two operations) and asserts two crystals
#     from 400 of the 1,000 units beside it with the buffer at 0; N1b charges
#     160,000 and 80,000 more (400,000 FE in all) for five operations, five
#     crystals and all 1,000 units; N2 proves one operation alone costs 80,000
#     FE and 200 units; N3 proves a dirty slurry beside it makes no progress,
#     spends no FE, and is refused by the machine's own tank by name; N4 pulls
#     the crystals out through a hopper;
#  O  the crystallizing sweep: every carried crystallizing row (7 — the seven
#     slurries the washer makes) runs in a real crystallizer with its clean
#     slurry consumed and its crystal made.
# N0  the four crystals slice 10 adds (lead, osmium, tin, uranium): 1,000 units
#     of clean_lead beside a charged crystallizer make exactly two crystal_lead
#     and leave the neighbour at 600.
#  P  the crystal leg (slice 11): one crystal_lead in a charged injection chamber
#     beside a 4,000-unit hydrogen chloride tank becomes exactly one shard_lead
#     for 80,000 FE, leaving the tank at 3,800 and the machine's own gas empty;
#     the seven crystal->shard rows are swept in H.
#  Q  the ore chain for lead, osmium, tin and uranium (slice 12): a dirty dust in
#     a charged enrichment chamber becomes its dust for 10,000 FE, the dust in a
#     smelter becomes its ingot, and the clump in a crusher becomes the dirty
#     dust — the three new rows of the jar's chain, each in a real machine.
#  R  the enriching sweep: every carried enriching row (136, slice 12's four
#     dirty dust -> dust rows included) runs in a real enrichment chamber with
#     its declared input count and output stack, and a no-recipe control does
#     nothing.
#  S  the smelting table's named rows (slice 13): a coal ore becomes coal, clay
#     becomes terracotta, a tin dust becomes this port's ingot_tin for 10,000 FE
#     each, and a dirt the table has no row for stays put and makes nothing.
#  U  the four metals' storage-block and nugget family (slice 14): every carried
#     recipe of the family — block from nine ingots, nine ingots from a block,
#     ingot from nine nuggets, nine nuggets from an ingot, for lead, osmium, tin
#     and uranium — crafts in a real table with its declared id and count, its
#     grid emptied and its ingredients consumed; eight ingots and a dirt in the
#     same grid make nothing; the placed block breaks into exactly one of itself.
#  T  the smelting sweeps: every carried smelting row — the 163 the 26.3 data
#     expands to (rows smelted: 163 of 163) and the jar's own 7 dust -> ingot
#     rows (rows smelted: 7 of 7) — runs in a real Energized Smelter with its
#     input consumed and its declared output stack, plus a no-recipe control.
#  V  the Chemical Dissolution Chamber (slice 15), the head of the slurry chain:
#     a tank's 1,000 units of sulfuric acid beside a charged dissolution chamber
#     turn one iron ore into exactly 1,000 units of dirty_iron for 40,000 FE
#     (100 ticks x 400 FE) while the tank loses exactly the 100 units the
#     per-tick recipe spends and the chamber keeps none of the acid; water in
#     the chamber's own tank makes no progress and spends no FE (the readout
#     unchanged) and a dirt in the slot makes nothing and draws no acid; the
#     washer beside it then takes one unit of that slurry **through
#     mekanism:chemical** and finishes one clean_iron row.
#  V3 the 12-row dissolution sweep (slice 16): for every carried row a tank and
#     chamber pair is charged to 160,000 FE with 100 x the row's acid per tick in
#     the tank, and the chamber's parsed /mekanism info must say input empty x0,
#     output <slurry> <amount>/10000 and 120000/160000 FE while the source tank
#     holds exactly 1000 - 100 x the row's acid. A dirt no-row control keeps its
#     slot, spends no FE and draws no acid. SELFTEST=1 flips the deepslate-iron
#     row's expected output, so the run FAILS on the slurry the machine really made.
#  K  the five basic factories (slice 7): three processes of the same operation
#     side by side — three ore in a basic purifying factory become nine clumps
#     for 3 x 40,000 FE and 3 x 200 oxygen, one ore costs one operation (40,000
#     and 200), and 40,000 FE buys 200 progress ticks at 3 x 200 FE/t rather
#     than three operations; a basic smelting factory smelts three ore into
#     three ingots for its 30,000 FE buffer, an input with no recipe burns
#     nothing, and a factory's progress array survives save + relaunch.
#
# NOCABLE=1 runs the cable scenario without the cable and asserts that the
# smelter and the chamber stay empty (the differential control passing).
# SELFTEST=1 flips phase V3's deepslate-iron row output assertion (it must FAIL
# on the dirty_iron the chamber really made): the newest family's own check is
# the one proven able to fail, and the flip moves with the newest family.
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
INJECTION=mekanism:chemical_injection_chamber
SHARD=mekanism:shard_iron
PURIFICATION=mekanism:purification_chamber
SMELTING_FACTORY=mekanism:basic_smelting_factory
ENRICHING_FACTORY=mekanism:basic_enriching_factory
CRUSHING_FACTORY=mekanism:basic_crushing_factory
INJECTING_FACTORY=mekanism:basic_injecting_factory
PURIFYING_FACTORY=mekanism:basic_purifying_factory
WASHER=mekanism:chemical_washer
CRYSTALLIZER=mekanism:chemical_crystallizer
DISSOLUTION=mekanism:chemical_dissolution_chamber
CRYSTAL=mekanism:crystal_iron
BLOCK_LEAD=mekanism:block_lead
BLOCK_OSMIUM=mekanism:block_osmium
BLOCK_TIN=mekanism:block_tin
BLOCK_URANIUM=mekanism:block_uranium
CLUMP=mekanism:clump_iron
DIRTY=mekanism:dirty_dust_iron

[ -n "$GAME" ] || { echo "set GAME=<the launcher-built horizon-client> (see the header)" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL (set CTL=)" >&2; exit 1; }
[ -f "$PORT/horizon-mod.toml" ] || { echo "missing the package: $PORT/horizon-mod.toml" >&2; exit 1; }
[ -f "$PORT/mekanism.hmod" ] || { echo "missing the package: $PORT/mekanism.hmod" >&2; exit 1; }

# `/tmp` is a 7.5 GB tmpfs that has been 100 % full; the launcher leg exports
# E2E_TMP under ~/.cache/horizon-e2e/scratch for exactly that reason.
TMP_ROOT=${E2E_TMP:-/tmp/opencode}
mkdir -p "$TMP_ROOT"
SCRATCH=$(mktemp -d "$TMP_ROOT"/mekanism-home.XXXXXX)
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
    for _ in $(seq 1 30); do
        ctl wait 4 >/dev/null
        if [ "$(chem_lines)" -gt "$before" ]; then
            sleep 0.2
            chem_last
            return
        fi
    done
    # Never return an older line: under load a reply can trail the command by
    # more than 48 ticks, and the previous phase's `chemical:` line would then
    # be mistaken for this command's answer (the 2026-11-24 false failure).
    fail "no new chemical answer after /mekanism chemical ${ask[*]} (last: $(chem_last))"
}

# How many `/mekanism fluid ...` answers the log holds so far, and the newest:
# the fluid family is the fluid capability's, as `chemical: ` is its own.
fluid_lines() { grep -acE "\[CHAT\] fluid: " "$HORIZON_HOME/logs/latest.log" 2>/dev/null || true; }
fluid_last() { grep -aE "\[CHAT\] fluid: " "$HORIZON_HOME/logs/latest.log" 2>/dev/null | tail -1 | sed 's/.*\[CHAT\] //' || true; }

# Run one `/mekanism fluid` command **once** and return its answer: a command
# that moves a fluid must never be re-sent by a retry loop.
fluid_once() { # fluid_once ARGS...
    local ask=("$@") before
    before=$(fluid_lines)
    say "mekanism fluid ${ask[*]}" >/dev/null
    for _ in $(seq 1 30); do
        ctl wait 4 >/dev/null
        if [ "$(fluid_lines)" -gt "$before" ]; then
            sleep 0.2
            fluid_last
            return
        fi
    done
    fail "no new fluid answer after /mekanism fluid ${ask[*]} (last: $(fluid_last))"
}

# One command, then assert its answer says TEXT.
expect_fluid() { # expect_fluid TEXT -- ARGS...
    local want=$1 line
    shift 2
    line=$(fluid_once "$@")
    echo "  fluid $*: $line"
    case "$line" in
        *"$want"*) ;;
        *) fail "expected /mekanism fluid $* to say '$want', got '$line'" ;;
    esac
}

# `/mekanism info` at the tank: its line is the tank's, not "No block entity
# there.", and it arrives as the same `chemical: ` family.
tank_info() {
    local before
    before=$(chem_lines)
    say "mekanism info 16 64 26" >/dev/null
    for _ in $(seq 1 30); do
        ctl wait 4 >/dev/null
        if [ "$(chem_lines)" -gt "$before" ]; then
            sleep 0.2
            chem_last
            return
        fi
    done
    fail "no new tank info answer after /mekanism info 16 64 26 (last: $(chem_last))"
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

# Fill a chemical tank to 1000 and assert the amount from the block entity's
# saved data, not from chat feedback: under a long loaded leg the game can
# throttle a command's `[CHAT] chemical:` line past the reply window even
# though the insert landed (recorded on 2026-11-24). A retry only runs when the
# saved amount is still 0, so it cannot double-insert.
acid_tank_1000() { # acid_tank_1000 X Y Z
    local x=$1 y=$2 z=$3 attempt amt
    for attempt in 1 2 3; do
        say "mekanism chemical insert 1000 mekanism:sulfuric_acid $x $y $z" >/dev/null
        ctl wait 8 >/dev/null
        amt=$(block_data "$x" "$y" "$z" amount)
        case "$amt" in
            *1000*)
                echo "  chemical insert 1000 mekanism:sulfuric_acid $x $y $z: $amt"
                return 0 ;;
            *0)
                ;;
            *)
                fail "chemical insert at $x,$y,$z left an unexpected amount: $amt" ;;
        esac
    done
    fail "chemical insert at $x,$y,$z never landed: $amt"
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
    # No hostile spawns for the rest of the run: the crafting phase needs
    # survival (a creative hand does not consume an ingredient), and a wandering
    # Enderman killed the player mid-check on the 2026-11-22 no-cable leg.
    ctl command "difficulty peaceful"
    ctl command "execute in minecraft:the_end run gamerule doMobSpawning false"
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
BLOCKS=("$SMELTER" "$CHAMBER" "$CRUSHER" "$INJECTION" "$PURIFICATION" "$CUBE" "$CABLE" "$TANK" \
    "$SMELTING_FACTORY" "$ENRICHING_FACTORY" "$CRUSHING_FACTORY" "$INJECTING_FACTORY" \
    "$PURIFYING_FACTORY" "$WASHER" "$CRYSTALLIZER" "$DISSOLUTION" \
    "$BLOCK_LEAD" "$BLOCK_OSMIUM" "$BLOCK_TIN" "$BLOCK_URANIUM")
ITEMS=("mekanism:dust_iron" "mekanism:dust_gold" "mekanism:dust_copper" \
    "mekanism:shard_iron" "mekanism:shard_gold" "mekanism:shard_copper" \
    "mekanism:shard_lead" "mekanism:shard_osmium" "mekanism:shard_tin" \
    "mekanism:shard_uranium" \
    "mekanism:clump_iron" "mekanism:clump_gold" "mekanism:clump_copper" \
    "mekanism:clump_lead" "mekanism:clump_osmium" "mekanism:clump_tin" \
    "mekanism:clump_uranium" \
    "mekanism:dirty_dust_iron" "mekanism:dirty_dust_gold" "mekanism:dirty_dust_copper" \
    "mekanism:dirty_dust_lead" "mekanism:dirty_dust_osmium" "mekanism:dirty_dust_tin" \
    "mekanism:dirty_dust_uranium" \
    "mekanism:dust_lead" "mekanism:dust_osmium" "mekanism:dust_tin" \
    "mekanism:dust_uranium" \
    "mekanism:ingot_lead" "mekanism:ingot_osmium" "mekanism:ingot_tin" \
    "mekanism:ingot_uranium" \
    "mekanism:nugget_lead" "mekanism:nugget_osmium" "mekanism:nugget_tin" \
    "mekanism:nugget_uranium" \
    "mekanism:crystal_iron" "mekanism:crystal_gold" "mekanism:crystal_copper" \
    "mekanism:crystal_lead" "mekanism:crystal_osmium" "mekanism:crystal_tin" \
    "mekanism:crystal_uranium" \
    "$DROPPER")
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
    place "$((-6 + i))" 64 26 "$id"
    i=$((i + 1))
done
# Past the array: twenty blocks now stand at x=-6..13, so the probe sits at 15
# (a new member of BLOCKS moves every hard-coded probe, as slices 5-9 learned).
end "setblock 15 64 26 mekanism:not_a_block" >/dev/null 2>&1 || true
ctl wait 3
[ "$(block_name 15 64 26)" = "minecraft:air" ] \
    || fail "an unknown block id was accepted: $(block_name 15 64 26)"
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
# G. The Chemical Injection Chamber: the capability's first consumer (slice 5)
# ---------------------------------------------------------------------------
# The machine holds its own 210-unit tank and pulls the gas it needs from the
# tank beside it **through `mekanism:chemical`** on its tick. The assertions are
# the jar's numbers: 400 FE/t x 200 ticks = 80,000 FE per operation, and 200
# ticks x the recipe's per-tick amount of the gas (1 for raw ore) = 200 units.
# The gas is read back on both sides: the neighbour's tank through the
# capability, and the machine's own tank from its saved data.
echo "== G1: a neighbour tank's gas reaches the chamber through the capability, and one operation takes exactly 200 of it =="
place 0 66 12 "$TANK"
expect_chem "inserted 4000 mekanism:hydrogen_chloride into mekanism:basic_chemical_tank (4000/64000)" \
    -- insert 4000 mekanism:hydrogen_chloride 0 66 12
place 1 66 12 "$INJECTION"
ctl command "execute in minecraft:the_end run item replace block 1 66 12 container.0 with minecraft:raw_iron 3" >/dev/null
ctl wait 6
expect_contains 1 66 12 "$INJECTION" "0/160000 FE, progress 0/200, input minecraft:raw_iron x3"
say "mekanism energy 160000 1 66 12"
ctl wait 6
# The machine starts on its first charged tick, so the assertion here is that it
# took the energy at all (the exact 80,000 is asserted after the operation).
expect_energy 1 66 12 "$INJECTION" gt 0
ctl wait 230
# The jar's c:raw_materials/iron x3 + 1 hydrogen chloride per tick -> 8 shards.
expect_contains 1 66 12 "$INJECTION" "input empty x0, output $SHARD x8"
# 200 ticks x 400 FE: exactly half the 160,000 buffer.
expect_energy 1 66 12 "$INJECTION" eq 80000
# The gas came from the neighbour, through the capability: 4,000 - 200.
expect_chem "holds mekanism:hydrogen_chloride 3800/64000 (answered by mekanism)" \
    -- read 0 66 12
TANK_LEFT=$(block_data 0 66 12 amount)
echo "  data get block 0 66 12 amount: $TANK_LEFT"
case "$TANK_LEFT" in
    *"3800"*) ;;
    *) fail "the neighbour tank should hold 3800 after one operation: $TANK_LEFT" ;;
esac
# ...and the machine kept none of it: it pulled one tick's gas per tick and
# spent it, so the consumer's own buffer reads 0.
MACHINE_GAS=$(block_data 1 66 12 amount)
echo "  data get block 1 66 12 amount: $MACHINE_GAS"
case "$MACHINE_GAS" in
    *" 0"*|*": 0"*) ;;
    *) fail "the chamber's own tank should be empty after the operation: $MACHINE_GAS" ;;
esac
expect_chem "holds nothing 0/210 (answered by mekanism)" -- read 1 66 12

echo "== G2: without a tank beside it the chamber makes no progress and spends no FE =="
# Two controls in one wait: no neighbour at all (4,66,12), and a neighbour whose
# gas the recipe does not take (oxygen at 7,66,12 for a raw-iron operation).
place 4 66 12 "$INJECTION"
place 7 66 12 "$TANK"
expect_chem "inserted 4000 mekanism:oxygen into mekanism:basic_chemical_tank (4000/64000)" \
    -- insert 4000 mekanism:oxygen 7 66 12
place 8 66 12 "$INJECTION"
for x in 4 8; do
    ctl command "execute in minecraft:the_end run item replace block $x 66 12 container.0 with minecraft:raw_iron 3" >/dev/null
done
say "mekanism energy 160000 4 66 12"
say "mekanism energy 160000 8 66 12"
ctl wait 6
ctl wait 230
expect_contains 4 66 12 "$INJECTION" "progress 0/200, input minecraft:raw_iron x3, output empty x0"
expect_energy 4 66 12 "$INJECTION" eq 160000
expect_contains 8 66 12 "$INJECTION" "progress 0/200, input minecraft:raw_iron x3, output empty x0"
expect_energy 8 66 12 "$INJECTION" eq 160000
# The wrong-gas tank was not drawn from at all.
expect_chem "holds mekanism:oxygen 4000/64000" -- read 7 66 12

# ---------------------------------------------------------------------------
# H. The sweep: every carried injecting row through a real chamber
# ---------------------------------------------------------------------------
echo "== H: every carried injecting recipe in a real Chemical Injection Chamber =="
if [ "$NOCABLE" = 1 ]; then
    echo "  (NOCABLE: the sweep is skipped with the cable scenario)"
elif [ "$SELFTEST" = 1 ]; then
    echo "  (selftest: the sweep is skipped)"
else
    IROWS=$SCRATCH/injecting-rows.json
    python3 -u "$PORT/tools/gen_injecting.py" --rows >"$IROWS" \
        || fail "the injecting rows could not be read from the jar"
    rows=$(jq 'length' "$IROWS")
    # Slice 11's seven crystal rows are part of this table: a tag mapping that
    # drops them again must fail the leg, not quietly sweep 72.
    [ "$rows" -ge 79 ] || fail "the jar yielded only $rows injecting rows"
    echo "  rows from the jar's mekanism:injecting data: $rows"
    HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
        python3 -u "$PORT/tools/inject_sweep.py" "$SOCK" "$CTL" "$IROWS" 6 66 -4 --batch 6 \
        || fail "the injecting sweep failed"
    HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
        python3 -u "$PORT/tools/inject_sweep.py" "$SOCK" "$CTL" --control 20 66 -4 \
        || fail "the injection chamber's wrong-gas control failed"
fi

# ---------------------------------------------------------------------------
# I. The Purification Chamber: oxygen, the jar's clump table, and the chain
# ---------------------------------------------------------------------------
# The jar's numbers: 200 FE/t x 200 ticks = 40,000 FE an operation, an 80,000 FE
# insert-only buffer, the same 210-unit gas tank the injection chamber has, and
# the recipe's per-tick oxygen (1 for an ore, 2 for a raw block). The machine
# pulls the oxygen from the tank beside it through `mekanism:chemical`, exactly
# as the injection chamber pulls hydrogen chloride.
echo "== I1: one iron ore becomes three clumps for 40,000 FE and exactly 200 oxygen =="
place 0 66 16 "$TANK"
expect_chem "inserted 4000 mekanism:oxygen into mekanism:basic_chemical_tank (4000/64000)" \
    -- insert 4000 mekanism:oxygen 0 66 16
place 1 66 16 "$PURIFICATION"
ctl command "execute in minecraft:the_end run item replace block 1 66 16 container.0 with minecraft:iron_ore 1" >/dev/null
ctl wait 6
expect_contains 1 66 16 "$PURIFICATION" "0/80000 FE, progress 0/200, input minecraft:iron_ore x1"
say "mekanism energy 80000 1 66 16"
ctl wait 6
# The machine starts on its first charged tick, so the assertion here is that it
# took the energy at all (the exact 40,000 is asserted after the operation).
expect_energy 1 66 16 "$PURIFICATION" gt 0
ctl wait 230
# The jar's c:ores/iron x1 + 1 oxygen per tick -> 3 clumps.
expect_contains 1 66 16 "$PURIFICATION" "input empty x0, output $CLUMP x3"
# 200 ticks x 200 FE: exactly half the 80,000 buffer.
expect_energy 1 66 16 "$PURIFICATION" eq 40000
# The oxygen came from the neighbour, through the capability: 4,000 - 200.
expect_chem "holds mekanism:oxygen 3800/64000 (answered by mekanism)" \
    -- read 0 66 16
TANK_LEFT=$(block_data 0 66 16 amount)
echo "  data get block 0 66 16 amount: $TANK_LEFT"
case "$TANK_LEFT" in
    *"3800"*) ;;
    *) fail "the neighbour tank should hold 3800 after one operation: $TANK_LEFT" ;;
esac
MACHINE_GAS=$(block_data 1 66 16 amount)
echo "  data get block 1 66 16 amount: $MACHINE_GAS"
case "$MACHINE_GAS" in
    *" 0"*|*": 0"*) ;;
    *) fail "the chamber's own tank should be empty after the operation: $MACHINE_GAS" ;;
esac
expect_chem "holds nothing 0/210 (answered by mekanism)" -- read 1 66 16

echo "== I2: without oxygen, or with the injection chamber's hydrogen chloride, nothing moves =="
# Two controls in one wait: no neighbour at all (4,66,16), and a neighbour whose
# gas the jar's iron-ore recipe does not take (hydrogen chloride at 7,66,16).
place 4 66 16 "$PURIFICATION"
place 7 66 16 "$TANK"
expect_chem "inserted 4000 mekanism:hydrogen_chloride into mekanism:basic_chemical_tank (4000/64000)" \
    -- insert 4000 mekanism:hydrogen_chloride 7 66 16
place 8 66 16 "$PURIFICATION"
for x in 4 8; do
    ctl command "execute in minecraft:the_end run item replace block $x 66 16 container.0 with minecraft:iron_ore 1" >/dev/null
done
say "mekanism energy 80000 4 66 16"
say "mekanism energy 80000 8 66 16"
ctl wait 6
ctl wait 230
expect_contains 4 66 16 "$PURIFICATION" "progress 0/200, input minecraft:iron_ore x1, output empty x0"
expect_energy 4 66 16 "$PURIFICATION" eq 80000
expect_contains 8 66 16 "$PURIFICATION" "progress 0/200, input minecraft:iron_ore x1, output empty x0"
expect_energy 8 66 16 "$PURIFICATION" eq 80000
expect_chem "holds mekanism:hydrogen_chloride 4000/64000" -- read 7 66 16

echo "== I3: the jar's chain: clump -> crusher -> dirty dust -> enriching chamber -> dust =="
if [ "$SELFTEST" = 1 ]; then
    echo "  (selftest: the chain needs phase I1's clumps)"
else
    # A clump into a real crusher: the jar's from_clump row, 10,000 FE.
    place 0 68 16 "$CRUSHER"
    ctl command "execute in minecraft:the_end run item replace block 0 68 16 container.0 with $CLUMP 1" >/dev/null
    say "mekanism energy 20000 0 68 16"
    ctl wait 6
    ctl wait 230
    expect_contains 0 68 16 "$CRUSHER" "input empty x0, output $DIRTY x1"
    expect_energy 0 68 16 "$CRUSHER" eq 10000
    # The dirty dust into an enrichment chamber: upstream's
    # `processing/<material>/dust/from_dirty_dust.json` is `mekanism:enriching`.
    # There is no hopper in this check (recorded gap), so the item the crusher
    # made is put into the chamber by hand, as slices 1-2 did with their dust.
    place 3 68 16 "$CHAMBER"
    ctl command "execute in minecraft:the_end run item replace block 3 68 16 container.0 with $DIRTY 1" >/dev/null
    say "mekanism energy 20000 3 68 16"
    ctl wait 6
    ctl wait 230
    expect_contains 3 68 16 "$CHAMBER" "input empty x0, output $DUST x1"
    expect_energy 3 68 16 "$CHAMBER" eq 10000
    # ...and the same dirty dust in a purification chamber makes nothing: the
    # jar has no purifying recipe for it, so the port does not invent one.
    place 6 68 16 "$PURIFICATION"
    ctl command "execute in minecraft:the_end run item replace block 6 68 16 container.0 with $DIRTY 1" >/dev/null
    say "mekanism energy 80000 6 68 16"
    ctl wait 6
    ctl wait 230
    expect_contains 6 68 16 "$PURIFICATION" "progress 0/200, input $DIRTY x1, output empty x0"
    expect_energy 6 68 16 "$PURIFICATION" eq 80000
    echo "  a clump crushed into a dirty dust and that dirty dust enriched into a dust; the purification chamber has no row for a dirty dust"
fi

# ---------------------------------------------------------------------------
# J. The sweep: every carried purifying row through a real chamber
# ---------------------------------------------------------------------------
echo "== J: every carried purifying recipe in a real Purification Chamber =="
if [ "$NOCABLE" = 1 ]; then
    echo "  (NOCABLE: the sweep is skipped with the cable scenario)"
elif [ "$SELFTEST" = 1 ]; then
    echo "  (selftest: the sweep is skipped)"
else
    PROWS=$SCRATCH/purifying-rows.json
    python3 -u "$PORT/tools/gen_purifying.py" --rows >"$PROWS" \
        || fail "the purifying rows could not be read from the jar"
    rows=$(jq 'length' "$PROWS")
    [ "$rows" -ge 15 ] || fail "the jar yielded only $rows purifying rows"
    echo "  rows from the jar's mekanism:purifying data: $rows"
    HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
        python3 -u "$PORT/tools/gas_sweep.py" "$SOCK" "$CTL" "$PROWS" 6 66 -8 --batch 6 \
        --kind purifying \
        || fail "the purifying sweep failed"
    HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
        python3 -u "$PORT/tools/gas_sweep.py" "$SOCK" "$CTL" --control 20 66 -8 \
        --kind purifying \
        || fail "the purification chamber's wrong-gas control failed"
fi

# ---------------------------------------------------------------------------
# B. Recipes: the five shaped recipes in a real table, and the blocks' loot
# ---------------------------------------------------------------------------
echo "== B1: the sixteen carried shaped recipes in a real crafting table =="
place 10 64 28 minecraft:crafting_table
ctl command "execute in minecraft:the_end run tp @s 9.5 64 28.5" >/dev/null
ctl wait 4
ctl command "gamemode survival @s" >/dev/null
ctl wait 3
python3 -u "$PORT/tools/craft_check.py" "$SOCK" "$CTL" 10 64 28 \
    || fail "the crafting-table recipe check failed"
ctl command "gamemode creative @s" >/dev/null
ctl command "clear @s" >/dev/null
ctl wait 3

# ---------------------------------------------------------------------------
# U. The storage-block and nugget family (slice 14): every carried recipe of the
#    family in a real table, a dirt counterfactual, and the block's own loot
# ---------------------------------------------------------------------------
# The jar declares four recipes per metal and this port carries all sixteen
# (`tools/carry_storage.py` translates them to 26.3's ingredient dialect and
# nothing else): a 3x3 block from nine ingots, a shapeless nine ingots from a
# block, a 3x3 ingot from nine nuggets and a shapeless nine nuggets from an
# ingot. The rows are read back from the committed recipe and tag files, so the
# scenario crafts what the package ships. (The selftest's flip moved to phase V
# with slice 15: it always inverts the newest family's own assertion.)
echo "== U: slice 14's storage-block and nugget family in a real crafting table =="
# B1 ends in creative; a creative table does not consume an ingredient and a
# creative break drops nothing, so this phase puts the player back in survival
# for both halves.
ctl command "gamemode survival @s" >/dev/null
ctl wait 3
STORE_ROWS=$SCRATCH/storage-rows.json
python3 -u "$PORT/tools/carry_storage.py" --rows >"$STORE_ROWS" \
    || fail "the storage-block and nugget rows could not be read"
rows=$(jq 'length' "$STORE_ROWS")
[ "$rows" -ge 16 ] || fail "the package yielded only $rows storage-block and nugget rows, expected 16"
echo "  rows from the jar's storage-block and nugget recipes: $rows"
# Not the selftest's flip any more: since slice 15 the newest family is the
# dissolution chamber, and its own assertion in phase V is the one inverted
# (the notes' lesson: the flip moves with the newest family).
python3 -u "$PORT/tools/craft_family.py" "$SOCK" "$CTL" "$STORE_ROWS" 10 64 28 \
    || fail "the storage-block and nugget recipe check failed"
# The counterfactual: the same nine-ingot grid with dirt where the mod's own
# ingredient goes must leave the result empty, so the check cannot pass on a
# table that crafts whatever it is handed.
python3 -u "$PORT/tools/craft_family.py" "$SOCK" "$CTL" "$STORE_ROWS" 10 64 28 --control \
    || fail "the dirt-in-the-grid control crafted something"
echo "== U: the placed block drops exactly one of itself =="
ctl command "clear @s" >/dev/null
ctl give minecraft:iron_pickaxe 1 >/dev/null
ctl wait 3
ctl select minecraft:iron_pickaxe >/dev/null
ctl wait 2
place 12 64 27 "$BLOCK_LEAD"
ctl command "execute in minecraft:the_end run tp @s 12.5 64 28.5" >/dev/null
ctl wait 5
ctl look-at-block 12 64 27 >/dev/null 2>&1 || true
reply=$(ctl break 12 64 27) || fail "cannot break $BLOCK_LEAD: $reply"
got=$(jq -r --arg d "$BLOCK_LEAD" '.drops[$d] // 0' <<<"$reply")
[ "$got" = 1 ] || fail "$BLOCK_LEAD dropped $got of itself: $reply"
echo "  $BLOCK_LEAD broke into $got of itself"
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
    x=$((-6 + i))
    place "$x" 64 26 "$id"
    i=$((i + 1))
done
i=0
for id in "${BLOCKS[@]}"; do
    x=$((-6 + i))
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
    expect_contains 0 64 22 "$CRUSHER" "input empty x0, output minecraft:gravel x1"
    # 200 ticks at 50 FE/t is exactly 10,000 FE: the buffer is emptied.
    expect_energy 0 64 22 "$CRUSHER" eq 0
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
    python3 -u "$PORT/tools/gen_crushing.py" --rows >"$ROWS" \
        || fail "the crushing rows could not be read from the jar"
    rows=$(jq 'length' "$ROWS")
    # 156 rows since slice 13 (152 + the four `dust/from_ingot` rows): a tag
    # mapping that drops one must fail the leg, not quietly sweep a smaller table.
    [ "$rows" -ge 156 ] || fail "the jar yielded only $rows crushing rows, expected 156"
    echo "  rows from the jar's mekanism:crushing data: $rows"
    python3 -u "$PORT/tools/crush_sweep.py" "$SOCK" "$CTL" "$ROWS" 6 64 -8 --batch 10 \
        || fail "the crushing sweep failed"
    python3 -u "$PORT/tools/crush_sweep.py" "$SOCK" "$CTL" --control 0 64 24 \
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
else
    expect_contains 0 64 12 "$CHAMBER" "input empty x0, output $DUST x2"
    # The cube's 20,000 FE all reach the chamber and one operation spends
    # 10,000, so the machine keeps exactly the other half.
    expect_energy 0 64 12 "$CHAMBER" eq 10000
fi

echo "== the chain: the chamber's two dust smelt into two more iron ingots =="
if [ "$NOCABLE" = 1 ]; then
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


# ---------------------------------------------------------------------------
# K. The basic factories (slice 7): three processes side by side
# ---------------------------------------------------------------------------
# The jar's numbers, re-read for this machine family: a basic factory runs
# `FactoryTier.BASIC`'s three processes, each with its own input/output pair
# and its own progress, over ONE buffer given by `Factory.setMachineData` — half
# the base machine's storage per process, never less than one tick's usage — and
# (for the gas factories) one tank of `MAX_GAS` per process. So a basic
# purifying factory holds 120,000 FE and 630 oxygen, and three ore in its three
# inputs cost 3 x 40,000 FE and 3 x 200 oxygen. Two counterfactuals keep the
# numbers honest: ONE ore must cost one operation (40,000 and 200, not three),
# and 40,000 FE at 3 x 200 FE/t must buy exactly 200 progress ticks — a factory
# that charged once per tick instead of once per process would finish all three.
echo "== K1: three ore in three processes become nine clumps for 600 oxygen and 120,000 FE =="
place 0 70 0 "$TANK"
expect_chem "inserted 4000 mekanism:oxygen into mekanism:basic_chemical_tank (4000/64000)" \
    -- insert 4000 mekanism:oxygen 0 70 0
place 1 70 0 "$PURIFYING_FACTORY"
for slot in 0 1 2; do
    ctl command "execute in minecraft:the_end run item replace block 1 70 0 container.$slot with minecraft:iron_ore 1" >/dev/null
done
ctl wait 6
expect_contains 1 70 0 "$PURIFYING_FACTORY" "0/120000 FE (3 processes), progress 0/200, input minecraft:iron_ore x3"
say "mekanism energy 120000 1 70 0"

echo "== K2: one ore in one process is one operation, not three =="
place 0 70 4 "$TANK"
expect_chem "inserted 4000 mekanism:oxygen into mekanism:basic_chemical_tank (4000/64000)" \
    -- insert 4000 mekanism:oxygen 0 70 4
place 1 70 4 "$PURIFYING_FACTORY"
ctl command "execute in minecraft:the_end run item replace block 1 70 4 container.0 with minecraft:iron_ore 1" >/dev/null
ctl wait 6
say "mekanism energy 120000 1 70 4"

echo "== K3: the buffer is shared: 40,000 FE buys 200 progress ticks, not three operations =="
place 0 70 8 "$TANK"
expect_chem "inserted 4000 mekanism:oxygen into mekanism:basic_chemical_tank (4000/64000)" \
    -- insert 4000 mekanism:oxygen 0 70 8
place 1 70 8 "$PURIFYING_FACTORY"
for slot in 0 1 2; do
    ctl command "execute in minecraft:the_end run item replace block 1 70 8 container.$slot with minecraft:iron_ore 1" >/dev/null
done
ctl wait 6
say "mekanism energy 40000 1 70 8"

echo "== K4: the item factory path: three ore smelt side by side for the 30,000 FE buffer =="
place 0 70 12 "$SMELTING_FACTORY"
for slot in 0 1 2; do
    ctl command "execute in minecraft:the_end run item replace block 0 70 12 container.$slot with minecraft:iron_ore 1" >/dev/null
done
ctl wait 6
expect_contains 0 70 12 "$SMELTING_FACTORY" "0/30000 FE (3 processes), progress 0/200, input minecraft:iron_ore x3"
say "mekanism energy 30000 0 70 12"

echo "== K5: a factory with an input it has no recipe for makes no progress and burns nothing =="
place 0 70 16 "$SMELTING_FACTORY"
ctl command "execute in minecraft:the_end run item replace block 0 70 16 container.0 with minecraft:diamond 1" >/dev/null
ctl wait 6
say "mekanism energy 30000 0 70 16"
ctl wait 6
ctl wait 230

# K1 (the selftest flips phase L's washer assertion now, so K runs as usual).
expect_contains 1 70 0 "$PURIFYING_FACTORY" "input empty x0, output mekanism:clump_iron x9"
expect_energy 1 70 0 "$PURIFYING_FACTORY" eq 0
expect_chem "holds mekanism:oxygen 3400/64000 (answered by mekanism)" -- read 0 70 0
# K2: one ore, one process, one operation.
expect_contains 1 70 4 "$PURIFYING_FACTORY" "input empty x0, output mekanism:clump_iron x3"
expect_energy 1 70 4 "$PURIFYING_FACTORY" eq 80000
expect_chem "holds mekanism:oxygen 3800/64000 (answered by mekanism)" -- read 0 70 4
# K3: no operation completes, the buffer is gone, and exactly 200 units of gas
# were spent for the 200 charged ticks.
expect_contains 1 70 8 "$PURIFYING_FACTORY" "0/120000 FE (3 processes), progress 67/200, input minecraft:iron_ore x3, output empty x0"
expect_energy 1 70 8 "$PURIFYING_FACTORY" eq 0
expect_chem "holds mekanism:oxygen 3800/64000 (answered by mekanism)" -- read 0 70 8
# K4: three operations, three ingots, the whole buffer.
expect_contains 0 70 12 "$SMELTING_FACTORY" "input empty x0, output minecraft:iron_ingot x3"
expect_energy 0 70 12 "$SMELTING_FACTORY" eq 0
# K5: the control.
expect_contains 0 70 16 "$SMELTING_FACTORY" "progress 0/200, input minecraft:diamond x1, output empty x0"
expect_energy 0 70 16 "$SMELTING_FACTORY" eq 30000

# ---------------------------------------------------------------------------
# L. The Chemical Washer: the port's first fluid machine (slice 8)
# ---------------------------------------------------------------------------
# Two capabilities, two resource types, one machine. The **water** arrives through
# the game's own `horizon:fluids` (`ModFluidTanks` on the block, the contents in
# the entity's `fluids` list) and the **slurries** through `mekanism:chemical` (two
# tanks: 0 in, 1 out). The jar's numbers: 200 FE/t x 200 ticks = 40,000 FE an
# operation, 5 mB of water and 1 unit of dirty slurry per operation, the clean
# slurry out. `chemicals are not fluids` is asserted in both directions: water
# cannot go in through the chemical key, and a chemical tank answers no fluid
# handler.
echo "== L1: water through horizon:fluids, the tank's slurry through mekanism:chemical, one operation =="
place 0 72 0 "$TANK"
expect_chem "inserted 1000 mekanism:dirty_iron into mekanism:basic_chemical_tank (1000/64000)" \
    -- insert 1000 mekanism:dirty_iron 0 72 0
place 1 72 0 "$WASHER"
# The buffer is 80,000 FE: full, with no water and nothing in the machine's own
# slurry tank, so nothing is drawn from the tank beside it yet.
say "mekanism energy 80000 1 72 0"
ctl wait 4
expect_contains 1 72 0 "$WASHER" "80000/80000 FE, progress 0/200, water 0/10000 mB, input empty x0, output empty x0"
expect_chem "holds mekanism:dirty_iron 1000/64000" -- read 0 72 0
# The water arrives through the game's own `horizon:fluids` and lands in the
# block entity's saved `fluids` list. The machine holds no energy yet, so it does
# not start on it.
say "mekanism energy 0 1 72 0"
ctl wait 4
expect_fluid "inserted 10000 minecraft:water into mekanism:chemical_washer (10000/10000)" \
    -- insert 10000 1 72 0
WATER_SAVED=$(block_data 1 72 0 fluids[0].Amount)
echo "  data get block 1 72 0 fluids[0].Amount: $WATER_SAVED"
case "$WATER_SAVED" in
    *"10000"*) ;;
    *) fail "the water did not land in the block entity's saved fluids list: $WATER_SAVED" ;;
esac
# The same tank, read back through the capability by the game's own handler.
expect_fluid "holds minecraft:water 10000/10000 (answered by the game)" -- read 1 72 0
expect_contains 1 72 0 "$WASHER" "0/80000 FE, progress 0/200, water 10000/10000 mB, input empty x0, output empty x0"
# Exactly one operation's energy — 200 ticks x 200 FE — so the machine completes
# one operation and cannot afford a second. It pulls the one unit of dirty slurry
# it is about to spend from the tank beside it, so that tank loses exactly one.
say "mekanism energy 40000 1 72 0"
ctl wait 6
ctl wait 230
expect_contains 1 72 0 "$WASHER" "0/80000 FE, progress 0/200, water 9995/10000 mB, input empty x0, output mekanism:clean_iron x1"
# The three buffers, read from the saved data independently of `/mekanism info`.
WATER_LEFT=$(block_data 1 72 0 fluids[0].Amount)
DIRTY_LEFT=$(block_data 1 72 0 input_amount)
CLEAN_OUT=$(block_data 1 72 0 output_chemical)
CLEAN_AMOUNT=$(block_data 1 72 0 output_amount)
ENERGY_LEFT=$(block_data 1 72 0 energy)
echo "  data get block 1 72 0 fluids[0].Amount: $WATER_LEFT"
echo "  data get block 1 72 0 input_amount: ${DIRTY_LEFT:-<no such key>}"
echo "  data get block 1 72 0 output_chemical: $CLEAN_OUT"
echo "  data get block 1 72 0 output_amount: $CLEAN_AMOUNT"
echo "  data get block 1 72 0 energy: $ENERGY_LEFT"
case "$WATER_LEFT" in
    *"9995"*) ;;
    *) fail "the water tank should hold 9995 after one operation: $WATER_LEFT" ;;
esac
case "$DIRTY_LEFT" in
    *" 0"*|*": 0"*|"") ;;
    *) fail "the machine's own slurry tank should be empty after the operation: $DIRTY_LEFT" ;;
esac
case "$CLEAN_OUT" in
    *mekanism:clean_iron*) ;;
    *) fail "the output tank does not hold mekanism:clean_iron: $CLEAN_OUT" ;;
esac
case "$CLEAN_AMOUNT" in
    *"1"*) ;;
    *) fail "the output tank should hold one unit: $CLEAN_AMOUNT" ;;
esac
case "$ENERGY_LEFT" in
    *" 0"*|*": 0"*) ;;
    *) fail "one operation should spend the whole 40,000 FE charged: $ENERGY_LEFT" ;;
esac
# The tank beside it gave exactly the one unit the operation spent...
expect_chem "holds mekanism:dirty_iron 999/64000 (answered by mekanism)" -- read 0 72 0
# ...and the machine kept none of it, and draws nothing while it has no energy.
expect_chem "holds nothing 0/10000 (answered by mekanism)" -- read 1 72 0

echo "== L2: a clean slurry in the input makes nothing, draws no water and takes nothing from the tank beside it =="
place 4 72 0 "$TANK"
expect_chem "inserted 1000 mekanism:dirty_iron into mekanism:basic_chemical_tank (1000/64000)" \
    -- insert 1000 mekanism:dirty_iron 4 72 0
place 5 72 0 "$WASHER"
expect_fluid "inserted 1000 minecraft:water into mekanism:chemical_washer (1000/10000)" -- insert 1000 5 72 0
# The jar has no washing row whose input is a clean slurry: no recipe, so the
# progress resets and nothing is spent (upstream's mismatched recipe).
expect_chem "inserted 1000 mekanism:clean_iron into mekanism:chemical_washer (1000/10000)" \
    -- insert 1000 mekanism:clean_iron 5 72 0
say "mekanism energy 80000 5 72 0"
ctl wait 6
ctl wait 230
expect_contains 5 72 0 "$WASHER" "80000/80000 FE, progress 0/200, water 1000/10000 mB, input mekanism:clean_iron x1000, output empty x0"
expect_energy 5 72 0 "$WASHER" eq 80000
expect_chem "holds mekanism:dirty_iron 1000/64000" -- read 4 72 0

echo "== L3: dirty slurry beside a machine with no water spends no FE and draws nothing =="
place 7 72 0 "$TANK"
expect_chem "inserted 1000 mekanism:dirty_iron into mekanism:basic_chemical_tank (1000/64000)" \
    -- insert 1000 mekanism:dirty_iron 7 72 0
place 8 72 0 "$WASHER"
say "mekanism energy 80000 8 72 0"
ctl wait 6
ctl wait 230
# No water is upstream's mismatched recipe: the progress resets, no energy is
# spent, and the machine does not siphon a neighbour's slurry it cannot wash.
expect_contains 8 72 0 "$WASHER" "80000/80000 FE, progress 0/200, water 0/10000 mB, input empty x0, output empty x0"
expect_energy 8 72 0 "$WASHER" eq 80000
expect_chem "holds mekanism:dirty_iron 1000/64000" -- read 7 72 0

echo "== L4: water is not a chemical, a chemical tank is not a fluid tank, and the clean slurry drains =="
# Direction 1: the chemical key refuses water **by name** (it is not a chemical).
expect_chem "unknown chemical minecraft:water" -- insert 100 minecraft:water 1 72 0
# Direction 2: a Basic Chemical Tank declares no fluid tank, so the fluid
# capability answers nothing there.
place 0 72 4 "$TANK"
expect_fluid "no fluid handler at 0 72 4" -- insert 100 0 72 4
# The output is a chemical and it leaves through the chemical key: tank 1 of the
# washer's handler, into the gauge dropper, at the dropper's 250-per-call rate.
if [ "$SELFTEST" = 1 ]; then
    echo "  (selftest: the drain is skipped)"
else
    ctl command "clear @s" >/dev/null
    ctl give "$DROPPER" 1 >/dev/null || fail "give $DROPPER failed"
    ctl wait 3
    ctl select "$DROPPER" >/dev/null
    ctl command "gamemode survival @s" >/dev/null
    ctl wait 3
    expect_chem "drained 1 mekanism:clean_iron into the gauge dropper (tank 0/10000, dropper 1/16000)" \
        -- drain-tank 250 1 1 72 0
    CLEAN_LEFT=$(block_data 1 72 0 output_amount)
    echo "  data get block 1 72 0 output_amount: $CLEAN_LEFT"
    case "$CLEAN_LEFT" in
        *"0"*) ;;
        *) fail "the output tank did not give up its one unit: $CLEAN_LEFT" ;;
    esac
    ctl command "gamemode creative @s" >/dev/null
    ctl wait 2
fi

# ---------------------------------------------------------------------------
# M. The sweep: every carried washing row through a real washer
# ---------------------------------------------------------------------------
echo "== M: every carried washing recipe in a real Chemical Washer =="
if [ "$NOCABLE" = 1 ]; then
    echo "  (NOCABLE: the sweep is skipped with the cable scenario)"
elif [ "$SELFTEST" = 1 ]; then
    echo "  (selftest: the sweep is skipped)"
else
    WROWS=$SCRATCH/washing-rows.json
    python3 -u "$PORT/tools/gen_washing.py" --rows >"$WROWS" \
        || fail "the washing rows could not be read from the jar"
    rows=$(jq 'length' "$WROWS")
    [ "$rows" -ge 7 ] || fail "the jar yielded only $rows washing rows"
    echo "  rows from the jar's mekanism:washing data: $rows"
    HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
        python3 -u "$PORT/tools/wash_sweep.py" "$SOCK" "$CTL" "$WROWS" 2 72 -4 --batch 7 \
        || fail "the washing sweep failed"
fi

# ---------------------------------------------------------------------------
# N. The Chemical Crystallizer: a clean slurry into a crystal (slices 9-10)
# ---------------------------------------------------------------------------
# The jar's numbers: 400 FE/t x 200 ticks = 80,000 FE an operation, a 160,000 FE
# insert-only buffer (exactly two operations), one 10,000-unit tank that takes a
# clean slurry with a crystallizing row and nothing else, and one output slot.
# 200 units of a clean slurry make one crystal, and the slurry is spent when the
# operation finishes. The tank beside the machine is the chemical side; the
# crystal leaves through the container's own slot.
echo "== N0: slice 10's four crystals — 1,000 units of clean_lead, one charge, two crystal_lead =="
place 0 72 14 "$TANK"
expect_chem "inserted 1000 mekanism:clean_lead into mekanism:basic_chemical_tank (1000/64000)" \
    -- insert 1000 mekanism:clean_lead 0 72 14
place 1 72 14 "$CRYSTALLIZER"
# The same arithmetic as N1, on a row this port did not have at 0.2.8: one
# 160,000 FE charge is exactly two operations, so two crystals from 400 of the
# 1,000 units beside it and 600 left.
say "mekanism energy 160000 1 72 14"
ctl wait 6
ctl wait 430
expect_contains 1 72 14 "$CRYSTALLIZER" "0/160000 FE, progress 0/200, input empty x0, output mekanism:crystal_lead x2"
expect_energy 1 72 14 "$CRYSTALLIZER" eq 0
expect_chem "holds mekanism:clean_lead 600/64000 (answered by mekanism)" -- read 0 72 14
expect_chem "holds nothing 0/10000 (answered by mekanism)" -- read 1 72 14

# ---------------------------------------------------------------------------
# P. The crystal leg (slice 11): a crystal into a shard through the chamber
# ---------------------------------------------------------------------------
# The jar's `processing/<material>/shard/from_crystal.json`: one crystal plus
# one unit of hydrogen chloride per operating tick become one shard of that
# material. The operation is the shared 200 ticks, so it spends 200 units and
# 400 FE/t x 200 = 80,000 FE. The seven rows this phase's generator now carries
# are all swept in phase H; this is the headline on the newest material (the
# slice-10 crystal).
echo "== P: slice 11's crystal leg — one crystal_lead + 4,000 HCl -> one shard_lead =="
place 0 72 16 "$TANK"
expect_chem "inserted 4000 mekanism:hydrogen_chloride into mekanism:basic_chemical_tank (4000/64000)" \
    -- insert 4000 mekanism:hydrogen_chloride 0 72 16
place 1 72 16 "$INJECTION"
ctl command "execute in minecraft:the_end run item replace block 1 72 16 container.0 with mekanism:crystal_lead 1" >/dev/null
ctl wait 6
# One full buffer (two operations' worth) so the assertion is about the
# operation's exact 80,000, not about a buffer that merely ran dry.
say "mekanism energy 160000 1 72 16"
ctl wait 6
ctl wait 430
# Slice 12 moved the `SELFTEST=1` flip to the newest family (the ore chain,
# phase Q): this phase asserts the injection chamber's real answer on every run.
expect_contains 1 72 16 "$INJECTION" "80000/160000 FE, progress 0/200, input empty x0, output mekanism:shard_lead x1"
expect_energy 1 72 16 "$INJECTION" eq 80000
# The gas came from the neighbour through the capability: 4,000 - 200.
expect_chem "holds mekanism:hydrogen_chloride 3800/64000 (answered by mekanism)" -- read 0 72 16
# ...and the machine kept none of it: it spent each tick's unit as it took it.
expect_chem "holds nothing 0/210 (answered by mekanism)" -- read 1 72 16

# ---------------------------------------------------------------------------
# Q. The ore chain for the four extra metals (slice 12): clump -> crusher ->
#    dirty dust -> enricher -> dust -> smelter -> ingot
# ---------------------------------------------------------------------------
# The jar declares the whole chain per material: `dirty_dust/from_clump.json`
# (crushing, one -> one), `dust/from_dirty_dust.json` (enriching, one -> one)
# and `ingot/from_dust_smelting.json` (a `minecraft:smelting` recipe the
# Energized Smelter takes). Each machine is charged with its whole 20,000 FE
# buffer; one operation is 200 ticks at 50 FE/t, so exactly 10,000 FE is spent
# and the buffer reads 10,000 afterwards. The three machines run side by side in
# one wait.
echo "== Q: slice 12's chain — a lead clump, its dirty dust, its dust and its ingot =="
place 2 64 18 "$CHAMBER"
place 4 64 18 "$SMELTER"
place 6 64 18 "$CRUSHER"
ctl command "execute in minecraft:the_end run item replace block 2 64 18 container.0 with mekanism:dirty_dust_lead 1" >/dev/null
ctl command "execute in minecraft:the_end run item replace block 4 64 18 container.0 with mekanism:dust_lead 1" >/dev/null
ctl command "execute in minecraft:the_end run item replace block 6 64 18 container.0 with mekanism:clump_lead 1" >/dev/null
ctl wait 6
say "mekanism energy 20000 2 64 18"
say "mekanism energy 20000 4 64 18"
say "mekanism energy 20000 6 64 18"
ctl wait 6
ctl wait 230
# Slice 13 moved the `SELFTEST=1` flip to the newest family (the smelting table,
# phase S): this phase asserts the chain's real answer on every run.
expect_contains 2 64 18 "$CHAMBER" "10000/20000 FE, progress 0/200, input empty x0, output mekanism:dust_lead x1"
expect_energy 2 64 18 "$CHAMBER" eq 10000
expect_contains 4 64 18 "$SMELTER" "10000/20000 FE, progress 0/200, input empty x0, output mekanism:ingot_lead x1"
expect_energy 4 64 18 "$SMELTER" eq 10000
expect_contains 6 64 18 "$CRUSHER" "10000/20000 FE, progress 0/200, input empty x0, output mekanism:dirty_dust_lead x1"
expect_energy 6 64 18 "$CRUSHER" eq 10000

# ---------------------------------------------------------------------------
# R. The sweep: every carried enriching row through a real Enrichment Chamber
# ---------------------------------------------------------------------------
# The parity rule's debt for the 132 rows the port carried since slice 2; slice
# 12 grows the table to 136 (four dirty dust -> dust rows) and pays it.
echo "== R: every carried enriching recipe in a real Enrichment Chamber =="
if [ "$NOCABLE" = 1 ]; then
    echo "  (NOCABLE: the sweep is skipped with the cable scenario)"
elif [ "$SELFTEST" = 1 ]; then
    echo "  (selftest: the sweep is skipped)"
else
    EROWS=$SCRATCH/enriching-rows.json
    python3 -u "$PORT/tools/gen_enriching.py" --rows >"$EROWS" \
        || fail "the enriching rows could not be read from the jar"
    rows=$(jq 'length' "$EROWS")
    # Slice 12's four dirty dust -> dust rows are part of this table: a tag
    # mapping that drops them again must fail the leg, not quietly sweep 132.
    [ "$rows" -ge 136 ] || fail "the jar yielded only $rows enriching rows"
    echo "  rows from the jar's mekanism:enriching data: $rows"
    HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
        python3 -u "$PORT/tools/enrich_sweep.py" "$SOCK" "$CTL" "$EROWS" 6 64 -6 --batch 10 \
        || fail "the enriching sweep failed"
    HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
        python3 -u "$PORT/tools/enrich_sweep.py" "$SOCK" "$CTL" --control 28 64 -6 \
        || fail "the enrichment chamber's no-recipe control failed"
fi

echo "== N1: two operations, the buffer's whole 160,000 FE, from 1,000 units =="
place 0 72 10 "$TANK"
expect_chem "inserted 1000 mekanism:clean_iron into mekanism:basic_chemical_tank (1000/64000)" \
    -- insert 1000 mekanism:clean_iron 0 72 10
place 1 72 10 "$CRYSTALLIZER"
# 400 FE/t x 200 ticks = 80,000 FE per operation, and the 160,000 buffer holds
# exactly two of them: one charge spends it all on two crystals and 400 units of
# the neighbour's slurry. (The slice-9 brief's headline asked for five crystals
# 230 ticks after one charge; that is one operation, not five — five operations
# are 1,000 ticks and 400,000 FE, which is what N1b charges.)
say "mekanism energy 160000 1 72 10"
ctl wait 6
ctl wait 430
expect_contains 1 72 10 "$CRYSTALLIZER" "0/160000 FE, progress 0/200, input empty x0, output $CRYSTAL x2"
expect_energy 1 72 10 "$CRYSTALLIZER" eq 0
# Two operations x 200 units: the neighbour gave 400 and has 600 left; the
# machine's own tank is empty (it spends what it drew when the operation
# finishes).
expect_chem "holds mekanism:clean_iron 600/64000 (answered by mekanism)" -- read 0 72 10
expect_chem "holds nothing 0/10000 (answered by mekanism)" -- read 1 72 10
# The second operation is the one the first 0.2.8 run lost: a slot holding
# one material's crystal stopped the draw loop at another material's row.
echo "== N1b: three more operations, three more charges, five crystals from 1,000 units =="
for charge in 160000 80000; do
    say "mekanism energy $charge 1 72 10"
    ctl wait 4
    ctl wait 430
done
expect_contains 1 72 10 "$CRYSTALLIZER" "0/160000 FE, progress 0/200, input empty x0, output $CRYSTAL x5"
expect_energy 1 72 10 "$CRYSTALLIZER" eq 0
CRYSTAL_ENERGY=$(block_data 1 72 10 energy)
echo "  data get block 1 72 10 energy: $CRYSTAL_ENERGY"
case "$CRYSTAL_ENERGY" in
    *" 0"*|*": 0"*) ;;
    *) fail "five operations should spend the whole 400,000 FE charged: $CRYSTAL_ENERGY" ;;
esac
# 5 operations x 200 units: the tank beside it is empty, and the machine's
# own tank too.
expect_chem "holds nothing 0/64000 (answered by mekanism)" -- read 0 72 10
expect_chem "holds nothing 0/10000 (answered by mekanism)" -- read 1 72 10

echo "== N2: one operation costs 80,000 FE and 200 units, and makes one crystal =="
place 4 72 8 "$TANK"
expect_chem "inserted 1000 mekanism:clean_iron into mekanism:basic_chemical_tank (1000/64000)" \
    -- insert 1000 mekanism:clean_iron 4 72 8
place 5 72 8 "$CRYSTALLIZER"
say "mekanism energy 80000 5 72 8"
ctl wait 6
ctl wait 230
expect_contains 5 72 8 "$CRYSTALLIZER" "0/160000 FE, progress 0/200, input empty x0, output $CRYSTAL x1"
expect_energy 5 72 8 "$CRYSTALLIZER" eq 0
# The tank beside it gave exactly the 200 units the operation spent...
expect_chem "holds mekanism:clean_iron 800/64000 (answered by mekanism)" -- read 4 72 8
# ...and the machine kept none of it.
expect_chem "holds nothing 0/10000 (answered by mekanism)" -- read 5 72 8

echo "== N3: a dirty slurry makes no progress, spends no FE, and the tank refuses it by name =="
place 0 72 12 "$TANK"
expect_chem "inserted 1000 mekanism:dirty_iron into mekanism:basic_chemical_tank (1000/64000)" \
    -- insert 1000 mekanism:dirty_iron 0 72 12
place 1 72 12 "$CRYSTALLIZER"
# The machine's own tank takes a clean slurry with a row and nothing else
# (upstream's `containsRecipe`): a dirty slurry is refused, and the answer says so.
expect_chem "inserted 0 mekanism:dirty_iron into mekanism:chemical_crystallizer (0/10000)" \
    -- insert 100 mekanism:dirty_iron 1 72 12
say "mekanism energy 160000 1 72 12"
ctl wait 6
ctl wait 230
# A dirty slurry is not a crystallizing input, so the machine resets its progress,
# spends no FE and does not siphon the tank beside it.
expect_contains 1 72 12 "$CRYSTALLIZER" "160000/160000 FE, progress 0/200, input empty x0, output empty x0"
expect_energy 1 72 12 "$CRYSTALLIZER" eq 160000
expect_chem "holds mekanism:dirty_iron 1000/64000" -- read 0 72 12

echo "== N4: a hopper under the output slot pulls the crystals into a chest =="
place 1 71 10 minecraft:hopper
place 1 70 10 minecraft:chest
ctl wait 4
ctl wait 100
CHEST=$(ctl container 1 70 10)
echo "  ctl container 1 70 10: $CHEST"
echo "$CHEST" \
    | jq -e '[.slots[] | select(.item == "mekanism:crystal_iron") | .count] | add == 5' \
        >/dev/null \
    || fail "the chest did not take the five crystals the output slot held: $CHEST"
# The output slot is empty again, and the machine makes no more: its
# neighbour tank is empty.
expect_contains 1 72 10 "$CRYSTALLIZER" "output empty x0"

# ---------------------------------------------------------------------------
# O. The sweep: every carried crystallizing row through a real crystallizer
# ---------------------------------------------------------------------------
echo "== O: every carried crystallizing recipe in a real Chemical Crystallizer =="
if [ "$NOCABLE" = 1 ]; then
    echo "  (NOCABLE: the sweep is skipped with the cable scenario)"
elif [ "$SELFTEST" = 1 ]; then
    echo "  (selftest: the sweep is skipped)"
else
    CROWS=$SCRATCH/crystallizing-rows.json
    python3 -u "$PORT/tools/gen_crystallizing.py" --rows >"$CROWS" \
        || fail "the crystallizing rows could not be read from the jar"
    rows=$(jq 'length' "$CROWS")
    # The washer makes seven slurries and slice 10 carries a crystal for every
    # one, so the sweep must see all seven rows — a dropped row is a dead washer
    # recipe, not a smaller sweep.
    [ "$rows" -ge 7 ] || fail "the jar yielded only $rows crystallizing rows, expected 7"
    echo "  rows from the jar's mekanism:crystallizing data: $rows"
    HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
        python3 -u "$PORT/tools/crystal_sweep.py" "$SOCK" "$CTL" "$CROWS" 6 72 -4 --batch 3 \
        || fail "the crystallizing sweep failed"
fi

# ---------------------------------------------------------------------------
# S. The carried smelting table's headline (slice 13): three named vanilla rows,
#    one of the jar's own dust rows, and a no-recipe control.
# ---------------------------------------------------------------------------
# The Energized Smelter smelts every `minecraft:smelting` recipe — upstream adds
# them all to its own `mekanism:smelting` type — so its table is the 163 rows the
# 26.3 data expands to plus the jar's seven dust -> ingot rows; phase T sweeps
# all 170. This phase is the headline on named rows: an ore into its item, a
# block into its fired product, a tin dust into this port's own ingot, and a
# dirt the table has no row for, which must stay put and make nothing.
echo "== S: the smelting table's named rows — coal ore, clay, a tin dust and a dirt control =="
# Stand where the row is built: a `setblock` in a chunk the player has left
# answers "Could not set the block".
ctl command "execute in minecraft:the_end run tp @s 10.5 64 -10.5" >/dev/null
ctl wait 10
place 0 64 -10 "$SMELTER"
place 2 64 -10 "$SMELTER"
place 4 64 -10 "$SMELTER"
place 6 64 -10 "$SMELTER"
ctl command "execute in minecraft:the_end run item replace block 0 64 -10 container.0 with minecraft:coal_ore 1" >/dev/null
ctl command "execute in minecraft:the_end run item replace block 2 64 -10 container.0 with minecraft:clay 1" >/dev/null
ctl command "execute in minecraft:the_end run item replace block 4 64 -10 container.0 with mekanism:dust_tin 1" >/dev/null
ctl command "execute in minecraft:the_end run item replace block 6 64 -10 container.0 with minecraft:dirt 1" >/dev/null
ctl wait 6
say "mekanism energy 20000 0 64 -10"
say "mekanism energy 20000 2 64 -10"
say "mekanism energy 20000 4 64 -10"
say "mekanism energy 20000 6 64 -10"
ctl wait 6
ctl wait 230
# Slice 14 moved the `SELFTEST=1` flip to the newest family (the storage-block
# and nugget recipes, phase U): this phase asserts the smelting table's real
# answer on every run.
# One operation is 200 ticks at 50 FE/t: 10,000 FE of the 20,000 charged.
expect_contains 0 64 -10 "$SMELTER" "10000/20000 FE, progress 0/200, input empty x0, output minecraft:coal x1"
expect_energy 0 64 -10 "$SMELTER" eq 10000
expect_contains 2 64 -10 "$SMELTER" "10000/20000 FE, progress 0/200, input empty x0, output minecraft:terracotta x1"
expect_energy 2 64 -10 "$SMELTER" eq 10000
# The jar's own row, in the same machine as the vanilla ones.
expect_contains 4 64 -10 "$SMELTER" "10000/20000 FE, progress 0/200, input empty x0, output mekanism:ingot_tin x1"
expect_energy 4 64 -10 "$SMELTER" eq 10000
# The control: dirt has no row, so the machine keeps it, spends nothing and
# makes nothing.
expect_contains 6 64 -10 "$SMELTER" "20000/20000 FE, progress 0/200, input minecraft:dirt x1, output empty x0"
expect_energy 6 64 -10 "$SMELTER" eq 20000

# ---------------------------------------------------------------------------
# T. The sweeps: every carried smelting row through a real Energized Smelter
# ---------------------------------------------------------------------------
# The parity rule's debt for a table the port has carried since slice 1, paid
# the way phase R pays the enriching one: 163 vanilla rows and the jar's 7 own
# rows, each one operation in its own machine, the input consumed and the
# declared output stack asserted. Batch 20 keeps 163 rows to nine waits; the
# ctl calls per row, not the ticks, are the sweep's cost.
echo "== T: every carried smelting recipe in a real Energized Smelter =="
if [ "$NOCABLE" = 1 ]; then
    echo "  (NOCABLE: the sweeps are skipped with the cable scenario)"
elif [ "$SELFTEST" = 1 ]; then
    echo "  (selftest: the sweeps are skipped)"
else
    ctl command "execute in minecraft:the_end run tp @s 12.5 64 -12.5" >/dev/null
    ctl wait 10
    SURFACE_ROWS=$SCRATCH/smelting-rows.json
    if [ -n "$RES" ]; then
        python3 -u "$PORT/tools/gen_smelting.py" --resources "$RES" --rows >"$SURFACE_ROWS" \
            || fail "the 26.3 smelting rows could not be read"
    else
        python3 -u "$PORT/tools/gen_smelting.py" --rows >"$SURFACE_ROWS" \
            || fail "the 26.3 smelting rows could not be read"
    fi
    rows=$(jq 'length' "$SURFACE_ROWS")
    # 163 vanilla rows: a recipe or tag mapping that drops one must fail the leg,
    # not quietly sweep a smaller table.
    [ "$rows" -ge 163 ] || fail "the 26.3 data yielded only $rows smelting rows, expected 163"
    echo "  rows from the 26.3 data's minecraft:smelting recipes: $rows"
    python3 -u "$PORT/tools/smelt_sweep.py" "$SOCK" "$CTL" "$SURFACE_ROWS" -6 64 -12 --batch 20 \
        || fail "the smelting sweep failed"
    JAR_ROWS=$SCRATCH/smelting-jar-rows.json
    python3 -u "$PORT/tools/gen_smelting.py" --mekanism-rows >"$JAR_ROWS" \
        || fail "the jar's dust rows could not be read"
    rows=$(jq 'length' "$JAR_ROWS")
    [ "$rows" -ge 7 ] || fail "the jar yielded only $rows dust -> ingot smelting rows, expected 7"
    echo "  rows from the jar's processing/*/ingot/from_dust_smelting.json: $rows"
    python3 -u "$PORT/tools/smelt_sweep.py" "$SOCK" "$CTL" "$JAR_ROWS" -6 64 -14 --batch 10 \
        --label "jar " || fail "the jar's dust -> ingot sweep failed"
    python3 -u "$PORT/tools/smelt_sweep.py" "$SOCK" "$CTL" --control 0 64 -16 \
        || fail "the smelter's no-recipe control failed"
fi

echo "== persistence: save, quit and relaunch read the block entities back =="
# Stand in the arena first: a `setblock` in a chunk the player has left answered
# "Could not set the block" and left the cable in place, so the player is brought
# back before the row is cleared and the cleared blocks are read back.
ctl command "execute in minecraft:the_end run tp @s 0.5 64 3.5" >/dev/null
ctl wait 20
# Cut the smelter's cable so nothing feeds it while it stalls, empty its buffer
# so the operation cannot start early, then leave it mid-operation: 3,000 FE is
# exactly 60 ticks of work at 50 FE/t, from progress 0.
if [ "$NOCABLE" = 0 ]; then
    end "setblock 1 64 0 minecraft:air" >/dev/null
fi
end "setblock 2 64 0 minecraft:air" >/dev/null
ctl wait 6
echo "  cleared for the stall: 1,64,0 $(block_name 1 64 0), 2,64,0 $(block_name 2 64 0)"
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
# A washer left mid-operation with all three of its tanks loaded: 13,400 FE is
# exactly 67 ticks at 200 FE/t, so it stalls at progress 67 with its water and its
# ten units of dirty slurry untouched (both are spent at the operation's finish)
# and the output tank empty.
place 4 72 4 "$WASHER"
expect_fluid "inserted 1000 minecraft:water into mekanism:chemical_washer (1000/10000)" -- insert 1000 4 72 4
expect_chem "inserted 10 mekanism:dirty_iron into mekanism:chemical_washer (10/10000)"     -- insert 10 mekanism:dirty_iron 4 72 4
say "mekanism energy 0 4 72 4"
ctl wait 6
say "mekanism energy 13400 4 72 4"
ctl wait 4
ctl wait 130
# A crystallizer left mid-operation too: 26,800 FE is exactly 67 ticks at
# 400 FE/t, so it stalls at progress 67 with its 400 units of clean slurry still
# in its tank (the slurry is spent when the operation finishes).
place 8 72 4 "$CRYSTALLIZER"
expect_chem "inserted 400 mekanism:clean_iron into mekanism:chemical_crystallizer (400/10000)"     -- insert 400 mekanism:clean_iron 8 72 4
say "mekanism energy 0 8 72 4"
ctl wait 6
say "mekanism energy 26800 8 72 4"
ctl wait 4
ctl wait 130
echo "  before save, chamber: $(info_once 0 64 16 "$CHAMBER")"
echo "  before save, cube:    $(info_once 2 64 16 "$CUBE")"
echo "  before save, washer:  $(info_once 4 72 4 "$WASHER")"
echo "  before save, crystallizer: $(info_once 8 72 4 "$CRYSTALLIZER")"
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
echo "  after relaunch, factory: $(info_once 1 70 8 "$PURIFYING_FACTORY")"
expect_contains 1 70 8 "$PURIFYING_FACTORY" "0/120000 FE (3 processes), progress 67/200, input minecraft:iron_ore x3"
echo "  after relaunch, washer: $(info_once 4 72 4 "$WASHER")"
# The water and the slurry are spent when the operation **finishes** (upstream's
# `TwoInputCachedRecipe.twoInputCalculateOperationsThisTick` takes them in
# `finishProcessing`), so a mid-operation washer still holds all 1,000 mB.
expect_contains 4 72 4 "$WASHER" "0/80000 FE, progress 67/200, water 1000/10000 mB, input mekanism:dirty_iron x10, output empty x0"
echo "  after relaunch, crystallizer: $(info_once 8 72 4 "$CRYSTALLIZER")"
expect_contains 8 72 4 "$CRYSTALLIZER" "0/160000 FE, progress 67/200, input mekanism:clean_iron x400, output empty x0"
echo "  (the saved compound survives: FE, progress and slots read back)"

# ---------------------------------------------------------------------------
# V. The Chemical Dissolution Chamber (slice 15): an ore and sulfuric acid into
#    a dirty slurry, and the washer beside it taking that slurry
# ---------------------------------------------------------------------------
# The jar's numbers: 400 FE/t x 100 ticks = 40,000 FE an operation (a 160,000 FE
# buffer), two 10,000-unit tanks, and every row `per_tick_usage: true` — one unit
# of sulfuric acid a tick for an ore, so 100 units an operation. The chamber
# pulls the tick's acid from the tank beside it through `mekanism:chemical` and
# keeps none of it; what it makes is in its second tank, which the washer beside
# it takes through the same capability. The rows are read back from the
# generated table, so the scenario and the table cannot drift apart.
echo "== V1: one iron ore dissolves into 1,000 units of dirty_iron for 40,000 FE and exactly 100 acid =="
DROWS=$SCRATCH/dissolving-rows.json
python3 -u "$PORT/tools/gen_dissolving.py" --rows >"$DROWS" \
    || fail "the dissolution rows could not be read from the jar"
rows=$(jq 'length' "$DROWS")
# Twelve rows: three vanilla ores in their two 26.3 forms, and the raw ore / raw
# block rows for iron, gold and copper. A mapping that drops one must fail the
# leg, not quietly check a smaller table.
[ "$rows" -ge 12 ] || fail "the jar yielded only $rows dissolution rows, expected 12"
ORE_ROW=$(jq -r '.[] | select(.input == "minecraft:iron_ore")' "$DROWS")
ACID_PER_TICK=$(jq -r '.acid_per_tick' <<<"$ORE_ROW")
ORE_OUT=$(jq -r '.output' <<<"$ORE_ROW")
ORE_AMOUNT=$(jq -r '.output_amount' <<<"$ORE_ROW")
ACID_SPENT=$((ACID_PER_TICK * 100))
ACID_LEFT=$((1000 - ACID_SPENT))
echo "  rows from the jar's mekanism:dissolution data: $rows"
echo "  minecraft:iron_ore: $ACID_PER_TICK acid a tick x 100 ticks = $ACID_SPENT, -> $ORE_AMOUNT $ORE_OUT"
place 0 74 0 "$TANK"
acid_tank_1000 0 74 0
place 1 74 0 "$DISSOLUTION"
ctl command "execute in minecraft:the_end run item replace block 1 74 0 container.0 with minecraft:iron_ore 1" >/dev/null
ctl wait 6
expect_contains 1 74 0 "$DISSOLUTION" "0/160000 FE, progress 0/100, input minecraft:iron_ore x1, acid empty 0/10000, output empty 0/10000"
say "mekanism energy 160000 1 74 0"
ctl wait 6
# The machine starts on its first charged tick, so the assertion here is that it
# took the energy at all (the exact 40,000 is asserted after the operation).
expect_energy 1 74 0 "$DISSOLUTION" gt 0
ctl wait 120
expect_contains 1 74 0 "$DISSOLUTION" "input empty x0, acid empty 0/10000, output $ORE_OUT $ORE_AMOUNT/10000"
# 100 ticks x 400 FE: exactly a quarter of the 160,000 buffer.
expect_energy 1 74 0 "$DISSOLUTION" eq 120000
# The chamber's own saved tank holds the slurry; its acid tank is empty because
# it pulled one tick's acid per tick and spent it.
OUT_CHEM=$(block_data 1 74 0 output_chemical)
OUT_AMOUNT=$(block_data 1 74 0 output_amount)
ACID_AMOUNT=$(block_data 1 74 0 acid_amount)
echo "  data get block 1 74 0 output_chemical: $OUT_CHEM"
echo "  data get block 1 74 0 output_amount: $OUT_AMOUNT"
echo "  data get block 1 74 0 acid_amount: $ACID_AMOUNT"
case "$OUT_CHEM" in
    *"$ORE_OUT"*) ;;
    *) fail "the chamber's output tank does not hold $ORE_OUT: $OUT_CHEM" ;;
esac
case "$OUT_AMOUNT" in
    *"$ORE_AMOUNT"*) ;;
    *) fail "the chamber's output tank should hold $ORE_AMOUNT: $OUT_AMOUNT" ;;
esac
case "$ACID_AMOUNT" in
    *" 0"*|*": 0"*) ;;
    *) fail "the chamber's own acid tank should be empty after the operation: $ACID_AMOUNT" ;;
esac
# The acid came from the neighbour, through the capability: 1,000 - 100.
# The neighbour's amount is read from its saved data (the chat `chemical: read`
# line can be throttled on a long loaded leg; the 0.2.14 evidence proves the
# capability answers "(answered by mekanism)" for this same read).
TANK_LEFT=$(block_data 0 74 0 amount)
echo "  data get block 0 74 0 amount: $TANK_LEFT"
case "$TANK_LEFT" in
    *"$ACID_LEFT"*) ;;
    *) fail "the neighbour tank should hold $ACID_LEFT after one operation: $TANK_LEFT" ;;
esac
# ...and a wrong chemical in the machine's own tank makes no progress and spends
# no FE: water at the acid tank, exactly the counterfactual the unit test pins.
place 4 74 0 "$DISSOLUTION"
ctl command "execute in minecraft:the_end run item replace block 4 74 0 container.0 with minecraft:iron_ore 1" >/dev/null
say "mekanism energy 160000 4 74 0"
ctl wait 6
ctl command "execute in minecraft:the_end run data merge block 4 74 0 {acid_chemical:\"minecraft:water\",acid_amount:1000}" >/dev/null
ctl wait 120
expect_contains 4 74 0 "$DISSOLUTION" "160000/160000 FE, progress 0/100, input minecraft:iron_ore x1, acid minecraft:water 1000/10000, output empty 0/10000"
expect_energy 4 74 0 "$DISSOLUTION" eq 160000
# An item the table has no row for makes nothing, spends nothing and, because it
# could not work, does not siphon the acid beside it.
place 5 74 0 "$TANK"
acid_tank_1000 5 74 0
place 6 74 0 "$DISSOLUTION"
ctl command "execute in minecraft:the_end run item replace block 6 74 0 container.0 with minecraft:dirt 1" >/dev/null
say "mekanism energy 160000 6 74 0"
ctl wait 6
ctl wait 120
expect_contains 6 74 0 "$DISSOLUTION" "160000/160000 FE, progress 0/100, input minecraft:dirt x1, acid empty 0/10000, output empty 0/10000"
expect_energy 6 74 0 "$DISSOLUTION" eq 160000
CONTROL_TANK_LEFT=$(block_data 5 74 0 amount)
echo "  data get block 5 74 0 amount: $CONTROL_TANK_LEFT"
case "$CONTROL_TANK_LEFT" in
    *1000*) ;;
    *) fail "the dirt control's neighbour tank should still hold 1000: $CONTROL_TANK_LEFT" ;;
esac

echo "== V2: the washer beside the chamber takes the slurry the chamber made =="
place 2 74 0 "$WASHER"
expect_fluid "inserted 1000 minecraft:water into mekanism:chemical_washer (1000/10000)" \
    -- insert 1000 2 74 0
say "mekanism energy 40000 2 74 0"
ctl wait 6
ctl wait 230
# One washing operation: 5 mB of water and one unit of the chamber's dirty_iron
# become one unit of clean_iron for 40,000 FE. The washer found the slurry in the
# chamber's **second** tank through `mekanism:chemical`, and the chamber lost it.
expect_contains 2 74 0 "$WASHER" "0/80000 FE, progress 0/200, water 995/10000 mB, input empty x0, output mekanism:clean_iron x1"
CHAMBER_LEFT=$(block_data 1 74 0 output_amount)
echo "  data get block 1 74 0 output_amount: $CHAMBER_LEFT"
case "$CHAMBER_LEFT" in
    *"$((ORE_AMOUNT - 1))"*) ;;
    *) fail "the chamber should hold $((ORE_AMOUNT - 1)) units after the washer took one: $CHAMBER_LEFT" ;;
esac

echo "== V3: all twelve carried dissolution rows in real chambers, with a dirt control =="
# The rows come from the generator the package's table is built from, so the
# scenario cannot drift from what the chamber actually carries. The tool checks
# every row resolves its input before spending energy and reports a row that
# silently fails to resolve instead of skipping it; SELFTEST flips the
# deepslate-iron row's expected output and must fail on the real dirty_iron.
SWEEP_FLAGS=()
if [ "$SELFTEST" = 1 ]; then SWEEP_FLAGS+=(--selftest); fi
HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
    python3 -u "$PORT/tools/dissolve_sweep.py" "$SOCK" "$CTL" --control 8 74 4
HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
    python3 -u "$PORT/tools/dissolve_sweep.py" "$SOCK" "$CTL" "$DROWS" 8 74 4 \
        --batch 6 "${SWEEP_FLAGS[@]}"

# ---------------------------------------------------------------------------
# W. The acid chain (slice 17): sulfur dust -> SO2 -> SO3 -> locally made acid,
#    which then runs the dissolution chamber. SELFTEST=acid flips only this
#    family's acid assertion; SELFTEST=1 keeps flipping the dissolution sweep.
# ---------------------------------------------------------------------------
echo "== W: the locally made acid chain, from sulfur dust to dirty iron =="
ACID_FLAGS=()
if [ "$SELFTEST" = 1 ] || [ "$SELFTEST" = acid ]; then ACID_FLAGS+=(--selftest); fi
HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
    python3 -u "$PORT/tools/acid_chain.py" "$SOCK" "$CTL" 12 76 8 --control
HORIZON_LOG="$HORIZON_HOME/logs/latest.log" \
    python3 -u "$PORT/tools/acid_chain.py" "$SOCK" "$CTL" 12 76 8 "${ACID_FLAGS[@]}" \
    || fail "the acid chain did not run end to end"

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
    echo "PASS: the chemical capability works (an insert lands in the tank's saved data, part of it drains into the gauge dropper, an unknown id and a block that answers no chemical both move nothing), its consumers pull a gas from a neighbour through the same capability and spend exactly one operation's worth on one operation (or nothing without a usable gas), the five basic factories run three processes of the same operation side by side (three ore -> nine clumps for 3 x 40,000 FE and 600 oxygen; one ore -> three clumps for one operation; 40,000 FE buys 200 progress ticks, not three operations), the washer takes water through the game's own fluid capability and dirty slurry through the chemical one and turns one operation into 5 mB of water and one unit of dirty slurry spent for one clean unit (while a clean slurry, an empty water tank, water at the chemical key and a chemical tank at the fluid key all move nothing), the crystallizer turns 200 units of a clean slurry into one crystal for 80,000 FE and five operations into five crystals, refusing a dirty slurry at its tank while a dirty slurry in a neighbour tank spends nothing, slice 12's four metals run their chain (dirty dust -> dust -> ingot, clump -> dirty dust) for 10,000 FE an operation, the chemical dissolution chamber dissolves one iron ore into 1,000 units of dirty_iron for 40,000 FE and 100 of a neighbour tank's sulfuric acid while water and a dirt make nothing, all twelve carried rows and the dirt control in the dissolution sweep run in real chambers, and without a cable the charged cube moves no energy into the adjacent smelter or chamber, which never smelt or enrich"
elif [ "$SELFTEST" = 1 ] || [ "$SELFTEST" = acid ]; then
    echo "SELFTEST finished without failing (this is a bug in the test)"
    exit 1
else
    echo "PASS: the twenty blocks, their forty-five items and seventeen shaped recipes and their loot tables work, and the four metals' storage-block and nugget family runs both ways in a real table (a block from nine ingots, nine ingots from a block, an ingot from nine nuggets and nine nuggets from an ingot, each with its id, its count, its grid emptied and its ingredients consumed, while eight ingots and a dirt make nothing and the placed block breaks into exactly one of itself), every carried crushing recipe runs in a real crusher (156 rows, slice 13's four ingot -> dust rows included), every carried smelting recipe - the 163 rows the 26.3 data expands to and the jar's own seven dust -> ingot rows - runs in a real Energized Smelter with its input consumed and its output stack asserted (a coal ore into coal, clay into terracotta and a tin dust into ingot_tin for 10,000 FE each, while a dirt the table has no row for stays put), every carried enriching recipe runs in a real enrichment chamber (136 rows, slice 12's four dirty dust -> dust rows included), every carried injecting recipe - the seven crystal->shard rows slice 11 adds included - runs in a real chemical injection chamber and every carried purifying recipe in a real purification chamber, both pulling their gas from a neighbour's tank through mekanism:chemical and spending exactly the operation's gas - 200 oxygen for one iron ore into three clumps at 40,000 FE - every carried washing recipe runs in a real chemical washer, whose water arrives through horizon:fluids and whose dirty slurry arrives through mekanism:chemical, one operation spending exactly 5 mB of water, one unit of dirty slurry and 40,000 FE for one unit of clean slurry, every carried crystallizing recipe - all seven slurries the washer makes, lead through uranium included - runs in a real chemical crystallizer that turns 200 units of its clean slurry into one crystal for 80,000 FE (five operations into five crystals from 1,000 units, a lead tank's 1,000 units into exactly two crystal_lead with 600 left, a hopper pulling them out, a dirty slurry refused by the tank and making nothing in a neighbour, and the crystallizer's own crystal_lead in an injection chamber becoming exactly one shard_lead for 80,000 FE and 200 of a neighbour tank's hydrogen chloride), the chemical dissolution chamber dissolves one iron ore into exactly 1,000 units of dirty_iron for 40,000 FE and exactly 100 of a neighbour tank's sulfuric acid, while water in its own tank and a dirt in its slot make nothing and spend nothing, and the washer beside it takes that slurry through mekanism:chemical and finishes one clean_iron row, every one of the twelve carried dissolution rows runs in a real chamber with its declared item count, per-tick acid, slurry and amount (the input consumed, exactly 40,000 FE spent and the source tank short exactly its acid, while a dirt no-row control keeps its slot and draws nothing), the jar's chain runs end to end (a clump crushes into a dirty dust and that dirty dust enriches into a dust, while a purification chamber has no row for it), and slice 12's four metals run their own chain in real machines (a dirty_dust_lead enriches into dust_lead for 10,000 FE, that dust smelts into ingot_lead, and a clump_lead crushes into dirty_dust_lead), the five basic factories run three processes of the same operation side by side (three ore in the purifying factory become nine clumps for 3 x 40,000 FE and exactly 600 of the neighbour's oxygen, one ore costs one operation, and 40,000 FE buys 200 progress ticks rather than three operations, while a smelting factory turns three ore into three ingots for its 30,000 FE buffer), the smelter, cube, cable and enrichment chamber work end to end - the cube's FE reaches the chamber through the cable and turns one ore into two dust, that dust smelts into two ingots, an unsmeltable input, a no-gas and a wrong-gas neighbour do nothing - the menus open, the machines', the factories' and the crystallizer's FE, progress and tanks survive a save and relaunch, and the chemical capability moves one chemical into the tank's saved data, drains part of it into the gauge dropper through the item capability, and refuses an unknown id by name while a block that answers no chemical moves nothing"
fi
