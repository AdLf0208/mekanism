#!/usr/bin/env bash
# One slot lease for the final Mekanism slice-2 evidence: the default e2e, the
# no-cable differential, the fail-capable self-test, the staged check and test
# on the final sources, the content package and the native package.
# Run through: build-slot.sh e2e --job … --port ports/mekanism -- tools/final-evidence.sh
set -uo pipefail
export LC_ALL=C
exec 1>&2

PORT=$(cd "$(dirname "$0")/.." && pwd)
CLONE=$(cd "$PORT/../.." && pwd)
E=$CLONE/ports/_shared/evidence/mekanism
mkdir -p "$E"
OUT=${OUT:-/tmp/opencode}
H=$CLONE/ports/_shared/pins/horizon-1d63e0a
cd "$PORT"
echo "port $PORT evidence $E"

step() { echo "### $*"; }

step "e2e (default)"
tools/e2e.sh >"$E/e2e-default.log" 2>&1
rc=$?
echo "default e2e exit: $rc"
[ "$rc" -eq 0 ] || { tail -5 "$E/e2e-default.log" >&2; exit 1; }

step "e2e (NOCABLE=1, the differential)"
NOCABLE=1 tools/e2e.sh >"$E/e2e-nocable.log" 2>&1
rc=$?
echo "nocable e2e exit: $rc"
[ "$rc" -eq 0 ] || { tail -5 "$E/e2e-nocable.log" >&2; exit 1; }

step "e2e (SELFTEST=1, must fail)"
SELFTEST=1 tools/e2e.sh >"$E/e2e-selftest.log" 2>&1
rc=$?
echo "selftest e2e exit: $rc (non-zero is required)"
[ "$rc" -ne 0 ] || { echo "SELFTEST passed: the scenario cannot fail" >&2; exit 1; }

step "format"
cargo fmt --check
echo "fmt exit: $?"

step "staged check"
python3 tools/horizon.py check --horizon "$H" >"$OUT/mek-check-final.log" 2>&1
rc=$?
echo "check exit: $rc"
grep -c "warning" target/horizon-check.log || true

step "staged test"
python3 tools/horizon.py test --horizon "$H" >"$OUT/mek-test-final.log" 2>&1
rc=$?
echo "test exit: $rc"
grep -E "test result" target/horizon-test.log || true

step "content package"
TOOL=$HORIZON_TARGET_DIR/debug/cargo-horizon-mod
[ -x "$TOOL" ] || TOOL=$CLONE/ports/_shared/build-slots/slot-1/target/debug/cargo-horizon-mod
[ -x "$TOOL" ] || { echo "no cargo-horizon-mod binary in the slot target" >&2; exit 1; }
(cd content && "$TOOL" build) >"$OUT/mek-hmod-build.log" 2>&1
rc=$?
echo "hmod build exit: $rc"
[ "$rc" -eq 0 ] || { cat "$OUT/mek-hmod-build.log" >&2; exit 1; }
cp content/target/horizon-mod/mekanism.hmod mekanism.hmod
ls -l mekanism.hmod
(cd "$CLONE/ports/_shared/pins/horizon-1d63e0a" && "$TOOL" check "$PORT/mekanism.hmod" --json) >"$E/hmod-check.json" 2>&1 || true
cat "$E/hmod-check.json"

step "native package"
python3 tools/package.py

echo "ALL STEPS DONE"
