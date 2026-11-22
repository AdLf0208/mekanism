# Validation

## 0.2.7 — the Chemical Washer, slice 8 (2026-11-22)

Validated on Linux against the campaign pin **Horizon
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, `package_format` 6,
`content_api` "0.2", rust 1.99.1) and mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`,
both recorded in `horizon-mod.toml`'s `[native]`. Every build, test and game leg ran
through `ports/tools/build-slot.sh` (one lease per step, `CARGO_INCREMENTAL=0`, no
`--target-dir`).

| Check | Command | Result |
|---|---|---|
| runtime + glue rules, RED first | `cargo test --offline` on the untouched tree | **147 errors**, the new API missing (`slice8-unit-red.log`) |
| tests | `cargo test --offline` | **108 passed**, 0 failed |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`slice8-clippy.log`) |
| format | `cargo fmt --check` (never `--all`) | clean (`slice8-clippy.log`) |
| recipe tables | the six `gen_*.py --check` (`HORIZON_RES` set) | 163 / 132 / 148 / 72 / 15 / **7** rows |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` | **179,723 B**, `cmp` identical, sha256 `38a7bca520354e06932a914fbcd70adb0f2c2f12624b74598a855bcc4a7458a4` (`slice8-build.log`, `slice8-build-second.log`, `slice8-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | exit 0, `mekanism 0.2.7 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, 233 entries (`slice8-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, exit 0 (`slice8-fits.log`, sha printed beside it) |
| fits, a different build | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, exit 0 — the run that proves something (`slice8-fits.log`) |
| archive | `unzip -l mekanism.hmod` | 233 entries, no `Cargo.lock` |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (233 entries, no packed `Cargo.lock`) |
| in game | `build-slot.sh run --step e2e -- bash tools/e2e.sh` on the launcher build | **`PASS` + `EXIT:0`** (`../_shared/evidence/mekanism-e2e/e2e-package.log`) |
| unmodded game | `GAME=$CONTROL_GAME CONTROL=0 tools/e2e.sh` | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** (`e2e-control.log`) |
| fail-capable | `SELFTEST=1 tools/e2e.sh` | **`FAIL: expected /mekanism info at 1,72,0 to say 'output empty x0', got '… output mekanism:clean_iron x1'` + `EXIT:1`** (`e2e-selftest.log`) |
| differential | `NOCABLE=1 tools/e2e.sh` | **`PASS` + `EXIT:0`** (`e2e-nocable.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |
| launcher build | `launcher-game.sh build mekanism` (in a lease) | `Compiling mekanism (1 of 1)`, `native-0a875da-46a111dc6bcd` (`slice8-launcher-build.log`) |

The in-game leg proves, per the parity rule: the fourteen blocks and thirteen items are give-able and
placeable, an unknown id is refused, the **fifteen** carried shaped recipes craft (id, count,
ingredients consumed, eight wrong-ingredient controls empty), the fourteen blocks' loot tables drop
the block, every carried crushing (148), injecting (72), purifying (15) and **washing (7)** recipe
still runs — and the new family, the port's first **fluid** machine: water arrives through the
game's own `horizon:fluids` (the block entity's saved `fluids` list reads it back, the read names
`the game` as the answering provider), the dirty slurry arrives through `mekanism:chemical`, and one
operation spends exactly **40,000 FE, 5 mB of water and one unit of dirty slurry** to make **one unit
of clean slurry** (the tank beside it 1,000 → 999, the machine's own tank empty); a clean slurry makes
nothing, an empty water tank spends no FE and draws nothing, water is refused by the chemical key, a
chemical tank answers no fluid handler, and the clean slurry drains through the chemical key's tank 1;
the washer's progress and all three tanks survive a save and relaunch. See [notes.md](notes.md)
§"Slice 8 — the Chemical Washer" for the transcript.

## 0.2.6 — the basic factory family, slice 7 (2026-11-22)

Validated on Linux against the campaign pin **Horizon
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, `package_format` 6,
`content_api` "0.2", rust 1.99.1) and mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`,
both recorded in `horizon-mod.toml`'s `[native]`. Every build, test and game leg ran
through `ports/tools/build-slot.sh` (one lease per step, `CARGO_INCREMENTAL=0`, no
`--target-dir`).

| Check | Command | Result |
|---|---|---|
| runtime + glue rules, RED first | `cargo test --offline` on the untouched tree | **71 errors**, the new API missing (`slice7-unit-red.log`; `slice7-unit-red-2.log` for the gas ask) |
| tests | `cargo test --offline` | **92 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` (never `--all`) | clean (`sdk-fmt.log`) |
| recipe tables | the five `gen_*.py --check` (`HORIZON_RES` set) | 163 / 132 / 148 / 72 / 15 rows (`slice7-recipe-tables.log`) |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` | **161,073 B**, `cmp` identical, sha256 `6633dde7616f86fb8d8a6baad3cbaa1f842d977ae500a207000345a8a8b60e67` (`slice7-build.log`, `slice7-build-second.log`, `slice7-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | exit 0, `mekanism 0.2.6 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, 220 entries (`slice7-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, exit 0 (`slice7-fits.log`, sha printed beside it) |
| fits, a different build | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, exit 0 — the run that proves something (`slice7-fits.log`) |
| archive | `unzip -l mekanism.hmod` | 220 entries, no `Cargo.lock` |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (220 entries, no packed `Cargo.lock`) |
| in game | `build-slot.sh run --step e2e -- bash tools/e2e.sh` on the launcher build | **`PASS` + `EXIT:0`** (`../_shared/evidence/mekanism-e2e/e2e-package.log`) |
| unmodded game | `GAME=$CONTROL_GAME CONTROL=0 tools/e2e.sh` | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** (`e2e-control.log`) |
| fail-capable | `SELFTEST=1 tools/e2e.sh` | **`FAIL: expected /mekanism info at 1,70,0 to say 'output empty x0', got '… output mekanism:clump_iron x9'` + `EXIT:1`** (`e2e-selftest.log`) |
| differential | `NOCABLE=1 tools/e2e.sh` | **`PASS` + `EXIT:0`** (`e2e-nocable.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |
| launcher build | `launcher-game.sh build mekanism` (in a lease) | `Compiling mekanism (1 of 1)`, `native-0a875da-c9d32f247453` (`slice7-launcher-build.log`) |

The in-game leg proves, per the parity rule: the thirteen blocks and thirteen items are give-able and
placeable, an unknown id is refused, the **fourteen** carried shaped recipes craft (id, count,
ingredients consumed, seven wrong-ingredient controls empty), the thirteen blocks' loot tables drop
the block, every carried crushing (148), injecting (72) and purifying (15) recipe still runs, and the
new family: a basic purifying factory beside a tank of 4,000 oxygen turns **three ore in three
processes** into **nine clumps** for **3 × 40,000 FE** and exactly **600** units — the tank at
3,400 — with the buffer at 0; **one** ore in one process costs **one** operation (3 clumps, 40,000
FE, 200 oxygen, the tank at 3,800); **40,000 FE in the 120,000 FE buffer buys exactly 200 progress
ticks** (67 + 67 + 66) and no operation, while a per-factory charge would have finished three; a
basic smelting factory turns three ore into three ingots for its 30,000 FE buffer, a diamond makes
nothing, and the factories' FE, progress array and six slots survive a save and relaunch. See
[notes.md](notes.md) §"Slice 7 — the basic factory family" for the transcript.

## 0.2.5 — the Purification Chamber, slice 6 (2026-11-21)

Validated on Linux against the campaign pin **Horizon
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, `package_format` 6,
`content_api` "0.2", rust 1.99.1) and mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`,
both recorded in `horizon-mod.toml`'s `[native]`. Every build, test and game leg ran
through `ports/tools/build-slot.sh` (one lease per step, `CARGO_INCREMENTAL=0`, no
`--target-dir`).

| Check | Command | Result |
|---|---|---|
| runtime + glue rules, RED first | `cargo test --offline` on the untouched tree | **34 errors**, the new API missing (`slice6-unit-red.log`) |
| tests | `cargo test --offline` | **78 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` (never `--all`) | clean (`sdk-fmt.log`) |
| recipe tables | the five `gen_*.py --check` | 163 / **132** / **148** / 72 / **15** rows (`slice6-recipe-tables.log`) |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` | **124,275 B**, `cmp` identical, sha256 `ac8289c802ffdba25a1b3f7a96fea5c0904b8c7a65d20ca49417dea6f9363828` (`sdk-build.log`, `sdk-build-second.log`, `sdk-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | exit 0, `mekanism 0.2.5 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, 159 entries (`sdk-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, exit 0 (`sdk-fits.log`, sha printed beside it) |
| fits, a different build | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, exit 0 — the run that proves something (`sdk-fits.log`) |
| archive | `unzip -l mekanism.hmod` | 159 entries, no `Cargo.lock` (`sdk-entries.txt`) |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (`sdk-verify-package.log`) |
| in game | `build-slot.sh run --step e2e -- bash tools/e2e.sh` on the launcher build | **`PASS` + `EXIT:0`** (`../_shared/evidence/mekanism-e2e/e2e-package.log`) |
| unmodded game | `GAME=$CONTROL_GAME CONTROL=0 tools/e2e.sh` | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** (`e2e-control.log`) |
| fail-capable | `SELFTEST=1 tools/e2e.sh` | **`FAIL: expected /mekanism info at 1,66,16 to say 'output empty x0', got '… output mekanism:clump_iron x3'` + `EXIT:1`** (`e2e-selftest.log`) |
| differential | `NOCABLE=1 tools/e2e.sh` | **`PASS` + `EXIT:0`** (`e2e-nocable.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |
| launcher build | `launcher-game.sh build mekanism` (in a lease) | `Compiling mekanism (1 of 1)`, `native-0a875da-6e2fafb2fa07` (`slice6-launcher-build.log`) |

The in-game leg proves, per the parity rule: the eight blocks and thirteen items are give-able and
placeable, an unknown id is refused, the **nine** carried shaped recipes craft (id, count,
ingredients consumed, six wrong-ingredient controls empty), the eight blocks' loot tables drop the
block, every carried crushing recipe (148), every carried injecting recipe (72) and every carried
purifying recipe (15) run in a real machine, the machines' chain and persistence hold, the chemical
capability moves and refuses as before — and **its second consumer**: a Purification Chamber beside a
tank of oxygen pulls the gas **through `mekanism:chemical`**, turns one iron ore into the jar's
three `mekanism:clump_iron` for exactly 40,000 FE (200 × 200), and leaves the neighbour tank at
4,000 − 200 with its own tank empty; with no tank beside it, and with a tank of the injection
chamber's hydrogen chloride, it makes no progress, spends no FE and draws nothing; and the jar's
chain runs end to end (clump → crusher → dirty dust → enriching chamber → dust, while a purification
chamber has no row for a dirty dust). See [notes.md](notes.md) §"Slice 6 — the Purification Chamber"
for the transcript.

## 0.2.4 — the Chemical Injection Chamber, slice 5 (2026-11-21)

Validated on Linux against the campaign pin **Horizon
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, `package_format` 6,
`content_api` "0.2", rust 1.99.1) and mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`,
both recorded in `horizon-mod.toml`'s `[native]`. Every build, test and game leg ran
through `ports/tools/build-slot.sh` (one lease per step, `CARGO_INCREMENTAL=0`, no
`--target-dir`).

| Check | Command | Result |
|---|---|---|
| runtime + glue rules, RED first | `cargo test --offline` on the untouched tree | **`error[E0432]`**, the new API missing (`slice5-unit-red.log`) |
| tests | `cargo test --offline` | **72 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` (never `--all`) | clean (`sdk-fmt.log`) |
| recipe tables | `gen_smelting.py --check`, `gen_enriching.py --check`, `gen_crushing.py --check`, `gen_injecting.py --check` | 163 / 129 / 145 / **72** rows (`sdk-recipe-tables.log`) |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` | **105,226 B**, `cmp` identical, sha256 `dc37efbcea33fc5e7c0feb26fa2a126f947b107aa566dd90dc9cd91df49dac6c` (`sdk-build.log`, `sdk-build-second.log`, `sdk-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | exit 0, `mekanism 0.2.4 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, 122 entries (`sdk-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, exit 0 (`sdk-fits.log`, sha printed beside it) |
| fits, a different build | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, exit 0 — the run that proves something (`sdk-fits.log`) |
| archive | `unzip -l mekanism.hmod` | 122 entries, no `Cargo.lock` (`sdk-entries.txt`) |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (`sdk-verify-package.log`) |
| in game | `build-slot.sh run --step e2e -- bash tools/e2e.sh` on the launcher build | **`PASS` + `EXIT:0`** (`../_shared/evidence/mekanism-e2e/e2e-package.log`) |
| unmodded game | `GAME=$CONTROL_GAME CONTROL=0 tools/e2e.sh` | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** (`e2e-control.log`) |
| fail-capable | `SELFTEST=1 tools/e2e.sh` | **`FAIL: expected /mekanism info at 1,66,12 to say 'output empty x0', got '… output mekanism:shard_iron x8'` + `EXIT:1`** (`e2e-selftest.log`) |
| differential | `NOCABLE=1 tools/e2e.sh` | **`PASS` + `EXIT:0`** (`e2e-nocable.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |
| launcher build | `launcher-game.sh build mekanism` (in a lease) | `Compiling mekanism (1 of 1)`, `native-0a875da-8480d1934f65` (`slice5-launcher-build.log`) |

The in-game leg proves, per the parity rule: the seven blocks and seven items are give-able and
placeable, an unknown id is refused, the **eight** carried shaped recipes craft (id, count,
ingredients consumed, five wrong-ingredient controls empty), the seven blocks' loot tables drop the
block, every carried crushing recipe (145) runs in a real crusher, the machines' chain and
persistence hold, the chemical capability moves and refuses as before — and **its consumer**: a
Chemical Injection Chamber beside a tank of hydrogen chloride pulls the gas **through
`mekanism:chemical`**, turns three raw iron into the jar's eight `mekanism:shard_iron` for exactly
80,000 FE, and leaves the neighbour tank at 4,000 − 200 with its own tank empty; with no tank beside
it, with a tank of oxygen, and with a tank of hydrogen (a gas the raw-iron recipe does not take) it
makes no progress, spends no FE and draws nothing — and **all 72 carried injecting rows** run in a
real chamber with their declared input count, output stack and gas cost. See [notes.md](notes.md)
§"Slice 5 — the Chemical Injection Chamber" for the transcript.

## 0.2.3 — the chemical capability, slice 4 (2026-11-21)

Validated on Linux against the campaign pin **Horizon
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, `package_format` 6,
`content_api` "0.2", rust 1.99.1) and mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`,
both recorded in `horizon-mod.toml`'s `[native]`. Every build, test and game leg ran
through `ports/tools/build-slot.sh` (one lease per step, `CARGO_INCREMENTAL=0`, no
`--target-dir`).

| Check | Command | Result |
|---|---|---|
| runtime rules, RED first | `cargo test --offline chemical_tests` on `todo!()` bodies | **6 failed**, `not yet implemented` (`slice4-unit-red.log`) |
| glue saved shape, RED first | `cargo test --offline the_saved_chemical` on `todo!()` bodies | **1 failed**, `not yet implemented` (`slice4-glue-red.log`) |
| tests | `cargo test --offline` | **63 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` (never `--all`) | clean (`sdk-fmt.log`) |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` | **84,180 B**, `cmp` identical, sha256 `5e304608de3b1094a2b9ce62b5fe9245299cc0587e4bacc44201b303cf8fd989` (`sdk-build.log`, `sdk-build-second.log`, `sdk-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | exit 0, `mekanism 0.2.3 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, 94 entries (`sdk-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, exit 0 (`sdk-fits.log`, sha printed beside it) |
| fits, a different build | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, exit 0 — the run that proves something (`sdk-fits.log`) |
| archive | `unzip -l mekanism.hmod` | 94 entries, no `Cargo.lock` |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (`sdk-verify-package.log`) |
| in game | `build-slot.sh run --step e2e -- bash tools/e2e.sh` on the launcher build | **`PASS` + `EXIT:0`** (`../_shared/evidence/mekanism-e2e/e2e-package.log`) |
| unmodded game | `GAME=$CONTROL_GAME CONTROL=0 tools/e2e.sh` | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** (`e2e-control.log`) |
| fail-capable | `SELFTEST=1 tools/e2e.sh` | **`FAIL: expected /mekanism chemical insert 4000 mekanism:hydrogen … to say 'inserted 0 …'` + `EXIT:1`** (`e2e-selftest.log`) |
| differential | `NOCABLE=1 tools/e2e.sh` | **`PASS` + `EXIT:0`** (`e2e-nocable.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |
| launcher build | `launcher-game.sh build mekanism` (in a lease) | `Compiling mekanism (1 of 1)`, `native-0a875da-4d3c54971561` (`slice4-launcher-build.log`) |

The in-game leg proves, per the parity rule: the six blocks and four items are give-able and
placeable, an unknown id is refused, the **seven** carried shaped recipes craft (id, count,
ingredients consumed, four wrong-ingredient controls empty), the six blocks' loot tables drop
the block, every carried crushing recipe runs in a real crusher, the machines' chain and
persistence hold — and **the chemical capability**: `mekanism:chemical` is among the game's
published block capabilities, an insert through the key lands in the tank's saved data
(`chemical`/`amount` read back exactly), part of it drains into the gauge dropper through the
item key (buffer delta **and** the stack's `mekanism:chemical` component), an unknown chemical
id is refused **by name** and moves nothing, and the same insert against a crusher (a block
entity that answers no chemical) moves nothing. See [notes.md](notes.md) §"Slice 4 — the
chemical capability" for the transcript.

## 0.2.2 — the Crusher, slice 3 (2026-11-21)

Validated on Linux against the campaign pin **Horizon
`0a875da91dbb0fdcb7ba8c42c11341aa229af51a`** (Nightly 2026-11-20, `package_format` 6,
`content_api` "0.2", rust 1.99.1) and mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`,
both recorded in `horizon-mod.toml`'s `[native]`. Every build, test and game leg ran
through `ports/tools/build-slot.sh` (one lease per step, `CARGO_INCREMENTAL=0`, no
`--target-dir`).

| Check | Command | Result |
|---|---|---|
| tests, RED first | `cargo test --offline` before the feature | **31 errors**, the new API missing (`slice3-unit-red.log`) |
| tests | `cargo test --offline` | **52 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` (never `--all`) | clean (`sdk-fmt.log`) |
| recipe tables | `gen_smelting.py --check`, `gen_enriching.py --check`, `gen_crushing.py --check` | 163 / 129 / **145** rows (`sdk-recipe-tables.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` twice | **68,652 B**, `cmp` identical, sha256 `d52219f7…7d38f6` (`sdk-build.log`, `sdk-build-second.log`, `sdk-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | exit 0, `native-mod`, 26.3, 82 entries (`sdk-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, exit 0 (`sdk-fits.log`) |
| fits, a different build | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, exit 0 — the run that proves something (`sdk-fits.log`) |
| archive | `unzip -l mekanism.hmod` | 82 entries, no `Cargo.lock` (`sdk-entries.txt`) |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** |
| in game | `ports/tools/launcher-game.sh run mekanism` | **`PASS` + `EXIT:0`** (`../_shared/evidence/mekanism-e2e/e2e-package.log`) |
| unmodded game | `GAME=$CONTROL_GAME tools/e2e.sh` | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** (`e2e-control.log`) |
| fail-capable | `SELFTEST=1 tools/e2e.sh` | **`FAIL: … 'output empty x0', got '… output minecraft:gravel x1'` + `EXIT:1`** (`e2e-selftest.log`) |
| differential | `NOCABLE=1 tools/e2e.sh` | **`PASS` + `EXIT:0`** (`e2e-nocable.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |

The in-game leg proves, per the parity rule: the five carried shaped recipes craft (id,
count, ingredients consumed, two wrong-ingredient controls empty), the five blocks' loot
tables drop the block, every carried block and item is give-able and placeable, an
unknown id is refused, the crusher runs one operation for exactly 10,000 FE and does
nothing without energy or without a recipe, its menu opens, and **all 145 carried
crushing rows** run in a real crusher with their declared input count and output stack.
See [notes.md](notes.md) §"Slice 3 — the Crusher" for the transcript.

## 0.2.0 — the SDK package (2026-11-18, historical)

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
