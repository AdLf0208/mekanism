#!/usr/bin/env bash
# The registry check and the two `fits` runs for slice 20, one lease.
set -uo pipefail
cd "$(dirname "$0")/.."
cargo horizon-mod check mekanism.hmod
echo "CHECK_EXIT:$?"
cargo horizon-mod fits mekanism.hmod --build 0a875da
echo "FITS_PIN_EXIT:$?"
cargo horizon-mod fits mekanism.hmod --build 0b14f8a
echo "FITS_OTHER_EXIT:$?"
