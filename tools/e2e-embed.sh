#!/usr/bin/env bash
# The embedded-content proof in one lease: the scenario (must PASS, with the
# control leg first) and the selftest (must exit non-zero), each writing its own
# evidence log with its own EXIT line. Run through the slot registry:
#   ports/tools/build-slot.sh e2e --job mk-embed-e2e --port ports/mekanism \
#       --horizon <abs>/ports/_shared/pins/horizon-1d63e0a -- bash tools/e2e-embed.sh
set -uo pipefail
PORT=$(cd "$(dirname "$0")/.." && pwd)
E=$PORT/../_shared/evidence/mekanism
mkdir -p "$E"
cd "$PORT"

status=0
echo "== embedded scenario (no --dev-mod; control leg first)"
env EMBEDDED=1 CONTROL=1 bash tools/e2e.sh > "$E/e2e-embedded.log" 2>&1
scenario=$?
echo "  scenario exit $scenario"
tail -3 "$E/e2e-embedded.log"
[ "$scenario" = 0 ] || { echo "FAIL: the embedded scenario did not pass"; status=1; }

echo "== embedded selftest (must exit non-zero)"
env EMBEDDED=1 SELFTEST=1 bash tools/e2e.sh > "$E/e2e-embedded-selftest.log" 2>&1
selftest=$?
echo "  selftest exit $selftest"
tail -3 "$E/e2e-embedded-selftest.log"
[ "$selftest" != 0 ] || { echo "FAIL: the selftest did not fail"; status=1; }

echo "EMBEDDED-E2E:$status"
exit $status
