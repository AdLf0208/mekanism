# Validation

## 0.2.0 — the SDK package (current, 2026-11-18)

Validated on Linux with rustc 1.97.0 against the pinned SDK nightly
`0b14f8acfd97fa2c750e42795249dcc981178ca4` (`package_format` 6, `content_api`
"0.2", rust 1.99.1) and mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, both
recorded in `horizon-mod.toml`'s `[native]`.

Every step ran through the campaign's slot registry
(`ports/tools/build-slot.sh`, `BUILD_LOCAL=1`, never a bare `cargo`), with
`CARGO_INCREMENTAL=0`, directly in the port: the SDK package has no staging
step and no staged tree.

| Check | Command | Result |
|---|---|---|
| format | `cargo fmt --check` (never `--all`) | clean |
| runtime tests, before any glue | `cargo test --offline` | 33 passed, 0 failed |
| glue red | `cargo test --offline` (tests written, bodies `todo!()`) | 7 failed, `not yet implemented` |
| tests | `cargo test --offline` | 42 passed, 0 failed (33 `runtime::`, 9 `support::`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | 0 warnings |
| recipe tables | `python3 tools/gen_smelting.py --check`, `tools/gen_enriching.py --check` | 163 smelting rows (73 recipes); 129 enriching rows (322 recipes, 202 skipped) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | 57,808 B, exit 0 |
| reproducible | the same build again | identical bytes, sha256 `162e40e7…3cba` |
| registry check | `cargo horizon-mod check mekanism.hmod` | exit 0, `native-mod`, 26.3 |
| fits the pin | `cargo horizon-mod fits mekanism.hmod --build nightly` | `0b14f8a made for`, exit 0 |
| archive | `unzip -l mekanism.hmod` | 66 entries: 57 content files, 3 `src/`, 4 root docs, 2 manifests, `uses.json`; no `Cargo.lock` |
| `runtime.rs` | `unzip -p mekanism.hmod src/runtime.rs \| sha256sum` | `dc2dc959…b6e90`, the retired file byte for byte |
| content carry | 55 carried files byte-compared against the retired content artifact | 0 differing (2 licence copies added) |
| surface | `unzip -p mekanism.hmod uses.json` | 286 entries |

**Not run: the in-game check.** A native `.hmod` is compiled by
`horizon-launcher native-build . --trust-native-code`, and the launcher is not
installed in this environment, so there is no `0b14f8a` client for
`tools/e2e.sh`. The script is updated for that game (`GAME=`, optional
`CONTROL_GAME=`, `SELFTEST=1` kept); the missing run is a recorded gap in
[notes.md](notes.md).

Evidence: `ports/_shared/evidence/mekanism/sdk-*` (`sdk-test.log`,
`sdk-clippy.log`, `sdk-fmt.log`, `sdk-unit-red.log`, `sdk-build.log`,
`sdk-build-second.log`, `sdk-reproducible.txt`, `sdk-check.log`,
`sdk-check-json.log`, `sdk-fits.log`, `sdk-entries.txt`, `sdk-sha256.txt`,
`sdk-runtime-sha256.txt`, `sdk-uses.json`, `sdk-uses-summary.txt`).

## 0.1.0 — the retired `.hnmod` + content `.hmod` split (historical)

Validated on Linux (Rust 1.97.0) on 2026-11-08 against Horizon
`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9` and horizon-mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57` (schema 6), the revisions the retired
`compatibility.json` carried. Every build, test and game run went through the
campaign's slot registry, with `CARGO_INCREMENTAL=0` and no `--target-dir`.

| Check | Command (via the slot runner) | Result |
|---|---|---|
| format | `cargo fmt --check` | clean |
| runtime tests | `cargo test --offline` | 33 passed, 0 failed |
| runtime lints | `cargo clippy --all-targets --offline -- -D warnings` | 0 warnings |
| recipe tables | `python3 tools/gen_smelting.py --check`, `tools/gen_enriching.py --check` | 163 smelting rows (73 recipes); 129 enriching rows (322 recipes, 202 skipped) |
| stage | `python3 tools/horizon.py stage --horizon …/horizon-1d63e0a` | exit 0, 7 injections, 2 modules |
| staged check | `python3 tools/horizon.py check …` | `cargo_exit: 0`, 0 warnings |
| embedded tests | `python3 tools/horizon.py test …` | 33 passed, 0 failed, 1240 filtered out |
| fast build | `python3 tools/horizon.py build …` | `cargo_exit: 0` |
| in-game check | `build-slot.sh e2e -- … -- tools/e2e.sh` | PASS, `EXIT:0` |
| differential | `NOCABLE=1 … tools/e2e.sh` | PASS, `EXIT:0` |
| fail-capable run | `SELFTEST=1 … tools/e2e.sh` | FAIL as required (non-zero) |
| persistence | the e2e's save → quit → relaunch phase (`tools/persist-repro.sh` for the standalone repro) | FE, progress and slots read back unchanged |
| content package | `cargo-horizon-mod build` in `content/` | 24,322 B, 56 entries, content only |
| `.hmod` load check | `tools/hmod-load.sh` | the four blocks and two items register; the same ids are unknown without the package |
| native package | `python3 tools/package.py` | 19,911 B, 11 entries, pin recorded |

`tools/final-evidence.sh` ran the three e2e variants, the staged check and
test, the content build and the native package in one slot lease and wrote its
logs to `ports/_shared/evidence/mekanism/`. Those tools and those logs belong
to the retired format and are **historical**: the commands cannot be re-run as
written, and the retired tree they drove is in
`ports/_shared/evidence/mekanism/retired-hnmod-0.1.0/`.

No Minecraft resources or player data were read or changed: every game run
used a fresh `mktemp -d` scratch home and its own world, and the prepared 26.3
root read-only.
