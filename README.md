# Mekanism for Horizon — slices 1–11

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
- `mekanism:crusher` — 50 FE/t, 200 ticks per operation (10,000 FE), 20,000 FE
  buffer and the same three-slot menu; the jar's 145 `mekanism:crushing` rows
  this port can finish (cobblestone → gravel, gravel → sand, wool → string,
  ingots → dusts, the stone/deepslate/tuff/copper conversions and more);
- `mekanism:chemical_injection_chamber` — 400 FE/t, 200 ticks per operation
  (80,000 FE), a 160,000 FE buffer and a **210-unit gas tank of its own** that
  it tops up from a neighbouring tank **through `mekanism:chemical`**; the jar's
  79 `mekanism:injecting` rows this port can finish (ore and raw ore → shards,
  **every crystal the crystallizer makes → its material's shard**, copper
  oxidising, concrete solidifying, coral reviving, dirt → mud, terracotta →
  clay), with the seven `mekanism:shard_{iron,gold,copper,lead,osmium,tin,uranium}`
  items;
- `mekanism:chemical_washer` — **the port's first fluid machine**: 200 FE/t,
  200 ticks per operation (40,000 FE), an 80,000 FE buffer, a 10,000 mB **water
  tank** declared through the game's own `horizon:fluids` capability (its
  contents in the block entity's `fluids` list) and two 10,000-unit slurry tanks
  through `mekanism:chemical` — dirty in, clean out — over the jar's seven
  `mekanism:washing` rows (5 mB of water + 1 unit of a dirty slurry → 1 unit of
  its clean slurry, for copper, gold, iron, lead, osmium, tin and uranium);
- `mekanism:chemical_crystallizer` — **the washer's consumer**: 400 FE/t, 200
  ticks per operation (80,000 FE), a 160,000 FE buffer (exactly two operations)
  and a 10,000-unit tank that takes a clean slurry with a row and nothing else
  (upstream's `containsRecipe` predicate), over **all seven** `mekanism:crystallizing`
  slurries the washer can make — **200 units of a clean slurry make one**
  `mekanism:crystal_{copper,gold,iron,lead,osmium,tin,uranium}`, and a hopper
  pulls the crystals out;
- **the five basic factories** — `mekanism:basic_{smelting,enriching,crushing,injecting,purifying}_factory`
  — each running three processes of its recipe side by side over one buffer and,
  for the two gas factories, one shared tank (half the base machine's storage per
  process);
- `mekanism:basic_energy_cube` — 4,000,000 FE at 4,000 FE/t;
- `mekanism:basic_universal_cable` — moves 8,000 FE/t between the block
  entities beside it, on its own tick;
- **the chemical capability** (`mekanism:chemical`, the block side, and
  `mekanism:item_chemical`, the item side), the surface this pack's own
  integrations ask for and no Horizon build publishes:
  - `mekanism:basic_chemical_tank` — one tank of 64,000 units of one chemical,
    kept as `chemical` + `amount` in the block entity's saved data and answered
    by a capability provider;
  - `mekanism:gauge_dropper` — carries 16,000 units of one chemical in its
    `mekanism:chemical` component (`{id, amount}`), moving 250 a call, through
    the item side of the same capability.

  The port's `src/chemical.rs` publishes the key, the item key and the
  `Chemical` type; the id table is Mekanism's own (`runtime::CHEMICALS`, 32
  named chemicals), and an id outside it is refused by name.

Read [notes.md](notes.md) for the slice boundary, the deviations, the gap list
and the migration to the SDK format, and [FINDINGS.md](FINDINGS.md) for the
upstream analysis (the numbers and the files read from tag
`v1.21.1-10.7.19.85`).

The recipe tables are generated into `src/runtime.rs` by
`tools/gen_smelting.py` (the 26.3 vanilla smelting recipes),
`tools/gen_enriching.py`, `tools/gen_crushing.py`, `tools/gen_injecting.py`,
`tools/gen_purifying.py`, `tools/gen_washing.py` and `tools/gen_crystallizing.py`
(the jar's `mekanism:*` recipes over the items and chemicals the port registers);
all seven have `--check`, and `--rows` feeds the in-game sweeps
(`tools/crush_sweep.py`, `tools/inject_sweep.py`, `tools/gas_sweep.py`,
`tools/wash_sweep.py`, `tools/crystal_sweep.py`) so the scenario and the table
cannot drift apart. `tools/carry_crystallizer.py --check` and
`tools/carry_shards.py --check` re-derive the crystallizer's and the seven
shards' carried art from the jar.

```sh
# the pure rules and the glue's declarations, on the host target
cargo test --offline
cargo clippy --all-targets --offline -- -D warnings
cargo fmt --check                 # NOT `--all`: that would check the fetched SDK

# the chemical capability is also exercised in game: `/mekanism chemical list|read|insert|drain|dropper`

# the package: `cargo horizon-mod` fetches .horizon/sdk/ (gitignored) on first use
rm -f Cargo.lock                  # a packed lock changes the artifact's sha256
cargo horizon-mod build           # -> target/horizon-mod/mekanism.hmod
cp target/horizon-mod/mekanism.hmod ./mekanism.hmod
cargo horizon-mod check mekanism.hmod
cargo horizon-mod fits mekanism.hmod --build 0a875da   # the pin: `made for`
cargo horizon-mod fits mekanism.hmod --build 0b14f8a   # a different build: `fits`
```

`cargo horizon-mod build` **packs**; it does not compile the game crate. The
native compile is a launcher step (`horizon-launcher native-build .`), so the
in-game check needs a launcher-built game:

```sh
ports/tools/launcher-game.sh run mekanism      # builds the game and runs the leg
```

Run every build and every game leg through the campaign's slot registry
(`../tools/build-slot.sh`), never a bare `cargo` in a port. Requirements: Linux,
a Rust toolchain (the pinned SDK is rust 1.99.1; this crate also builds on
1.97.0), Python 3.12+, and `jq` for the e2e.
