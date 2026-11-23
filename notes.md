# Mekanism → Horizon, slices 1–11 (the ore-processing machines, the cable, the chemical chain, the factories, the washer, the crystallizer and the crystal → shard leg)

> **0.2.0 (2026-11-18): migrated to the flat SDK package** (`api = "0.2"`, one
> `impl mod_api::Hooks`, `src/runtime.rs` byte-identical, content at the package
> root). The registry closed this port's former submission **#780** with the
> `.hnmod` purge, and this version supersedes it. Everything below describing
> `mixins/`, the plan, `tools/horizon.py` and the split content `.hmod` is the
> **0.1.0 history**; [`## Migration to the SDK format`](#migration-to-the-sdk-format-020-2026-11-18)
> at the end is the current shape, the helper→hook table and the new traps.

Ported from **Mekanism 10.7.19.85** (`mekanism/Mekanism`, **MIT**), the exact
ATM10 jar `Mekanism-1.21.1-10.7.19.85.jar` (Modrinth SHA-1
`b78945c40cfe7640408f3fd1e44da385a8c8b805`, byte-identical to the pack's
`modlist.json` entry; provenance and hashes in
`ports/_shared/carry-work/mekanism/SOURCE.md`). Upstream behaviour was read
from tag `v1.21.1-10.7.19.85` (commit
`a00109e4856fd38b9c5b3dd7f22ce4a59cd65a80`) — analysis only, no Java code was
translated. See `FINDINGS.md` for the files read and the numbers.

**LOCAL-ONLY** until the maintainer approves publishing (ARCHITECTURE.md §8).
MIT is licence rank 0 (permissive); the approval is still required.

Status: **slices 1–7 implemented and verified in-game** — an Energized Smelter
(50 FE/t × 200 t, 20,000 FE buffer), an Enrichment Chamber on the same machine
core (ore → two dust, raw ore 3 → 4, raw block 1 → 12, plus the jar's other
recipes over items the port registers), a Crusher on the same core (the jar's
145 `mekanism:crushing` rows this port can finish), a Chemical Injection Chamber
(400 FE/t × 200 t, 160,000 FE buffer, a 210-unit gas tank it fills from a
neighbour through `mekanism:chemical`, the jar's 79 `mekanism:injecting` rows this
port can finish, the seven crystal → shard rows included), a Basic Energy Cube
(4,000,000 FE), a Basic Universal Cable (8,000 FE/t), the chemical capability
(one 64,000-unit tank block and a chemical item) and its two consumers (the
Chemical Injection Chamber and the Purification Chamber), and the **five basic
factories** — smelting, enriching, crushing, injecting and purifying — each
running three processes of the same recipe side by side over one buffer and, for
the two gas factories, one tank. **Slice 8** added the Chemical Washer (the port's
first fluid machine) and **slice 9** the Chemical Crystallizer; their sections
are below, and **slice 10** the four crystals that complete the crystallizer's seven slurries, and
**slice 11** the crystal → shard leg that carries every one of those crystals on into the injecting
chamber's shards. Everything else in Mekanism is a planned gap (below).

## Class + tier, approach, licence

- **Class `content`, tier `deep`** (`ports/ledger.csv`, from
  `ports/_shared/source-map/atm10-sources.csv`), licence **MIT → rank 0**.
- The **slice shape is machines** (BE, energy, recipes, menus), not the mod's
  whole deep tier: entities, renderers, dimensions and holographic multiblocks
  are deferred (`horizon/docs/modpacks.md`: a machines feature needs a block
  entity, its state in NBT, a container/menu, and the energy balance in its
  own tick).
- Queue position: the #2 core base by dependency weight (9 pending ATM10 mods
  require it); at licence rank 0, core, mechanics-first, it is a legitimate
  first Mekanism slice. Slice 2 continues the **ore-processing chain** in
  dependency order: every later ore machine (crusher, purification, injection,
  factories) is the same `ItemStackToItemStackRecipe` machine shape, so the
  chamber is the smallest next step that makes the chain produce dust and the
  smelter able to finish it.

## The pin, and a correction to ARCHITECTURE.md §3

Campaign pin **Horizon `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**, mixins
**`5a64af938ec86b00d53d16a408aabc0038fb3e57`** (schema 6);
`compatibility.json` and `mixins/mekanism/horizon-mixin.toml` carry it, with
the `game.rs` hash `64813cf4…f80f`.

`ARCHITECTURE.md` §3 says "there is no FE hook yet". At this pin that is **no
longer true**: API **142** adds `native_block_entity_energy` (declare a block
entity's energy buffer — capacity, per-call insert/extract limits and the faces
each reaches; the amount is the int **`energy`** in the entity's saved
compound) plus the helpers `native_energy`, `native_energy_insert`,
`native_energy_extract` and `native_move_energy` (`docs/native-mods.md:1194`,
`crates/horizon-client/src/game/machines/native_energy.rs`). Both machines use
it, and so do the cube and the cable; the energy lives in each block entity's
NBT and the cable balances it on its own tick.

## Behaviour (slice 1)

| Machine | Upstream | This port |
|---|---|---|
| **Energized Smelter** | 50 FE/t, `BASE_TICKS_REQUIRED` 200 (10,000 FE per operation), buffer 20,000 FE, consumes one input item when the operation finishes, result is a stack | same constants (`SMELTER_OPERATION`, `SMELTER_CAPACITY`, `runtime::process_tick`); the input is consumed at the finish, not the start |
| progress rules | no energy → **held**; no/wrong input → **reset**; output full → held, no energy spent | same, as `IdleReason::{NoEnergy, NoInput, NoRecipe, NotEnoughInput, NoOutputSpace}` (unit-tested) |
| recipes | `mekanism:smelting`, and `MekanismRecipeType` adds **every `minecraft:smelting` recipe** to it | the port carries the 26.3 vanilla smelting table (73 recipe files expanded over item tags → 163 input→output rows), generated by `tools/gen_smelting.py` into `runtime.rs`, **plus** the jar's own three dust → ingot `minecraft:smelting` recipes (`MEKANISM_ROWS`) |
| slots / menu | input, output, energy slot (a battery item); Mekanism's own GUI | three slots at `0/1/2` (input 0, unused 1, output 2) shown as the vanilla **furnace menu**: its flame bar draws the stored energy fraction and its arrow the operation progress (`data = [energy·200/capacity, 200, progress, 200]`) |
| **Basic Energy Cube** | 4,000,000 FE, 4,000 FE/t (`EnergyCubeTier.BASIC`), no tick | same, through `native_block_entity_energy`; one call moves at most 4,000 FE (the tier rate; upstream's external automation is unlimited — a recorded deviation) |
| **Basic Universal Cable** | a buffered transmitter: an 8,000 FE buffer (`CableTier.BASIC`) that pulls from PULL sides and emits to acceptors each tick; cables share one network buffer | one 8,000 FE buffer per cable; on its own tick it pulls from every adjacent provider and pushes into every adjacent acceptor (`runtime::cable_plan`, unit-tested for the caps and conservation). A line relays hop by hop instead of as one shared buffer — a recorded deviation |

## Behaviour (slice 2): Enrichment Chamber

| | Upstream | This port |
|---|---|---|
| machine | `TileEntityEnrichmentChamber` on `TileEntityElectricMachine`/`TileEntityProgressMachine`/`TileEntityRecipeMachine` | the same shared core as the smelter (`runtime::process_tick`), differing only in the operation and the recipe table |
| energy | `UsageConfig.enrichmentChamber` **50 FE/t**, `StorageConfig.enrichmentChamber` **20,000 FE**, insert-only capability | `CHAMBER_USAGE`/`CHAMBER_CAPACITY`, same `native_block_entity_energy` buffer (all faces insert, none extract) |
| speed | `BASE_TICKS_REQUIRED` **200** ticks, so **10,000 FE per operation** | `CHAMBER_OPERATION = { ticks: 200, usage: 50 }` |
| recipe type | `mekanism:enriching` (`ItemStackToItemStackRecipe`); unlike `SMELTING`, no vanilla recipes are added | `runtime::enriching` table, generated by `tools/gen_enriching.py` from the jar's 322 `mekanism:enriching` recipes, kept where both sides exist in this port: **129 rows** |
| input count | `InputHelper.use` shrinks by the ingredient's count × operations; too few items is `NOT_ENOUGH_INPUT`, which **resets** the progress | `Recipe::input` (1, or 3 for raw ore); `IdleReason::NotEnoughInput` resets, unit-tested |
| ore → dust | `#c:ores/<material>` ×1 → **2** dust (processed/iron, gold, copper among the registered items) | `minecraft:iron_ore`/`deepslate_iron_ore` → 2 × `mekanism:dust_iron`, same for gold and copper |
| raw forms | `#c:raw_materials/<material>` ×**3** → **4** dust; `#c:storage_blocks/raw_<material>` ×1 → **12** dust | same three rows per material |
| other vanilla-item recipes | `#c:ores/coal` → 2 coal, diamond/emerald → 2, lapis/redstone → 12, quartz → 2, ancient debris → 2 scrap, `#c:gravels` → flint, `#c:sands` → gravel, `#c:gunpowders` → flint, and the jar's stone/tuff/deepslate/dye conversions | all carried (they are real enriching recipes whose items the port has) |
| dropped | recipes whose tag members or outputs the port does not register (`mekanism:dust_osmium`, `enriched_*`, `salt`, dyes from modded flowers, AE2/BoP/BWG compat) | skipped by the generator, which prints them; recorded as a gap |
| slots / menu | input/output/energy slots, Mekanism's own GUI | the same three-slot vanilla furnace menu as the smelter |
| block | `BlockFactoryMachine`, hardness 3.5 / resistance 16, `facing` × `active` states, the `mekanism:block/machine` model with the chamber's textures | barrel template (hardness 3.5 / resistance 16, stone sounds, requires a tool), the same `machine` model and the chamber's front/side/back/top/bottom textures; `open` replaces `active` and is never set, so the idle front shows — the smelter's recorded deviation |

### Hooks

| Hook (API) | Helper | What it does |
|---|---|---|
| `block_entity` (15/129) | `block_entity.rs` | Gives the four blocks a block entity. |
| `block_entity_ticks` (15) | `ticks.rs` | Both machines and the cable tick; the cube is storage only. |
| `block_entity_container` (16/17) | `container.rs` | Both machines' 3 slots (`minecraft:furnace`), hoppers fill slot 0 and take slot 2. |
| `block_entity_energy` (142) | `energy.rs` | The four buffers (`energy` in the saved compound). |
| `block_entity_tick` (17) | `tick.rs` | One machine tick (smelt or enrich) / move energy one tick. |
| `register_commands` (8) | `register.rs` | `/mekanism info\|energy\|progress`. |
| `perform_command` (8) | `perform.rs` | The command bodies. |

Harness addition (recorded, like `/enderstorage info`):
`/mekanism info <x> <y> <z>` prints a machine's FE, progress and slots;
`/mekanism energy <amount> <x> <y> <z>` (op 2) sets its FE;
`/mekanism progress <x> <y> <z>` prints a machine's progress. They are the
controls the in-game check uses; the machines work without them.

## Persistence: the reported slice-1 loss does not reproduce

Slice 1 recorded "one save + relaunch run lost the BE compound (energy/progress
reset)" and removed its persistence phase rather than leave a false claim. This
slice investigated it first (`systematic-debugging`) and **could not reproduce
it**; the phase is back, and it now passes.

Evidence (`tools/persist-repro.sh`, shipped so it can be re-run): a cube and a
smelter in The End, `/mekanism energy` values set, `data get block` read live,
save, quit (waiting for `session.lock` to go), relaunch the same world, read
again. Five runs, one variable each:

| Run | Save path | Cube FE | Smelter |
|---|---|---|---|
| 1 | `save` | 6666 → 6666 | progress 0, 2 ore + 2 ingots → same |
| 2 | `save-all` | 6666 → 6666 | same |
| 3 | none (quit's closing save) | 6666 → 6666 | same |
| 4 | `save` | 6666 → 6666 | same |
| 5 | `save`, after teleporting 3,000 blocks away and back (chunk unload + reload) | 6666 → 6666 | same |

(`data get block` also shows the live compound: `{..., energy: 6666}` and
`{..., energy: 0, progress: 0, Items: [...]}`, identical before and after.)

The code cannot lose it on any mutating path, which is why there is no fix to
make:

- every machine tick writes `energy`/`progress`/slots into the entity it is
  handed, and `put_back_mod_block_entity` marks the chunk modified
  (`native-hooks` at the pin: `crates/horizon-client/src/game/native_hooks.rs:11976-11989`);
- `/mekanism energy` writes through `native_with_mod_block_entity`, which
  marks the chunk modified (`native_hooks.rs:10668-10688`);
- a cable's cross-block moves write through `native_energy_store`, which marks
  it (`game/machines/native_energy.rs:81-90`);
- the port's own `support.rs` `mark_modified` (line 99) covers both machines'
  and the cable's ticks, and its only excluded outcomes (`NoEnergy`,
  `NoOutputSpace`) change nothing;
- saving encodes the compound with the entity (`save.rs:645-654`) and loading
  decodes it (`save.rs:923-933`); `save_level` writes every modified chunk that
  is loaded (`game.rs:16678-16684`) and the closing save does the same per
  stage (`game/saving.rs:148-171`).

So the earlier observation was either a harness mistake (a read at another
machine's position, or a relaunch racing the old process' save) or a
different port version; it is not reproducible here and no code path in the
port can drop the compound. The e2e now asserts it, so a regression would be
caught: after the whole scenario a smelter is left at **0 FE / progress
60/200 / 1 ore** (3,000 FE = exactly 60 ticks at 50 FE/t), a chamber at
**4,321 FE** and an uncabled cube at **6,666 FE**; `save`, quit, relaunch reads
all three back unchanged.

## Content (`.hmod`)

Carried from the jar: the two machines' plain model chain
(`mekanism:block/machine` + one model each), the smelter's six textures, the
chamber's seven plus its `front_active` texture (carried, unused: the idle
front shows, as the smelter does), the cube's and the cable's textures, the
three dusts' item models and textures, the upstream lang keys for the four
blocks and three items, and the four crafting recipes as they are (their
Mekanism ingredients do not exist yet, so they are not craftable — a
documented gap; the e2e gives the items).

Patched where the carry cannot work: upstream's cube and cable use **custom
model loaders** (`mekanism:energy_cube`, `mekanism:transmitter` with an OBJ),
which Horizon cannot load. The cube is a plain `block/cube_all` over
`mekanism:block/models/energy_cube`; the cable is a plain central post
(`[6,0,6]..[10,16,10]`) over its own texture, with matching `shapes` and
`collision_shapes`. Both machines' blockstates cover the barrel template's
`facing`×`open` (12 variants, one model), and the block declarations use the
upstream numbers (smelter and chamber 3.5/16, cube 2/2.4, cable 1/6; all
`requires_tool`, stone sounds). Loot tables are the 26.3 self-drop shape;
`data/minecraft/tags/block/mineable/pickaxe.json` lists the four.

The three dusts are plain items on the `minecraft:gunpowder` template (stack
64) with the jar's `item/generated` models and textures.

## Gaps / deviations

1. **The rest of Mekanism.** Ore processing beyond smelting, enriching,
   crushing and injection (Purification and the Ore Washer — both need a second
   gas/fluid input — plus factories and the rest of the chain from the shards
   onward: the shards → clumps → dirty dust → dust steps need the purification
   chamber and the crusher on this port's items), factories, the
   advanced/elite/ultimate tiers, the creative cube, upgrades, side
   configuration, redstone control, ejectors, the Configurator, the gas system,
   the Metallurgic Infuser, thermal distillation, the Digital Miner, the
   Teleporter, jetpacks/armour and the multiblocks are **not ported**.
   `FINDINGS.md` lists the next slices in dependency order.
2. **The enrichment table is the jar's recipes over the items this port
   registers**: 129 of 322 rows. Osmium/tin/lead/uranium dusts and their ores,
   the `enriched_*` items, `salt`, `hdpe_sheet`, dye-from-flower recipes and
   the AE2/Biomes O' Plenty/Biomes We've Gone compat recipes are dropped
   because one side does not exist yet (`tools/gen_enriching.py` prints every
   skipped file). Adding a modded ore later means adding its tag members to
   the generator's `TAGS`.
3. **No pipes or hoppers in the check**: the chamber's two dust are read out of
   its menu and given to the smelter's input with `item replace`, standing in
   for the transport a later slice (or a hopper — the containers already
   declare their input/output faces) would provide.
4. **The machines' GUIs** are the vanilla furnace menu (3 slots, flame and
   arrow bars); Mekanism's own machine screen, its energy gauge and its
   side-config buttons are not drawn.
5. **The cube and the cable models** are plain replacements for upstream's
   custom model loaders; the cable does not draw connecting arms and the cube
   does not draw its port panels/LEDs.
6. **Recipes are not craftable**: the carried recipes need
   `mekanism:steel_casing`, `mekanism:alloys/basic`, `c:circuits/basic`,
   `c:ingots/steel`, `mekanism:energy_tablet`, `c:dusts/redstone` and
   `c:ingots/iron`, which the port does not register. The machines are given
   by command in the check.
7. **One cable call moves at most 4,000 FE per side for the cube** (the tier
   rate), while upstream's external automation is unlimited; the cable itself
   is capped at its 8,000 FE buffer. A cable line relays hop by hop instead of
   through one shared network buffer.
8. **No energy item**: the machines' energy slot is unused and there is no
   energy tablet (`native_item_energy_buffer`, API 307, would be the hook).
9. **Breaking a machine** drops the block item only; upstream copies the
   machine's energy and contents into the item (`mekanism:energy`, `:items`
   components), which the port does not.
10. **No `active` block state** on either machine (upstream swaps the front
    texture), so both always show the idle front.
11. **No config file**: the upstream usage/storage values are the compiled
    defaults (50 FE/t and 20,000 FE for each machine; 4,000,000/4,000 for the
    cube; 8,000 for the cable).

Skills: superpowers/using-superpowers,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

## Verification

Pin **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`**, mixins
**`5a64af938ec86b00d53d16a408aabc0038fb3e57`** (schema 6). Every build, test
and e2e ran through `ports/tools/build-slot.sh` (never a bare `cargo`,
`CARGO_INCREMENTAL=0`, no `--target-dir`), with
`--horizon ports/_shared/pins/horizon-1d63e0a`; the e2e used the pin-matched
shared `horizon-ctl` (`ports/_shared/bin/horizon-1d63e0a5/`). The RED run
(`unit-red.log`) is the new tests before the feature existed (47 errors).

### Ladder

| Step | Command (through the slot runner) | Result |
|---|---|---|
| runtime tests (RED) | `cargo test --offline` before the chamber existed | **47 errors**, the new tests' API missing (`unit-red.log`) |
| runtime tests | `cargo test --offline` in the port | **33 passed**, 0 failed |
| runtime lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, **0 warnings** |
| format | `cargo fmt --check` | clean |
| recipe tables | `tools/gen_smelting.py --check`, `tools/gen_enriching.py --check` | 163 smelting rows; **129 enriching rows** from 322 jar recipes (202 skipped) |
| stage | `tools/horizon.py stage` | exit 0, 7 injections, 2 modules |
| check | `tools/horizon.py check` | `cargo_exit: 0`, **0 warnings** (`target/horizon-check.log`) |
| embedded test | `tools/horizon.py test` | `test result: ok. **33 passed**; 0 failed; 1240 filtered out`, `cargo_exit: 0` |
| fast build | `tools/horizon.py build` | `cargo_exit: 0` (game binary in the slot) |
| e2e | `build-slot.sh e2e -- … -- tools/e2e.sh` | **PASS, `EXIT:0`** |
| differential | `NOCABLE=1 tools/e2e.sh` | **PASS, `EXIT:0`** (no energy reaches either machine) |
| fail-capable run | `SELFTEST=1 tools/e2e.sh` | **FAIL as required** (non-zero) |

All of the above (the three e2e variants, the staged check and test on the final
tree, the content build and the native package) also ran in a single slot lease
through `tools/final-evidence.sh`; every line below comes from that final run.

### The in-game check (evidence `ports/_shared/evidence/mekanism/e2e-default.log`)

The End arena, y=64: smelter `0,64,0` + cable `1,64,0` + cube `2,64,0`; chamber
`0,64,12` + cable `1,64,12` + cube `2,64,12`; a smelter with no energy input
(`0,64,4`), a cube-and-smelter pair with no cable (`0/2,64,8`), a chamber with
no cable (`0,64,16` + cube `2,64,16`) and a chamber with an unenrichable input
(`0,64,20`). Quoted verbatim:

```
== the machines place and their block entities answer ==
  info(0,64,0): mekanism:energized_smelter: 0/20000 FE, progress 0/200, input empty x0, output empty x0
  info(2,64,0): mekanism:basic_energy_cube: 0/4000000 FE, progress 0/0, input empty x0, output empty x0
  info(0,64,12): mekanism:enrichment_chamber: 0/20000 FE, progress 0/200, input empty x0, output empty x0
  info(1,64,0): mekanism:basic_universal_cable: 0/8000 FE, progress 0/0, input empty x0, output empty x0
  info(1,64,12): mekanism:basic_universal_cable: 0/8000 FE, progress 0/0, input empty x0, output empty x0
== control 1: no energy, so an ore does not smelt ==
  info(0,64,0): mekanism:energized_smelter: 0/20000 FE, progress 0/200, input minecraft:iron_ore x1, output empty x0
== the cube charges, and the cable moves the energy to the smelter ==
  energy(0,64,0): 7700
  energy(2,64,0): 0
== two ore smelt into two iron ingots over two operations ==
  info(0,64,0): mekanism:energized_smelter: 5450/20000 FE, progress 91/200, input minecraft:iron_ore x2, output empty x0
  info(0,64,0): mekanism:energized_smelter: 5350/20000 FE, progress 0/200, input empty x0, output minecraft:iron_ingot x2
== control 2: an unsmeltable input makes no progress and burns nothing ==
  info(0,64,4): mekanism:energized_smelter: 10000/20000 FE, progress 0/200, input minecraft:diamond x1, output empty x0
  energy(0,64,4): 10000
== control 3: a cube beside a smelter with no cable transfers nothing, and one operation costs 10,000 FE ==
  info(0,64,8): mekanism:energized_smelter: 10000/20000 FE, progress 0/200, input empty x0, output minecraft:iron_ingot x1
  energy(0,64,8): 10000
  energy(2,64,8): 10000
== the chamber enriches one ore into two iron dust, fed by the cable ==
  info(0,64,12): mekanism:enrichment_chamber: 0/20000 FE, progress 0/200, input minecraft:iron_ore x1, output empty x0
  info(0,64,12): mekanism:enrichment_chamber: 10000/20000 FE, progress 0/200, input empty x0, output mekanism:dust_iron x2
  energy(0,64,12): 10000
== the chamber opens its three-slot menu, holding the two dust ==
== the chain: the chamber's two dust smelt into two more iron ingots ==
  info(0,64,0): mekanism:energized_smelter: 0/20000 FE, progress 0/200, input mekanism:dust_iron x2, output minecraft:iron_ingot x2
  info(0,64,0): mekanism:energized_smelter: 0/20000 FE, progress 0/200, input empty x0, output minecraft:iron_ingot x4
  energy(0,64,0): 0
== control 4: an input the chamber has no recipe for burns nothing ==
  info(0,64,20): mekanism:enrichment_chamber: 10000/20000 FE, progress 0/200, input minecraft:diamond x1, output empty x0
  energy(0,64,20): 10000
== control 5: a charged cube with no cable to the chamber transfers nothing ==
  energy(0,64,16): 0
  energy(2,64,16): 10000
== persistence: save, quit and relaunch read the block entities back ==
  before save, smelter: mekanism:energized_smelter: 0/20000 FE, progress 60/200, input minecraft:iron_ore x1, output minecraft:iron_ingot x4
  before save, chamber: mekanism:enrichment_chamber: 4321/20000 FE, progress 0/200, input empty x0, output empty x0
  before save, cube:    mekanism:basic_energy_cube: 6666/4000000 FE, progress 0/0, input empty x0, output empty x0
  after relaunch, smelter: mekanism:energized_smelter: 0/20000 FE, progress 60/200, input minecraft:iron_ore x1, output minecraft:iron_ingot x4
  info(0,64,16): mekanism:enrichment_chamber: 4321/20000 FE, progress 0/200, input empty x0, output empty x0
  info(2,64,16): mekanism:basic_energy_cube: 6666/4000000 FE, progress 0/0, input empty x0, output empty x0
  (the saved compound survives: FE, progress and slots read back)
PASS: the smelter, cube, cable and enrichment chamber work end to end - the cube's FE reaches the chamber through the cable and turns one ore into two dust, that dust smelts into two ingots, an unsmeltable input and an uncabled neighbour do nothing, both three-slot menus open, and the machines' FE, progress and slots survive a save and relaunch
EXIT:0
```

What the numbers prove:

- the cable moved all 10,000 FE from the cube into the smelter, which spent
  46 ticks × 50 FE before the reading: **7,700 = 10,000 − 2,300**, so the
  transfer **conserves energy** (the cube reads 0 and the cable keeps nothing);
- two ore became **two iron ingots** over two operations, leaving 5,350 FE;
- an unsmeltable input left the progress at 0 and the 10,000 FE untouched;
- a cube beside a smelter with **no cable** kept all 10,000 FE while that
  smelter smelted exactly one ore for exactly 10,000 FE — the differential
  shows the cable is what moves the energy, and pins the per-operation cost;
- the chamber's cube delivered its whole 20,000 FE through the second cable;
  one enriching operation cost 10,000 and left **10,000**, and the output slot
  held **`mekanism:dust_iron` ×2** — the ore → 2 dust behaviour, over the
  cable, with both menus opening;
- the two dust then smelted into **two more ingots** (the output slot goes from
  2 to 4), which is the completed chain ore → dust → ingot, and the smelter
  read 0 FE afterwards: exactly two operations of 10,000 FE each;
- a diamond in the chamber made no progress and burned nothing, and a charged
  cube with no cable to its chamber transferred nothing;
- after `save`, quit and relaunch the smelter still read `0/20000 FE, progress
  60/200, input minecraft:iron_ore x1, output minecraft:iron_ingot x4`, the
  chamber `4321/20000 FE` and the cube `6666/4000000 FE` — energy, progress and
  slots all survived the round trip (the section above explains why the
  slice-1 loss did not reproduce).

### The differential without the cable (evidence `e2e-nocable.log`)

`NOCABLE=1` runs the same scenario with no cable in either row: the charged
cubes keep all **10,000 / 20,000 FE**, the smelter stays at **0 FE** and its two
ore never smelt, and the chamber stays at **0 FE** with its ore untouched
(`progress 0/200, input minecraft:iron_ore x1, output empty x0`). The
persistence phase still passes (the smelter at progress 60, the chamber 4,321,
the cube 6,666). `PASS`, `EXIT:0`.

### Fail-capable check (evidence `e2e-selftest.log`)

`SELFTEST=1` flips the smelting assertion (the output must stay empty). The
same scenario then **fails** as required:

```
FAIL: expected /mekanism info at 0,64,0 to say 'output empty x0', got
'mekanism:energized_smelter: 5350/20000 FE, progress 0/200, input empty x0,
output minecraft:iron_ingot x2'
```

Two earlier attempts also failed on real mismatches while the phases were
being made deterministic (`e2e-attempt-short-output.log`: the smelter already
held two ingots, so the chain ends at four; `e2e-attempt-residual-energy.log`:
a machine still holding energy had already started the operation when its
value was set), so the assertions bite.

### Content package

`content/` built with the pin's `cargo-horizon-mod`:
`mekanism.hmod` **24,322 B**, SHA-256
`bd59bc76fe53984536780f365119bc80a35fc9958de7fa42ab9166a86fe1017e`, **56
entries** (four blocks, three dust items, the models/textures/blockstates/lang,
the four carried recipes, four loot tables, the pickaxe tag and the creative
tab); `horizon-mod check --json` reports content only, API 0.2,
`"capabilities":["registries"]`, no code.

Load check (`tools/hmod-load.sh`, evidence `hmod-load.log`): the shared
unmodded `1d63e0a5` game with `--dev-mod mekanism.hmod` registers the four
blocks (`mekanism:energized_smelter`, `basic_energy_cube`,
`basic_universal_cable`, `enrichment_chamber`) and gives the smelter and
`mekanism:dust_iron` items; the **same world without `--dev-mod`** answers
`Unknown block type` for the smelter and the chamber, so the check cannot pass
vacuously.

### Native package

`tools/package.py`: `mekanism-0.1.0.hnmod` **19,911 B**, SHA-256
`79315d9cd0c10942db65a1576ab7461009904f0d978a0bbee169eafc0f3e6e03`, 11
entries; `horizon-native.json` pins `horizon` = the campaign pin and `mixins` =
`5a64af93…`, game 26.3, schema 1.

## Embedded content (AE2 pattern) — the port is one `.hnmod`

The registry stores **one file per project** (`AGENT_PLAYBOOK.md` §1.0): an
`.hnmod` is code-only and an `.hmod` is content, so the split port above could
not be listed faithfully. It now ships its content **inside** the bundle, the
pattern `charginggadgets` proved in game (PR #526) and `ARCHITECTURE.md` §8
records as accepted by the gate:

- **`mixins/mekanism/data/mekanism-content.hmod`** — the built
  `content/mekanism.hmod`, byte for byte (`tools/package.sh` builds and syncs
  it; `tools/package.py` refuses a copy that differs).
- **`mixins/mekanism/embedded_content.rs`** with the one `embedded-content`
  injection (HEAD, cancellable, `target = "horizon_client::game::mods::content_root"`,
  bindings `root`/`enabled`/`dev`): `native_data!` compiles the `.hmod` into the
  game, the helper writes it to `.horizon-cache/mekanism-embedded-content.hmod`
  when the copy on disk differs, appends it to the dev list and breaks out of
  `content_root` with the extended list. An installed copy of the mod still
  wins (it is found first and the id dedupes).
- **`tools/package.py`** packs `mixins/<id>/data/` into the archive
  (`docs/native-mods.md`, "Data files") and **`tools/package.sh`** rebuilds the
  `.hmod`, syncs it, writes the `.hnmod` into the port root (never `target/`)
  and runs `cargo horizon-mod check <file> --kind native-mod`.
- **`tools/ladder-embed.sh`** is the ladder (the content build must run
  **before** `stage`: the injected module's `native_data!` reads the staged
  `native-data/mekanism/`), and **`tools/e2e-embed.sh`** runs both game legs in
  one lease.

Artifacts and evidence (this pin, 2026-11-10):

| Item | Evidence | Result |
|---|---|---|
| `.hnmod` (content embedded) | `ports/mekanism/mekanism.hnmod` | **36,422 B**, SHA-256 `66ae69763229c005bf7db6643bbecce273869e8f2eca988f6de3ee57edcac0b4`, **13 entries** incl. `mixins/mekanism/data/mekanism-content.hmod` (24,322 B, `bd59bc76…1017e`) |
| native check | `package.log` (`target/ladder/check-native.log`) | `mekanism 0.1.0 (native-mod, format native-1/1d63e0a5…/5a64af93…, for 26.3)` — passes |
| ladder | `fmt.log`, `clippy.log`, `test-green.log`, `stage.log`, `check.log`, `test.log`, `build.log` | fmt clean, clippy `-D warnings` clean, 33 pure tests, stage/check **exit 0**, embedded test **33 passed**, fast build |
| e2e, one file only (`EMBEDDED=1`, no `--dev-mod`) | `e2e-embedded.log` | **PASS**, `EXIT:0`: control leg first (same binary, `.horizon-cache` unusable → `minecraft:air` at both probe positions, `Unknown block type`, while `/mekanism info` still answers `No block entity there.`), then the whole smelter/cube/cable/chamber scenario and the save+relaunch |
| embed fail-capability | `e2e-embedded-selftest.log` | `EMBEDDED=1 SELFTEST=1` → the flipped enrichment assertion fails, `EXIT:1` |

Harness facts this slice added:

- **Beware the runner's lease fd (2026-11-10).** Between ~13:00 and 14:35 every
  `python3 tools/horizon.py build` failed `cargo_exit: 101` with `could not
  parse/generate dep info at …/deps/<crate>-<hash>.d: No such file or
  directory` on five attempts, locally and on the second machine. The cause was
  campaign-wide, not this port: a closed `flock` fd made the lease a no-op, so
  jobs shared a slot and deleted each other's `stage/modded`
  (`AGENTS.md`, "Never close the lease fd in the parent shell"). After the fix
  the selftest leg was re-run and the superseded `e2e-selftest.log` (2026-11-08,
  pre-embedding) was replaced with a proper `FAIL: expected /mekanism info at
  0,64,0 to say 'output empty x0'` + `EXIT:1` verdict.
- **`native_data!` resolves against the staged `native-data/<mod>/`**, so the
  ladder must build the content and copy it into `mixins/<id>/data/` before
  `stage`; packaging after `build` alone fails `check` with
  `couldn't read …/native-data/mekanism/mekanism-content.hmod`.
- **The End's exit portal sits at 0,64,0** (bedrock) — a control that expects
  `minecraft:air` there fails for the wrong reason. The control now probes
  (8,64,8) and (10,64,8), inside the arena's cleared volume.
- **A control leg needs its own scratch home and working directory.** Sharing
  `HORIZON_HOME` and the game folder with the main run mixed the two sessions'
  `latest.log` (the main run's `/mekanism info` replies went missing) and left a
  world-lock race; the control now runs in `$SCRATCH/control` with its own
  `HORIZON_HOME`/`XDG_*`, and the closed subshell propagates its failure.
- `latest.log` is still the primary read for `/mekanism` replies, but
  `info_once` now falls back to `ctl messages` for the line, which removed the
  remaining empty-reply flake.

### Known pin/harness findings

- **Persistence**: five save/relaunch runs at this pin keep a mod block
  entity's compound (`energy`, `progress`, `Items`), including across a chunk
  unload; the slice-1 note that one run lost it did not reproduce (see the
  section above, which lists the code paths that make it impossible to lose).
- `ctl messages` reads the window client's chat, whose sync trails the server;
  the e2e reads the game's `latest.log` for `/mekanism` replies instead.
- A leftover `horizon-client` from another port's load check held a slot lock
  for 3½ hours (a leaked inherited lock fd); it was killed to free the queue.

## Reviewer repairs (2026-11-11) — PR review items and what changed


Each item below is quoted from `ports/_shared/horizon-mods/reviews/pr-*-mekanism.md` (the review is the specification), with what this repair did about it.

- **Reviewer:** [P1] the four machines cannot be crafted: recipes need unregistered items/tags, and the cube used the unimplemented `mekanism:mek_data` type
  **Repair:** Fixed in content: the four block recipes are now `minecraft:crafting_shaped` with the mod's own patterns and vanilla stand-ins (iron/redstone/glass/copper, iron block for the casing, redstone block for the tablet), so this slice can craft its own blocks without the absent steel/alloy/circuit content. Recorded in the description as a deviation.
- **Reviewer:** [P2] the cube's menu and side configuration are advertised but not implemented
  **Repair:** Corrected in the entry: the description now says the cube has **no menu and no side configuration** and that all six faces are inputs and outputs.
- **Reviewer:** licence notice
  **Repair:** `LICENSE` + `LICENSE-NOTICE.md` in the `.hnmod`, `data/mekanism/license/` in the carried `.hmod`.
- **Reviewer:** registry artifact delivery
  **Repair:** **Staging (orchestrator, not ours)**: the artifact is not in registry release/staging as `<sha256>.<ext>`; the orchestrator answers it with `horizon-mods#686` (no write access to the registry's releases). Every repackaged artifact below has a fresh sha256 for that re-push, and the author release must be re-uploaded from the port repository.

Skills: superpowers/using-superpowers,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

## Migration to the SDK format (0.2.0, 2026-11-18)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70; PR #17992 removes the old files) retired
`.hnmod`: on 2026-11-18 the registry closed **#780** (`mekanism` 0.1.0) with that single reason, and
an `.hnmod` no longer loads. **0.2.0** is the same behaviour republished as one flat SDK `.hmod`.
Pins: SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`** (`package_format` 6,
`content_api` "0.2", rust 1.99.1), mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`**, launcher 0.2.78, `cargo-horizon-mod` from
`horizon-rust/horizon@main`. The pinned SDK is cached in-tree at `../_shared/sdk/0b14f8a/` and
fetched into `.horizon/sdk/` (gitignored) by the build.

**Order done, in the brief's order:** the retired paths were moved into
`../_shared/evidence/mekanism/retired-hnmod-0.1.0/` **before** the first `cargo horizon-mod build`
(the tool refuses a root holding retired package paths); `src/runtime.rs` was copied with `cp` and
proved with `cmp` + `sha256sum` (`dc2dc959…b6e90`); `cargo test` ran green on the new crate
**before any glue existed** (33 runtime tests); the glue's tests were written first and ran **red on
`todo!()`** (`7 failed … not yet implemented`); then the glue was implemented and the whole crate went
green (41 tests).

### File map (0.1.0 → 0.2.0)

| 0.1.0 path | Now | Evidence |
|---|---|---|
| `mixins/mekanism/runtime.rs` | `src/runtime.rs`, **byte-identical** (`cp`, `cmp`, and the packaged entry's sha256) | `retired-hnmod-0.1.0/mixins/mekanism/runtime.rs` |
| `mixins/mekanism/support.rs` (480 lines, `mekanism_state`) | `src/support.rs` (the declarations, the two Parts, the command bodies) + `src/lib.rs` (the one `Hooks`) | `retired-hnmod-0.1.0/mixins/mekanism/support.rs` |
| `mixins/mekanism/horizon-mixin.toml` (`[[module]]` ×2, `[[injection]]` ×8) | no plan: `Registrar` declarations and typed `Hooks`/Part methods | `retired-hnmod-0.1.0/mixins/` |
| `mixins/mekanism/data/mekanism-content.hmod` + `embedded_content.rs` (AE2 pattern) | **gone**: the package carries `assets/` and `data/` itself | `retired-hnmod-0.1.0/mixins/` |
| `content/` (`horizon-mod.toml`, `assets/`, `data/`, `mekanism.hmod`) | the package root: `horizon-mod.toml`, `assets/`, `data/` | `retired-hnmod-0.1.0/content/`, `…/mekanism-0.1.0-content.hmod` |
| `mekanism.hnmod` (39,122 B, embedded content) | `mekanism.hmod` (57,808 B: code **and** content) | `retired-hnmod-0.1.0/mekanism.hnmod` |
| the reviewed content rebuild (25,796 B, with `data/mekanism/license/`) | re-carried: the same 57 files, licence copies included | `retired-hnmod-0.1.0/mekanism.hmod` |
| `compatibility.json`, `rust-toolchain.toml`, `horizon-native.json` | `[native]` in `horizon-mod.toml`; the pinned commit decides the toolchain | `retired-hnmod-0.1.0/` |
| `tools/horizon.py`, `package.py`, `package.sh`, `test_stage.py`, `test_template.py`, `stamp_license.py`, `hmod-load.sh`, `check-native.sh`, `ladder-embed.sh`, `e2e-embed.sh`, `selftest-embed.sh`, `final-evidence.sh`, `tools/audit/`, `.github/workflows/ci.yml`, the port `AGENTS.md` | deleted: staging, `.hnmod` packaging, the plugin-injection audit and the embedded-content drivers belong to the retired format | `retired-hnmod-0.1.0/tools/`, `…/.github/`, `…/AGENTS.md` |
| `Cargo.lock` | not kept: it is packed and changes the artifact (trap 12) | not kept |

**Kept and updated:** `tools/e2e.sh` (a launcher-built game: `GAME=`, optional `CONTROL_GAME=`,
`SELFTEST=1` kept), `tools/persist-repro.sh` (same launch shape), `tools/gen_smelting.py` and
`tools/gen_enriching.py` (now writing and checking `src/runtime.rs`), `notes.md`, `FINDINGS.md`,
`VALIDATION.md`, `README.md`, `LICENSE`, `LICENSE-NOTICE.md`.

**Content carry, diffed:** the 55 carried files under `assets/` (38) and `data/` (17) were compared
byte for byte against the retired content artifact — **0 differing**. Two files are **added**:
`data/mekanism/license/{license,notice}.txt`, byte-identical to `LICENSE` and `LICENSE-NOTICE.md`,
which is what the reviewed 0.1.0 content rebuild shipped (the reviewer's licence item). The new
package carries 57 content files, 3 `src/` files, 4 root documents, 2 manifests and `uses.json`:
66 entries.

### Helper → hook table (every former `[[injection]]` and `[[module]]`)

| 0.1.0 injection (hook, API) / module | 0.2.0 | Mapping notes |
|---|---|---|
| `[[module]] runtime` (`mekanism`) | `pub mod runtime;` in `src/lib.rs` | an ordinary module now; `pub` so its items are not `dead_code` |
| `[[module]] state` (`mekanism_state`) | `mod support;` | the glue, no `//!`-free constraint any more (that was the old planner) |
| `embedded-content` (`content_root` HEAD, cancellable) | **none** | one `.hmod` carries the content and the code; the helper existed only to smuggle a separate `.hmod` in |
| `block-entity` (`block_entity`, 15/129) | **declarations**: `registrar.block_entity_type("energized_smelter"/"enrichment_chamber"/"basic_energy_cube"/"basic_universal_cable").blocks(&[..]).build()` | the old hook answered `Compound::new()` for the four blocks; a declared type already starts with an empty compound, so no `BlockEntityPart::block_entity` is needed |
| `block-entity-ticks` (`block_entity_ticks`, 15) | `BlockEntityPart::block_entity_ticks` → `event.ticks = true` on `MachinePart` (smelter, chamber) and `CablePart` | the cube is storage only and carries no Part, so it never ticks (as 0.1.0) |
| `block-entity-container` (`block_entity_container`, 16/17) | `registrar.block_container(&block, support::container())` for the smelter and the chamber | 3 slots, every face reaches all three, inputs `[0]`, outputs `[2]`, `minecraft:furnace` — unchanged |
| `block-entity-energy` (`block_entity_energy`, 142) | `registrar.block_energy(&block, …)` ×4: `machine_battery(20_000)`, `cube_battery()`, `cable_battery()` | the amount stays the int `energy` in the saved compound; the limits and faces are unchanged |
| `block-entity-tick` (`block_entity_tick`, 17) | `BlockEntityPart::block_entity_tick` → `MachinePart` / `CablePart` | `event.pos`, `event.entity.{nbt,items,data}`, `event.reply` replace the old bindings; the machines' and the cable's own id tests are gone (a Part only hears its own type) |
| `register-commands` (`register_commands`, 8) | `Hooks::register_commands` (`event.commands`) | same three usages: `mekanism info <pos:block_pos>`, `mekanism energy <amount:integer> <pos:block_pos>` (op 2), `mekanism progress <pos:block_pos>` |
| `perform-command` (`perform_command`, 8, string words) | `Hooks::run_mod_command` (`CommandInvocation`) | typed `BlockPos`/`Integer`; `Flow::Cancel` = handled; the answer text is byte-identical to 0.1.0 |

The engine calls inside the glue moved too, and each has one successor:

| 0.1.0 call | 0.2.0 |
|---|---|
| `game.native_max_stack_size(&item)` | `host.max_stack_size(item_key)` |
| `game.native_energy(&dim, at, side)` | `level.capability(host, &ENERGY, at, Some(side))` + `EnergyHandler::{amount, capacity}` |
| `game.native_energy_extract` / `native_energy_insert` | `EnergyHandler::extract` / `::insert` inside `host.transaction(…)` |
| `game.native_with_mod_block_entity(pos, f)` | `host.block_entity(level, pos)` → `BlockEntityRef::{data, item_slot, merge_data}` |
| `game.world.modified.insert((chunk))` | the tick reply: `Changed` by default, `Unchanged` for an idle machine (no energy / no output room) |
| `entity.data[0..4]` (menu bars) | the tick payload's `data` slots, the documented field of a mod block entity |
| `NativeCommand::new` / `::op` | `Command::new` / `Command::op` |
| `game.native_feedback(Text::literal(..))` | `host.commands().feedback(Text::literal(..))` |
| `/mekanism` at a foreign block entity | `None` → "No block entity there." (0.1.0 never saw one: `native_with_mod_block_entity` filters to this mod's own entities) |

### The ladder (one build slot, `CARGO_INCREMENTAL=0`, no `--target-dir`)

`build-slot.sh` still speaks the retired `tools/horizon.py`, so it is used as the **slot guard**
(`BUILD_LOCAL=1`), with the SDK ladder run directly in the port: there is no staging step and no
staged tree.

| Step | Command | Result |
|---|---|---|
| runtime tests before any glue | `cargo test --offline` | **33 passed**, 0 failed |
| glue red | `cargo test --offline` with the glue's tests written and its bodies `todo!()` | **7 failed** (`not yet implemented`), 33 passed — `../_shared/evidence/mekanism/sdk-unit-red.log` |
| tests | `cargo test --offline` | **42 passed**, 0 failed (33 `runtime::`, 9 `support::`) — `sdk-test.log` |
| lints | `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings — `sdk-clippy.log` |
| format | `cargo fmt --check` (**not** `--all`) | clean — `sdk-fmt.log` |
| recipe tables | `gen_smelting.py --check`, `gen_enriching.py --check` | 163 smelting rows; 129 enriching rows (202 skipped) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `Built ./target/horizon-mod/mekanism.hmod` — `sdk-build.log` |
| second build | the same again | **identical bytes** — `sdk-reproducible.txt` |
| registry check | `cargo horizon-mod check mekanism.hmod` | `mekanism 0.2.0 (native-mod, format native-1/0b14f8a…/e59da90…, for 26.3)` / `57566 bytes, SHA-256 86c2d94a…bda8, unsigned`, **exit 0** — `sdk-check.log`, `sdk-check-json.log` |
| fits | `cargo horizon-mod fits mekanism.hmod --build nightly` | `0b14f8a  made for`, exit 0 — `sdk-fits.log` |
| runtime untouched | `unzip -p mekanism.hmod src/runtime.rs \| sha256sum` | `dc2dc959…b6e90` = the retired file — `sdk-runtime-sha256.txt` |
| entries | `unzip -l mekanism.hmod` | **66 files**, **no `Cargo.lock`** — `sdk-entries.txt` |
| surface | `unzip -p mekanism.hmod uses.json` | **286 entries** — `sdk-uses.json`, `sdk-uses-summary.txt` |

`uses.json` in one line: 8 hooks (`block_entity`, `block_entity_ticks`, `block_entity_tick`,
`register_commands`, `run_mod_command` and the macro's block/item/command subscriptions), 29 methods
(including `DefaultCapabilitiesRegistrar::block_container`/`block_energy`,
`EnergyHandler::{amount,capacity,insert,extract}`, `HostTransaction::transaction`,
`BlockEntityPart::block_entity_tick`, `Commands::feedback`), the `horizon:energy` capability, 63
handles, 144 types and both macros.

### Gaps after the migration

Everything in *Gaps / deviations* above still stands (the rest of Mekanism, the 129-of-322 enriching
table, no pipes, the vanilla furnace GUI, plain cube/cable models, recipes, the 4,000 FE cable call,
no energy item, no `active` state, no config). Added by this format:

1. **No 0.2.0 in-game run.** A native `.hmod` is compiled by the launcher
   (`horizon-launcher native-build . --trust-native-code`), and `horizon-launcher` is not installed
   here, so there is no `0b14f8a` client for `tools/e2e.sh`. The 0.1.0 runs (the scenario, the
   no-cable differential and the selftest) remain the behaviour's evidence for a `runtime.rs` that
   is byte-identical; what changed is the transport, not the rules.
2. **The cable measures its moves instead of planning them** (below), so `runtime::cable_plan`,
   `Endpoint`, `Move` and `Plan` are tested but no longer called by the glue. The arithmetic that
   decides each move (`energy_after_extract`, `energy_after_insert`, `clamp_energy`) is still the
   runtime's.
3. **No `/mekanism` usage text.** Brigadier parses `<pos:block_pos>` and `<amount:integer>` before the
   hook runs, so a malformed command never reaches it; the three "Usage: …" answers of 0.1.0 are
   unreachable and gone. No e2e assertion used them.
4. **No log lines.** The SDK exports no logger and a native mod may not add a crate; the retired
   bundled `log::warn!` lived in the embedded-content helper, which has no successor.
5. **The chunk is dirtied by the tick reply**, not by `game.world.modified.insert`: an idle machine
   (no energy or no output room) answers `Unchanged`, and the engine leaves the chunk and the
   containers alone. Observable only as fewer chunk saves, not as different behaviour.
6. **The two tick bodies are not unit-tested.** What they *declare* and the state they write are
   (`support::tests`), and the arithmetic under them is (`runtime::tests`), but the per-face cable
   loop and the tick's slot/menu writes need a `ServerHost`: the SDK offers a fake one only through
   `mod_api::testing`, gated on a `testing` feature this package's manifest must not add (it would
   change the artifact's Cargo.toml). Their proof is the in-game leg — which is gap 1. A future
   slice can either take that feature deliberately or drive the loop through a command.

### `## Upstream defects fixed`

None found in this migration. The 0.1.0 slice recorded deviations (the 4,000 FE cable call, the
vanilla furnace menu, the plain models), not defects.

### Behaviour that changed with the format (deliberate, and why)

- **The cable's moves are measured, not planned.** 0.1.0 asked `native_energy` for each neighbour's
  `can_insert`/`can_extract` and planned every face before executing (`cable_plan`). The SDK's energy
  capability exposes a side's amount and capacity but **not** its per-call insert/extract limits or
  whether it can extract at all, so a plan would have to assume both ways are open. That throttles a
  line: the smelter beside the cable is insert-only, and after one operation it *holds* energy, so a
  plan would put its residual charge in the pull set, take nothing, and leave the cable short of room
  it never used. Each face's pull and push is therefore executed and **measured** in one
  transaction, capped by the runtime's own arithmetic and the cable's real room or contents. Energy is
  conserved exactly as the plan proved it was; a line no longer loses throughput to a machine's
  residual charge.
- **The machines' slots are sized from the container** rather than resized defensively on every tick:
  the doc says a mod block entity's `items` holds as many slots as its container declares. The resize
  stays as a guard for an entity created before the declaration, and answering `Changed` in that tick
  is what persists it.
- **`/mekanism` filters by block id.** The old read was this mod's entities only; the new one asks the
  block first, so a chest at the position answers "No block entity there." instead of a machine at
  0 FE. Nothing in 0.1.0 could reach that path.

### Traps this migration hit (the next port must know)

1. **The retired paths must be gone before the first build.** `unexpected package path:
   mixins/mekanism/runtime.rs` — the root may hold package paths only. `mixins/`, the `.hnmod` and
   the old content `.hmod` all moved to evidence first. `tools/audit/` is refused too: it is a second
   crate (`[workspace] members`), unlike `tools/*.py`.
2. **Two artifacts shared the name `mekanism.hmod`.** The port root held the 0.1.0 content build and
   `content/` held a reviewed rebuild; the new package is the same name, so both had to move before
   the build or the deliverable would have been overwritten. Check for that before `cp`.
3. **`cargo test` builds the SDK path dependency.** `--offline` works once `.horizon/sdk/` is copied
   in (`cp -a ../framedblocks/.horizon/sdk .horizon/sdk`), otherwise the first `cargo horizon-mod`
   command fetches 21 MB.
4. **An `Infallible` transaction closure makes `let Ok(..) else` irrefutable** — clippy's
   `irrefutable_let_patterns` under `-D warnings`. Return `Result<i32, ()>` from
   `host.transaction(…)` and keep the `else`.
5. **A five-field registration tuple trips `clippy::type_complexity`.** A `MachineDecl` struct is the
   fix; the loop then reads as the table it is.
6. **`cargo fmt` would rewrite `src/runtime.rs`.** `cargo fmt --check` is the gate, and when the new
   files need formatting run `rustfmt --edition 2024 src/lib.rs src/support.rs` only, then `cmp`
   `runtime.rs` against the retired copy.
7. **A declared block entity type starts with an empty compound,** so the old `block_entity` hook
   (which returned exactly that) has no successor method: the four types are declarations, and only
   the three ticked ones carry a Part.
8. **`block_entity_ticks` dispatches by block, `block_entity_tick` by block entity kind.** One Part
   attached to the declared type answers both, which is why one Part per block is enough and why the
   old `match entity.id`/block-id tests disappear.
9. **A ticked block entity is out of the world during its own tick** — but only *itself*. The cable
   reads and writes its **neighbours** through `level.capability(…)`, which answers for them; no
   `level_tick_post` queue is needed because the cable never needs its own buffer through the
   capability (it reads `energy` from its payload) and never enumerates entities.
10. **Do not `mem::take` the tick payload.** `Tag::Compound(event.entity.nbt.clone())` and write it
    back at the end: an early return (no level, no neighbours) must not drop the machine's saved
    state.
11. **`Unchanged` discards this Part's payload edits,** so an idle tick must not rely on having
    written anything to the entity; the host's writes to neighbours keep their effects either way.
12. **A packed `Cargo.lock` changes the artifact** and pins versions newer than the SDK's own lock:
    `rm -f Cargo.lock` right before the build, `/Cargo.lock` in `.gitignore`.
13. **`gen_smelting.py`/`gen_enriching.py` wrote `mixins/mekanism/runtime.rs`.** Repointed to
    `src/runtime.rs`; their `--check` runs (163 and 129 rows) are what proves the generated table
    region still matches the jar and the 26.3 recipes — and it is also what proves the
    byte-identical move did not disturb the tables.

### Changelog (0.2.0) — supersedes #780

> **0.2.0 — republished as an SDK `.hmod`.** Horizon's mod-API merge (launcher ≥ 0.2.70) retired
> `.hnmod` and closed **#780** (`mekanism` 0.1.0) on 2026-11-18; this version supersedes it. One flat
> `mekanism.hmod` now carries the code *and* the content (the Energized Smelter, the Enrichment
> Chamber, the Basic Energy Cube, the Basic Universal Cable, the three dusts, their models, textures,
> blockstates, recipes, loot, tag and lang): `src/lib.rs` is the one `impl mod_api::Hooks`,
> `src/support.rs` the glue, and `src/runtime.rs` is byte-identical to 0.1.0, so the machines, the
> recipes, the energy arithmetic and the cable behave exactly as before. Built against SDK
> `0b14f8acfd97fa2c750e42795249dcc981178ca4` / mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`
> for 26.3. The `.hnmod`, the mixin plan, the eight per-hook helpers and the embedded-content
> injection are gone; the four blocks' containers and buffers are `Registrar` declarations, the
> machines' and the cable's ticks are Parts, and `/mekanism` is a typed `run_mod_command`. The cable
> now measures each neighbour's move instead of planning it, because the SDK's energy capability
> does not expose a side's insert/extract limits; energy is conserved as before. Package: **66
> entries, 57,808 B**, sha256
> `162e40e77430c4ec8a3692976ba50fffe039b49a8a6fa7ced400a74a0d663cba`; `check` and `fits` exit 0.
> **No 0.2.0 in-game run**: the launcher is not installed here (gap 1).

### Migration verification

| Check | Command | Result |
|---|---|---|
| Runtime tests in the new crate, before the glue | `cargo test --offline` | **33 passed**, 0 failed |
| Glue red on `todo!()` | `cargo test --offline` | **7 failed**, the glue's API missing (`not yet implemented`) |
| Whole crate | `cargo test --offline` | **42 passed**, 0 failed (`sdk-test.log`) |
| Lints | `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| Formatting | `cargo fmt --check` | clean (`sdk-fmt.log`) |
| Package | `cargo horizon-mod build` | 57,808 B, exit 0 (`sdk-build.log`) |
| Same package twice | `cargo horizon-mod build` again | identical bytes (`sdk-reproducible.txt`) |
| Registry check | `cargo horizon-mod check mekanism.hmod` | exit 0, sha256 `162e40e7…3cba` (`sdk-check.log`) |
| Fits the pinned build | `cargo horizon-mod fits … --build nightly` | `0b14f8a made for`, exit 0 (`sdk-fits.log`) |
| `runtime.rs` untouched | `unzip -p mekanism.hmod src/runtime.rs \| sha256sum` | `dc2dc959…b6e90` = the retired file (`sdk-runtime-sha256.txt`) |
| Content carry unchanged | byte-compared the 55 carried files against the retired content artifact | 0 differing; 2 licence copies added |
| Archive shape | `unzip -l mekanism.hmod` | 66 entries, no `Cargo.lock`, both licence files in (`sdk-entries.txt`) |
| Skills line | `python3 ports/tools/check-skills.py --notes ports/mekanism --strict` | PASS |

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

## Review fixes (2026-11-18)

`var/review-all/1231-mekanism.md`, the one finding fixed; version **0.2.1** (58,950 B, sha256
`2cf935e4b568d876535fafec9f0eb7c7aaf8c851ff30f424e1931b993ec76c5a`).

- **Info disclosure (src/support.rs:502, src/lib.rs:47).** `/mekanism info` read a loaded
  machine's input/output item ids and counts at player-selected coordinates, bypassing reach and
  the GUI. It now needs an operator (permission ≥ 2) or a player in the same level within 8
  blocks; any other caller gets a generic refusal. Rule: `runtime::may_inspect`, pinned by
  `a_readout_needs_an_operator_reach_or_the_same_level`.

### In-game harness corrections (documented)

- The control leg probed 0,64,0, which is the End exit portal's bedrock in this world; an unknown
  block id leaves bedrock in place, so it now probes 4,64,4 and 6,64,4.
- Under `--headless` a mod container screen opens but `ctl screen`/`close`/`quit` never answer
  while it is open (no screen tick). The menu step now reads `.screen` from the `ctl interact`
  reply, runs last, and `cleanup` ends the captured game pid; the three slots stay pinned by the
  container unit test, and item movement is exercised in-game through the same slots by `item
  replace`.
- The canonical leg was re-run through `launcher-game.sh run mekanism` after the scenario fixes
  and passes (`PASS` + `EXIT:0`); an earlier attempt stopped in `e2e-guard.sh` with
  `p: unbound variable` after the chamber section (a guard bug, not the port).
- `runtime.rs` gained the reach rule and its test (43 tests); it is no longer byte-identical to the
  retired bundle.

Ladder: `cargo test` 43 passed; `cargo clippy --all-targets -- -D warnings` exit 0;
`cargo fmt --check` clean; `cargo horizon-mod build` twice byte-identical; `check` and `fits`
exit 0. In-game: `PASS` + `EXIT:0` (`ports/_shared/evidence/mekanism-e2e/e2e-launcher.log`);
`GAME=$CONTROL_GAME CONTROL=0` → `FAIL` (`Unknown block type`) + `EXIT:1`.

## Slice 3 — the Crusher (0.2.2, 2026-11-21)

### The scope authority, quoted

The port's own gap list said (verbatim, `FINDINGS.md`, "Next slices (planned gap list)" item 1):

> **Ore processing chain (the rest of the tier)**: the **Enrichment
> Chamber** is done (slice 2); next are the Crusher (`mekanism:crushing`,
> the same `ItemStackToItemStackRecipe` machine shape), Purification Chamber
> (needs oxygen, a gas system), Ore Washer (needs water) and Chemical
> Injection Chamber (gases). The first fits the shared machine core directly;
> the gas-based ones need a gas capability that does not exist at the pin.

and `notes.md`'s own gap 1 lists "Crusher" first among the machines that are not ported.

**Chosen: the Crusher.** It is the next machine in the jar's processing chain and the only one of the
four that needs no new capability: purification, washing and injection all need a **gas** (oxygen,
water as a gas, chemical injection), which the pin has no capability for, while crushing is the same
`ItemStackToItemStackRecipe` machine the smelter and chamber already run. It fits the tier as a
**machines-shaped feature inside the `deep` row**: one block entity, its saved state, a container and
menu, and its energy balance in its own tick — no entity, renderer, dimension or custom vanilla hook,
so the slice does not grow past the tier.

### What the jar says (upstream truth, read from `v1.21.1-10.7.19.85`)

| value | source | port |
|---|---|---|
| usage 50 FE/t | `UsageConfig.java:54` (`crusher`, default `50L`) | `CRUSHER_USAGE = 50` |
| storage 20,000 FE | `StorageConfig.java:57` (`crusher`, `20_000L, 1`) | `CRUSHER_CAPACITY = 20_000` |
| 200 ticks per operation | `TileEntityCrusher` → `TileEntityElectricMachine(BASE_TICKS_REQUIRED)` | `CRUSHER_TICKS = 200` (10,000 FE per operation) |
| recipe type `mekanism:crushing` | `TileEntityCrusher.getRecipeType()` → `MekanismRecipeType.CRUSHING` | `runtime::crushing` |
| the same progress rules | `TileEntityRecipeMachine`/`CachedRecipe` (shared with the other two) | `runtime::process_tick`, unchanged |
| block 3.5 / 16, `facing`×`active` | `BlockFactoryMachine`, `blockstates/crusher.json` | barrel template (3.5/16, `facing`×`open`), the same recorded deviation as the other two machines |

### The recipe table: 145 of the jar's 313 crushing recipes

`tools/gen_crushing.py` reads every `"type": "mekanism:crushing"` file in the jar and keeps a row where
both sides exist in this port:

- **tags expanded to their 26.3 members** (`c:cobblestones/normal`, `c:gravels`,
  `c:sandstone/{uncolored,red}_blocks`, `minecraft:wool` ×16, `c:rods/{blaze,breeze}`,
  `c:ingots/{iron,gold,copper}`, `c:obsidians/normal`), and `neoforge:compound` unions
  (`polished_or_smooth_basalt_to_basalt`);
- **312 recipes with `neoforge:conditions`** (AE2, Biomes O' Plenty, Biomes We've Gone) are another
  mod's items and are not carried; **190 rows are skipped in total**, each printed by the generator:
  the bio-fuel family (`mekanism:bio_fuel`/`block_bio_fuel` are not registered), clumps
  (`c:clumps/*` — purification, which needs oxygen, is not ported), the gem/coal/charcoal/obsidian
  dusts (`mekanism:dust_{diamond,emerald,lapis_lazuli,quartz,coal,charcoal,obsidian,netherite}` are
  not registered), and bronze/steel/lead/tin/uranium (no items).

Families carried: cobblestone→gravel→sand, the stone/deepslate/tuff/blackstone/quartz/prismarine
building-block conversions, the dewax family (40 waxed→plain copper rows), wool→string ×4,
bone→bone meal ×6, bone block→×9, blaze rod→4 powder, breeze rod→6 wind charge, prismarine→shards,
dripstone block→4 pointed dripstone, honeycomb block→4 honeycomb, music disc 5→9 fragments,
flint→gunpowder, and the three ingots→dusts (iron, gold, copper).

### The two defects this slice found and fixed

**1. Every carried shaped recipe was in the 1.21.1 dialect and could never craft.** The four recipes
0.2.0/0.2.1 shipped (and the new crusher's) wrote a key ingredient as `{"item": "minecraft:iron_ingot"}`,
which is the NeoForge 1.21.1 shape. 26.3's shaped-recipe parser takes a **bare item id**
(`data/minecraft/recipe/iron_block.json`: `"#": "minecraft:iron_ingot"`), and rejects the object —
the server log at the recipe phase said it plainly:

```
[Server thread/ERROR]: recipes: invalid ingredient (unresolved tag or malformed input): {"item":"minecraft:iron_ingot"}
[Server thread/ERROR]: recipes: invalid ingredient (unresolved tag or malformed input): {"item":"minecraft:copper_ingot"}
```

The grid was filled correctly and the result slot stayed empty, so the recipes were dead data — a
defect `check`, `fits` and the unit tests all pass, which is exactly the class the parity rule was
written for. **Fixed:** all five carried recipes use bare item ids. Proven in game by
`tools/craft_check.py` (each recipe crafts its declared id and count, the grid empties, the
distinctive ingredient is consumed, and two wrong-ingredient controls leave the result empty).

**2. The port's own `Stack`/`Recipe` shadowed the SDK's types and made `fits` claim a break.**
`cargo horizon-mod` records a package's struct literals against the SDK type of the same name, so
`uses.json` claimed `type:Recipe` (with the SDK's nine fields) and `type:Stack` for the pure core's
`Recipe { input, output }` and `Stack { item, count }` — types the package does not use. `fits
--build 0b14f8a` then failed with *"the type `Recipe` can no longer be built with a literal"*, a
dependency the port never had (`HANDOVER.md:782` records the same class of false positive on other
ports' `Item`/`Config`). **Fixed:** the pure-core types are `MachineRecipe` and `ItemAmount`;
`uses.json` no longer lists either, and `fits` against `0b14f8a` prints `fits`.

### Red → green

The tests were written first and ran red on the untouched tree
(`../_shared/evidence/mekanism/slice3-unit-red.log`):

```
error[E0425]: cannot find value `CRUSHER` in this scope
   --> src/support.rs:622:37
622 |         assert_eq!(machine_capacity(CRUSHER), Some(CRUSHER_CAPACITY));
    |                                     ^^^^^^^ not found in this scope
...
error[E0433]: cannot find module or crate `crushing` in this scope
    --> src/runtime.rs:1930:20
1930|         assert_eq!(crushing::lookup("minecraft:dirt"), None);
error: could not compile `mekanism` (lib test) due to 31 previous errors
```

Then the table generator, the constants, the module and the third `MachineDecl` made them green:
**52 tests** (was 43).

### Coverage — every family this slice ships

| family | status | proof |
|---|---|---|
| the Crusher block, its block entity, 20,000 FE buffer, three slots, furnace menu | **tested** | `C` line `mekanism:crusher: 0/20000 FE, ...`, the menu opens (`ctl interact` → `screen: container`) |
| one crushing operation: 10,000 FE, 200 ticks, input consumed, output in slot 2 | **tested** | `C`: cobblestone → gravel and `energy(0,64,22): 0` after one operation |
| no energy → nothing happens (progress held) | **tested** | `C`: the uncharged crusher at 2,64,22 stays `progress 0/200, input minecraft:cobblestone x1` |
| an input with no recipe → nothing, no energy spent | **tested** | `D` control: dirt still in slot 0 after 225 ticks, slot 2 empty |
| **all 145 carried crushing recipes** (input count, output id and count, input consumed) | **tested** | `D`: `rows crushed: 145 of 145` (`tools/crush_sweep.py`, ten crushers per batch) |
| the five carried shaped recipes | **tested** | `B1` (`tools/craft_check.py`): five crafts + two wrong-ingredient controls |
| the five blocks' loot tables | **tested** | `B2`: each block drops its own item (`ctl break` → `.drops`) |
| the five blocks and three dusts give/place/read back; an unknown id refused | **tested** | `A1`/`A2` |
| clumps → dirty dust (the crusher's ore-processing role) | **not ported** | needs the Purification Chamber (oxygen, a gas capability) |
| gem/coal/charcoal/obsidian/netherite dusts from crushing | **not ported** | those `mekanism:dust_*` items are not registered; the rows are dropped and printed by the generator |
| bio fuel | **not ported** | `mekanism:bio_fuel`/`block_bio_fuel` are not registered |
| the `active` front texture and the `crusher_active` model | **not ported** | carried in `assets/`, unused: the block template's `open` state stands in for `active` (the smelter's/chamber's recorded deviation) |

### The ladder (one lease per step, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result |
|---|---|---|
| runtime + glue tests (RED) | `cargo test --offline` on the untouched tree | **31 errors** (`slice3-unit-red.log`) |
| tests | `cargo test --offline` | **52 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` | clean (`sdk-fmt.log`) |
| recipe tables | `gen_smelting.py --check`, `gen_enriching.py --check`, `gen_crushing.py --check` | 163 / 129 / **145** rows (`sdk-recipe-tables.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `Built ./target/horizon-mod/mekanism.hmod` (`sdk-build.log`) |
| same package twice | the same again | **`cmp` identical** (`sdk-build-second.log`, `sdk-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | `mekanism 0.2.2 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **68,652 B, SHA-256 `d52219f7…7d38f6`**, exit 0 (`sdk-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, `EXIT:0` (`sdk-fits.log`) |
| fits, **a different build** | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` (`sdk-fits.log`) — the run that proves something |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (`82 entries`, no packed `Cargo.lock`) |
| archive shape | `unzip -l mekanism.hmod` | 82 files, no `Cargo.lock` (`sdk-entries.txt`) |

### In-game evidence (`--offscreen`, the launcher build)

`ports/tools/launcher-game.sh run mekanism` → `PASS` + `EXIT:0`
(`../_shared/evidence/mekanism-e2e/e2e-package.log`), quoted from the new phases:

```
== A1: every carried block and item is give-able ==
  5 blocks and 3 items are give-able
== A2: every block stands where it is put, and an unknown id is refused ==
  5 blocks place and read back; an unknown id is refused
== B1: the five carried shaped recipes in a real crafting table ==
  energized smelter: 4 iron + 2 redstone + 2 glass + iron block -> {"count": 1, "item": "mekanism:energized_smelter"}
  enrichment chamber: 4 iron + 2 redstone + 2 gold + iron block -> {"count": 1, "item": "mekanism:enrichment_chamber"}
  crusher: 4 redstone + 2 copper + 2 magma + iron block -> {"count": 1, "item": "mekanism:crusher"}
  basic energy cube: 4 iron + 2 redstone blocks + 2 copper + iron block -> {"count": 1, "item": "mekanism:basic_energy_cube"}
  basic universal cable: 2 iron + redstone, eight at a time -> {"count": 8, "item": "mekanism:basic_universal_cable"}
  smelter with a stick where the glass goes -> null
  crusher with a stick where the magma goes -> null
recipe PASS: all five carried shaped recipes craft with their declared id and count, their ingredients are consumed, and both wrong-ingredient controls leave the result empty
== B2: every block's loot table drops the block ==
  5 blocks each dropped their own item
== C: the crusher crushes for exactly 10,000 FE, and not without energy ==
  info(0,64,22): mekanism:crusher: 0/20000 FE, progress 0/200, input minecraft:cobblestone x1, output empty x0
  info(0,64,22): mekanism:crusher: 0/20000 FE, progress 0/200, input empty x0, output minecraft:gravel x1
  energy(0,64,22): 0
  info(2,64,22): mekanism:crusher: 0/20000 FE, progress 0/200, input minecraft:cobblestone x1, output empty x0
  energy(2,64,22): 0
  the crusher opens its three-slot menu
== D: every carried crushing recipe in a real crusher ==
  rows from the jar's mekanism:crushing data: 145
  minecraft:cobblestone x1 -> {"count": 1, "item": "minecraft:gravel"}
  minecraft:iron_ingot x1 -> {"count": 1, "item": "mekanism:dust_iron"}
  minecraft:waxed_copper_block x1 -> {"count": 1, "item": "minecraft:copper_block"}
  minecraft:white_wool x1 -> {"count": 4, "item": "minecraft:string"}
  ...
rows crushed: 145 of 145
sweep PASS: all 145 carried crushing recipes ran in a real crusher with their declared input count and output stack, and the input was consumed
== control: a crusher with no recipe for its input crushes nothing ==
  crushed dirt: [{"count": 1, "item": "minecraft:dirt"}, null, null]
  control PASS: the dirt is still in the input and nothing was made
```

What the numbers prove: 20,000 FE was set and **10,000 left after one operation**, so the cost is
exactly 200 ticks × 50 FE/t; the crusher beside it, never charged, kept its cobblestone and made
nothing (the energy is what crushes); the dirt stayed put after the same 225 ticks (no recipe, no
spend); and all 145 rows produced exactly the jar's output stack with the input consumed.

The rest of the leg (slices 1–2) is unchanged and still passes: the cube→cable→chamber→smelter chain,
the five controls, the three-slot menus and the save/relaunch round trip.

| leg | result | evidence |
|---|---|---|
| package (default) | **`PASS` + `EXIT:0`** | `e2e-package.log` |
| unmodded game (`GAME=$CONTROL_GAME`) | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** | `e2e-control.log` |
| fail-capable (`SELFTEST=1`) | **`FAIL: expected /mekanism info at 0,64,22 to say 'output empty x0', got 'mekanism:crusher: 0/20000 FE, progress 0/200, input empty x0, output minecraft:gravel x1'` + `EXIT:1`** | `e2e-selftest.log` |
| differential (`NOCABLE=1`) | **`PASS` + `EXIT:0`** (no energy reaches the smelter or chamber) | `e2e-nocable.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |

### Pin

Slice 3 is built and run at the campaign pin **Horizon `0a875da91dbb0fdcb7ba8c42c11341aa229af51a`**
(Nightly 2026-11-20, `package_format` 6, `content_api` 0.2, toolchain 1.99.1) + mixins
**`e59da9094f0c2fdda6c627fe9eff22dec6a906f0`** (unchanged). The `[native]` block carries it,
`.horizon/sdk` is the SDK release's `api-source.tar.xz` (SHA-256 verified against its `SHA256SUMS`),
and `fits` is green against both this pin and the previous one (`0b14f8a`).

### Gaps after slice 3

Everything in *Gaps / deviations* and *Gaps after the migration* above still stands. Changed by this
slice: the crusher is ported (the ore-processing chain is now smelter, chamber, crusher); the five
block recipes are craftable in-game now that their dialect is fixed, which closes most of gap 6 (the
machines' other ingredients — steel casing, alloys, circuits — are still represented by vanilla
stand-ins, a recorded deviation); the dusts a crusher would make (`diamond`, `emerald`, `lapis`,
`quartz`, `coal`, `charcoal`, `obsidian`, `netherite`) are not registered, so those rows are dropped.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,verification-before-completion; BeautifulRust/rust-best-practices

### Changelog (0.2.2) — supersedes 0.2.1

> **0.2.2 — the Crusher, and the carried recipes actually craft.** One new machine on the shared
> recipe-machine core: 50 FE/t, 200 ticks per operation (10,000 FE), a 20,000 FE insert-only buffer,
> the same three slots and vanilla furnace menu, running the jar's `mekanism:crushing` table over the
> items this port has — **145 rows** (cobblestone→gravel→sand, the stone/deepslate/tuff/blackstone/
> quartz/prismarine conversions, the 40 waxed→plain copper rows, wool→4 string, bone→6 bone meal,
> blaze rod→4 powder, breeze rod→6 wind charge, prismarine shards, and iron/gold/copper ingots → their
> dusts). Two defects fixed: every carried shaped recipe used the 1.21.1 `{"item": …}` ingredient
> shape, which 26.3 rejects (`recipes: invalid ingredient`), leaving all five uncraftable — they now
> use bare item ids and craft in game; and the pure core's `Stack`/`Recipe` shadowed the SDK's types
> in `uses.json`, making `fits` report a break the package did not have — they are `ItemAmount` and
> `MachineRecipe` now. Pin moved to SDK **`0a875da9…`** (mixins unchanged). Package: **82 entries,
> 68,652 B**, sha256 `d52219f7f369bded568a031d8a8e31d8e0a8a2cb8b0fc19bafe43112187d38f6`;
> `check` exit 0; `fits` `made for` at the pin and `fits` at `0b14f8a`; 52 tests, clippy
> `-D warnings` clean; in game `PASS` + `EXIT:0` with all 145 crushing rows and the five recipes
> proven, the unmodded game `EXIT:1`, `SELFTEST=1` `EXIT:1`, `NOCABLE=1` `PASS` + `EXIT:0`.

## Slice 4 — the chemical capability (0.2.3, 2026-11-21)

### The scope authority, quoted

The gap line this slice comes from is the port's own (`notes.md` "Gaps / deviations" item 1 and
`FINDINGS.md` "Next slices" 1 and 6), and the slice-3 note names the missing surface twice:

> Ore processing beyond smelting, enriching and crushing (Purification, Ore Washer, Chemical
> Injection — all three need a gas), factories, … **the gas system**, …

> **Machines that need new capabilities**: Metallurgic Infuser, Thermal Distillation, Electrolytic
> Separator, Digital Miner (requires a gas capability and a mod-defined resource type).

The campaign's evidence adds the demand side: at the pin the game publishes **11** capabilities and
none is chemical (`surface.json`: energy, fluids, items, item_energy, item_fluids, item_items,
entity_automation, entity_energy, entity_fluids, entity_items, storage), and two ports stopped on
exactly that — `refinedstorage_mekanism_integration` (`blocked(mekanism:chemical_handler)`) and
`mekanisticrouters` (`blocked(no-parent-module-api,no-chemical-capability)`).

**Chosen: the chemical capability**, as the smallest honest provider of it: one resource (a chemical
id + amount), **one** `CapabilityKey` published by the shipped crate, one block entity holding one
buffer, and one item that carries a chemical through the item side of the same capability. Not the
gas system, not the factories, not osmium/alloys, not a multiblock — half a chemical system is only
a chemical system if one round trip through the key works, so the slice is that round trip and
nothing else.

It fits the **`deep` tier** as a machines-shaped feature: a block entity with its own saved state,
its capability provider and its item, no entity, renderer, dimension or custom vanilla hook.

### How the SDK's capability mechanism works (read before designing the key)

`ports/_shared/sdk/0a875da/crates/horizon-mod-api/src/parts/capabilities.rs`:

- `PartRegistrar::capability(table, shares_names, name, target, before_game, (provider, handler))` keys
  providers by **`name`** and refuses one name answered with **two handler types** (it compares the
  handler's `type_name`), because a lookup of a mod's own capability downcasts to the trait object the
  asker names (`OwnProvider` holds `Arc<dyn Any>`; only a mod linked with the defining crate knows its
  handler type).
- `CapabilityKey<T> { name, default, handler }`: `CapabilityKey::new("mymod:chemicals")` is a **mod's
  own** capability, answered only by its Parts (`LevelRef::capability` → the table's providers, then
  the game's `default`). The game's own keys (`ITEMS`, `ENERGY`, `FLUIDS`) fill in `default`.
- The fixture the brief points at (`#[cfg(any(test, feature = "testing"))] pub mod tests`) is exactly
  this shape: `Chemical(pub String)`, `CHEMICALS = CapabilityKey::new("test:chemical")`, and a
  `ChemTank` keeping `chemical` + `amount` in a block entity's saved data. It is a **test** fixture,
  so the key here is the same shape in a shipped module (`src/chemical.rs`), not a copy of it.
- Item capabilities are a separate table with their own key type (`ItemCapabilityKey`), as fluids are
  (`FLUIDS` / `ITEM_FLUIDS`), which is why the item side is `mekanism:item_chemical`.

### What the jar says (upstream truth, read from `v1.21.1-10.7.19.85`)

| value | source | port |
|---|---|---|
| 64 buckets of one chemical, 1,000 out per tick | `ChemicalTankTier.java:12` (`BASIC(BaseTier.BASIC, 64 * BUCKET_VOLUME, BUCKET_VOLUME)`) | `TANK_CAPACITY = 64_000` |
| the tank's one tank, drain and fill slots, side IO | `TileEntityChemicalTank.java:117-133` (`getInitialChemicalTanks`, `getInitialInventory`) | one tank; **no slots** (recorded deviation) |
| the chemical registry's 32 named chemicals | `MekanismChemicals.java:19-58` | `runtime::CHEMICALS` (ids only, sorted) |
| a chemical stack serialises as `{id, amount}` | `ChemicalStack.java:80-83` (`MAP_CODEC`, `SerializationConstants.ID`/`AMOUNT`) | the item component `mekanism:chemical` |
| the gauge dropper holds 16,000, moves 250 a call | `GearConfig.java:159-163` (`gaugeDropperCapacity` 16 × 1,000, `gaugeDroppedTransferRate` 250) | `DROPPER_CAPACITY`, `DROPPER_TRANSFER` |
| it is a chemical container: `Capabilities.CHEMICAL.getCapability(stack)` | `ItemGaugeDropper.java:78-96` (`use`, shift-clears its tanks) | `ITEM_CHEMICAL` provider over the stack's component |
| the tank block: machine properties 3.5/16, no `active`, no `facing` | `MekanismBlockTypes.java:836`, `createMachine(...)` at 928-935 | barrel-free block template (`minecraft:iron_block`) with 3.5/16 |
| the tank's shape is inset (`BlockShapes.CHEMICAL_TANK`) | `MekanismBlockTypes.java:930` | full cube (recorded deviation) |

### The slice, in the port's three layers

- **`src/chemical.rs`** — the published surface: `Chemical` (a namespaced id), `CHEMICAL` (the block
  key, `mekanism:chemical`), `ITEM_CHEMICAL` (the item key, `mekanism:item_chemical`) and
  `ITEM_COMPONENT` (`mekanism:chemical`, the component an item's chemical lives in). `Chemical::new`
  checks the *shape*; whether an id is a chemical Mekanism registers is `runtime::known_chemical`, so a
  compound written by a later version still reads back instead of being dropped.
- **`src/runtime.rs`** — the rules: `CHEMICALS` (32 ids), `known_chemical`, `Tank` (`chemical` +
  `amount`, `amount == 0` is empty whatever the id), `insert_chemical` (unknown id → `Refusal::
  UnknownChemical(<the id>)`; a different chemical than the held one → 0; capped by the room left),
  `extract_chemical`, `dropper_move` (at most 250 a call), and the three capacities.
- **`src/support.rs`** — the glue: `ChemicalTank(BlockSide)` answering `CHEMICAL` for the
  `basic_chemical_tank` block entity type (reads and writes `chemical` + `amount` through
  `BlockEntityRef::{data, merge_data}`, so the chunk is marked changed and a transaction can roll it
  back), `ItemChemicalTank(ItemPlace)` answering `ITEM_CHEMICAL` for `mekanism:gauge_dropper` (the
  `{id, amount}` component rewritten through `ItemComponentHandler::write`, i.e. an exchange of one),
  `read_tank`/`write_tank` for the saved shape, and the `/mekanism chemical …` readouts.

The one structural choice worth naming: the tank **does not tick**. Nothing here moves on its own, so
a capability call is the only writer — which is why the slice sidesteps the `level_tick_post`
write-back trap entirely (a ticked block entity's writes are overwritten by its own write-back unless
queued to `level_tick_post`).

### Commands (harness, recorded like `/mekanism info`)

| command | what it does |
|---|---|
| `/mekanism chemical list` | the id table's size and a member, plus `block_capability_names(host)` — the game's published capability names, where `mekanism:chemical` now appears |
| `/mekanism chemical read <pos>` | asks `mekanism:chemical` at `pos` through `LevelRef::capability` and prints what the **handler** answers — and `(answered by <mod>)`, which is the table's word, not this port's |
| `/mekanism chemical insert <amount> <id> <pos>` (op 2) | inserts through the key, exactly as another mod's crate would; an id outside the table is refused **by name** |
| `/mekanism chemical drain <amount> <pos>` (op 2) | extracts through the block key and inserts into the gauge dropper in the caller's main hand through the item key, at the dropper's 250-per-call rate; what the dropper refuses goes back into the tank |
| `/mekanism chemical dropper` | what the held dropper carries, read through the item key |

`/mekanism info` at a chemical tank now answers the tank line instead of "No block entity there." (the
tank has no FE buffer, so the machine line would have been a lie).

### Red → green

The runtime rules were written as tests first and ran red on `todo!()` bodies
(`../_shared/evidence/mekanism/slice4-unit-red.log`):

```
---- runtime::chemical_tests::an_unknown_chemical_is_refused_by_name stdout ----
thread 'runtime::chemical_tests::an_unknown_chemical_is_refused_by_name' panicked at src/runtime.rs:1939:5:
not yet implemented: a tank takes a chemical
...
test result: FAILED. 0 passed; 6 failed; 0 ignored; 0 measured; 52 filtered out
```

The glue's saved-shape test ran red the same way (`slice4-glue-red.log`:
`support::tests::the_saved_chemical_and_amount_read_back … not yet implemented: the tank's saved
shape`, `1 failed`). Then the implementations made the whole crate green: **63 tests** (was 52) —
6 new `runtime::chemical_tests`, 3 `chemical::tests`, 2 `support::tests`.

### Coverage — every family this slice ships

| family | status | proof |
|---|---|---|
| the capability is published by name, answerable from another crate's lookup | **tested** | `F1`: `published capabilities horizon:items, horizon:energy, horizon:fluids, horizon:storage, mekanism:chemical`; `F2`/`F3` ask it through `LevelRef::capability` (the consumer path) and get `(answered by mekanism)` |
| the 32-id chemical table; an unknown id refused **by name** | **tested** | `F1` (`32 chemicals, mekanism:hydrogen among them`), `F4` (`unknown chemical mekanism:unobtainium`, tank unchanged at 3,750), unit tests `the_chemical_table_is_mekanism_s_named_chemicals`, `an_unknown_chemical_is_refused_by_name` |
| the Basic Chemical Tank: block, block entity, 64,000-unit tank, `chemical` + `amount` saved data | **tested** | `A1`/`A2` (give/place/read back), `B2` (loot), `F2` (`data get block … chemical: "mekanism:hydrogen"`, `amount: 4000`), `F2` `/mekanism info` answers the tank line |
| a chemical inserted through the block key | **tested** | `F2`: `inserted 4000 mekanism:hydrogen into mekanism:basic_chemical_tank (4000/64000)` |
| a partial extract through the block key into the item | **tested** | `F3`: `drained 250 … (tank 3750/64000, dropper 250/16000)`; `data get block … amount: 3750`; the stack's `components: {"mekanism:chemical": {amount: 250, id: "mekanism:hydrogen"}}` |
| the Gauge Dropper: item, 16,000 units, 250 a call, its `mekanism:chemical` component | **tested** | `A1` (give), `B1` (crafts), `F3` (the component and the 250 cap), `F3` `chemical dropper: … holds mekanism:hydrogen 250/16000`, unit tests `the_gauge_dropper_moves_at_most_its_transfer_rate` |
| a block entity that answers no chemical moves nothing (counterfactual) | **tested** | `F5`: `no chemical handler at 18 64 26`, and `data get block 18 64 26 chemical` → `Found no elements matching chemical` |
| the tank's drain/fill slots and side IO | **not ported** | `TileEntityChemicalTank`'s two `ChemicalInventorySlot`s are a convenience over the same tank; the capability is the interface this slice ships |
| the tank's inset shape and its own GUI | **not ported** | a full cube over the tank texture (the cube's and cable's recorded deviation); no screen |
| chemical *items* (gases as items), slurries, pigments, infuse types | **not ported** | the id table names them; no item, no registry entry, no recipe produces one |
| the chemical-block machine family (washer, injection, separator) | **not ported** | they need this capability *and* their own machines; this slice is the capability they were blocked on |

### The ladder (one lease per step, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result |
|---|---|---|
| runtime rules (RED) | `cargo test --offline chemical_tests` on `todo!()` bodies | **6 failed**, `not yet implemented` (`slice4-unit-red.log`) |
| glue saved shape (RED) | `cargo test --offline the_saved_chemical` on `todo!()` bodies | **1 failed**, `not yet implemented` (`slice4-glue-red.log`) |
| tests | `cargo test --offline` | **63 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` | clean (`sdk-fmt.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `Built ./target/horizon-mod/mekanism.hmod` (`sdk-build.log`) |
| same package twice | the same again | **`cmp` identical** (`sdk-build-second.log`, `sdk-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | `mekanism 0.2.3 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **84,180 B, SHA-256 `5e304608…8fd989`**, exit 0 (`sdk-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, `EXIT:0` (`sdk-fits.log`, sha printed first) |
| fits, **a different build** | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, `EXIT:0` (`sdk-fits.log`) |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (94 entries, no packed `Cargo.lock`) |
| archive shape | `unzip -l mekanism.hmod` | 94 files, no `Cargo.lock` |

The launcher build (`launcher-game.sh build mekanism`, in a slot) printed `Compiling mekanism (1 of
1)`, which is the trap's check that the binary under test carries this package.

### In-game evidence (`--offscreen`, the launcher build)

`ports/_shared/evidence/mekanism-e2e/e2e-package.log` → `PASS` + `EXIT:0`, quoted from phase F:

```
== F1: the chemical capability is published by name, with Mekanism's ids ==
  chemical list: chemical: 32 chemicals, mekanism:hydrogen among them; published capabilities horizon:items, horizon:energy, horizon:fluids, horizon:storage, mekanism:chemical
== F2: an insert through the capability lands in the tank's saved data ==
  chemical insert 4000 mekanism:hydrogen 16 64 26: chemical: inserted 4000 mekanism:hydrogen into mekanism:basic_chemical_tank (4000/64000)
  data get block 16 64 26 chemical: 16, 64, 26 has the following block data: "mekanism:hydrogen"
  data get block 16 64 26 amount: 16, 64, 26 has the following block data: 4000
  chemical read 16 64 26: chemical: mekanism:basic_chemical_tank holds mekanism:hydrogen 4000/64000 (answered by mekanism)
  mekanism info 16 64 26: chemical: mekanism:basic_chemical_tank holds mekanism:hydrogen 4000/64000 (answered by mekanism)
== F3: part of it drains into the gauge dropper through the item capability ==
  chemical drain 1000 16 64 26: chemical: drained 250 mekanism:hydrogen into the gauge dropper (tank 3750/64000, dropper 250/16000)
  data get block 16 64 26 amount: 16, 64, 26 has the following block data: 3750
  data get entity @s SelectedItem: Player… has the following entity data: {components: {"mekanism:chemical": {amount: 250, id: "mekanism:hydrogen"}}, count: 1, id: "mekanism:gauge_dropper"}
  chemical dropper: chemical: the gauge dropper holds mekanism:hydrogen 250/16000
== F4: an unknown chemical id is refused by name and moves nothing ==
  chemical insert 500 mekanism:unobtainium 16 64 26: chemical: unknown chemical mekanism:unobtainium
  data get block 16 64 26 amount: 16, 64, 26 has the following block data: 3750
== F5: the counterfactual: a block entity that answers no chemical moves nothing ==
  chemical insert 1000 mekanism:hydrogen 18 64 26: chemical: no chemical handler at 18 64 26
  data get block 18 64 26 chemical: Found no elements matching chemical
  the same insert moved nothing into a block that answers no chemical
```

What the numbers prove:

- **the key is real and published**: `mekanism:chemical` is in the game's own
  `block_capability_names` list beside `horizon:energy`/`horizon:fluids`/`horizon:items`, and every
  read answers `(answered by mekanism)` — the lookup went through the table by name, not through this
  port's block ids;
- **the insert round trip**: 4,000 of `mekanism:hydrogen` went in through the capability and the block
  entity's **saved data** says `chemical: "mekanism:hydrogen"`, `amount: 4000` — the two `/data get
  block` reads are the "read the buffer back by its saved data, assert id and amount exactly" leg;
- **the partial extract**: asking for 1,000 moved **250**, because the dropper's own transfer rate is
  250; the tank reads **3,750** (4,000 − 250) and the dropper's stack carries
  `{amount: 250, id: "mekanism:hydrogen"}` in its `mekanism:chemical` component — the buffer delta and
  the item's payload asserted independently, and the item read through its own capability says
  `250/16000`;
- **the unknown id** is refused **by name** (`unknown chemical mekanism:unobtainium`) and the tank is
  untouched at 3,750;
- **the counterfactual**: the same `insert 1000 mekanism:hydrogen` against a **crusher** — a real mod
  block entity with a container and an energy buffer, which answers `horizon:items` and
  `horizon:energy` but not this key — answers `no chemical handler at 18 64 26` and its saved data has
  no chemical at all. Paired with F2 (the same insert into the tank moves 4,000), it is a
  two-sided differential, not a one-sided absence.

| leg | result | evidence |
|---|---|---|
| package (default) | **`PASS` + `EXIT:0`** | `e2e-package.log` |
| unmodded game (`GAME=$CONTROL_GAME`) | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** | `e2e-control.log` |
| fail-capable (`SELFTEST=1`) | **`FAIL: expected /mekanism chemical insert 4000 mekanism:hydrogen 16 64 26 to say 'inserted 0 mekanism:hydrogen', got '…inserted 4000…'` + `EXIT:1`** | `e2e-selftest.log` |
| differential (`NOCABLE=1`) | **`PASS` + `EXIT:0`** (phase F passes with no cable too) | `e2e-nocable.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |

The `SELFTEST=1` flip moved to **this** slice's assertion (it used to flip the crusher's output): the
newest family is the one that has to prove it can fail, and it fails on the insert assertion, which
runs before the smelting phases.

### What the harness taught this slice

1. **A creative player's hand gives without losing the stack** (`ItemPlace::exchange` →
   `extract` + `insert`, and a creative hand's extract is vanilla's give-without-taking). The drain
   probe first showed **two** droppers: the plain one still in the hand and the filled one in the next
   inventory slot (plus 750 units gone after three retries). The phase now switches to survival for the
   exchange (as a bucket behaves) and back to creative after it. This is the same class as the
   `NoAI`/item-entity harness quirks: record it, do not "fix" the game.
2. **A command that moves something must never be retried by an assertion loop.** F3's first version
   used the `info_once`-style "ask again until the answer matches" and drained 250 **three** times. The
   helper now counts the `chemical: ` lines in the log, sends the command **once**, and waits for a
   *new* line; the assertion is applied to that line only.
3. **Chat feedback can trail the command** by a second or two under load, so the wait is on the log
   line count, not on a fixed sleep.

### Gaps after slice 4

Everything in *Gaps / deviations* and *Gaps after the migration* and *Gaps after slice 3* still stands.
Changed: the port now **publishes** the capability the pack's integrations were blocked on, so
`refinedstorage_mekanism_integration` and `mekanisticrouters` can ask `mekanism:chemical` and get an
answer; the id table (32 names) is the registry the gas-based machines will validate against. Still
missing for those machines: the chemicals as *items*/registry entries, the slurries and pigments, the
tank's slots and GUI, and the machines themselves.

### `## Upstream defects fixed`

None found in this slice. The 250-per-call gauge-dropper rate and the 64,000-unit tank are upstream's
own numbers, carried as they are; the deviations (plain cube, no slots, no GUI, no chemical items) are
scope, not defects.

### Deviations this slice records

- **The tank block is a full cube** over `mekanism:block/models/chemical_tank`; upstream's authored
  model is an inset 10/16 tank with a valve, and its `BlockShapes.CHEMICAL_TANK` is not carried.
- **No drain/fill slots and no side IO**: the same tank, reached through the capability instead.
- **The tank does not tick** (upstream ticks to move its slots and its dumping mode); nothing here
  moves on its own.
- **The saved compound names `chemical` + `amount`** (the SDK's `test:chemical` fixture shape) while
  the item's component uses `{id, amount}` (upstream's `ChemicalStack` codec): a block entity's saved
  state is the port's own, a component is a serialised stack.
- **The chemicals exist as ids only**: `runtime::CHEMICALS` validates and names them, but no item, no
  registry entry and no texture ships for any of them.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

### Changelog (0.2.3) — supersedes 0.2.2

> **0.2.3 — the chemical capability.** This pack's integrations ask for a chemical handler and no
> Horizon build publishes one (11 capabilities, none chemical), so this version ships the smallest
> honest provider: **`mekanism:chemical`** (a `CapabilityKey` published by the package, plus
> `mekanism:item_chemical` for the item side, with the `Chemical` resource type), answered by a new
> **Basic Chemical Tank** — one tank of 64,000 units of one chemical, kept as `chemical` + `amount` in
> its block entity's saved data — and carried by a new **Gauge Dropper** item that holds 16,000 units
> in its `mekanism:chemical` component and moves 250 a call. The id table is Mekanism's own 32 named
> chemicals, and an id outside it is refused by name. Proven in game through the capability itself: an
> insert lands in the saved data, part of it drains into the dropper through the item key, an unknown
> id is refused and moves nothing, and the same insert against a crusher (a block entity that answers
> no chemical) moves nothing. `/mekanism chemical list|read|insert|drain|dropper` are the harness
> readouts. Package: **94 entries, 84,180 B**, sha256
> `5e304608de3b1094a2b9ce62b5fe9245299cc0587e4bacc44201b303cf8fd989`; `check` exit 0; `fits` `made
> for` at the pin and `fits` at `0b14f8a`; 63 tests, clippy `-D warnings` clean; in game `PASS` +
> `EXIT:0`, the unmodded game `EXIT:1`, `SELFTEST=1` `EXIT:1`, `NOCABLE=1` `PASS` + `EXIT:0`.

## Slice 5 — the Chemical Injection Chamber (0.2.4, 2026-11-21)

### The scope authority, quoted

The slice-4 report named the consumer as the next slice, and the port's own gap list says the same
(`notes.md` "Gaps / deviations" item 1 and `FINDINGS.md` "Next slices" 1):

> Ore processing beyond smelting, enriching and crushing (Purification, Ore Washer, Chemical
> Injection — all three need a gas), …

and `FINDINGS.md`'s refinement after slice 4:

> the **gas capability they were blocked on now exists** (`mekanism:chemical`), so the Purification
> Chamber, Ore Washer and Chemical Injection Chamber are unblocked on the capability and blocked
> only on their own machines and on the chemicals as items.

**Chosen: the Chemical Injection Chamber** (`mekanism:injecting`), the **consumer** half of the
capability slice 4 published: a machine that holds its own gas tank *and* pulls a chemical from a
neighbour's tank **through `mekanism:chemical`** on its tick. It reuses the shared machine core the
Smelter, Enrichment Chamber and Crusher run (block entity, energy buffer, 200 ticks, three slots,
furnace menu) with one extra input: a gas amount per operation. It fits the **`deep` tier** as a
machines-shaped feature — one block entity with its own saved state, a container, an energy balance
and a capability consumer; no entity, renderer, dimension or custom vanilla hook.

### What the jar says (upstream truth, read from `v1.21.1-10.7.19.85`)

| value | source | port |
|---|---|---|
| usage **400 FE/t** | `UsageConfig.java:64` (`chemicalInjectionChamber`, default `400L`) | `INJECTION_USAGE = 400` |
| storage **160,000 FE** | `StorageConfig.java:76-77` (`chemicalInjectionChamber`, `160_000L, 1`) | `INJECTION_CAPACITY = 160_000` |
| 200 ticks per operation, so **80,000 FE** | `TileEntityChemicalInjectionChamber` → `TileEntityAdvancedElectricMachine(BASE_TICKS_REQUIRED)` | `INJECTION_OPERATION = { 200, 400 }` |
| the machine's **own gas tank is 210 units** | `TileEntityAdvancedElectricMachine.java:66` (`MAX_GAS = 210`) | `INJECTION_TANK = 210` |
| insert-only for automation | `getInitialChemicalTanks` → `ConstantPredicates.notExternal()` when `randomizedConsumption` (the default, `true`) makes `allowExtractingChemical()` false | the chamber answers `mekanism:chemical` with `insert_chemical` capped at 210 and its own `extract` taking nothing from a foreign asker |
| **the gas is spent per tick, `chemical_input.amount` each** | `chemicalInputHandler = InputHelper.getConstantInputHandler`; `ItemStackConstantChemicalToObjectCachedRecipe.useResources` → `use(recipeChemical, operations × multiplier)` with `ChemicalUsageMultiplier.constantUse(BASE_TICKS_REQUIRED, …)` = 1 per tick | `per_tick` per operating tick, so a whole operation takes `per_tick × 200` |
| no gas (or a gas the recipe does not take) **holds** the progress | the constant-usage handler overrides `resetProgress` to `tracker.updateOperations(0)` | `IdleReason::NoChemical` (held, no energy spent) |
| recipe type `mekanism:injecting` | `TileEntityChemicalInjectionChamber.getRecipeType()` | `runtime::injecting` |
| block 3.5 / 16, `facing`×`active` | `MekanismBlockTypes.chemicalInjectionChamber` (`createMachine`), `blockstates/chemical_injection_chamber.json` | barrel template (3.5/16, `facing`×`open`), the same recorded deviation as the other machines |

The per-tick reading matters and is easy to get wrong: the jar writes `per_tick_usage: true` on all
91 injecting recipes, `baseTotalUsage` is the machine's **tick count** (200) and the multiplier is
**1 per tick**, which is then multiplied by the recipe's `chemical_input.amount`. So water vapour at
`amount: 1` costs 200 units an operation and a coral block at `amount: 5` costs 1,000 — not 1 and 5.

### The recipe table: 72 of the jar's 91 injecting recipes

`tools/gen_injecting.py` reads every `"type": "mekanism:injecting"` file in the jar and keeps a row
where both sides exist in this port:

- **item tags expanded to their 26.3 members** (`c:ores/{iron,gold,copper}`, `c:raw_materials/*`,
  `c:storage_blocks/raw_*`), each expansion its own row;
- **the chemical side is a list**: the jar writes either one id (`mekanism:oxygen`,
  `mekanism:hydrogen_chloride`) or a chemical **tag** — `mekanism:water_vapor` holds
  `mekanism:water_vapor` and `mekanism:steam`, and `recipe.test` accepts either, so one row names
  both and the machine runs on a tank of either;
- **22 skipped, each printed**: `c:bricks/normal`, `c:gunpowders` (output `mekanism:dust_sulfur`),
  `c:crystals/*` (no crystal item), the lead/osmium/tin/uranium ore and raw tags (no item), the
  plutonium pellet, and `from_crystal` rows — all items or outputs this port does not register.

Families carried: **copper oxidising** (27 rows: block, chiseled, bulb, door, grate, trapdoor, cut
copper and its slab/stairs, all three stages), **concrete solidifying** (16 coloured powders →
concrete), **coral reviving** (15 dead corals, blocks and fans), **dirt → mud**, **terracotta → clay**,
and the **shards** — `c:ores/{iron,gold,copper}` → 4 shards, `c:raw_materials/*` ×3 → 8, raw blocks
×1 (+2 gas) → 24, for the three materials, with the three `mekanism:shard_*` items carried.

### The consumer, in the port's three layers

- **`src/runtime.rs`** — `InjectionRecipe { input, chemical: &'static [&'static str], per_tick,
  output }`, `injecting::{recipe_for, lookup, ROW_COUNT}`, `process_injection_tick` (the shared
  machine rules plus the gas gates) and `injection_pull(recipe, tank) -> Option<(&'static str, u32)>`
  — **one tick's gas**, the chemical to ask for and how much. `Tank::spend` and `room_for` are the
  two helpers it shares with the older machines (`room_for` is the refactor of `process_tick`'s slot
  arithmetic, so both machines use one copy).
- **`src/support.rs`** — `InjectionPart`: `top_up_tank` (asks a neighbour through `CHEMICAL`, only
  when this tick could work) and `pull_chemical` (each move measured, each neighbour asked only when
  it holds the wanted chemical, the whole draw inside one `host.transaction`), then
  `process_injection_tick`, then `write_state` + `write_tank` into the payload. The chamber also
  **answers** `mekanism:chemical` with `ChemicalTank { capacity: 210 }` — the same handler the tank
  block uses, with the machine's capacity — so another mod can fill the machine directly, and its
  own `/mekanism chemical read` shows `0/210`.
- **Content** — `mekanism:chemical_injection_chamber` (barrel template 3.5/16, the upstream
  blockstate's `facing`×`open`, the upstream model, `_active` model and all 11 textures), the three
  `mekanism:shard_{iron,gold,copper}` items (the jar's `item/generated` models and textures), loot,
  the shaped recipe (vanilla stand-ins: 4 gold + 3 iron + iron block + copper block, the upstream
  pattern over ingredients this port has), the pickaxe tag, the creative tab and the lang keys.

The one structural choice worth naming, because it is a **deviation**: upstream's machine does not
pull. Its tank is filled by automation *pushing* into the chemical capability, and the machine only
runs when its tank already holds ≥ one operation's `per_tick` amount. This port's machine **pulls**
the gas it needs, one tick at a time, because that is the behaviour the slice is about (a consumer of
the capability) and because it makes the conservation provable: a whole operation draws exactly
`200 × per_tick` units from the neighbour and leaves the machine's own tank empty. Upstream's
buffer-plus-push model is reachable through the same capability (the chamber answers inserts), so no
behaviour is lost — only the automation direction differs.

### Red → green

The runtime rules and the glue's shape were written as tests first and ran red on the untouched tree
(`../_shared/evidence/mekanism/slice5-unit-red.log`):

```
error[E0432]: unresolved imports `crate::runtime::INJECTION_CAPACITY`, `crate::runtime::INJECTION_OPERATION`, `crate::runtime::INJECTION_TANK`, `crate::runtime::INJECTION_TICKS`, `crate::runtime::injecting`, `crate::runtime::injection_pull`, `crate::runtime::process_injection_tick`, `crate::runtime::room_for`
  --> src/support.rs:19:21
   |
19 |     EMPTY_CHEMICAL, INJECTION_CAPACITY, INJECTION_OPERATION, INJECTION_TANK, INJECTION_TICKS,
   |                     ^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^  ^^^^^^^^^^^^^^^ no `INJECTION_TICKS` in `runtime`
```

then the constants, the generator's table, the two runtime functions and the Part made the crate
green: **72 tests** (was 63) — 8 new `runtime::injection_tests`, 2 `support::tests`. One test-only
mistake was caught by the red run itself (the mid-operation "gas ran out" assertion fired on the
tick that emptied the tank, not the next one) and fixed in the test, not the rule.

### Coverage — every family this slice ships

| family | status | proof |
|---|---|---|
| the Chemical Injection Chamber block, block entity, 160,000 FE buffer, three slots, furnace menu | **tested** | `A1`/`A2` (give/place/read back), `B2` (loot), `B1` (crafts), `G1` (`0/160000 FE, …` and the machine's own line) |
| one injection operation: 80,000 FE, 200 ticks, the input consumed, the jar's output stack | **tested** | `G1`: `raw_iron x3` → `mekanism:shard_iron x8`, `energy … eq 80000` |
| the gas is pulled **through the capability** from a neighbour, one tick's worth per tick | **tested** | `G1`: tank `3,800/64,000` after `4,000`, machine's own tank `0`, `holds nothing 0/210 (answered by mekanism)` |
| no tank beside it → no progress, no FE spent | **tested** | `G2` (4,66,12): `progress 0/200, input minecraft:raw_iron x3`, `eq 160000` |
| a gas the recipe does not take → no progress, no FE, nothing drawn | **tested** | `G2` (7/8,66,12): oxygen tank, same idle line, tank still `4000/64000`; sweep control: a hydrogen tank at a raw-iron chamber, `5000/64000` unchanged |
| **all 72 carried injecting recipes** (input count, output id and count, input consumed, the tank short exactly `200 × per_tick`) | **tested** | `H`: `rows injected: 72 of 72` (`tools/inject_sweep.py`, six pairs per batch) |
| the iron/gold/copper shards: items, models, textures, give-able | **tested** | `A1` (give-able, 3 items) |
| the eight carried shaped recipes (the chamber's added) | **tested** | `B1` (`tools/craft_check.py`): eight crafts + five wrong-ingredient controls |
| the seven blocks' loot tables | **tested** | `B2`: each block drops its own item |
| `c:crystals/*` shard recipes, lead/osmium/tin/uranium shards, gunpowder → sulfur, plutonium pellets | **not ported** | those items are not registered; the generator prints each skipped file |
| the `active` front texture and the `_active` model | **not ported** | carried in `assets/`, unused: the block template's `open` state stands in for `active` (the other machines' recorded deviation) |
| factories (injecting factory), speed/energy/chemical upgrades, side configuration, the machine's own GUI | **not ported** | out of this slice (the same gaps as the other machines) |

### The ladder (one lease per step, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result |
|---|---|---|
| runtime + glue tests (RED) | `cargo test --offline` on the untouched tree | **`error[E0432]`**, the new API missing (`slice5-unit-red.log`) |
| tests | `cargo test --offline` | **72 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` | clean (`sdk-fmt.log`) |
| recipe tables | `gen_smelting.py --check`, `gen_enriching.py --check`, `gen_crushing.py --check`, `gen_injecting.py --check` | 163 / 129 / 145 / **72** rows (`sdk-recipe-tables.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `Built ./target/horizon-mod/mekanism.hmod` (`sdk-build.log`) |
| same package twice | the same again | **`cmp` identical** (`sdk-build-second.log`, `sdk-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | `mekanism 0.2.4 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **105,226 B, SHA-256 `dc37efbc…9dac6c`**, exit 0 (`sdk-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, `FIT-PIN-EXIT:0` (`sdk-fits.log`, sha printed first) |
| fits, **a different build** | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, `FIT-OTHER-EXIT:0` (`sdk-fits.log`) — the run that proves something |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (122 entries, no packed `Cargo.lock`) |
| archive shape | `unzip -l mekanism.hmod` | 122 files, no `Cargo.lock` (`sdk-entries.txt`) |

The launcher build (`launcher-game.sh build mekanism`, in a lease) printed `Compiling mekanism (1 of
1)` and produced `native-0a875da-8480d1934f65`, which is the trap's check that the binary under test
carries this package.

### In-game evidence (`--offscreen`, the launcher build)

`../_shared/evidence/mekanism-e2e/e2e-package.log` → `PASS` + `EXIT:0`, quoted from phase G:

```
== G1: a neighbour tank's gas reaches the chamber through the capability, and one operation takes exactly 200 of it ==
  chemical insert 4000 mekanism:hydrogen_chloride 0 66 12: chemical: inserted 4000 mekanism:hydrogen_chloride into mekanism:basic_chemical_tank (4000/64000)
  info(1,66,12): mekanism:chemical_injection_chamber: 0/160000 FE, progress 0/200, input minecraft:raw_iron x3, output empty x0
  energy(1,66,12): 157200
  info(1,66,12): mekanism:chemical_injection_chamber: 80000/160000 FE, progress 0/200, input empty x0, output mekanism:shard_iron x8
  energy(1,66,12): 80000
  chemical read 0 66 12: chemical: mekanism:basic_chemical_tank holds mekanism:hydrogen_chloride 3800/64000 (answered by mekanism)
  data get block 0 66 12 amount: 0, 66, 12 has the following block data: 3800
  data get block 1 66 12 amount: 1, 66, 12 has the following block data: 0
  chemical read 1 66 12: chemical: mekanism:chemical_injection_chamber holds nothing 0/210 (answered by mekanism)
== G2: without a tank beside it the chamber makes no progress and spends no FE ==
  chemical insert 4000 mekanism:oxygen 7 66 12: chemical: inserted 4000 mekanism:oxygen into mekanism:basic_chemical_tank (4000/64000)
  info(4,66,12): mekanism:chemical_injection_chamber: 160000/160000 FE, progress 0/200, input minecraft:raw_iron x3, output empty x0
  energy(4,66,12): 160000
  info(8,66,12): mekanism:chemical_injection_chamber: 160000/160000 FE, progress 0/200, input minecraft:raw_iron x3, output empty x0
  energy(8,66,12): 160000
  chemical read 7 66 12: chemical: mekanism:basic_chemical_tank holds mekanism:oxygen 4000/64000 (answered by mekanism)
```

and from phase H, the sweep (`tools/inject_sweep.py`) and its control:

```
== H: every carried injecting recipe in a real Chemical Injection Chamber ==
  rows from the jar's mekanism:injecting data: 72
  minecraft:black_concrete_powder x1 + mekanism:water_vapor x1/t -> {"count": 1, "item": "minecraft:black_concrete"} (chemical: mekanism:basic_chemical_tank holds mekanism:water_vapor 4800/64000 (answered by mekanism))
  ...
  minecraft:copper_block x1 + mekanism:oxygen x1/t -> {"count": 1, "item": "minecraft:exposed_copper"} (chemical: … holds mekanism:oxygen 4800/64000 …)
  minecraft:deepslate_iron_ore x1 + mekanism:hydrogen_chloride x1/t -> {"count": 4, "item": "mekanism:shard_iron"} (chemical: … holds mekanism:hydrogen_chloride 4800/64000 …)
  minecraft:dirt x1 + mekanism:water_vapor x1/t -> {"count": 1, "item": "minecraft:mud"} (chemical: … holds mekanism:water_vapor 4800/64000 …)
  ...
  minecraft:raw_copper_block x1 + mekanism:hydrogen_chloride x2/t -> {"count": 24, "item": "mekanism:shard_copper"} (chemical: … holds mekanism:hydrogen_chloride 4600/64000 …)
  minecraft:raw_iron x3 + mekanism:hydrogen_chloride x1/t -> {"count": 8, "item": "mekanism:shard_iron"} (chemical: … holds mekanism:hydrogen_chloride 4800/64000 …)
  minecraft:raw_iron_block x1 + mekanism:hydrogen_chloride x2/t -> {"count": 24, "item": "mekanism:shard_iron"} (chemical: … holds mekanism:hydrogen_chloride 4600/64000 …)
  ...
rows injected: 72 of 72
sweep PASS: all 72 carried injecting recipes ran in a real chamber with their declared input count and output stack, the input was consumed, and the neighbour tank lost exactly the gas the operation spent
== control: a tank of a chemical the recipe does not take moves nothing ==
  gas: chemical: mekanism:basic_chemical_tank holds mekanism:hydrogen 5000/64000 (answered by mekanism)
  machine: [{"count": 3, "item": "minecraft:raw_iron"}], 160000 FE
  control PASS: the raw iron is still there, nothing was made and no gas was drawn
```

What the numbers prove:

- **the gas came through the capability, not from the chamber**: the tank beside the chamber read
  `4,000` before the operation and `3,800` after, through `/mekanism chemical read` (the key, not the
  block id), and the two `/data get block` reads are the same fact from the saved data on **both**
  sides — the neighbour's `amount: 3800` and the machine's `amount: 0`. A machine that had made its
  own gas, or over-drawn a buffer, could not produce that pair;
- **the operation is exactly one operation**: 200 ticks × 400 FE = 80,000 FE, half of the 160,000
  buffer, and the input went from three raw iron to none while slot 2 gained the jar's **eight**
  `mekanism:shard_iron` (the `per_tick_usage` recipe `raw_ore ×3 + 1 hydrogen chloride → 8 shards`);
- **the gas cost is the recipe's, per tick**: every one of the 72 rows left its tank at exactly
  `5,000 − 200 × per_tick` — 4,800 for a one-unit row, 4,600 for the two-unit raw-block row, and
  4,000/3,500 for the three- and five-unit coral rows — so the table's `per_tick` column is what the
  machine spent, not a guess;
- **the controls are two-sided**: with no tank the machine held its progress at 0 and its 160,000 FE
  intact; with a tank of **oxygen** beside a raw-iron operation nothing moved either; and in the
  sweep's control a tank of **hydrogen** (a chemical the port knows, and the one the slice-4 report
  happened to name) was not drawn from at all, because the jar's raw-iron recipe takes
  `hydrogen_chloride`. The same phase with the tank present turns three raw iron into eight shards,
  so the absence is the machine's rule, not a broken harness.

| leg | result | evidence |
|---|---|---|
| package (default) | **`PASS` + `EXIT:0`** | `e2e-package.log` |
| unmodded game (`GAME=$CONTROL_GAME CONTROL=0`) | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** | `e2e-control.log` |
| fail-capable (`SELFTEST=1`) | **`FAIL: expected /mekanism info at 1,66,12 to say 'output empty x0', got 'mekanism:chemical_injection_chamber: 80000/160000 FE, progress 0/200, input empty x0, output mekanism:shard_iron x8'` + `EXIT:1`** | `e2e-selftest.log` |
| differential (`NOCABLE=1`) | **`PASS` + `EXIT:0`** (phase G passes with no cable too) | `e2e-nocable.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |

The `SELFTEST=1` flip moved to **this** slice's assertion (it was the chemical insert in slice 4):
the newest family is the one that has to prove it can fail, and it fails on the chamber's output
assertion, which runs before the crafting and machine phases.

### What the harness taught this slice

1. **A new member of the `BLOCKS` array moves every hard-coded probe.** Adding the chamber shifted
   the A2 "unknown block id is refused" probe onto the Basic Chemical Tank's slot, and the phase
   failed on a *correct* block. The probe now stands at `8,64,26`, past the array. The same class as
   the End portal's bedrock: a harness constant that a content change invalidates.
2. **A charged machine has already started.** `expect_contains … "160000/160000 FE, progress 0/200"`
   after `mekanism energy` is a race the machine wins: by the next `/mekanism info` it had run 64
   ticks. The charged state is asserted as `gt 0`, and the exact 80,000 after the operation.
3. **`ctl container` may elide empty trailing slots** (a machine holding only its input answered one
   slot, not three), so every slot assertion indexes defensively (`held[2] if len(held) > 2 else
   None`) instead of assuming a length.
4. `E2E_TMP` is now honoured: the scratch home goes under the launcher's `~/.cache/horizon-e2e/scratch`
   when the leg exports it, instead of a 7.5 GB `/tmp` tmpfs that has been 100 % full.

### Gaps after slice 5

Everything in *Gaps / deviations*, *Gaps after the migration*, *Gaps after slice 3* and *Gaps after
slice 4* still stands. Changed: the **injection chamber is ported**, so the ore-processing chain is
now smelter · enrichment · crusher · injection, and the port ships the three shards as items. Still
missing for the rest of the chain: the Purification Chamber (oxygen, still unported) and the
Ore Washer (water), the clumps/crystals/slurries, the factories, and the chemicals as items.

### `## Upstream defects fixed`

None found in this slice. Two upstream numbers were **misread first and then read correctly from the
source**, which is worth recording because a port that guessed would have shipped them wrong:

- `UsageConfig.chemicalInjectionChamber` is **400 FE/t** (not the 50 of the three simpler machines),
  so an operation costs 80,000 FE and the buffer is 160,000;
- the per-tick gas usage is `recipe.amount × 1/tick` for 200 ticks (`constantUse`), not the
  `chemical_input.amount` as a flat per-operation cost — which is why a coral block's `amount: 5`
  row costs 1,000 units.

### Deviations this slice records

- **The machine pulls its gas; upstream is pushed into.** Upstream's tank is a 210-unit buffer that
  automation fills through the chemical capability; the machine runs only when the tank already holds
  a tick's worth. This port keeps the buffer (210, insert-only, answered through
  `mekanism:chemical`) *and* lets the machine pull one tick's gas from an adjacent handler through
  the same key, because the slice is the capability's consumer. A whole operation therefore draws
  exactly `200 × per_tick` from the neighbour and leaves the machine's tank empty. A push-driven
  setup still works: the tank accepts inserts up to 210.
- **`randomizedConsumption` is not modelled.** Upstream's default (`true`) makes the gas usage
  statistical (a Poisson mean around one unit per tick); the port has no config, so it spends the
  deterministic `constantUse` amount — the same expected cost, exactly reproducible.
- **The chamber's own tank is not shown in `/mekanism info`** (the line stays the six-field machine
  line the other machines answer); it is read through `mekanism:chemical`
  (`/mekanism chemical read <pos>` → `0/210`) and through its saved `chemical`/`amount`.
- **The block is the barrel template** (`facing`×`open`, the idle front), not upstream's
  `facing`×`active` blockstate; the `_active` model and textures are carried, unused.
- **No upgrades, no factory, no ejector, no side configuration** — the same recorded gaps as the
  other machines.
- **The pull resolves because the provider is this same crate.** A capability lookup downcasts by the
  handler's **Rust type**, so `LevelRef::capability(host, &CHEMICAL, …)` answers only when the
  provider is this crate's `ChemicalTank` behind this crate's `Chemical` type — the limit
  `HANDOVER.md` §9k measured with `mekanisticrouters` (a second crate's same-named key and its own
  `Chemical` type is passed over). That is exactly the case here, so the consumer half works; a
  *third-party* consumer would need the shared `ChemicalResource` §9k asks `horizon-mod-vocab` for,
  and this slice deliberately does not try to solve it.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

### Changelog (0.2.4) — supersedes 0.2.3

> **0.2.4 — the Chemical Injection Chamber, the chemical capability's first consumer.** A new machine
> on the shared recipe-machine core with a gas input: **400 FE/t × 200 ticks (80,000 FE an
> operation)**, a **160,000 FE** insert-only buffer, a **210-unit gas tank of its own** (insert-only,
> answered through `mekanism:chemical`), the same three slots and vanilla furnace menu, and the jar's
> `mekanism:injecting` table over the items this port has — **72 rows**: copper oxidising, concrete
> solidifying, coral reviving, dirt → mud, terracotta → clay, and ore/raw ore/raw block →
> **shards** for iron, gold and copper, with the three `mekanism:shard_*` items carried (models,
> textures, tab, lang). The machine **pulls one tick's gas at a time from a neighbour's tank through
> `mekanism:chemical`** and spends exactly `200 × per_tick` units an operation, leaving its own tank
> empty — the capability's consumer half, proven in game with the operation's FE, the product and the
> neighbour tank's delta all asserted, with no-tank and wrong-gas controls. Package: **122 entries,
> 105,226 B**, sha256 `dc37efbcea33fc5e7c0feb26fa2a126f947b107aa566dd90dc9cd91df49dac6c`; `check`
> exit 0; `fits` `made for` at the pin and `fits` at `0b14f8a`; 72 tests, clippy `-D warnings` clean;
> in game `PASS` + `EXIT:0` with all 72 injecting rows, the unmodded game `EXIT:1`, `SELFTEST=1`
> `EXIT:1`, `NOCABLE=1` `PASS` + `EXIT:0`.

## Slice 6 — the Purification Chamber (0.2.5, 2026-11-21)

### The scope authority, quoted

The slice comes from the port's own gap list. `notes.md` "Gaps / deviations" item 1 and
`FINDINGS.md` "Next slices" 1 both name it:

> Ore processing beyond smelting, enriching, crushing and injection (**Purification** and the Ore
> Washer — both need a second gas/fluid input — plus factories and the rest of the chain from the
> shards onward: the shards → clumps → dirty dust → dust steps need the purification chamber and the
> crusher on this port's items), …

> … next are the **Purification Chamber** (needs oxygen) and the Ore Washer (needs water as a gas).
> Both are `ItemStackChemicalToItemStack` machines of the same shape, so each is a slice like this
> one — with its own recipe table and a second gas id the table already validates against.

**Chosen: the Purification Chamber** (`mekanism:purifying`) — the same gas machine the injection
chamber is, on **oxygen**, with the jar's clump table. It fits the **`deep` tier** as a
machines-shaped feature (one block entity with its own saved state, a container, an energy balance
and a capability consumer), same as slice 5.

### One brief/artefact disagreement, and what the jar says

The brief for this slice asked to prove "the dirty dust back into this chamber gives a dust". The
jar does not: `data/mekanism/recipe/processing/<material>/dust/from_dirty_dust.json` is
`"type": "mekanism:enriching"`, and `mekanism:purifying` has **28** recipes, every one of them a
`processing/<material>/clump/from_*.json`. So the port follows the artefact: the chain is proven as
**clump → crusher → dirty dust → enrichment chamber → dust**, and phase I3 also puts a dirty dust
into a purification chamber as a counterfactual (no progress, no FE spent) so the table's rule is
two-sided. The alternative — inventing a purifying row — would have been a defect, not fidelity.

### What the jar says (upstream truth, read from `v1.21.1-10.7.19.85`)

| value | source | port |
|---|---|---|
| usage **200 FE/t** | `UsageConfig.java:56` (`purificationChamber`, default `200L`) | `PURIFY_USAGE = 200` |
| storage **80,000 FE** | `StorageConfig.java:57-61` (`purificationChamber`, `80_000L, 1`) | `PURIFY_CAPACITY = 80_000` |
| 200 ticks per operation, so **40,000 FE** | `TileEntityPurificationChamber` → `TileEntityAdvancedElectricMachine(BASE_TICKS_REQUIRED)` | `PURIFY_OPERATION = { 200, 200 }` |
| the machine's own gas tank is **210** units | `TileEntityAdvancedElectricMachine.java:66` (`MAX_GAS = 210`) | `PURIFY_TANK = 210` |
| insert-only for automation | `useStatisticalMechanics()` is the config default `true` → `allowExtractingChemical()` false → `ConstantPredicates.notExternal()` | answered through `mekanism:chemical`, inserts up to 210, foreign extract takes nothing |
| the gas is spent per tick | `chemical_input.amount` under `per_tick_usage: true`, through `ChemicalUsageMultiplier.constantUse` (1 unit/tick) | `per_tick` per operating tick, so an operation takes `per_tick × 200` |
| no gas (or another gas) **holds** the progress | the constant-usage handler overrides `resetProgress` to `updateOperations(0)` | `IdleReason::NoChemical` (held, no FE spent) |
| recipe type `mekanism:purifying` | `TileEntityPurificationChamber.getRecipeType()` | `runtime::purifying` |
| block 3.5 / 16, `facing`×`active` | `MekanismBlockTypes.PURIFICATION_CHAMBER` (`createFactoryMachine`), `blockstates/purification_chamber.json` | barrel template (3.5/16, `facing`×`open`), the same recorded deviation as the other machines |

The numbers were re-read for **this** machine rather than carried over from slice 5: it is **200
FE/t**, not the injection chamber's 400 and not the three simpler machines' 50, so an operation
costs 40,000 FE and the buffer is 80,000.

### The recipe table: 15 of the jar's 28 purifying recipes

`tools/gen_purifying.py` reads every `"type": "mekanism:purifying"` file in the jar and keeps a row
where both sides exist in this port:

- **item tags expanded to their 26.3 members** (`c:ores/{iron,gold,copper}`, `c:raw_materials/*`,
  `c:storage_blocks/raw_*`, `c:shards/{iron,gold,copper}`), each expansion its own row;
- the jar's 28 recipes are 4 files × 7 materials; the port registers iron, gold and copper, so 12
  recipes expand to **15 rows**: ore ×1 → **3** clumps (2 ores each for iron and copper, gold's two
  26.3 ores in the same expansion the other tables use), raw ore ×1 → **2**, raw block ×1 (+2 gas)
  → **18**, shard ×1 → **1**;
- **16 skipped, each printed**: the lead/osmium/tin/uranium ore, raw and shard tags — no such item
  in this port.

### The machine, in the port's three layers

- **`src/runtime.rs`** — `GasRecipe` (renamed from `InjectionRecipe`: the two machines run the same
  `ItemStackChemicalToItemStack` recipe type, so one shape serves both), `gas_pull` and
  `process_gas_tick` (renamed with it), `purifying::{recipe_for, lookup, ROW_COUNT}` and the five
  `PURIFY_*` constants.
- **`src/support.rs`** — `GasPart { operation, capacity, tank, recipe_for }` replaces the
  injection-only `InjectionPart`; the two machines are two `GasDecl` rows and one `Part`. The
  registration still answers `mekanism:chemical` with the machine's own 210-unit tank.
- **Content** — `mekanism:purification_chamber` (barrel template 3.5/16, the upstream blockstate's
  `facing`×`open`, the upstream model, `_active` model and all 12 textures), the three
  `mekanism:clump_{iron,gold,copper}` and three `mekanism:dirty_dust_*` items (the jar's
  `item/generated` models and textures), loot, the shaped recipe (vanilla stand-ins: 4 gold + 2
  redstone + 2 copper around an **Enrichment Chamber**, the upstream `ACA/OPO/ACA` pattern over
  ingredients this port has), the pickaxe tag, the creative tab and the lang keys.

Registering the clumps and dirty dusts also carries the two rows the chain needs, which the crusher
and enrichment generators had been printing as skipped: `c:clumps/*` → 1 dirty dust (crusher,
**145 → 148** rows) and `c:dirty_dusts/*` → 1 dust (enriching, **129 → 132** rows).

### Red → green

The runtime rules and the glue's declaration were written as tests first and ran red on the
untouched tree (`../_shared/evidence/mekanism/slice6-unit-red.log`):

```
error[E0425]: cannot find value `PURIFICATION` in this scope
    --> src/support.rs:1328:37
     |
1328 |         assert_eq!(machine_capacity(PURIFICATION), Some(PURIFY_CAPACITY));
     |                                     ^^^^^^^^^^^^ not found in this scope
…
error: could not compile `mekanism` (lib test) due to 34 previous errors
```

Then the constants, the generator's table, the shared gas core and the `GasPart` made the crate
green: **78 tests** (was 72) — 5 new `runtime::purification_tests`, 1 `support::tests`.

### Coverage — every family this slice ships

| family | status | proof |
|---|---|---|
| the Purification Chamber block, block entity, 80,000 FE buffer, three slots, furnace menu | **tested** | `A1`/`A2` (give/place/read back), `B2` (loot), `B1` (crafts), `I1` (`0/80000 FE, ...`) |
| one purification operation: 40,000 FE, 200 ticks, the input consumed, the jar's output stack | **tested** | `I1`: `iron_ore x1` → `mekanism:clump_iron x3`, `energy eq 40000` |
| oxygen pulled **through the capability** from a neighbour, one tick's worth per tick | **tested** | `I1`: tank `4000 → 3800`, machine's own tank `0` (`holds nothing 0/210 (answered by mekanism)`) |
| no tank beside it → no progress, no FE spent | **tested** | `I2` (4,66,16): `progress 0/200, input minecraft:iron_ore x1`, `eq 80000` |
| hydrogen chloride (the injection chamber's gas) → no progress, no FE, nothing drawn | **tested** | `I2` (7/8,66,16): same idle line, tank still `4000/64000`; sweep control: a hydrogen chloride tank at an iron-ore chamber, `5000/64000` unchanged |
| **all 15 carried purifying recipes** (input count, output id and count, input consumed, tank short exactly `200 × per_tick`) | **tested** | `J`: `rows purifying: 15 of 15` (`tools/gas_sweep.py --kind purifying`) |
| the jar's chain: clump → crusher → dirty dust | **tested** | `I3`: crusher reads `output mekanism:dirty_dust_iron x1`, `10000/20000 FE` left |
| dirty dust → dust (the jar's **enriching** row) | **tested** | `I3`: enrichment chamber reads `output mekanism:dust_iron x1`, `10000/20000 FE` left |
| a dirty dust in a purification chamber makes nothing (counterfactual) | **tested** | `I3`: `progress 0/200, input mekanism:dirty_dust_iron x1`, `eq 80000` |
| the three clumps and three dirty dusts: items, models, textures, give-able | **tested** | `A1` (6 items give-able) |
| the nine carried shaped recipes | **tested** | `B1` (`tools/craft_check.py`): nine crafts + six wrong-ingredient controls |
| the eight blocks' loot tables | **tested** | `B2`: each block drops its own item |
| lead/osmium/tin/uranium clumps and their ores | **not ported** | those items are not registered; the generator prints each skipped file |
| the `active` front texture and the `_active` model | **not ported** | carried in `assets/`, unused: the block template's `open` state stands in for `active` (the other machines' recorded deviation) |
| factories (purifying factory), upgrades, side configuration, the machine's own GUI | **not ported** | out of this slice (the same gaps as the other machines) |

### The ladder (one lease per step, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result |
|---|---|---|
| runtime + glue tests (RED) | `cargo test --offline` on the untouched tree | **34 errors**, the new API missing (`slice6-unit-red.log`) |
| tests | `cargo test --offline` | **78 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` | clean (`sdk-fmt.log`) |
| recipe tables | `gen_smelting.py --check`, `gen_enriching.py --check`, `gen_crushing.py --check`, `gen_injecting.py --check`, `gen_purifying.py --check` | 163 / **132** / **148** / 72 / **15** rows (`slice6-recipe-tables.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `Built ./target/horizon-mod/mekanism.hmod` (`sdk-build.log`) |
| same package twice | the same again | **`cmp` identical** (`sdk-build-second.log`, `sdk-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | `mekanism 0.2.5 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **124,275 B, SHA-256 `ac8289c8…3828`**, exit 0 (`sdk-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, `FIT-PIN-EXIT:0` (`sdk-fits.log`, sha printed first) |
| fits, **a different build** | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, `FIT-OTHER-EXIT:0` (`sdk-fits.log`) — the run that proves something |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (159 entries, no packed `Cargo.lock`) |
| archive shape | `unzip -l mekanism.hmod` | 159 files, no `Cargo.lock` (`sdk-entries.txt`) |

The launcher build (`launcher-game.sh build mekanism`, in a lease) printed `Compiling mekanism (1 of
1)` and produced `native-0a875da-6e2fafb2fa07`, which is the trap's check that the binary under test
carries this package.

### In-game evidence (`--offscreen`, the launcher build)

`../_shared/evidence/mekanism-e2e/e2e-package.log` → `PASS` + `EXIT:0`, quoted from phases I and J:

```
== I1: one iron ore becomes three clumps for 40,000 FE and exactly 200 oxygen ==
  chemical insert 4000 mekanism:oxygen 0 66 16: chemical: inserted 4000 mekanism:oxygen into mekanism:basic_chemical_tank (4000/64000)
  info(1,66,16): mekanism:purification_chamber: 0/80000 FE, progress 0/200, input minecraft:iron_ore x1, output empty x0
  energy(1,66,16): 78800
  info(1,66,16): mekanism:purification_chamber: 40000/80000 FE, progress 0/200, input empty x0, output mekanism:clump_iron x3
  energy(1,66,16): 40000
  chemical read 0 66 16: chemical: mekanism:basic_chemical_tank holds mekanism:oxygen 3800/64000 (answered by mekanism)
  data get block 0 66 16 amount: 0, 66, 16 has the following block data: 3800
  data get block 1 66 16 amount: 1, 66, 16 has the following block data: 0
  chemical read 1 66 16: chemical: mekanism:purification_chamber holds nothing 0/210 (answered by mekanism)
== I2: without oxygen, or with the injection chamber's hydrogen chloride, nothing moves ==
  chemical insert 4000 mekanism:hydrogen_chloride 7 66 16: chemical: inserted 4000 mekanism:hydrogen_chloride into mekanism:basic_chemical_tank (4000/64000)
  info(4,66,16): mekanism:purification_chamber: 80000/80000 FE, progress 0/200, input minecraft:iron_ore x1, output empty x0
  energy(4,66,16): 80000
  info(8,66,16): mekanism:purification_chamber: 80000/80000 FE, progress 0/200, input minecraft:iron_ore x1, output empty x0
  energy(8,66,16): 80000
  chemical read 7 66 16: chemical: mekanism:basic_chemical_tank holds mekanism:hydrogen_chloride 4000/64000 (answered by mekanism)
== I3: the jar's chain: clump -> crusher -> dirty dust -> enriching chamber -> dust ==
  info(0,68,16): mekanism:crusher: 10000/20000 FE, progress 0/200, input empty x0, output mekanism:dirty_dust_iron x1
  energy(0,68,16): 10000
  info(3,68,16): mekanism:enrichment_chamber: 10000/20000 FE, progress 0/200, input empty x0, output mekanism:dust_iron x1
  energy(3,68,16): 10000
  info(6,68,16): mekanism:purification_chamber: 80000/80000 FE, progress 0/200, input mekanism:dirty_dust_iron x1, output empty x0
  energy(6,68,16): 80000
  a clump crushed into a dirty dust and that dirty dust enriched into a dust; the purification chamber has no row for a dirty dust
== J: every carried purifying recipe in a real Purification Chamber ==
  rows from the jar's mekanism:purifying data: 15
  minecraft:copper_ore x1 + mekanism:oxygen x1/t -> {"count": 3, "item": "mekanism:clump_copper"} (chemical: mekanism:basic_chemical_tank holds mekanism:oxygen 4800/64000 (answered by mekanism))
  …
  minecraft:raw_iron_block x1 + mekanism:oxygen x2/t -> {"count": 18, "item": "mekanism:clump_iron"} (chemical: mekanism:basic_chemical_tank holds mekanism:oxygen 4600/64000 (answered by mekanism))
rows purifying: 15 of 15
sweep PASS: all 15 carried purifying recipes ran in a real machine with their declared input count and output stack, the input was consumed, and the neighbour tank lost exactly the gas the operation spent
== control: a tank of a gas the purifying recipe does not take moves nothing ==
  gas: chemical: mekanism:basic_chemical_tank holds mekanism:hydrogen_chloride 5000/64000 (answered by mekanism)
  machine: [{"count": 1, "item": "minecraft:iron_ore"}], 80000 FE
  control PASS: the input is still there, nothing was made and no gas was drawn
```

What the numbers prove:

- **the operation is exactly one operation at the jar's price**: 200 ticks × 200 FE = 40,000 FE —
  the buffer goes 80,000 → **40,000**, the input leaves the slot and slot 2 gains the jar's
  **three** `mekanism:clump_iron`;
- **the oxygen is the recipe's, per tick**: the neighbour tank goes 4,000 → **3,800** (exactly
  `200 × 1`), and the machine's own tank reads **0**, so nothing was manufactured or over-drawn;
- **the controls are two-sided**: with no tank the machine held its progress at 0 and its 80,000 FE
  intact; with the **injection chamber's** hydrogen chloride beside it nothing moved either, and the
  tank was not drawn from at all — the gas id in the table is what decides, not the presence of a
  tank;
- **every row costs what it declares**: a one-unit row leaves 4,800, the raw-block rows (2/tick)
  leave 4,600, and each produced the jar's stack (18 for a raw block, 3 for an ore, 2 for a raw ore,
  1 for a shard);
- **the chain runs end to end**: the clump crushed into a dirty dust for 10,000 FE, that dirty dust
  enriched into a dust for 10,000 FE, and the same dirty dust in a purification chamber made
  nothing with all 80,000 FE intact.

| leg | result | evidence |
|---|---|---|
| package (default) | **`PASS` + `EXIT:0`** | `e2e-package.log` |
| unmodded game (`GAME=$CONTROL_GAME CONTROL=0`) | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** | `e2e-control.log` |
| fail-capable (`SELFTEST=1`) | **`FAIL: expected /mekanism info at 1,66,16 to say 'output empty x0', got 'mekanism:purification_chamber: 40000/80000 FE, progress 0/200, input empty x0, output mekanism:clump_iron x3'` + `EXIT:1`** | `e2e-selftest.log` |
| differential (`NOCABLE=1`) | **`PASS` + `EXIT:0`** (phase I passes with no cable too) | `e2e-nocable.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |

The `SELFTEST=1` flip moved to **this** slice's assertion (it was the injection chamber's output):
the newest family is the one that has to prove it can fail.

### What the harness taught this slice

1. **A Python tool's stdout is block-buffered when it is piped**, so a sweep that prints only per
   batch can go silent for minutes while it is working: the first D-sweep run was killed by
   `e2e-guard`'s 900 s idle limit with no output after `rows … 148` (kept as
   `../_shared/evidence/mekanism/e2e-attempt-idle-timeout.log`). Every tool call in `tools/e2e.sh`
   is `python3 -u` now, so the guard sees real liveness; the run then passed. The guard itself is
   real: its `EXIT:124` was a scheduling signal, not a port defect.
2. **A new member of the `BLOCKS` array moves the A2 probe again**: the unknown-id probe stood at
   `8,64,26`, which is now the eighth block's slot, so it moved to `10,64,26`. (Same class as slice
   5's note.)
3. **The brief's chain step and the jar disagree**: `from_dirty_dust` is `mekanism:enriching`, not
   `mekanism:purifying` (see above). The e2e therefore proves the jar's chain and keeps the
   purification chamber's missing row as a counterfactual, rather than inventing a recipe.

### Gaps after slice 6

Everything in *Gaps / deviations*, *Gaps after the migration*, *Gaps after slice 3*, *Gaps after
slice 4* and *Gaps after slice 5* still stands. Changed: the **purification chamber is ported**, so
the ore-processing chain is now smelter · enrichment · crusher · injection · purification, and the
port ships the three clumps and three dirty dusts, which also carries the crusher's clump rows and
the enrichment chamber's dirty-dust rows. Still missing: the **Ore Washer** (water as a gas), the
crystals and slurries, the factories, the chemicals as items, and the `c:ores/gold` expansion still
omits `minecraft:nether_gold_ore` in all three ore-table generators (a pre-existing gap of the tag
expansion, not introduced here).

### `## Upstream defects fixed`

None found in this slice. The one candidate — a purification chamber "should" turn a dirty dust into
a dust — is not a defect: the jar's own data puts that step in the enrichment chamber, and the port
follows the data.

### Deviations this slice records

- **The machine pulls its gas; upstream is pushed into** — the slice-5 deviation, unchanged and for
  the same reason (this slice is the same consumer through `mekanism:chemical`).
- **`randomizedConsumption` is not modelled**: the deterministic `constantUse` amount (1 unit/tick)
  is spent instead of the statistical mean, exactly as the injection chamber records.
- **The block is the barrel template** (`facing`×`open`), not upstream's `facing`×`active`; the
  `_active` model and 12 textures are carried, unused.
- **No upgrades, no factory, no ejector, no side configuration, no GUI** — the other machines' gaps.
- **The purification chamber has no recipe for a dirty dust** — the jar's rule, proven in `I3`.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

### Changelog (0.2.5) — supersedes 0.2.4

> **0.2.5 — the Purification Chamber, the jar's clump table and the ore-processing chain's middle
> step.** A new machine on the shared gas-machine core (the same `ItemStackChemicalToItemStack`
> machine the injection chamber is): **200 FE/t × 200 ticks (40,000 FE an operation)**, an **80,000
> FE** insert-only buffer, the same **210-unit** oxygen tank answered through `mekanism:chemical`,
> the same three slots and vanilla furnace menu, and the jar's `mekanism:purifying` table over the
> items this port has — **15 rows**: ore → **3 clumps**, raw ore → **2**, raw block → **18** and
> shard → **1** for iron, gold and copper, with the six `mekanism:clump_*` and
> `mekanism:dirty_dust_*` items carried (models, textures, tab, lang). Registering them also carries
> the crusher's clump → dirty dust rows (145 → 148) and the enrichment chamber's dirty dust → dust
> rows (129 → 132), so the jar's chain is complete in game: clump → crusher → dirty dust → enriching
> chamber → dust, while a purification chamber has no row for a dirty dust (the jar's
> `from_dirty_dust` is an enriching recipe). The machine pulls one tick's oxygen at a time from a
> neighbour tank through `mekanism:chemical` and spends exactly `200 × per_tick` units an operation.
> Package: **159 entries, 124,275 B**, sha256
> `ac8289c802ffdba25a1b3f7a96fea5c0904b8c7a65d20ca49417dea6f9363828`; `check` exit 0; `fits` `made
> for` at the pin and `fits` at `0b14f8a`; 78 tests, clippy `-D warnings` clean; in game `PASS` +
> `EXIT:0` with all 15 purifying rows and the chain proven, the unmodded game `EXIT:1`, `SELFTEST=1`
> `EXIT:1`, `NOCABLE=1` `PASS` + `EXIT:0`.

## Slice 7 — the basic factory family (0.2.6, 2026-11-22)

### The scope authority, quoted

The slice is the port's own next gap, named by both lists:

> **Factories** (`basic_smelting_factory` etc.): the same recipe in parallel, hearts of the
> `TileEntityFactory` family; needs multi-operation-per-tick (`operationsPerTick`) and a larger
> energy buffer. (`FINDINGS.md` "Next slices" 2)

> Ore processing beyond smelting, enriching, crushing and injection … plus factories and the rest of
> the chain … (`notes.md` "Gaps / deviations" 1)

and the 0.2.5 report's own next-slice proposal: **the basic factory family — `mekanism:basic_purifying_factory`
plus its four siblings**, the same recipes in parallel, `operationsPerTick`, a larger buffer and one
input/output pair per process.

**Chosen: all five Basic factories** (`basic_smelting_factory`, `basic_enriching_factory`,
`basic_crushing_factory`, `basic_injecting_factory`, `basic_purifying_factory`) — one block per item
machine the port already runs. It fits the **`deep` tier** as a machines-shaped feature (five block
entities with their own saved state, containers, energy balances, and two capability consumers), and
it ships nothing the port does not already have a recipe table for.

### What the jar says (upstream truth, re-read for this family)

Every number was re-read from `v1.21.1-10.7.19.85`; none is carried over from slices 5–6, whose
usage figures differ from each other and from these.

| value | source | port |
|---|---|---|
| **three processes** in a basic factory | `common/tier/FactoryTier.java` — `BASIC(BaseTier.BASIC, 3)` | `FACTORY_PROCESSES = 3` |
| **buffer = max(base storage / 2, base usage) × processes** | `common/content/blocktype/Factory.java` — `setMachineData`: `clampToLong(Math.max(origEnergy.getConfigStorage() * 0.5, origEnergy.getUsage()) * tier.processes)` | `factory_capacity`, so **30,000** FE for smelting/enriching/crushing (20,000/2 × 3), **240,000** for injecting (160,000/2 × 3) and **120,000** for purifying (80,000/2 × 3) |
| **usage is the base machine's, per process** | `MekanismBlockTypes.…withEnergyConfig(MekanismConfig.usage.<machine>, …)`; `UsageConfig.java`: smelter/enrichment/crusher **50**, purification **200**, injection **400** | `SMELTER_OPERATION`/`CHAMBER_OPERATION`/`CRUSHER_OPERATION`/`PURIFY_OPERATION`/`INJECTION_OPERATION` — a working process spends its own tick, so three working processes cost 3 × usage per tick |
| **200 ticks per operation** | `TileEntityFactory.BASE_TICKS_REQUIRED = 10 * TICKS_PER_SECOND` | `FACTORY_TICKS = 200` |
| **one input and one output slot per process** | `TileEntityItemToItemFactory.addSlots`: a `FactoryInputInventorySlot` and an `OutputInventorySlot` at x = base + i × mult, one `FactoryRecipeCacheLookupMonitor` per process | slots 0–2 inputs, 3–5 outputs; one progress per process |
| **`operationsPerTick`** | `TileEntityFactory.recalculateUpgrades`: `MekanismUtils.getOperationsPerTick(this, BASE_TICKS_REQUIRED, 1)`; `MekanismUtils.java:339` — one per tick unless a speed-upgraded operation takes less than a tick | `operations_per_tick(ticks × 10, 1)`; the port ships no upgrades, so **1**, and the rule is unit-tested |
| **one shared tank of `MAX_GAS` × processes** | `TileEntityItemStackChemicalToItemStackFactory.getInitialChemicalTanks`: `BasicChemicalTank.inputModern(capacity * tier.processes, …)`, `cap = AttributeFactoryType…` → `TileEntityAdvancedElectricMachine.MAX_GAS` (**210**) | `INJECTING_FACTORY_TANK` = `PURIFYING_FACTORY_TANK` = **630** |
| **one energy container for all processes** | `TileEntityFactory.getInitialEnergyContainers`: one `MachineEnergyContainer.input(this, …)`; `onUpdateServer` runs each process's monitor against it | one buffer for the block; every process's tick draws from it |
| recipe type per factory type | `TileEntityItemStackToItemStackFactory.getRecipeType` (SMELTING/ENRICHING/CRUSHING) and `TileEntityItemStackChemicalToItemStackFactory.getRecipeType` (INJECTING/PURIFYING) | the existing `smelting`, `enriching`, `crushing`, `injecting`, `purifying` tables |
| block 3.5 / 16, `facing` × `active` | `MekanismBlockTypes.getFactory(tier, type)` → `FactoryBuilder`, `blockstates/basic_<type>_factory.json` | barrel template (3.5/16, stone, requires a tool), `facing` × `open` — the other machines' recorded deviation |
| recipe `mekanism:mek_data`, `ACA/IPI/ACA` over `mekanism:alloys/basic`, `c:circuits/basic`, `c:ingots/iron` and the base machine | `data/mekanism/recipe/factory/basic/<type>.json` | `minecraft:crafting_shaped`, the same pattern and base machine, gold/redstone/iron as the stand-ins (the precedent of the nine earlier recipes) |

### The family, in the port's three layers

- **`src/runtime.rs`** — the `FACTORY_*` constants and the pure rules: `factory_capacity`
  (`Factory.setMachineData`), `factory_tank`, `operations_per_tick` (`getOperationsPerTick`), the
  `FactoryProcess` (progress + its own input/output pair), `process_factory_tick` and
  `process_gas_factory_tick` (the existing single-machine cores run once per process over one shared
  buffer/tank), and `factory_gas_want` (one tick's gas per **working** process, less what the tank
  already holds, and nothing at all while the buffer cannot pay for a tick).
- **`src/support.rs`** — `FactorPart`/`GasFactoryPart` and the two declaration tables
  (`FactoryDecl`, `GasFactoryDecl`), a six-slot container (`factory_container`, no menu), the
  `factory_capacity`/`factory_ticks` entries in the two tables `/mekanism` reads, the factory branch
  of `/mekanism info` (the sum of the three process slots, so nine clumps in three output slots
  read `x9`), the `progress` **int array** the factory saves (upstream's `SerializationConstants.PROGRESS`),
  and the 630-unit chemical capability on the two gas factories.
- **Content** — five blocks (`basic_{smelting,enriching,crushing,injecting,purifying}_factory`) with
  their blockstates/models/textures (the jar's per-type fronts, sides, backs, tops and bottoms, plus
  the shared back panel and the enriching `elements` texture), item models, block declarations,
  loot tables, the five shaped recipes, the pickaxe tag, the creative tab and the lang keys.

### Red → green

The new rules were written as tests first and ran red on the untouched tree
(`../_shared/evidence/mekanism/slice7-unit-red.log`, then `slice7-unit-red-2.log` for the gas-ask
function):

```
error[E0425]: cannot find value `FACTORY_SLOTS` in this scope
error[E0425]: cannot find value `SMELTING_FACTORY` in this scope
...
error[E0425]: cannot find function `factory_gas_want` in this scope
error: could not compile `mekanism` (lib test) due to 71 previous errors
```

Then the constants, the two tick loops and the declarations made the crate green: **92 tests**
(was 78) — 11 new `runtime::factory_tests` and 3 new `support::factory_support_tests`.

### Coverage — every family this slice ships

| family | status | proof |
|---|---|---|
| the five factory blocks, block entities, buffers and six slots | **tested** | `A1`/`A2` (13 blocks give-able/place/read back), `B2` (loot), `B1` (the five craft), `K1`/`K4` (`/mekanism info` reports each buffer and the three-process line) |
| a basic purifying factory: three ore in three processes → 9 clumps, 3 × 40,000 FE, 3 × 200 oxygen | **tested** | `K1`: `input empty x0, output mekanism:clump_iron x9`, `energy eq 0`, tank `3400/64000`; unit: `three_ore_in_three_processes_make_three_operations_side_by_side`, `a_gas_factory_spends_one_tick_of_each_process_s_gas` |
| **one** ore in one process is **one** operation (3 clumps, 40,000 FE, 200 gas) | **tested** | `K2`: `output mekanism:clump_iron x3`, `energy eq 80000`, tank `3800/64000`; unit: `one_process_working_costs_one_operation_not_the_whole_factory` |
| the buffer is shared per process: 40,000 FE buys 200 progress ticks, not three operations | **tested** | `K3`: `0/120000 FE (3 processes), progress 67/200, input minecraft:iron_ore x3, output empty x0`, `energy eq 0`, tank `3800/64000`; unit: `the_shared_buffer_pays_for_every_working_process` |
| a basic smelting factory: three ore → three ingots for the 30,000 FE buffer | **tested** | `K4`: `output minecraft:iron_ingot x3`, `energy eq 0` |
| a factory with an input it has no recipe for burns nothing | **tested** | `K5`: `progress 0/200, input minecraft:diamond x1`, `eq 30000` |
| the factories' FE, progress array and slots survive save + relaunch | **tested** | persistence phase: the drained K3 factory reads back `0/120000 FE (3 processes), progress 67/200, input minecraft:iron_ore x3` |
| `operationsPerTick` | **tested** | unit `a_whole_tick_operation_is_one_operation_per_tick` (200 ticks → 1; 0.5 tick → 2) |
| the five recipes craft, ingredients consumed | **tested** | `B1` (`craft_check.py`) and one wrong-ingredient factory control |
| the `active` front texture and the `_active` models | **not ported** | carried, unused: the block template's `open` stands in for `active` (the other machines' deviation) |
| the factories' own GUI (six slots, three progress bars, sorting) | **not ported** | `menu: None`; `/mekanism info` and hoppers reach the six slots |
| upgrades, side configuration, ejector, sorting | **not ported** | no upgrade system in this port, so `operationsPerTick` is the baseline 1 |
| advanced/elite/ultimate factories | **not ported** | `FactoryTier` rows 5/7/9; the five basic blocks are the slice |

### The ladder (one lease per step, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result |
|---|---|---|
| runtime + glue tests (RED) | `cargo test --offline` on the untouched tree | **71 errors**, the new API missing (`slice7-unit-red.log`; the gas ask, `slice7-unit-red-2.log`, 32 errors) |
| tests | `cargo test --offline` | **92 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` | clean (`sdk-fmt.log`) |
| recipe tables | the five `gen_*.py --check` (`HORIZON_RES` = the prepared 26.3 root) | 163 / 132 / 148 / 72 / 15 rows (`slice7-recipe-tables.log`) |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `Built ./target/horizon-mod/mekanism.hmod` (`slice7-build.log`) |
| same package twice | the same again | **`cmp` identical** (`slice7-build-second.log`, `slice7-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | `mekanism 0.2.6 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **161,073 B, SHA-256 `6633dde7…0e67`**, exit 0 (`slice7-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, `FIT-PIN-EXIT:0` (`slice7-fits.log`, sha printed first) |
| fits, **a different build** | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, `FIT-OTHER-EXIT:0` (`slice7-fits.log`) |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (220 entries, no packed `Cargo.lock`) |
| archive shape | `unzip -l mekanism.hmod` | 220 files, no `Cargo.lock` |

The launcher build (`launcher-game.sh build mekanism`, in a lease) printed `Compiling mekanism (1 of
1)` and produced `native-0a875da-c9d32f247453` (`slice7-launcher-build.log`). The stale
`native-0a875da-ec80a5edc241` build (same 0.2.6 manifest, older code) was removed first — the
launcher reuses a build when the manifest is byte-identical, so an edit inside an unpublished
version is invisible until the old `versions/native-*` folder and its export are gone.

### In-game evidence (`--offscreen`, the launcher build)

`../_shared/evidence/mekanism-e2e/e2e-package.log` → `PASS` + `EXIT:0`, quoted from phase K and the
persistence phase:

```
== K1: three ore in three processes become nine clumps for 600 oxygen and 120,000 FE ==
  chemical insert 4000 mekanism:oxygen 0 70 0: chemical: inserted 4000 mekanism:oxygen into mekanism:basic_chemical_tank (4000/64000)
  info(1,70,0): mekanism:basic_purifying_factory: 0/120000 FE (3 processes), progress 0/200, input minecraft:iron_ore x3, output empty x0
== K2: one ore in one process is one operation, not three ==
== K3: the buffer is shared: 40,000 FE buys 200 progress ticks, not three operations ==
== K4: the item factory path: three ore smelt side by side for the 30,000 FE buffer ==
  info(0,70,12): mekanism:basic_smelting_factory: 0/30000 FE (3 processes), progress 0/200, input minecraft:iron_ore x3, output empty x0
== K5: a factory with an input it has no recipe for makes no progress and burns nothing ==
  info(1,70,0): mekanism:basic_purifying_factory: 0/120000 FE (3 processes), progress 0/200, input empty x0, output mekanism:clump_iron x9
  energy(1,70,0): 0
  chemical read 0 70 0: chemical: mekanism:basic_chemical_tank holds mekanism:oxygen 3400/64000 (answered by mekanism)
  info(1,70,4): mekanism:basic_purifying_factory: 80000/120000 FE (3 processes), progress 0/200, input empty x0, output mekanism:clump_iron x3
  energy(1,70,4): 80000
  chemical read 0 70 4: chemical: mekanism:basic_chemical_tank holds mekanism:oxygen 3800/64000 (answered by mekanism)
  info(1,70,8): mekanism:basic_purifying_factory: 0/120000 FE (3 processes), progress 67/200, input minecraft:iron_ore x3, output empty x0
  energy(1,70,8): 0
  chemical read 0 70 8: chemical: mekanism:basic_chemical_tank holds mekanism:oxygen 3800/64000 (answered by mekanism)
  info(0,70,12): mekanism:basic_smelting_factory: 0/30000 FE (3 processes), progress 0/200, input empty x0, output minecraft:iron_ingot x3
  energy(0,70,12): 0
  info(0,70,16): mekanism:basic_smelting_factory: 30000/30000 FE (3 processes), progress 0/200, input minecraft:diamond x1, output empty x0
  energy(0,70,16): 30000
== persistence: save, quit and relaunch read the block entities back ==
  after relaunch, factory: mekanism:basic_purifying_factory: 0/120000 FE (3 processes), progress 67/200, input minecraft:iron_ore x3, output empty x0
```

and from phase A/B, the content and recipe half:

```
  13 blocks and 13 items are give-able
  13 blocks place and read back; an unknown id is refused
  13 blocks each dropped their own item
  basic purifying factory: 4 gold + 2 redstone + 2 iron + a mekanism:purification_chamber -> {"count": 1, "item": "mekanism:basic_purifying_factory", "slot": 200}
  (… the other four factories, the nine earlier recipes and the wrong-ingredient controls …)
```

What the numbers prove:

- **three processes, not three operations in a row**: three ore all leave their slots in the same
  200 ticks and the three output slots hold **nine** clumps; the buffer goes 120,000 → **0** and the
  neighbour tank 4,000 → **3,400**, so the electricity and the gas were charged **once per process**
  (`3 × 40,000 FE`, `3 × 200` units);
- **one ore costs one operation**: 3 clumps, the buffer at **80,000** (one 40,000 operation) and the
  tank at **3,800** (one 200-unit operation). A factory that silently ran one operation three times,
  or that charged per factory rather than per process, fails one of these two;
- **the shared buffer pays every working process**: with only 40,000 FE in a 120,000 FE buffer at
  3 × 200 FE/t, exactly **200** progress ticks happen (67 + 67 + 66), **no** operation completes and
  exactly **200** units of gas are drawn — a factory that charged 200 FE/t for the whole block would
  have finished all three;
- **the item path is the same core**: three ore become three ingots for the 30,000 FE buffer, and a
  diamond makes no progress with its 30,000 FE intact;
- **the state is the block entity's**: after `save`, quit and relaunch the drained factory still
  reads `progress 67/200` with its three ore in their three input slots.

| leg | result | evidence |
|---|---|---|
| package (default) | **`PASS` + `EXIT:0`** | `e2e-package.log` |
| unmodded game (`GAME=$CONTROL_GAME CONTROL=0`) | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** | `e2e-control.log` |
| fail-capable (`SELFTEST=1`) | **`FAIL: expected /mekanism info at 1,70,0 to say 'output empty x0', got … output mekanism:clump_iron x9` + `EXIT:1`** | `e2e-selftest.log` |
| differential (`NOCABLE=1`) | **`PASS` + `EXIT:0`** (the factory phases need no cable and still pass) | `e2e-nocable.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |
| regression | phases D/H/J | **148/148 crushing, 72/72 injecting, 15/15 purifying** rows still sweep |

The `SELFTEST=1` flip moved to **this** slice's assertion: the four older flips (crusher, smelter,
chamber, purification chamber) are now ordinary assertions, so the newest family is the one that
proves it can fail.

### What the harness taught this slice

1. **The first run found a real defect and the unit tests had not.** The factory drew its
   neighbour's gas while it had no energy, so `K3` read 3,797 units instead of 3,800: an unpowered
   factory sipped gas it could not use. `factory_gas_want` now takes the buffer and the operation
   and only counts the processes this tick can pay for, in the tick's own order; the regression test
   pins `energy 0 → no gas` and `one tick's energy → one process's gas`.
2. **The launcher reuses a build while the manifest is byte-identical.** After the gas fix the
   package was still 0.2.6, so `native-build` would have reused the pre-fix `native-0a875da-ec80a5edc241`;
   removing that version folder and its export made it print `Compiling mekanism (1 of 1)` again.
3. **A wandering Enderman killed the player during the survival crafting phase** on the no-cable
   leg. `arena()` now sets `difficulty peaceful` and `doMobSpawning false` alongside its
   `kill @e[type=!player]`, so the check no longer depends on what spawned while it ran.
4. **`gen_smelting.py` needs the prepared resources**: without `HORIZON_RES` it reads an absent
   `~/.local/horizon-client/26.3`, finds 0 recipes and reports the committed table as stale. Set
   `HORIZON_RES=/home/adlf/Desktop/horizon/.local/horizon-client/26.3` for the recipe-table gate.

### Gaps after slice 7

Everything in *Gaps / deviations*, *Gaps after the migration*, and *Gaps after slices 3–6* still
stands. Changed: the **basic factory tier is ported** for all five item machines. Still missing: the
advanced/elite/ultimate factory tiers (the same code with a different process count and buffer), the
Ore Washer (water as a gas), the crystals and slurries, the chemicals as items, upgrades (so
`operationsPerTick` stays 1), side configuration and the mod's own GUIs.

### `## Upstream defects fixed`

None found in this slice. The gas sip was a defect in **this port's** new code, not upstream's (the
upstream machine fills its tank from the chemical slot regardless of energy, but this port's
recorded deviation is to pull from a neighbour, and it only pulls what it is about to spend).

### Deviations this slice records

- **The factories have no GUI**: `menu: None`; the six slots are reachable by hoppers and
  `/mekanism info` reports their sum. Upstream draws a six-slot screen with three progress bars.
- **`operationsPerTick` is the baseline 1**: there is no upgrade system to make an operation take
  less than a tick; the rule and its formula are ported and tested, not exercised above the baseline.
- **The one tank is pulled from a neighbour**, not filled from a chemical slot item — the same
  deviation the injection and purification chambers record.
- **The block is the barrel template** (`facing`×`open`), not upstream's `facing`×`active`; the
  `_active` front textures are carried, unused.
- **No sorting, ejector, side configuration or security**, as on every other machine of this port.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

### Changelog (0.2.6) — supersedes 0.2.5

> **0.2.6 — the basic factory family.** Five new blocks — `basic_smelting_factory`,
> `basic_enriching_factory`, `basic_crushing_factory`, `basic_injecting_factory` and
> `basic_purifying_factory` — each running the same recipe **three times side by side**
> (`FactoryTier.BASIC.processes`), with one input and one output slot per process, its own progress
> and one shared buffer given by `Factory.setMachineData`: **max(base storage / 2, base usage) × 3**,
> so **30,000 FE** for the three item factories, **240,000** for the injecting factory and
> **120,000** for the purifying one. The two gas factories share one tank of **630** units
> (`MAX_GAS` 210 × 3) and draw one tick's gas for every **working** process through
> `mekanism:chemical`. Worked in the wrong direction: three ore in a basic purifying factory become
> **nine** clumps for **3 × 40,000 FE** and **600** oxygen, while **one** ore costs one operation
> (3 clumps, 40,000 FE, 200 oxygen) and 40,000 FE buys exactly **200** progress ticks, not three
> operations. `operationsPerTick` is ported and tested (baseline 1 without upgrades). The five
> shaped recipes use the upstream `ACA/IPI/ACA` pattern around each base machine with vanilla
> stand-ins for the alloys/circuits this port does not register. Package: **220 entries, 161,073 B**,
> sha256 `6633dde7616f86fb8d8a6baad3cbaa1f842d977ae500a207000345a8a8b60e67`; `check` exit 0; `fits`
> `made for` at the pin and `fits` at `0b14f8a`; 92 tests, clippy `-D warnings` clean; in game
> `PASS` + `EXIT:0` with the factory phase and the 148/72/15-row sweeps, the unmodded game `EXIT:1`,
> `SELFTEST=1` `EXIT:1` on the factory's own assertion, `NOCABLE=1` `PASS` + `EXIT:0`.

## Slice 8 — the Chemical Washer, the port's first fluid machine (0.2.7, 2026-11-22)

### The scope authority, quoted

The gap line this slice comes from is the port's own, named by all three of its lists:

> Ore processing beyond smelting, enriching, crushing and injection (**Purification** and the Ore
> Washer — both need a second gas/fluid input — plus factories …) (`notes.md` "Gaps / deviations" 1)

> the **Ore Washer** (water as a gas) … (`FINDINGS.md` "Next slices" 1, after slice 5)

> Still missing: … the **Ore Washer** (water as a gas), the crystals and slurries … (`notes.md`
> "Gaps after slice 7")

and the 0.2.6 report's next-slice proposal: **the Chemical Washer** — `FluidChemicalToChemicalRecipe`,
a 10,000 mB water tank plus a slurry tank in and a second out, 200 FE/t, 80,000 FE.

**Chosen: the Chemical Washer** (`mekanism:washing`). It fits the **`deep` tier** as a
machines-shaped feature — one block entity with its own saved state, three tanks, an energy balance
and a capability provider *and* consumer — and it is the port's first machine whose input is a
**fluid**, which is why it is also the slice that proves "chemicals are not fluids" in the positive
direction. No entity, renderer, dimension or custom vanilla hook.

### The fluid capability at the pin: enough, without a new API

The pin's SDK already carries the whole fluid surface, so this slice adds no capability:

| piece | where it is at `0a875da` | used for |
|---|---|---|
| `ModFluidTanks` (capacities, faces, fill/drain tanks) | `crates/horizon-mod-vocab/src/item/containers.rs:224` | the washer's one 10,000 mB water tank |
| `DefaultCapabilitiesRegistrar::block_fluids` | `crates/horizon-mod-api/src/parts/default_capabilities.rs:43` | the declaration |
| `FluidResource`, `FLUIDS` | `crates/horizon-mod-vocab/src/block/fluid_capability.rs:68` | the key the harness fills through |
| the `fluids` list in the entity's saved compound | `hooks/notes.rs:1610` ("the tanks' contents live in the entity's saved compound under `fluids`") | what the tick reads and writes |

`FluidResource` is one of the two resource types the SDK shares between crates (`ItemResource` is the
other), so unlike `mekanism:chemical` (§9k) a fluid capability is answerable across packages — the
e2e's read is `(answered by the game)`, not by this port. The one gap that remains is §9l's:
`native_move_fluids` is a `helpers[]` entry a package cannot call and refuses any fluid but
water/lava engine-side; this slice does not move fluids between blocks (the machine fills its own tank
through the key and spends it), so the gap does not block it.

### What the jar says (upstream truth, re-read from `v1.21.1-10.7.19.85`, `a00109e4`)

| value | source | port |
|---|---|---|
| usage **200 FE/t** | `UsageConfig.java:67` (`chemicalWasher`, `200L`) | `WASHER_USAGE` |
| storage **80,000 FE** | `StorageConfig.java:84-85` (`chemicalWasher`, `80_000L, 1`) | `WASHER_CAPACITY` |
| 200 ticks per operation, so **40,000 FE** | `TileEntityRecipeMachine`'s `BASE_TICKS_REQUIRED` | `WASHER_OPERATION` |
| water tank **10,000 mB**, input only | `TileEntityChemicalWasher.MAX_FLUID = 10 * BUCKET_VOLUME`; `BasicFluidTank.input(MAX_FLUID, fluid -> containsRecipeAB(...))` | `WASHER_WATER`; `block_fluids` with `inputs [0]`, `outputs []` |
| two slurry tanks of **10,000**, insert-only in / extract-only out | `MAX_SLURRY = 10 * BUCKET_VOLUME`; `BasicChemicalTank.inputModern` / `.output` | `WASHER_SLURRY`; `WasherTanks` (0 = dirty in, 1 = clean out) |
| recipe type `mekanism:washing` | `TileEntityChemicalWasher.getRecipeType()` → `MekanismRecipeType.WASHING` | `runtime::washing` |
| 5 mB water + 1 dirty slurry → 1 clean slurry, **per operation** | `processing/<m>/slurry/clean.json`; `TwoInputCachedRecipe` overrides no `useResources`, so both inputs are taken in `finishProcessing` | `WashRecipe`, `process_wash_tick` |
| a mismatched recipe (no water, a clean slurry, too little) **resets**; no energy or output room **holds** | `CachedRecipeHelper.twoInputCalculateOperationsThisTick` + `InputHandler.calculateOperationsCanSupport` (`resetProgress`) | `IdleReason::{NoFluid, NoRecipe, NotEnoughInput}` reset; `{NoEnergy, NoOutputSpace}` hold |
| slurries are chemicals of the same registry, `dirty_`/`clean_` × 7 materials | `MekanismChemicals.PROCESSED_RESOURCES` → `ChemicalDeferredRegister.registerSlurry`; `PrimaryResource` | `runtime::SLURRIES` (14 ids); `known_chemical` accepts them |
| 7 `mekanism:washing` recipes | one `clean.json` per `PrimaryResource` | 7 rows, every one carried |

### The slice in the port's three layers

- **`src/runtime.rs`** — `SLURRIES` (14 ids) and `known_slurry`, with `known_chemical` now answering
  both registries; `FluidTank` and `insert_fluid` (water and nothing else — upstream's tank predicate
  is `containsRecipeAB`, and every washing row takes water); `WashRecipe`, `washing::{recipe_for,
  ROW_COUNT}` (generated by `tools/gen_washing.py`); `process_wash_tick`, `room_in_tank` and
  `washer_pull`; the `WASHER_*`/`WATER` constants; `IdleReason::NoFluid`.
- **`src/support.rs`** — the block's declaration (no container: its inputs are tanks, so upstream's
  four slots are recorded as a deviation), its 80,000 FE buffer, `block_fluids` (`washer_fluids()`),
  the `fluids`-list read/write (`read_washer_fluid`/`write_washer_fluid`, the game's own shape),
  `WasherTanks` (two chemical tanks through `mekanism:chemical`, tank 0 insert-only and tank 1
  extract-only), `WasherPart` (the tick plus `washer_draw`, which pulls the operation's dirty slurry
  from a neighbour through the key — the gas machines' recorded "no pipes" deviation), the washer's
  `/mekanism info` line, `/mekanism fluid read|insert` (through `horizon:fluids`) and
  `/mekanism chemical drain-tank` (a named tank, so the clean output can be drained).
- **Content** — `mekanism:chemical_washer` with the jar's **own 28-element model** and its seven
  textures (`models/ports`, `ports_large`, `ports_led`, `ports_large_led`, `chemical_washer`,
  `chemical_washer_elements`, `steel_casing`), the blockstate, the block declaration (3.5/16, a tool),
  loot, the shaped recipe and the lang keys.

### Red → green

The runtime rules and the glue's declarations were written as tests first and ran red on the
untouched tree (`../_shared/evidence/mekanism/slice8-unit-red.log`):

```
error[E0425]: cannot find value `WASHER` in this scope
error[E0425]: cannot find function `process_wash_tick` in this scope
error: could not compile `mekanism` (lib test) due to 147 previous errors
```

Then the constants, the slurry table, the generator's washing table, the three-tank tick and the
declarations made the crate green: **108 tests** (was 92) — 16 new (12 `runtime::washer_tests`, and the
washer's declaration, water tank, saved shape and info line in `support::tests`).

### Coverage — every family this slice ships

| family | status | proof |
|---|---|---|
| the washer block, its block entity, 80,000 FE buffer | **tested** | `A1`/`A2` (give/place/read back), `B2` (loot), `B1` (crafts), `L1` (`80000/80000 FE`) |
| its one 10,000 mB water tank, filled **through `horizon:fluids`** | **tested** | `L1`: `fluid: inserted 10000 minecraft:water … (10000/10000)`, saved `fluids[0].Amount` 10000, `holds minecraft:water 10000/10000 (answered by the game)` |
| its two slurry tanks through `mekanism:chemical` (0 dirty in, 1 clean out) | **tested** | `L1` insert `(1000/10000)`, `chemical read` 0 → nothing after the operation; `L4` `drain-tank 250 1` → `drained 1 mekanism:clean_iron (tank 0/10000, dropper 1/16000)` |
| one washing operation: 200 ticks, 40,000 FE, 5 mB water, 1 dirty in, 1 clean out | **tested** | `L1`: `0/80000 FE, progress 0/200, water 9995/10000 mB, input empty x0, output mekanism:clean_iron x1`; saved `energy` 0, `fluids[0].Amount` 9995, `input_amount` 0, `output_chemical`/`output_amount`; the neighbour tank 1,000 → 999 |
| a clean slurry in the input: no recipe, no progress, no water, no FE, nothing drawn | **tested** | `L2`: `80000/80000 FE, progress 0/200, water 1000/10000 mB, input mekanism:clean_iron x1000, output empty x0`; the neighbour still 1,000 |
| dirty slurry beside a machine with no water: no progress, no FE, nothing drawn | **tested** | `L3`: `80000/80000 FE, progress 0/200, water 0/10000 mB, input empty x0, output empty x0`; the neighbour still 1,000 |
| **all 7 carried washing rows** (water spent, dirty consumed, clean made, one operation's FE) | **tested** | `M`: `rows washed: 7 of 7` (`tools/wash_sweep.py`) |
| the slurry ids are chemicals of Mekanism's registry | **tested** | unit `the_slurry_table_is_mekanism_s_fourteen_chemicals`; `L2` inserts `mekanism:clean_iron` through the key |
| water is not a chemical / a chemical tank is not a fluid tank | **tested** | `L4`: `unknown chemical minecraft:water`; `no fluid handler at 0 72 4` |
| the washer's three tanks survive save + relaunch | **tested** | persistence: `0/80000 FE, progress 67/200, water 1000/10000 mB, input mekanism:dirty_iron x10, output empty x0` before and after |
| the washer's four container slots (bucket in, dropper out, energy) | **not ported** | its inputs are tanks; recorded as a deviation (the machine reaches them through the capabilities, and `/mekanism info`/`drain-tank` are the harness's read/write path) |
| its own GUI | **not ported** | the other machines' gap: no screens in this port |
| the crystallizer and dissolution chamber (the rest of the slurry chain) | **not ported** | their own machines (`mekanism:crystallizing`, `mekanism:dissolving`); the slurries are ids this port now knows, so they are the next consumer |
| `active` / `open` face texture | **deviation** | the jar's washer has no `active` state at all; the barrel template's unused `open` state stands in, as on every other machine |
| upgrades, side configuration, ejector, security | **not ported** | the other machines' gaps |

### The ladder (one lease per step, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result |
|---|---|---|
| runtime + glue tests (RED) | `cargo test --offline` on the untouched tree | **147 errors**, the new API missing (`slice8-unit-red.log`) |
| tests | `cargo test --offline` | **108 passed**, 0 failed |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`slice8-clippy.log`) |
| format | `cargo fmt --check` (never `--all`) | clean (`slice8-clippy.log`) |
| recipe tables | the six `gen_*.py --check` (`HORIZON_RES` set) | 163 / 132 / 148 / 72 / 15 / **7** rows |
| package | `rm -f Cargo.lock && cargo horizon-mod build` | `Built ./target/horizon-mod/mekanism.hmod` (`slice8-build.log`) |
| same package twice | the same again | **`cmp` identical** (`slice8-build-second.log`, `slice8-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | `mekanism 0.2.7 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, **179,723 B, SHA-256 `38a7bca5…458a4`**, exit 0 (`slice8-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, `FIT-PIN-EXIT:0` (`slice8-fits.log`, sha printed first) |
| fits, **a different build** | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, `FIT-OTHER-EXIT:0` (`slice8-fits.log`) — the run that proves something |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (233 entries, no packed `Cargo.lock`) |
| archive shape | `unzip -l mekanism.hmod` | 233 files, no `Cargo.lock` |

The launcher build (`launcher-game.sh build mekanism`, in a lease) printed `Compiling mekanism (1 of
1)` and produced `native-0a875da-46a111dc6bcd` (`slice8-launcher-build.log`). The stale
`native-0a875da-c5974b56f2f6` build (same 0.2.7 manifest, older code) was removed first: the
launcher reuses a build while the manifest is byte-identical, so an edit inside an unpublished
version is invisible until the old version folder and its export are gone (the slice-7 trap, hit
once more here because the pull change landed after the first 0.2.7 build).

### In-game evidence (`--offscreen`, the launcher build)

`../_shared/evidence/mekanism-e2e/e2e-package.log` → `PASS` + `EXIT:0`, quoted from phases L and M:

```
== L1: water through horizon:fluids, the tank's slurry through mekanism:chemical, one operation ==
  chemical insert 1000 mekanism:dirty_iron 0 72 0: chemical: inserted 1000 mekanism:dirty_iron into mekanism:basic_chemical_tank (1000/64000)
  info(1,72,0): mekanism:chemical_washer: 80000/80000 FE, progress 0/200, water 0/10000 mB, input empty x0, output empty x0
  chemical read 0 72 0: chemical: mekanism:basic_chemical_tank holds mekanism:dirty_iron 1000/64000 (answered by mekanism)
  fluid insert 10000 1 72 0: fluid: inserted 10000 minecraft:water into mekanism:chemical_washer (10000/10000)
  data get block 1 72 0 fluids[0].Amount: 1, 72, 0 has the following block data: 10000
  fluid read 1 72 0: fluid: mekanism:chemical_washer holds minecraft:water 10000/10000 (answered by the game)
  info(1,72,0): mekanism:chemical_washer: 0/80000 FE, progress 0/200, water 10000/10000 mB, input empty x0, output empty x0
  info(1,72,0): mekanism:chemical_washer: 0/80000 FE, progress 0/200, water 9995/10000 mB, input empty x0, output mekanism:clean_iron x1
  data get block 1 72 0 fluids[0].Amount: 1, 72, 0 has the following block data: 9995
  data get block 1 72 0 input_amount: 1, 72, 0 has the following block data: 0
  data get block 1 72 0 output_chemical: 1, 72, 0 has the following block data: "mekanism:clean_iron"
  data get block 1 72 0 output_amount: 1, 72, 0 has the following block data: 1
  data get block 1 72 0 energy: 1, 72, 0 has the following block data: 0
  chemical read 0 72 0: chemical: mekanism:basic_chemical_tank holds mekanism:dirty_iron 999/64000 (answered by mekanism)
  chemical read 1 72 0: chemical: mekanism:chemical_washer holds nothing 0/10000 (answered by mekanism)
== L2: a clean slurry in the input makes nothing, draws no water and takes nothing from the tank beside it ==
  fluid insert 1000 5 72 0: fluid: inserted 1000 minecraft:water into mekanism:chemical_washer (1000/10000)
  chemical insert 1000 mekanism:clean_iron 5 72 0: chemical: inserted 1000 mekanism:clean_iron into mekanism:chemical_washer (1000/10000)
  info(5,72,0): mekanism:chemical_washer: 80000/80000 FE, progress 0/200, water 1000/10000 mB, input mekanism:clean_iron x1000, output empty x0
  energy(5,72,0): 80000
  chemical read 4 72 0: chemical: mekanism:basic_chemical_tank holds mekanism:dirty_iron 1000/64000 (answered by mekanism)
== L3: dirty slurry beside a machine with no water spends no FE and draws nothing ==
  info(8,72,0): mekanism:chemical_washer: 80000/80000 FE, progress 0/200, water 0/10000 mB, input empty x0, output empty x0
  energy(8,72,0): 80000
  chemical read 7 72 0: chemical: mekanism:basic_chemical_tank holds mekanism:dirty_iron 1000/64000 (answered by mekanism)
== L4: water is not a chemical, a chemical tank is not a fluid tank, and the clean slurry drains ==
  chemical insert 100 minecraft:water 1 72 0: chemical: unknown chemical minecraft:water
  fluid insert 100 0 72 4: fluid: no fluid handler at 0 72 4
  chemical drain-tank 250 1 1 72 0: chemical: drained 1 mekanism:clean_iron into the gauge dropper (tank 0/10000, dropper 1/16000)
== M: every carried washing recipe in a real Chemical Washer ==
  rows from the jar's mekanism:washing data: 7
  mekanism:dirty_copper x1 + 5 mB water -> {'mekanism:clean_copper': 1} (water 495, input 0, energy 40000)
  …
rows washed: 7 of 7
sweep PASS: all 7 carried washing recipes ran in a real Chemical Washer with their declared water and
slurry, the dirty slurry consumed, the clean slurry made, and exactly one operation's water and FE spent
== persistence: save, quit and relaunch read the block entities back ==
  before save, washer:  mekanism:chemical_washer: 0/80000 FE, progress 67/200, water 1000/10000 mB, input mekanism:dirty_iron x10, output empty x0
  after relaunch, washer: mekanism:chemical_washer: 0/80000 FE, progress 67/200, water 1000/10000 mB, input mekanism:dirty_iron x10, output empty x0
```

What the numbers prove:

- **the water arrives through the game's own fluid handler**: `fluid: inserted 10000
  minecraft:water …` is a `FLUIDS` capability insert, the block entity's saved `fluids` list reads
  `Amount: 10000`, and the read-back names `(answered by the game)` — the engine's provider, not this
  port's. That is the positive half of "chemicals are not fluids"; the negative half is the same run
  (`unknown chemical minecraft:water`, `no fluid handler at 0 72 4`);
- **one operation is exactly the jar's price**: 200 ticks x 200 FE = 40,000 FE (the buffer charged
  with exactly one operation reads **0** afterwards), **5 mB** of water (10,000 → 9,995) and **one
  unit** of dirty slurry (the neighbour's 1,000 → 999, pulled through `mekanism:chemical` and spent at
  the finish) for **one unit** of clean slurry — all four read from the saved data, not from the info
  line alone;
- **the tank beside it is the chemical side**: the machine's own 10,000-unit tank is empty after the
  operation (`holds nothing 0/10000`), so the slurry it spent came through the capability;
- **water and slurry are spent at the finish, not per tick**: the mid-operation washer (persistence
  phase) holds all 1,000 mB at progress 67/200;
- **every row costs what it declares**: each of the 7 washing rows ran in a real washer with 500 mB of
  water (495 after), its one unit of dirty slurry consumed, its own clean slurry made and 40,000 FE
  spent;
- **the state is the block entity's**: after `save`, quit and relaunch the washer still reads
  `progress 67/200` with its water and its ten units of dirty slurry.

| leg | result | evidence |
|---|---|---|
| package (default) | **`PASS` + `EXIT:0`** | `e2e-package.log` |
| unmodded game (`GAME=$CONTROL_GAME CONTROL=0`) | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** | `e2e-control.log` |
| fail-capable (`SELFTEST=1`) | **`FAIL: expected /mekanism info at 1,72,0 to say 'output empty x0', got '… output mekanism:clean_iron x1'` + `EXIT:1`** | `e2e-selftest.log` |
| differential (`NOCABLE=1`) | **`PASS` + `EXIT:0`** (phase L needs no cable and still passes) | `e2e-nocable.log` |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |

The `SELFTEST=1` flip moved to **this** slice's assertion (it was the basic purifying factory's
output): the newest family is the one that has to prove it can fail.

### What the harness taught this slice

1. **A machine that keeps working makes an exact FE reading impossible.** The first version charged
   the full 80,000 FE with 1,000 units in the tank; at any read the washer had done one operation and
   was 10–30 ticks into the next, so `40000/80000` could never be asserted. The phase now charges
   **exactly one operation** (40,000 FE) with the buffer's 80,000 proven separately, and the machine
   stops at 0 FE — which is also the stronger statement (the operation spends exactly 40,000).
2. **`ctl chat`'s reply is not the command's answer.** The first sweep read the feedback from the
   `ctl chat` reply and got `''`; every `/mekanism` answer has to be read from the game's
   `latest.log` (the gas and crushing sweeps already did).
3. **A `data get` value can hold a colon** (`"mekanism:clean_iron"`): splitting on the last `:` cuts
   the id in half. The sweep splits on `block data: ` instead.
4. **The idle assertion has to come before the charge** (the other machines' phases do): once the
   buffer pays, the machine works one tick per tick and no read catches it at progress 0.

### Gaps after slice 8

Everything in *Gaps / deviations*, *Gaps after the migration* and *Gaps after slices 3–7* still stands
where it is not superseded. Changed:

- the **Ore Washer is ported**, so the ore-processing chain is now smelter · enrichment · crusher ·
  injection · purification · **washer**;
- the port **publishes** the fluid side of a machine too (through the game's `horizon:fluids`), and
  its chemical id table now carries Mekanism's **14 slurries** (`runtime::SLURRIES`), so the
  crystallizer, the dissolution chamber and any slurry-consuming mod are unblocked on the ids;
- the washer has **no container** and **no GUI**: its four upstream slots (a bucket in, a dropper out,
  an energy slot) are recorded as a deviation, and `/mekanism info`, `/mekanism fluid read|insert`
  and `/mekanism chemical drain-tank` are the harness's read/write path;
- the washer **pulls** its dirty slurry from a neighbour (one operation's worth) instead of being
  filled by a pipe: the gas machines' recorded deviation, for the same reason (this port ships no
  pipes).

Still missing: the crystallizer and dissolution chamber (the slurries' other two machines), the
pigments, the advanced/elite/ultimate tiers, upgrades, side configuration, the mod's own GUIs.

### `## Upstream defects fixed`

None found in this slice. Every number is the jar's own, re-read for this machine; the two behaviours
that look odd are the artefact's and are carried as they are: the washer's block has **no `active`
state** (so the barrel template's unused `open` is the only stand-in), and both inputs are consumed
**at the finish**, not per tick (`TwoInputCachedRecipe` overrides no `useResources`) — the
mid-operation persistence phase pins the second one.

### Deviations this slice records

- **No container and no GUI**: the four upstream slots are conveniences over the same tanks; the
  tanks are the interface, reached through the two capabilities.
- **The machine pulls its slurry** from an adjacent chemical handler, one operation's worth per
  operation, and only when it could complete one — the gas machines' recorded deviation.
- **The block is the barrel template** (`facing` × `open`); upstream's washer has only `facing`, so
  the template's `open` state is never set. Its **model and seven textures are the jar's own** (the
  28-element model), unlike the other machines' plain cube — the only machine whose authentic model
  this port could carry unchanged.
- **The recipe uses vanilla stand-ins**: `IBI/CXC/ITI` with a bucket for the basic fluid tank,
  redstone for the ultimate circuit and an iron block for the steel casing, around this port's own
  Basic Chemical Tank (the precedent of the fourteen earlier recipes).
- **No upgrades, side configuration, ejector or security**, as on every other machine of this port.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

### Changelog (0.2.7) — supersedes 0.2.6

> **0.2.7 — the Chemical Washer, the port's first fluid machine.** It runs the jar's
> `FluidChemicalToChemicalRecipe` (`mekanism:washing`): **200 FE/t × 200 ticks (40,000 FE an
> operation)**, an **80,000 FE** insert-only buffer, a **10,000 mB water tank** declared through the
> game's own **`horizon:fluids`** capability — its contents in the block entity's `fluids` list, so
> water arrives through a real fluid handler — and **two 10,000-unit slurry tanks** through
> `mekanism:chemical` (dirty in, clean out), with **5 mB of water and one unit of a dirty slurry
> turned into one unit of its clean slurry per operation**, at the operation's finish as the jar does.
> The seven `mekanism:washing` rows (copper, gold, iron, lead, osmium, tin, uranium) are all carried,
> the port's chemical registry gains Mekanism's **14 slurries** (`dirty_*`/`clean_*`), and the block
> ships the jar's own 28-element model and seven textures. Worked in the wrong direction in game:
> water through the fluid key lands in the saved data and is spent 5 mB an operation, dirty slurry
> arrives through the chemical key (1,000 → 999 for one operation), a clean slurry makes nothing, an
> empty water tank spends no FE and draws nothing, water is refused by the chemical key, a chemical
> tank answers no fluid handler, the clean slurry drains through tank 1, and after `save`+relaunch the
> washer still holds its progress and all three tanks. Package: **233 entries, 179,723 B**, sha256
> `38a7bca520354e06932a914fbcd70adb0f2c2f12624b74598a855bcc4a7458a4`; `check` exit 0; `fits` `made
> for` at the pin and `fits` at `0b14f8a`; 108 tests, clippy `-D warnings` clean; in game `PASS` +
> `EXIT:0` with all 7 washing rows, the unmodded game `EXIT:1`, `SELFTEST=1` `EXIT:1` on the washer's
> own assertion, `NOCABLE=1` `PASS` + `EXIT:0`.

## Slice 9 — the Chemical Crystallizer (0.2.8, 2026-11-23)

### The scope authority, quoted

The gap line is the port's own, and it is the 0.2.7 report's `## Next slice`:

> the crystallizer (`mekanism:crystallizing`) — 200 units of a clean slurry into a
> `mekanism:crystal_<material>` — is the washer's consumer and needs **no new capability**: one
> chemical in, one item out, on the chemical key slice 4 published. (`notes.md` "Gaps after slice 8")

and `notes.md` "Gaps / deviations" 1 named the crystals among what was still missing
("Still missing: … the crystals and slurries …").

**Chosen: the Chemical Crystallizer** (`mekanism:crystallizing`) with the **three crystals this port
can finish** (iron, gold, copper). It is the washer's consumer — the last step of the jar's slurry
branch, clean slurry → crystal, which the enriching chamber then turns into a dust — and it needs no
new capability at all: the chemical key takes a slurry, and the product is an ordinary item in the
container's output slot. It fits the **`deep` tier** as a machines-shaped feature (one block entity,
its saved tank and buffer, one container, its energy balance in its own tick), with no entity,
renderer, dimension or custom vanilla hook.

### What the jar says (upstream truth, re-read from `v1.21.1-10.7.19.85`, `a00109e4`)

| value | source | port |
|---|---|---|
| usage **400 FE/t** | `UsageConfig.java:68` (`chemicalCrystallizer`, `400L`) | `CRYSTALLIZER_USAGE` |
| storage **160,000 FE** | `StorageConfig.java:86` (`chemicalCrystallizer`, `160_000L, 1`) | `CRYSTALLIZER_CAPACITY` |
| 200 ticks per operation, so **80,000 FE** | `TileEntityChemicalCrystallizer.BASE_TICKS_REQUIRED = 10 * SharedConstants.TICKS_PER_SECOND` (`:62`) | `CRYSTALLIZER_OPERATION` |
| one **10,000-unit** chemical tank, insert-only, and **its predicate is `containsRecipe`** | `MAX_CHEMICAL = 10 * BUCKET_VOLUME` (`:61`); `BasicChemicalTank.inputModern(MAX_CHEMICAL, this::containsRecipe, …)` (`:94`) | `CRYSTALLIZER_TANK`; `crystallizing::accepts` |
| **200 units → 1 crystal**, the slurry taken when the operation finishes | `processing/<m>/crystal/from_slurry.json` (`{"amount": 200, …}`); `OneInputCachedRecipe.crystallizing` (`:148`), whose `finishProcessing` calls `inputHandler.use` | `CrystallizeRecipe`, `process_crystal_tick` |
| a chemical with no row, an empty tank and too little slurry **reset**; no energy or no output room **hold** | `CachedRecipe`'s `NOT_ENOUGH_INPUT`/`mismatchedRecipe` vs `NOT_ENOUGH_ENERGY`/`NOT_ENOUGH_OUTPUT_SPACE` (`TRACKED_ERROR_TYPES`, `:49-54`) | `IdleReason::{NoChemicalInput, NoRecipe, NotEnoughInput}` reset; `{NoEnergy, NoOutputSpace}` hold |
| insert-only energy, one energy slot, one output slot, one chemical-container input slot | `MachineEnergyContainer.input` (`:109`); `configComponent.setupItemIOConfig(inputSlot, outputSlot, energySlot)` (`:99`) | energy through `block_energy` (insert-only faces); one output slot; **no** chemical-container or energy slot (recorded deviation) |
| block: `facing` only, **no `active` state**, hardness 3.5 / resistance 16, a tool | `blockstates/chemical_crystallizer.json` (4 variants); `MekanismBlockTypes.createMachine` | barrel template (3.5/16, `facing` × the template's unused `open`), the washer's recorded deviation |
| the jar's **composite** model (a cutout child and a translucent one) over seven textures | `models/block/chemical_crystallizer.json` (`neoforge:composite`) | flattened into one model by `tools/carry_crystallizer.py`, `render_type: cutout` (recorded deviation) |
| ten `mekanism:crystallizing` recipes | two in `recipe/crystallizing/` (lithium, brine→salt), eight in `processing/*/crystal/` (copper, gold, iron, lead, osmium, tin, uranium, plus antimatter) | `tools/gen_crystallizing.py`: **3 rows**, 7 skipped because their output item is not registered |

### The three carried crystals

Because the port registers no crystal items yet, a crystallizer row is only runnable if its **output
exists**: `tools/gen_crystallizing.py` keeps the three ore slurry rows and prints the seven it drops
(lithium dust, salt, the antimatter pellet, and `crystal_{lead,osmium,tin,uranium}` — the four
slurries the washer *can* make but whose crystals this port does not ship: gap 1 below). The three
crystals are plain items on the `minecraft:gunpowder` template (stack 64), `item/generated` models
and the jar's own textures; the crystallizer's seven textures and its model are the jar's bytes
(`tools/carry_crystallizer.py --check` re-derives all ten carried files).

### The slice in the port's three layers

- **`src/runtime.rs`** — `CrystallizeRecipe` (`chemical`, `amount`, `output`, `output_count`),
  `crystallizing::{recipe_for, accepts, slurries, slurries_that_fit, ROW_COUNT}` (the table generated
  from the jar), `process_crystal_tick` (the operation, its `Outcome`s and its energy), the
  `crystallizer_pull` rule, and the three constants. `CRYSTALLIZER_OPERATION.usage * ticks = 80,000`,
  so the buffer holds exactly two operations — a property the unit test pins and the e2e charges.
- **`src/support.rs`** — the block's declaration (hardness/resistance/tool, no container faces of its
  own beyond the one output slot), its 160,000 FE buffer, its **one-slot container** (`inputs []`,
  `outputs [0]`, so a hopper pulls the crystals), `CrystalTank(BlockSide)` answering
  `mekanism:chemical` for `mekanism:chemical_crystallizer` with upstream's *tank predicate*
  (`crystallizing::accepts`), `CrystallizerPart` (the tick: draw one operation's slurry from a
  neighbour through the key, then one operation's tick), and its `/mekanism info` line.
- **Content** — `mekanism:chemical_crystallizer` (the flattened jar model, seven textures, blockstate,
  3.5/16 + tool), the three `mekanism:crystal_{iron,gold,copper}` items with the jar's textures,
  their lang keys, the shaped recipe (vanilla stand-ins: 4 obsidian, 4 amethyst, 2 redstone, an iron
  block — the precedent of the fifteen earlier recipes), the loot table, the pickaxe tag and the
  creative tab.

### Red → green

The slice's tests were written first and ran red on the untouched tree
(`../_shared/evidence/mekanism/slice9-unit-red.log`), **67 errors**, e.g.:

```
error[E0433]: cannot find module or crate `crystallizing` in this scope
     --> src/runtime.rs:5993:20
      |
 5993 |         assert!(!crystallizing::accepts("mekanism:clean_lead"), "no crystal item");
error: could not compile `mekanism` (lib test) due to 67 previous errors
```

The **fix** this slice's first e2e forced has its own red (`slice9-fix-red.log`), on the tree with the
test and the glue call but not the rule:

```
error[E0425]: cannot find function `slurries_that_fit` in module `crystallizing`
     --> src/support.rs:1540:34
      |
 1540 |     for slurry in crystallizing::slurries_that_fit(output, limit) {
```

Then green: **124 tests** (was 108 at 0.2.7; +16 for the crystallizer's rules and this fix).

### The two defects the first 0.2.8 run found

**1. The crystallizer stopped after one operation (the port's own glue; fixed).** The first full e2e
left it at `80000/160000 FE, output crystal_iron x1` with the neighbour tank frozen at 800 and its own
tank empty — one operation, then 1,300 ticks of nothing, which is why the run's N1 failed on `x5`.
`tools/probe-crystal.sh` reproduced it in eight 100-tick reads (a tank of 1,000 beside a charged
machine), and the code showed the cause: `crystallizer_draw`'s empty-tank loop asked each slurry in
turn but used the room check as a **loop-wide** verdict — `return 0` on the first candidate whose
crystal did not match the slot. The slot held `crystal_iron`; the table's first row is
`clean_copper`, a different item, so the loop gave up before reaching `clean_iron`. A slot of one
material's crystals hid every other row.

**Fixed** by moving the rule into the pure core:
`crystallizing::slurries_that_fit(output, max_stack)` yields only the slurries whose crystal the slot
could take, and the glue iterates it. A full stack, or a slot of another item, yields nothing (so a
stalled crystallizer still siphons nothing — the slice-7 guard); a slot of `crystal_iron` yields
`clean_iron`. Pinned by
`a_slot_of_one_crystal_asks_for_that_materials_slurry_and_skips_the_rest`; proven in game by N1's
**second** operation (two crystals from one charge), which the buggy build could not produce.

**2. A harness coordinate collision (not a port defect; fixed in `tools/e2e.sh`).** N4's hopper/chest
under the crystallizer sat at `1,71,8`/`1,70,8` — directly on the purifying factory that phase K
leaves at `1,70,8` for the persistence check. The chest **replaced** that factory, so the relaunch
phase read a chest ("No block entity there.") and failed. N's column moved to `z=10`; the persistence
factory reads back unchanged.

### Coverage — every family this slice ships

Per the parity rule (AGENTS.md, 2026-11-20):

| family | status | proof |
|---|---|---|
| the crystallizer block, its block entity, its 160,000 FE buffer, its one output slot, its `/mekanism info` line | **tested** | N1 `info(1,72,10): mekanism:chemical_crystallizer: 0/160000 FE, progress 0/200, …`; `support::crystallizer_support_tests` |
| one crystallizing operation: 80,000 FE, 200 ticks, 200 units consumed at the finish, one crystal in the slot | **tested** | N2: 80,000 FE → `0/160000 FE, progress 0/200, output crystal_iron x1`, neighbour 800, machine tank 0 |
| the buffer holds exactly two operations; 1,000 units = five operations = 400,000 FE | **tested** | N1 (one charge → x2, 0 FE, neighbour 600) and N1b (two more charges → x5, neighbour 0) |
| a **dirty** slurry: no progress, no FE spent, and the machine's tank refuses it by name | **tested** | N3: `inserted 0 mekanism:dirty_iron … (0/10000)`, `160000/160000 FE, progress 0/200`, neighbour still 1000 |
| **all 3 carried crystallizing rows** (input amount, output id and count, slurry consumed) | **tested** | O: `rows crystallized: 3 of 3` (`tools/crystal_sweep.py`) |
| the three crystal items and the block: give-able, placeable, read back | **tested** | A1/A2 (the block and the three crystals are in the port's own lists) |
| the crystallizer's shaped recipe | **tested** | B1: 4 obsidian + 4 amethyst + 2 redstone + iron block → `mekanism:chemical_crystallizer`, plus the wrong-ingredient control |
| the crystallizer's loot table | **tested** | B2: each of the 15 blocks drops its own item |
| the crystals leave the output slot through a hopper | **tested** | N4: `ctl container` at the chest shows `crystal_iron` ×5, slot empty afterwards |
| progress and the tank survive a save and relaunch | **tested** | persistence: `0/160000 FE, progress 67/200, input mekanism:clean_iron x400` before and after |
| `crystal_{lead,osmium,tin,uranium}` | **not ported** | those crystal items are not registered, so `gen_crystallizing.py` drops their four rows (the washer can make the slurries — gap 1) |
| `mekanism:dust_lithium`, `mekanism:salt`, `mekanism:pellet_antimatter` | **not ported** | their output items are not registered; the three rows are dropped and printed |
| the `active` front texture, the composite (translucent) model, the chemical-container and energy slots, the GUI | **not ported** | recorded deviations below |

### The ladder (one lease per step, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result |
|---|---|---|
| runtime + glue rules, RED first | `cargo test --offline` on the untouched tree | **67 errors** (`slice9-unit-red.log`) |
| the fix, RED first | the same with the test and the call, not the rule | **`E0425: … slurries_that_fit`** (`slice9-fix-red.log`) |
| tests | `cargo test --offline` | **124 passed**, 0 failed (`sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| format | `cargo fmt --check` (never `--all`) | clean (`sdk-clippy.log`) |
| recipe tables | the seven `gen_*.py --check`, `carry_crystallizer.py --check` (`HORIZON_RES` set) | 163 / 132 / 148 / 72 / 15 / 7 / **3** rows; 10 carried files match the jar (`slice9-recipe-tables.log`) |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` | **193,757 B**, `cmp` identical, sha256 `ff719b85…e8945` (`slice9-build.log`, `slice9-build-second.log`, `slice9-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | exit 0, `mekanism 0.2.8 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, 250 entries (`slice9-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, exit 0 (`slice9-fits.log`) |
| fits, a different build | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, exit 0 — the run that proves something (`slice9-fits.log`) |
| archive | `unzip -l mekanism.hmod` | 250 entries, no `Cargo.lock` (`slice9-entries.txt`) |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (250 entries, no packed `Cargo.lock`) |
| unmodded game | `GAME=$CONTROL_GAME CONTROL=0 tools/e2e.sh` | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** (`e2e-control.log`) |
| fail-capable | `SELFTEST=1 tools/e2e.sh` | **`FAIL: expected /mekanism info at 1,72,10 to say 'output empty x0', got '… output mekanism:crystal_iron x2'` + `EXIT:1`** (`e2e-selftest.log`) |
| differential | `NOCABLE=1 tools/e2e.sh` | **`PASS` + `EXIT:0`** (`e2e-nocable.log`) |
| in game | `build-slot.sh run --step e2e -- bash tools/e2e.sh` on the launcher build | **`PASS` + `EXIT:0`** (`e2e-package.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |
| launcher build | `launcher-game.sh build mekanism` (in a lease, the stale `native-*` removed first) | `Compiling mekanism (1 of 1)`, `native-0a875da-8581709df4b9` (`slice9-launcher-build.log`) |

### In-game evidence (`--offscreen`, the launcher build)

`../_shared/evidence/mekanism-e2e/e2e-package.log`, quoted from this slice's phases:

```
== N1: two operations, the buffer's whole 160,000 FE, from 1,000 units ==
  chemical insert 1000 mekanism:clean_iron 0 72 10: chemical: inserted 1000 mekanism:clean_iron into mekanism:basic_chemical_tank (1000/64000)
  info(1,72,10): mekanism:chemical_crystallizer: 0/160000 FE, progress 0/200, input empty x0, output mekanism:crystal_iron x2
  energy(1,72,10): 0
  chemical read 0 72 10: chemical: mekanism:basic_chemical_tank holds mekanism:clean_iron 600/64000 (answered by mekanism)
  chemical read 1 72 10: chemical: mekanism:chemical_crystallizer holds nothing 0/10000 (answered by mekanism)
== N1b: three more operations, three more charges, five crystals from 1,000 units ==
  info(1,72,10): mekanism:chemical_crystallizer: 0/160000 FE, progress 0/200, input empty x0, output mekanism:crystal_iron x5
  energy(1,72,10): 0
  data get block 1 72 10 energy: 1, 72, 10 has the following block data: 0
  chemical read 0 72 10: chemical: mekanism:basic_chemical_tank holds nothing 0/64000 (answered by mekanism)
  chemical read 1 72 10: chemical: mekanism:chemical_crystallizer holds nothing 0/10000 (answered by mekanism)
== N2: one operation costs 80,000 FE and 200 units, and makes one crystal ==
  info(5,72,8): mekanism:chemical_crystallizer: 0/160000 FE, progress 0/200, input empty x0, output mekanism:crystal_iron x1
  energy(5,72,8): 0
  chemical read 4 72 8: chemical: mekanism:basic_chemical_tank holds mekanism:clean_iron 800/64000 (answered by mekanism)
  chemical read 5 72 8: chemical: mekanism:chemical_crystallizer holds nothing 0/10000 (answered by mekanism)
== N3: a dirty slurry makes no progress, spends no FE, and the tank refuses it by name ==
  chemical insert 100 mekanism:dirty_iron 1 72 12: chemical: inserted 0 mekanism:dirty_iron into mekanism:chemical_crystallizer (0/10000)
  info(1,72,12): mekanism:chemical_crystallizer: 160000/160000 FE, progress 0/200, input empty x0, output empty x0
  energy(1,72,12): 160000
  chemical read 0 72 12: chemical: mekanism:basic_chemical_tank holds mekanism:dirty_iron 1000/64000 (answered by mekanism)
== N4: a hopper under the output slot pulls the crystals into a chest ==
  ctl container 1 70 10: {"slots":[{"count":5,"item":"mekanism:crystal_iron"},null,…]}
  info(1,72,10): mekanism:chemical_crystallizer: 0/160000 FE, progress 0/200, input empty x0, output empty x0
== O: every carried crystallizing recipe in a real Chemical Crystallizer ==
  rows from the jar's mekanism:crystallizing data: 3
  mekanism:clean_copper x200 -> mekanism:crystal_copper x1 (tank 0, energy 80000)
  mekanism:clean_gold x200 -> mekanism:crystal_gold x1 (tank 0, energy 80000)
  mekanism:clean_iron x200 -> mekanism:crystal_iron x1 (tank 0, energy 80000)
rows crystallized: 3 of 3
  before save, crystallizer: mekanism:chemical_crystallizer: 0/160000 FE, progress 67/200, input mekanism:clean_iron x400, output empty x0
  after relaunch, crystallizer: mekanism:chemical_crystallizer: 0/160000 FE, progress 67/200, input mekanism:clean_iron x400, output empty x0
PASS: … the crystallizer turns 200 units of a clean slurry into one crystal for 80,000 FE and five
operations into five crystals, refusing a dirty slurry at its tank while a dirty slurry in a
neighbour tank spends nothing …
EXIT:0
```

What the numbers prove:

- one charge of 160,000 FE bought **exactly two** crystals and left the buffer at **0**, so the
  operation costs 80,000 FE (200 ticks × 400) and the buffer holds exactly two of them;
- the neighbour tank went 1,000 → **600** for those two operations (200 units each) and to **0** after
  five, so the machine spends exactly the row's amount per operation, and the machine's own tank reads
  0 (the slurry is spent at the finish);
- **two more charges** (160,000 + 80,000 = 400,000 FE in all) finished the five operations, which is
  the second operation the stall bug lost and the rest of the chain;
- one charge of 80,000 FE alone made **one** crystal and left the neighbour at 800, which pins the
  per-operation cost in the other direction;
- a dirty slurry beside it left `160000/160000 FE, progress 0/200` untouched and the neighbour at
  1,000, and the machine's own tank answered `inserted 0 … (0/10000)`: upstream's `containsRecipe`
  predicate refuses it **by name**, and nothing is siphoned;
- all **3** carried rows crystallized their declared crystal from their declared slurry amount;
- the hopper took the five crystals into the chest below and the output slot was empty afterwards;
- the crystallizer's progress and its 400 units of clean slurry survived `save`, quit and relaunch.

The rest of the leg is unchanged and still passes: the content/recipe/loot phases (now 15 blocks, 16
shaped recipes), the crusher and crushing sweep (148), the cable→cube→chamber→smelter chain, the
chemical capability, the injection/purifying/washing machines and sweeps (72/15/7), the factories
(slice 7) and the save/relaunch of every machine type.

### What the harness taught this slice

- **`setblock <air>` in a chunk the player has left answers `Could not set the block`** and leaves the
  block in place: the persistence phase's "cut the cable" silently did nothing on the first run. It
  now teleports the player back, clears the row and **prints what is there** (`cleared for the stall:
  1,64,0 minecraft:air, 2,64,0 minecraft:air`), so a failed clear is visible in the transcript rather
  than assumed.
- **A new phase's blocks must not land on an earlier phase's fixture.** The hopper chain that drains
  the crystallizer overwrote the purifying factory the persistence check reads (defect 2 above); the
  arena has rows at `y=70` (factories), `y=72` (the chemical machines), so the crystallizer's column
  moved to `z=10`.
- A `ctl wait N` is N **game ticks** (the replies count them), which is why a 200-tick operation is
  waited out with `ctl wait 430` rather than a wall-clock sleep; the log's tick numbers make a run's
  arithmetic checkable after the fact.

### Gaps after slice 9

1. **Four slurries have no crystal**: the washer makes `clean_{lead,osmium,tin,uranium}` and this port
   registers no `crystal_*` for them, so `gen_crystallizing.py` drops their four rows. Adding the four
   items + textures completes the washer→crystallizer→enricher chain for every row (a one-slice job).
2. **No chemical-container or energy slot**: upstream's `inputSlot` (a tank item's chemical) and
   `energySlot` are not carried; the tank is filled through `mekanism:chemical` and the buffer through
   `block_energy`, as every other machine of this port.
3. **The models**: the crystallizer's jar model is a `neoforge:composite` (a cutout child and a
   translucent one) and is flattened into one `cutout` model, so its glass draws cutout.
4. **No `active` state**: the jar's crystallizer has `facing` only, so the barrel template's unused
   `open` state is the only stand-in (the washer's recorded deviation, not a loss here).
5. **No GUI, upgrades, side configuration, ejector or security**, as on every other machine.
6. **Recipes use vanilla stand-ins** (obsidian/amethyst/redstone/iron block), the precedent of the
   fifteen earlier ones: the real ingredients (steel casing, ultimate circuit, enriched alloy) are not
   registered.
7. Everything else in *Gaps / deviations* and *Gaps after slice 8* still stands.

### `## Upstream defects fixed`

None found in this slice. The stall was this port's own glue (defect 1 above), and the jar's rows,
numbers and tank predicate are consistent with what it ships.

### Deviations this slice records

- **No input slot**: upstream's `ChemicalInventorySlot` takes a chemical container item; the port has
  the tank and the key instead (gap 2).
- **The draw-from-a-neighbour rule** is the gas machines' recorded deviation: the port ships no pipes,
  so the machine takes the operation's slurry from an adjacent handler — only when it could finish
  that operation (its crystal fits), which is the guard the stall fix tightened.
- **The crystals are plain items** on the gunpowder template with the jar's textures (no custom item
  behaviour, no `mekanism:crystal` component).
- **The four missing crystals and the seven dropped rows** are gap 1 above.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

### Changelog (0.2.8) — supersedes 0.2.7

> **0.2.8 — the Chemical Crystallizer, the washer's consumer.** It runs the jar's
> `mekanism:crystallizing` table over the three crystals this port registers: **400 FE/t × 200 ticks
> (80,000 FE an operation)**, a **160,000 FE** insert-only buffer (exactly two operations), one
> **10,000-unit** slurry tank whose predicate is upstream's `containsRecipe` (a clean slurry with a row
> and nothing else — a dirty slurry is refused by name), and one output slot a hopper can pull from,
> turning **200 units of a clean slurry into one `mekanism:crystal_<material>`** at the operation's
> finish. The three crystals (iron, gold, copper) are carried with the jar's textures, and the block
> ships the jar's model (flattened from its composite) and seven textures. Worked in game: one charge
> of 160,000 FE makes exactly **two** crystals and drains the neighbour by 400 units; 400,000 FE in
> three charges makes **five** from 1,000 units with the tank at 0; one 80,000 charge makes **one**
> for 200 units; a dirty slurry makes no progress, spends no FE and is refused by the machine's tank;
> all 3 rows run in a real crystallizer; a hopper empties the output slot; and the progress and the
> tank survive `save`+relaunch. The first run of this version caught and fixed a real stall: the
> empty-tank draw loop gave up on the whole table when the output slot held a *different* crystal, so
> the machine ran one operation and then nothing — the rule is now
> `crystallizing::slurries_that_fit`. Package: **250 entries, 193,757 B**, sha256
> `ff719b859f3cc0ca810a86b4d01fa60f23cc00c66a9d37a0eeec24da3ade8945`; `check` exit 0; `fits` `made
> for` at the pin and `fits` at `0b14f8a`; 124 tests, clippy `-D warnings` clean; in game `PASS` +
> `EXIT:0`; the unmodded game `EXIT:1`, `SELFTEST=1` `EXIT:1` on the crystallizer's own assertion,
> `NOCABLE=1` `PASS` + `EXIT:0`.

## Slice 10 — the four missing crystals (0.2.9, 2026-11-23)

### The scope authority, quoted

The gap line is this port's own, "Gaps after slice 9" item 1 (and the slice-9 coverage row for the
four items):

> **Four slurries have no crystal**: the washer makes `clean_{lead,osmium,tin,uranium}` and this port
> registers no `crystal_*` for them, so `gen_crystallizing.py` drops their four rows. Adding the four
> items + textures completes the washer→crystallizer chain for every row (a one-slice job).

**Chosen: the four crystals** (`mekanism:crystal_{lead,osmium,tin,uranium}`). It is not a new
family — it is the slice-9 crystallizer at four more rows — so it needs **no new capability, hook,
block or menu**: four item declarations, the jar's four item models and textures, and the
regenerated table. It sits on the `deep` row exactly as slice 9 did (the machines-shaped feature is
already shipped; this slice completes its data so no washer row dead-ends).

### What the jar says (re-read from `v1.21.1-10.7.19.85`, `a00109e4`)

| value | source | port |
|---|---|---|
| four more rows, `200` units → `1` crystal, `clean_<m>` → `crystal_<m>` for lead, osmium, tin, uranium | `data/mekanism/recipe/processing/{lead,osmium,tin,uranium}/crystal/from_slurry.json` | the regenerated `crystallizing::TABLE` (7 rows) |
| the four items' art | `assets/mekanism/{textures/item,models/item}/crystal_{lead,osmium,tin,uranium}.*` | `tools/carry_crystallizer.py` (now 18 carried files) |
| the four names | `item.mekanism.crystal_{lead,osmium,tin,uranium}` in the jar's `en_us.json` | `assets/mekanism/lang/en_us.json` |
| the stack size | the same `minecraft:gunpowder` template as iron/gold/copper (64) | `data/mekanism/horizon/item/crystal_<m>.json` |

`gen_crystallizing.py` now reports **10 source recipes, 7 rows, 3 skipped**: the rows it still leaves
out are `lithium` → `dust_lithium`, `brine` → `salt` and `antimatter` → `pellet_antimatter`, whose
output items this port does not register (gap 2 below).

### The family in the port's three layers

- **`src/runtime.rs`** — no new rule. The generated `crystallizing::TABLE` grows 3 → 7 rows, so
  `ROW_COUNT`, `recipe_for`, `accepts`, `slurries` and `slurries_that_fit` all read the new rows;
  the tests pin the four new materials, the table's order and the stall rule at a **late** row.
- **`src/support.rs`** — unchanged: the crystallizer's tick, tank, draw loop, info line and
  container are generic over the table, which is why the four rows work through the shipped glue.
- **Content** — four `data/mekanism/horizon/item/crystal_<material>.json`, four carried item models
  and textures, four lang keys and four creative-tab entries.

### Red → green

The tests were written first and ran red on the untouched table
(`../_shared/evidence/mekanism/slice10-unit-red.log`), **121 passed, 4 failed**, e.g.:

```
---- runtime::crystallizer_tests::every_crystallizing_row_is_the_jar_s stdout ----
thread 'runtime::crystallizer_tests::every_crystallizing_row_is_the_jar_s' panicked at src/runtime.rs:5911:9:
assertion `left == right` failed: the seven slurries the washer makes, each with its crystal
  left: 3
 right: 7

---- runtime::crystallizer_tests::the_four_new_crystals_complete_every_slurry_the_washer_makes stdout ----
thread 'runtime::crystallizer_tests::the_four_new_crystals_complete_every_slurry_the_washer_makes' (90632) panicked at src/runtime.rs:5956:36:
no row for mekanism:clean_lead: the washer can make it

test result: FAILED. 121 passed; 4 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Then green: **125 tests** (was 124; +1 rule test, and three existing tests extended).

### Coverage — every family this slice ships

Per the parity rule (AGENTS.md, 2026-11-20):

| family | status | proof |
|---|---|---|
| `mekanism:crystal_{lead,osmium,tin,uranium}` as items: give-able, in the creative tab, named | **tested** | A1 now gives **20** items (`… twenty items …` in the PASS line) |
| the four new crystallizing rows: input amount, output id and count, slurry consumed, one operation's FE | **tested** | O: `rows crystallized: 7 of 7` (`tools/crystal_sweep.py`), each row `(tank 0, energy 80000)` |
| a lead tank's 1,000 units beside one charge: exactly two `crystal_lead`, neighbour 600, machine tank empty | **tested** | N0: `0/160000 FE, progress 0/200, input empty x0, output mekanism:crystal_lead x2`, `energy 0`, `holds mekanism:clean_lead 600/64000 (answered by mekanism)`, `holds nothing 0/10000` |
| the stall rule at a late row (`crystal_lead` must ask for `clean_lead`, not the earlier rows) | **tested** | `a_slot_of_one_crystal_asks_for_that_materials_slurry_and_skips_the_rest` (unit) + N0 running **two** operations |
| `mekanism:dust_lithium`, `mekanism:salt`, `mekanism:pellet_antimatter` | **not ported** | their output items are not registered; the tool prints the three drops |
| the enricher leg for these four materials (`crystal_<m>` → `shard_<m>`, `dust_<m>`) | **not ported** | `mekanism:shard_{lead,osmium,tin,uranium}` and `dust_{lead,osmium,tin,uranium}` are not registered, so `gen_enriching.py` keeps dropping those rows (gap 2) |
| everything slice 9 proved (blocks, recipes, loot, sweeps, factories, persistence) | **still green** | the same package leg: `PASS` + `EXIT:0`, 7/7 crystallizing rows, all six e2e logs green |

### The ladder (one lease per step, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result |
|---|---|---|
| the four rows + the late-row stall case, RED first | `cargo test --offline` on the untouched table | **121 passed, 4 failed** (`slice10-unit-red.log`) |
| tests | `cargo test --offline` | **125 passed**, 0 failed (`slice10-test.log`, `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`slice10-clippy.log`) |
| format | `cargo fmt --check` (never `--all`) | clean (`slice10-clippy.log`) |
| recipe tables | the seven `gen_*.py --check` + `carry_crystallizer.py --check` (`HORIZON_RES` set) | 163 / 132 / 148 / 72 / 15 / 7 / **7** rows; **18** carried files match the jar (`slice10-recipe-tables.log`) |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` | **197,883 B**, `cmp` identical, sha256 `3f72a2ac…08a025` (`slice10-build.log`, `slice10-build-second.log`, `slice10-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | exit 0, `mekanism 0.2.9 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, 262 entries (`slice10-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, exit 0 (`slice10-fits.log`, sha printed beside it) |
| fits, a different build | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, exit 0 — the run that proves something (`slice10-fits.log`) |
| archive | `unzip -l mekanism.hmod` | 262 entries, no `Cargo.lock` (`slice10-entries.txt`) |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (262 entries, no packed `Cargo.lock`) |
| launcher build | `launcher-game.sh build mekanism` (in a lease, after removing every `native-*` tag that carried this mod) | `Compiling mekanism (1 of 1)`, `native-0a875da-40fc7fa7ae4b` (`slice10-launcher-build.log`) |
| in game | `build-slot.sh run --step e2e -- bash tools/e2e.sh` on the launcher build | **`PASS` + `EXIT:0`** (`../_shared/evidence/mekanism-e2e/e2e-package.log`) |
| unmodded game | `GAME=$CONTROL_GAME CONTROL=0 tools/e2e.sh` | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** (`e2e-control.log`) |
| fail-capable | `SELFTEST=1 tools/e2e.sh` | **`FAIL: expected /mekanism info at 1,72,14 to say 'output empty x0', got '… output mekanism:crystal_lead x2'` + `EXIT:1`** (`e2e-selftest.log`) |
| differential | `NOCABLE=1 tools/e2e.sh` | **`PASS` + `EXIT:0`** (`e2e-nocable.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |

### In-game evidence (`--offscreen`, the launcher build)

`../_shared/evidence/mekanism-e2e/e2e-package.log`, this slice's phases:

```
== N0: slice 10's four crystals — 1,000 units of clean_lead, one charge, two crystal_lead ==
  chemical insert 1000 mekanism:clean_lead 0 72 14: chemical: inserted 1000 mekanism:clean_lead into mekanism:basic_chemical_tank (1000/64000)
  info(1,72,14): mekanism:chemical_crystallizer: 0/160000 FE, progress 0/200, input empty x0, output mekanism:crystal_lead x2
  energy(1,72,14): 0
  chemical read 0 72 14: chemical: mekanism:basic_chemical_tank holds mekanism:clean_lead 600/64000 (answered by mekanism)
  chemical read 1 72 14: chemical: mekanism:chemical_crystallizer holds nothing 0/10000 (answered by mekanism)
…
  mekanism:clean_copper x200 -> mekanism:crystal_copper x1 (tank 0, energy 80000)
  mekanism:clean_gold x200 -> mekanism:crystal_gold x1 (tank 0, energy 80000)
  mekanism:clean_iron x200 -> mekanism:crystal_iron x1 (tank 0, energy 80000)
  mekanism:clean_lead x200 -> mekanism:crystal_lead x1 (tank 0, energy 80000)
  mekanism:clean_osmium x200 -> mekanism:crystal_osmium x1 (tank 0, energy 80000)
  mekanism:clean_tin x200 -> mekanism:crystal_tin x1 (tank 0, energy 80000)
  mekanism:clean_uranium x200 -> mekanism:crystal_uranium x1 (tank 0, energy 80000)
rows crystallized: 7 of 7
… PASS: … EXIT:0
```

What the numbers prove:

- the lead tank went 1,000 → **600** and two `crystal_lead` appeared for one 160,000 FE charge, so
  the new row costs one operation's 80,000 FE and 200 units exactly as iron/gold/copper do, and the
  buffer still holds exactly two operations;
- the machine's own tank reads empty, so the drawn 200 units are spent at the operation's finish;
- every one of the **seven** carried rows ran in a real crystallizer and left `tank 0, energy 80000`
  — no washer recipe dead-ends any more;
- the selftest flips **this** assertion (it expects `output empty x0` at 1,72,14 and gets
  `crystal_lead x2`), so the newest family's check is the one proven able to fail;
- the older legs are unchanged: the control game cannot give the blocks (`EXIT:1`), the differential
  without the cable still passes, and every machine's FE/progress/tank survives save + relaunch.

All **four** legs of this slice were run on the final launcher build
`native-0a875da-40fc7fa7ae4b`, whose exported `horizon-native-mods/mekanism/` is `cmp`-identical to
this tree (README, `src/runtime.rs`, `horizon-mod.toml`, the four item files, the lang file), so the
`PASS`/`EXIT` verdicts belong to the package recorded below and not to an earlier build.

### What the harness taught this slice

- **`native-*` tags in the launcher's versions folder carry this mod even when another job built
  it.** The folder held a stale 0.2.8 tag, and while this slice ran, concurrent jobs' builds (the
  shared `e2e` instance compiles every installed mod) produced **three** further 0.2.9 tags carrying
  this working tree. `built_tag` accepts any tag whose packaged manifest matches the port, so the e2e
  could have run a binary nobody had watched compile. All mekanism-carrying tags were removed before
  the final build, and the log's `Compiling mekanism (1 of 1)` plus `cmp` on the exported
  `horizon-native-mods/mekanism/` (README, `src/runtime.rs`, `horizon-mod.toml`, the four item files,
  the lang file — all identical to the tree) are the proof the four legs ran this slice's tree.
- **A phase that adds no block still needs a free column.** N0 starts a tank at `0,72,14` and a
  crystallizer at `1,72,14`; `z=10` already held slice 9's tank/crystallizer/hopper/chest chain and
  `z=12` the dirty-slurry control, so the new pair took the next free row rather than reusing one.
- The e2e is run from the **port** directory: the slot runner runs the command in the caller's cwd,
  and `bash tools/e2e.sh` from the campaign root is `EXIT:127`.

### Gaps after slice 10

1. The **enricher leg** for the seven crystals: the jar's `crystal_<m>` → `shard_<m>` and `dust_<m>`
   rows are still dropped because `mekanism:shard_{lead,osmium,tin,uranium}` and
   `mekanism:dust_{lead,osmium,tin,uranium}` are not registered (for iron/gold/copper the shards
   are, and the tool drops those three rows for the same reason). Registering the four shards and
   four dusts would close the chain into the smelter.
2. `mekanism:dust_lithium`, `mekanism:salt`, `mekanism:pellet_antimatter`: their output items are
   not registered, so the jar's three non-slurry crystallizing rows stay dropped.
3. The chemical-container and energy slots, the composite model, the `active` state and the GUI are
   slice 9's recorded deviations and still stand.
4. Everything else in *Gaps / deviations*, *Gaps after slice 8* and *Gaps after slice 9* still
   stands.

### `## Upstream defects fixed`

None found in this slice: the jar's four rows are consistent with its own items, textures and lang
keys, and the port's table regeneration reproduces them exactly.

### Deviations this slice records

- The four crystals are **plain items** on the gunpowder template with the jar's own textures — no
  custom item behaviour and no `mekanism:crystal` component, exactly as iron/gold/copper.
- The creative tab carries the four beside the existing three, in material order.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

### Changelog (0.2.9) — supersedes 0.2.8

> **0.2.9 — the four missing crystals.** The Chemical Crystallizer now runs **every** row the washer
> can feed it: `mekanism:crystal_{lead,osmium,tin,uranium}` join iron, gold and copper, so all
> **seven** `mekanism:crystallizing` slurries crystallize — 200 units of a clean slurry into one
> crystal for 80,000 FE — and no washer recipe dead-ends. The four items are the jar's own textures
> and models, named in lang and listed in the creative tab. Worked in game: a tank of 1,000
> `mekanism:clean_lead` beside a crystallizer charged with 160,000 FE yields exactly **two**
> `crystal_lead` with the neighbour at **600** and the buffer at 0; the sweep runs **7 of 7** rows
> and every one leaves `tank 0, energy 80000`; `SELFTEST=1` fails on the lead phase's own assertion
> (`output empty x0` vs `crystal_lead x2`); the unmodded game still cannot give the blocks and the
> no-cable differential still passes; every earlier leg is unchanged. Package: **262 entries,
> 197,883 B**, sha256 `3f72a2acaeed253f580e975d3fb6a7cc376bca57732c535b6e5e470e1b08a025`; `check`
> exit 0; `fits` `made for` at the pin and `fits` at `0b14f8a`; 125 tests, clippy `-D warnings`
> clean; in game `PASS` + `EXIT:0`, control `EXIT:1`, selftest `EXIT:1`, nocable `PASS` + `EXIT:0`,
> log gate 6/6.

## Slice 11 — the crystal → shard injection leg (0.2.10, 2026-11-23)

### The scope authority, quoted

The gap is this port's own, "Gaps after slice 10" item 1 (and the slice-10 coverage row for the
four shards):

> **The enricher leg** for the seven crystals: the jar's `crystal_<m>` → `shard_<m>` and
> `dust_<m>` rows are still dropped because `mekanism:shard_{lead,osmium,tin,uranium}` and
> `mekanism:dust_{lead,osmium,tin,uranium}` are not registered (for iron/gold/copper the shards
> are, and the tool drops those three rows for the same reason). Registering the four shards and
> four dusts would close the chain into the smelter.

**Chosen: the seven `crystal_<material>` → `shard_<material>` rows** of the Chemical Injection
Chamber. It is not a new family — it is the shipped gas machine on seven more rows — so it needs no
new capability, hook, block, menu or state: `tools/gen_injecting.py` gains the `c:crystals/*`
expansion, four shard items are declared with the jar's own art, and the generated table grows
**72 → 79 rows**. It remains a machines-shaped feature on the ledger's `deep` row, as every slice
since 1 has been.

**The optional half was not taken.** The brief allowed the four `dirty_dust_*` + four `dust_*` items
"if room remains … to close the crushing → enriching → smelting leg". Measuring the jar, it does not
close that leg: the crushing step into a dirty dust needs `mekanism:clump_{lead,osmium,tin,uranium}`
(not registered, so `gen_crushing.py` still drops those rows), and the smelting step needs
`mekanism:ingot_{lead,osmium,tin,uranium}` (the jar's `processing/<m>/ingot/from_dust_smelting.json`
outputs those four ids, which this port has no item for), so eight more items would add four
enriching rows that dead-end in both directions. The port also has no enriching sweep to prove a row
in game, and the parity rule forbids carrying a recipe that is never crafted. They stay a gap, and
this slice says so rather than adding dead data.

### What the jar says (re-read from `v1.21.1-10.7.19.85`, `a00109e4`)

| value | source | port |
|---|---|---|
| seven rows, `1` crystal + `1` unit of hydrogen chloride **per tick**, `200` ticks an operation, output `1` shard | `data/mekanism/recipe/processing/{copper,gold,iron,lead,osmium,tin,uranium}/shard/from_crystal.json` (`type: mekanism:injecting`, `per_tick_usage: true`) | `injecting::TABLE`, rows 1–7 of the regenerated table |
| the input is a **tag**, `c:crystals/<material>`, not an item id | the same files | `ITEM_TAGS["c:crystals/<m>"] = ["mekanism:crystal_<m>"]`: this port's crystal is the only member the game has, exactly as the previous tag expansions name the only 26.3 members |
| the four new items' art | `assets/mekanism/{textures/item,models/item}/shard_{lead,osmium,tin,uranium}.*` | `tools/carry_shards.py` (14 files: the four new pairs, re-deriving iron/gold/copper to prove the earlier carry was faithful) |
| the four names | `item.mekanism.shard_{lead,osmium,tin,uranium}` in the jar's `en_us.json` | `assets/mekanism/lang/en_us.json` |
| the stack size | the same `minecraft:gunpowder` template as every other item | `data/mekanism/horizon/item/shard_<m>.json` |

`gen_injecting.py` now reports **91 source recipes, 79 rows, 15 skipped** (was 72/22): the 15 are the
two tags with no 26.3 member (`c:bricks/normal`, `c:gunpowders`), the twelve lead/osmium/tin/uranium
ore, raw-ore and raw-block rows (26.3 has no such ore) and the plutonium pellet row. Every one of the
22 rows the old tool dropped for a *crystal* is now carried.

### The slice in the port's three layers

- **`src/runtime.rs`** — no new rule: the gas machine is already generic over the table
  (`process_gas_tick`, `gas_pull`, `recipe_for`). The generated `injecting::TABLE` grows 72 → 79, so
  `ROW_COUNT` reads 79 and the machine serves the seven rows through the shipped glue. One new test
  states the family rule: **every crystal the crystallizer makes injects into that material's shard**
  — input 1, hydrogen chloride only, 1 unit a tick, output 1 — and the test asserts the crystal list
  is exactly `crystallizing::ROW_COUNT` long, so a future crystal cannot be added without its leg.
- **`src/support.rs`** — unchanged. The chamber's tick, tank, draw loop, info line and container are
  generic over the table, which is why the seven rows work without a line of glue.
- **Content** — four `data/mekanism/horizon/item/shard_<material>.json`, four carried item models
  and textures (plus the three existing ones re-derived), four lang keys, four creative-tab entries.

### Red → green

The tests were written first and ran red on the untouched table
(`../_shared/evidence/mekanism/slice11-unit-red.log`), **124 passed, 2 failed**:

```
---- runtime::injection_tests::every_crystal_the_crystallizer_makes_injects_into_its_material_s_shard stdout ----

thread 'runtime::injection_tests::every_crystal_the_crystallizer_makes_injects_into_its_material_s_shard' (172027) panicked at src/runtime.rs:4813:36:
no injecting row for mekanism:crystal_copper

---- runtime::injection_tests::the_injection_table_is_the_jar_s_recipes_over_this_port_s_items stdout ----

thread 'runtime::injection_tests::the_injection_table_is_the_jar_s_recipes_over_this_port_s_items' (172031) panicked at src/runtime.rs:4788:9:
assertion `left == right` failed: the rows the generator keeps from the jar's 91 injecting recipes
  left: 72
 right: 79

test result: FAILED. 124 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

Then green: **126 tests** (was 125; +1 family test and the table test's count raised to 79).

### Coverage — every family this slice ships

Per the parity rule (AGENTS.md, 2026-11-20):

| family | status | proof |
|---|---|---|
| `mekanism:shard_{lead,osmium,tin,uranium}` as items: give-able, in the creative tab, named | **tested** | A1 now gives **24** items (`… twenty-four …` in the PASS line) |
| the seven `crystal_<m>` → `shard_<m>` rows: input count, output id and count, 200 units of gas, one operation's 80,000 FE | **tested** | H: `rows injecting: 79 of 79`, each new row `crystal_<m> x1 + hydrogen_chloride x1/t -> shard_<m> (count 1)` with the tank at 4,800 of 5,000 |
| the lead headline: one crystal + 4,000 HCl in a 160,000 FE buffer → exactly one shard_lead, 80,000 FE left, tank 3,800, machine tank empty | **tested** | P: `80000/160000 FE, progress 0/200, input empty x0, output mekanism:shard_lead x1`, `energy 80000`, `holds mekanism:hydrogen_chloride 3800/64000`, `holds nothing 0/210` |
| the newest family's assertion is fail-capable | **tested** | `SELFTEST=1`: `FAIL: … to say 'output empty x0', got '… output mekanism:shard_lead x1'` + `EXIT:1` |
| `mekanism:dirty_dust_{lead,osmium,tin,uranium}` + `mekanism:dust_{lead,osmium,tin,uranium}` | **not ported** | they do not close the leg: no clumps to crush into them and no `mekanism:ingot_*` to smelt them into (26.3 has neither), and no enriching sweep would prove the four rows — see "Gaps after slice 11" |
| the lead/osmium/tin/uranium **ore, raw ore and raw block** injecting rows | **not ported** | 26.3 has no lead/osmium/tin/uranium ore; the tool prints the twelve drops |
| everything slices 1–10 proved | **still green** | the same package leg: `PASS` + `EXIT:0`, 79/79 injecting rows, 148 crushing, 15 purifying, 7 washing, 7/7 crystallizing, all six e2e logs green |

### The ladder (one lease per step, `CARGO_INCREMENTAL=0`, no `--target-dir`)

| Step | Command | Result |
|---|---|---|
| the seven rows + the family test, RED first | `cargo test --offline` on the untouched table | **124 passed, 2 failed** (`slice11-unit-red.log`) |
| tests | `cargo test --offline` | **126 passed**, 0 failed (`slice11-test.log`, `sdk-test.log`) |
| lints | `cargo clippy --all-targets --offline -- -D warnings` | exit 0, 0 warnings (`slice11-clippy.log`, `sdk-clippy.log`) |
| format | `cargo fmt --check` (never `--all`) | clean (same step) |
| recipe tables | the seven `gen_*.py --check` + `carry_crystallizer.py --check` + `carry_shards.py --check` (`HORIZON_RES` set) | 163 / 132 / 148 / **79** / 15 / 7 / 7 rows; **18** + **14** carried files match the jar (`slice11-recipe-tables.log`) |
| package, twice | `rm -f Cargo.lock && cargo horizon-mod build` | **202,188 B**, `cmp` identical, sha256 `216345bb…533368b` (`slice11-build.log`, `slice11-build-second.log`, `slice11-reproducible.txt`) |
| registry check | `cargo horizon-mod check mekanism.hmod` | exit 0, `mekanism 0.2.10 (native-mod, format native-1/0a875da…/e59da90…, for 26.3)`, 274 entries (`slice11-check.log`, `sdk-check.log`) |
| fits, the pin | `cargo horizon-mod fits mekanism.hmod --build 0a875da` | `0a875da  made for`, exit 0, sha printed beside it (`slice11-fits.log`, `sdk-fits.log`) |
| fits, a different build | `cargo horizon-mod fits mekanism.hmod --build 0b14f8a` | `0b14f8a  fits`, exit 0 — the run that proves something (same log) |
| archive | `unzip -l mekanism.hmod` | 274 entries, no `Cargo.lock` |
| package gate | `python3 ports/tools/verify-sdk-package.py mekanism` | **1/1** (274 entries, no packed `Cargo.lock`) |
| launcher build | `launcher-game.sh build mekanism` (in a lease, after removing every `native-*` tag that carried this mod) | `Compiling mekanism (1 of 1)`, `native-0a875da-200d1e6142d3` (`slice11-launcher-build.log`) |
| in game | `build-slot.sh run --step e2e -- bash tools/e2e.sh` on the launcher build | **`PASS` + `EXIT:0`** (`../_shared/evidence/mekanism-e2e/e2e-package.log`) |
| unmodded game | `GAME=$CONTROL_GAME CONTROL=0 tools/e2e.sh` | **`FAIL: give mekanism:energized_smelter failed` + `EXIT:1`** (`e2e-control.log`) |
| fail-capable | `SELFTEST=1 tools/e2e.sh` | **`FAIL: expected /mekanism info at 1,72,16 to say 'output empty x0', got '… output mekanism:shard_lead x1'` + `EXIT:1`** (`e2e-selftest.log`) |
| differential | `NOCABLE=1 tools/e2e.sh` | **`PASS` + `EXIT:0`** (`e2e-nocable.log`) |
| log gate | `python3 ports/tools/check-e2e-logs.py mekanism-e2e --strict` | `logs=6 ok=6 missing-exit=0 bad-exit=0` |

The package-building steps ran with **`BUILD_LOCAL=1`**: the slot runner forwards `build`/`check` to
the second machine, whose cargo has no `cargo-horizon-mod` subcommand (`error: no such command:
horizon-mod`), while `test` and `lint` forward fine. The lease, the per-step job count and the
`CARGO_INCREMENTAL=0` rule are unchanged.

### In-game evidence (`--offscreen`, the launcher build)

`../_shared/evidence/mekanism-e2e/e2e-package.log`, this slice's phases:

```
== H: every carried injecting recipe in a real Chemical Injection Chamber ==
  rows from the jar's mekanism:injecting data: 79
  mekanism:crystal_copper x1 + mekanism:hydrogen_chloride x1/t -> {"count": 1, "item": "mekanism:shard_copper"} (chemical: mekanism:basic_chemical_tank holds mekanism:hydrogen_chloride 4800/64000 (answered by mekanism))
  mekanism:crystal_gold x1 + mekanism:hydrogen_chloride x1/t -> {"count": 1, "item": "mekanism:shard_gold"} (… 4800/64000 …)
  mekanism:crystal_iron x1 + mekanism:hydrogen_chloride x1/t -> {"count": 1, "item": "mekanism:shard_iron"} (… 4800/64000 …)
  mekanism:crystal_lead x1 + mekanism:hydrogen_chloride x1/t -> {"count": 1, "item": "mekanism:shard_lead"} (… 4800/64000 …)
  mekanism:crystal_osmium x1 + mekanism:hydrogen_chloride x1/t -> {"count": 1, "item": "mekanism:shard_osmium"} (… 4800/64000 …)
  mekanism:crystal_tin x1 + mekanism:hydrogen_chloride x1/t -> {"count": 1, "item": "mekanism:shard_tin"} (… 4800/64000 …)
  mekanism:crystal_uranium x1 + mekanism:hydrogen_chloride x1/t -> {"count": 1, "item": "mekanism:shard_uranium"} (… 4800/64000 …)
rows injecting: 79 of 79
sweep PASS: all 79 carried injecting recipes ran in a real machine …
…
== P: slice 11's crystal leg — one crystal_lead + 4,000 HCl -> one shard_lead ==
  chemical insert 4000 mekanism:hydrogen_chloride 0 72 16: chemical: inserted 4000 mekanism:hydrogen_chloride into mekanism:basic_chemical_tank (4000/64000)
  info(1,72,16): mekanism:chemical_injection_chamber: 80000/160000 FE, progress 0/200, input empty x0, output mekanism:shard_lead x1
  energy(1,72,16): 80000
  chemical read 0 72 16: chemical: mekanism:basic_chemical_tank holds mekanism:hydrogen_chloride 3800/64000 (answered by mekanism)
  chemical read 1 72 16: chemical: mekanism:chemical_injection_chamber holds nothing 0/210 (answered by mekanism)
…
  mekanism:clean_lead x200 -> mekanism:crystal_lead x1 (tank 0, energy 80000)
rows crystallized: 7 of 7
… PASS: … EXIT:0
```

What the numbers prove:

- all **seven** crystal rows ran in a real chamber from the item the crystallizer makes, each
  spending exactly 200 units of the tank's 5,000 (a tick's one unit over the 200-tick operation) and
  producing exactly one shard — no crystal the port ships dead-ends at the chamber any more;
- the lead headline charged the chamber to its whole 160,000 FE buffer and the operation spent
  exactly **80,000** (400 FE/t × 200 ticks), leaving `80000/160000` and the input slot empty with one
  `shard_lead` in the output — one crystal, one shard, one operation;
- the neighbour tank went 4,000 → **3,800** through `mekanism:chemical` and the machine's own 210-unit
  tank reads **0**, so the gas is drawn a tick at a time and spent, not buffered;
- the selftest flips **this** phase's assertion (`output empty x0` vs `shard_lead x1`), so the newest
  family's check is the one proven able to fail;
- the earlier legs are unchanged: 148 crushing, 15 purifying, 7 washing and 7/7 crystallizing rows,
  the five factories, the menus, save + relaunch, the control game's refusal (`EXIT:1`) and the
  no-cable differential (`PASS`).

All **four** legs ran on the final launcher build `native-0a875da-200d1e6142d3`, whose exported
`horizon-native-mods/mekanism/` is `cmp`-identical to this tree (`horizon-mod.toml`, `src/`, the lang
file, the four new item files, the creative tab, a new shard texture), so the `PASS`/`EXIT` verdicts
belong to the package recorded below.

### What the harness taught this slice

- **The brief's headline energy reading was arithmetically impossible, and the artefact wins.** It
  asked for `0/160000 FE` after one operation charged to 160,000; the jar's chamber is 400 FE/t ×
  200 ticks = **80,000** an operation, so a full buffer leaves **80,000**. The phase asserts
  `80000/160000 FE` and the exact 80,000 spend — the stronger claim, since it pins the operation's
  cost rather than the buffer running dry. (A crystallizer reads `0/160000` after two operations;
  that is where the number came from.)
- **Only one `native-*` tag carried this mod this time**, `native-0a875da-40fc7fa7ae4b` (slice 10's).
  It was removed before the final build, so the build log's `Compiling mekanism (1 of 1)` and
  `native-0a875da-200d1e6142d3` cannot be a stale binary another job produced. The other 24 tags in
  the versions folder carry other ports and were left alone.
- **A `c:` tag family whose member this port owns is carried, not dropped.** `c:crystals/<m>` was
  skipped for three slices because "no 26.3 member" meant "no vanilla member"; once the port registers
  the crystal itself, the tag's 26.3 expansion is that item, and the row is the jar's. The same
  reasoning is what keeps `c:ores/lead` dropped: 26.3 has no lead ore and this port registers none.
- **The sweep's row floor is a fail-capable assertion.** Phase H required `-ge 60`, which a
  regression to 72 would still pass; it now requires `-ge 79`, so losing the crystal rows fails the
  leg even if the unit test were deleted.
- **A headlined `setblock` in a chunk the player has left answers "Could not set the block"** — phase
  P's pair stands at y=72, z=16, and the player was in The End at y≈66..72 throughout; the helper's
  read-back check is what makes that visible instead of a silent no-op.

### Gaps after slice 11

1. **The four `dirty_dust_*` + four `dust_*` items** (slice 10's gap 1, second half): registering them
   adds four `mekanism:enriching` rows (`c:dirty_dusts/<m>` → `dust_<m>`), but the leg still does not
   close — the crushing step needs `mekanism:clump_{lead,osmium,tin,uranium}`, and the smelting step
   needs `mekanism:ingot_{lead,osmium,tin,uranium}`, which neither 26.3 nor this port has. A future
   slice that takes it should carry the four clumps and four ingots in the same version and add an
   enriching sweep, so the rows are provable rather than dead.
2. The jar's **lead/osmium/tin/uranium ore, raw ore and raw block** injecting rows (12) stay dropped:
   they need ores 26.3 does not have. Same for the tag's `c:storage_blocks/raw_*` members.
3. The jar's `data/c/tags/item/{crystals,shards}/<material>.json` and its `mekanism:tags/item/{crystals,shards}`
   parents are **still not declared** by the port (slice 10's note). The machine's table is Rust and
   the generator's `ITEM_TAGS` expansion plays the tag's role there, so nothing in this port breaks —
   but another mod asking the game for "any lead crystal" by tag would not see these items. Adding the
   14 tag files needs an in-game way to assert membership (the port has no tag readout today), so they
   stay a gap rather than an unprovable carry.
4. `mekanism:dust_lithium`, `mekanism:salt`, `mekanism:pellet_antimatter`: their output items are not
   registered, so the jar's three non-slurry crystallizing rows stay dropped (slice 10's gap 2).
5. The chemical-container and energy slots, the composite model, the `active` state and the GUI are
   slice 9's recorded deviations and still stand; everything else in *Gaps / deviations*, *Gaps after
   slice 8*, *Gaps after slice 9* and *Gaps after slice 10* still stands.

### `## Upstream defects fixed`

None found in this slice: the jar's seven rows are consistent with its own tag, items, textures and
lang keys, and the regenerated table reproduces them exactly (one crystal + 1 HCl/t → one shard).

### Deviations this slice records

- The four shards are **plain items** on the gunpowder template with the jar's own textures — no
  custom item behaviour, exactly as iron/gold/copper.
- `c:crystals/<material>` is expanded to this port's `mekanism:crystal_<material>`, the only crystal
  item the game has. In a pack that also installed another mod's crystals the tag would hold those
  too; here it holds one, and the rollout maps the jar's tag onto it rather than inventing an id.
- The creative tab carries the four beside the existing three, in material order.

Skills: superpowers/using-superpowers,brainstorming,writing-plans,test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices,rust-port

### Changelog (0.2.10) — supersedes 0.2.9

> **0.2.10 — the crystal → shard leg.** The Chemical Injection Chamber now runs the jar's
> `processing/<material>/shard/from_crystal.json` rows for **every** crystal the crystallizer makes:
> one `crystal_<material>` plus one unit of hydrogen chloride a tick, 200 ticks an operation, become
> one `shard_<material>` for 80,000 FE. `mekanism:shard_{lead,osmium,tin,uranium}` join iron, gold
> and copper (the jar's own textures, models, names and tab entries), so the chain runs
> washer → crystallizer → injection chamber → enriching chamber without a dead end on the crystal
> side. The injecting table is **79 rows of the jar's 91** (72 before; 15 skipped, none for a
> crystal). Worked in game: a 4,000-unit hydrogen chloride tank beside a chamber charged with
> 160,000 FE turns one `crystal_lead` into exactly one `shard_lead`, leaving `80000/160000 FE`,
> `progress 0/200`, the input empty and the neighbour at **3,800** with the machine's own tank at 0;
> the sweep runs **79 of 79** injecting rows, the seven crystal rows among them; `SELFTEST=1` fails
> on the lead injection's own assertion (`output empty x0`); the unmodded game still cannot give the
> blocks and the no-cable differential still passes; every earlier leg is unchanged. Package: **274
> entries, 202,188 B**, sha256 `216345bbec2e6136e03284bf2f9927ba8506c57f6269ee1f488626603533368b`;
> `check` exit 0; `fits` `made for` at the pin and `fits` at `0b14f8a`; 126 tests, clippy `-D
> warnings` clean; in game `PASS` + `EXIT:0`, control `EXIT:1`, selftest `EXIT:1`, nocable `PASS` +
> `EXIT:0`, log gate 6/6.
