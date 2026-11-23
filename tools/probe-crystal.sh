#!/usr/bin/env bash
# A one-purpose diagnostic (not part of the ladder): a Basic Chemical Tank of
# 1,000 clean iron beside a Chemical Crystallizer with a full 160,000 FE buffer,
# read every 100 game ticks, so a stall shows its tick.
#
#   eval "$(ports/tools/launcher-game.sh env mekanism)"
#   ports/tools/build-slot.sh run --job probe-mekanism --port ports/mekanism \
#       --step e2e -- bash ports/mekanism/tools/probe-crystal.sh
set -euo pipefail
export LC_ALL=C
exec 1>&2

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
GAME=${GAME:?set GAME (launcher-game.sh env)}
CTL=${CTL:?set CTL}
RES=${RES:-${HORIZON_RES:-}}
TMP_ROOT=${E2E_TMP:-/tmp/opencode}
mkdir -p "$TMP_ROOT"
SCRATCH=$(mktemp -d "$TMP_ROOT"/mekanism-probe.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
SOCK=$SCRATCH/probe.sock
mkdir -p "$XDG_CONFIG_HOME"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
say() { ctl chat "/$1" >/dev/null; }
end() { ctl command "execute in minecraft:the_end run $1"; }
info() { say "mekanism info $1 $2 $3" >/dev/null; ctl wait 4 >/dev/null; sleep 0.4
         grep -aE "\[CHAT\] mekanism:chemical_crystallizer: " "$HORIZON_HOME/logs/latest.log" | tail -1 | sed 's/.*\[CHAT\] //'; }
chem() { say "mekanism chemical $*" >/dev/null; ctl wait 4 >/dev/null; sleep 0.3
         grep -aE "\[CHAT\] chemical: " "$HORIZON_HOME/logs/latest.log" | tail -1 | sed 's/.*\[CHAT\] //'; }
cleanup() { local s=$?; ctl quit >/dev/null 2>&1 || true; sleep 1; rm -rf "$SCRATCH"; echo "EXIT:$s"; }
trap cleanup EXIT

GAME_PID=$(ctl launch --game "$GAME" --world ProbeCrystal --seed 20261105 --creative --offscreen --wait \
    ${RES:+--resources "$RES"} | jq -r '.pid // empty')
ctl wait-ready 120 >/dev/null
ctl wait 40 >/dev/null
ctl command "execute in minecraft:the_end run tp @s 0.5 72 9.5" >/dev/null
ctl wait 10
end "fill -4 71 4 8 71 14 minecraft:stone" >/dev/null
end "fill -4 72 4 8 80 14 minecraft:air" >/dev/null
ctl command "difficulty peaceful" >/dev/null
ctl wait 10

end "setblock 0 72 8 mekanism:basic_chemical_tank" >/dev/null
ctl wait 4
echo "tank insert: $(chem insert 1000 mekanism:clean_iron 0 72 8)"
end "setblock 1 72 8 mekanism:chemical_crystallizer" >/dev/null
ctl wait 4
ctl chat "/mekanism energy 160000 1 72 8" >/dev/null
ctl wait 4
echo "placed: $(ctl block 1 72 8 | jq -r .name)"

for round in 1 2 3 4 5 6 7 8; do
    ctl wait 100 >/dev/null
    echo "t+$((round * 100)): $(info 1 72 8)"
    echo "        tank: $(chem read 0 72 8)"
done
echo "raw: $(ctl command 'data get block 1 72 8' | jq -r '.chat[0] // ""')"
echo "PROBE: done"
