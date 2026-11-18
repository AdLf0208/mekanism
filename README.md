# Mekanism for Horizon — slices 1–2

One flat **SDK package** (`horizon-mod.toml`, `api = "0.2"`) for Mekanism
10.7.19.85 (MIT), the exact ATM10 jar. The carried blocks, items, models,
textures, blockstates, recipes, loot, tag and lang are the `assets/` and `data/`
here; the behaviour is `src/lib.rs` (one `impl mod_api::Hooks`) and
`src/support.rs` (the Parts and the command bodies), with the rules in
`src/runtime.rs`, which also builds and tests standalone.

- `mekanism:energized_smelter` — 50 FE/t, 200 ticks per operation (10,000 FE),
  20,000 FE buffer, three slots and the vanilla furnace menu;
- `mekanism:enrichment_chamber` — 50 FE/t, 200 ticks per operation (10,000 FE),
  20,000 FE buffer, the same three-slot menu; ore → two dust over the jar's
  `mekanism:enriching` recipes (raw ore 3 → 4, raw block 1 → 12), and
  `mekanism:dust_{iron,gold,copper}` for the smelter's dust → ingot recipes;
- `mekanism:basic_energy_cube` — 4,000,000 FE at 4,000 FE/t;
- `mekanism:basic_universal_cable` — moves 8,000 FE/t between the block
  entities beside it, on its own tick.

Read [notes.md](notes.md) for the slice boundary, the deviations, the gap list
and the migration to the SDK format, and [FINDINGS.md](FINDINGS.md) for the
upstream analysis (the numbers and the files read from tag
`v1.21.1-10.7.19.85`).

The recipe tables are generated into `src/runtime.rs` by
`tools/gen_smelting.py` (the 26.3 vanilla smelting recipes) and
`tools/gen_enriching.py` (the jar's `mekanism:enriching` recipes over the items
the port registers); both have `--check`.

```sh
# the pure rules and the glue's declarations, on the host target
cargo test --offline
cargo clippy --all-targets --offline -- -D warnings
cargo fmt --check                 # NOT `--all`: that would check the fetched SDK

# the package: `cargo horizon-mod` fetches .horizon/sdk/ (gitignored) on first use
rm -f Cargo.lock                  # a packed lock changes the artifact's sha256
cargo horizon-mod build           # -> target/horizon-mod/mekanism.hmod
cp target/horizon-mod/mekanism.hmod ./mekanism.hmod
cargo horizon-mod check mekanism.hmod
cargo horizon-mod fits mekanism.hmod --build nightly
```

`cargo horizon-mod build` **packs**; it does not compile the game crate. The
native compile is a launcher step (`horizon-launcher native-build .`), so the
in-game check needs a launcher-built game:

```sh
GAME=/path/to/launcher-built/horizon-client CTL=/path/to/horizon-ctl tools/e2e.sh
```

Run every build and every game leg through the campaign's slot registry
(`../tools/build-slot.sh`), never a bare `cargo` in a port. Requirements: Linux,
a Rust toolchain (the pinned SDK is rust 1.99.1; this crate also builds on
1.97.0), Python 3.12+, and `jq` for the e2e.
