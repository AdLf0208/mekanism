#!/usr/bin/env bash
# The non-e2e ladder for the embedded-content slice: fmt, clippy, the pure
# runtime tests, then the content build synced into the bundle's data/ (the
# injected `native_data!` compiles it in), the injected module's
# stage/check/test/build, and the packaging step (which rebuilds the .hmod,
# re-syncs it and writes the one .hnmod). Run inside a build slot:
#   ports/tools/build-slot.sh run --job mk-ladder --port ports/mekanism \
#       --step build -- bash tools/ladder-embed.sh
set -euo pipefail

PORT=$(cd "$(dirname "$0")/.." && pwd)
HORIZON=$PORT/../_shared/pins/horizon-1d63e0a
E=$PORT/../_shared/evidence/mekanism
mkdir -p "$E"
cd "$PORT"

echo "== cargo fmt --all -- --check"
cargo fmt --all -- --check 2>&1 | tee "$E/fmt.log"
echo "== rustfmt --check the injected helper"
rustfmt --check mixins/mekanism/embedded_content.rs 2>&1 | tee "$E/fmt-helper.log"
echo "== clippy"
cargo clippy --all-targets --locked --offline -p horizon-mekanism -- -D warnings 2>&1 | tee "$E/clippy.log"
echo "== pure runtime tests"
cargo test --offline --locked -p horizon-mekanism 2>&1 | tee "$E/test-green.log"
echo "== content build + sync into the bundle's data/ (the injection compiles it in)"
export CARGO_TARGET_DIR=$PORT/target/carry-tools
(cd content && cargo run --locked --offline --quiet \
    --manifest-path "$HORIZON/crates/cargo-horizon-mod/Cargo.toml" -- horizon-mod build)
cp content/target/horizon-mod/mekanism.hmod content/mekanism.hmod
mkdir -p mixins/mekanism/data
cp content/mekanism.hmod mixins/mekanism/data/mekanism-content.hmod
unset CARGO_TARGET_DIR
echo "== stage"
python3 tools/horizon.py stage --horizon "$HORIZON" 2>&1 | tee "$E/stage.log"
echo "== check"
python3 tools/horizon.py check --horizon "$HORIZON" 2>&1 | tee "$E/check.log"
echo "== embedded test"
python3 tools/horizon.py test --horizon "$HORIZON" 2>&1 | tee "$E/test.log"
echo "== fast build"
python3 tools/horizon.py build --horizon "$HORIZON" 2>&1 | tee "$E/build.log"
echo "== package (hmod + hnmod + native check)"
env CARGO_INCREMENTAL=0 tools/package.sh 2>&1 | tee "$E/package.log"
echo "LADDER-DONE"
