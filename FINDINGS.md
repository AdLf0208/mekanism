# Mekanism — upstream analysis for slices 1–2

Ported from **Mekanism 10.7.19.85** (`mekanism/Mekanism`, **MIT**), the exact
ATM10 jar (`Mekanism-1.21.1-10.7.19.85.jar`, Modrinth SHA-1
`b78945c40cfe7640408f3fd1e44da385a8c8b805`). Behaviour was read from the
upstream **tag `v1.21.1-10.7.19.85`** (commit
`a00109e4856fd38b9c5b3dd7f22ce4a59cd65a80`, 2026-04-10), i.e. the source of the
installed jar, cloned read-only to `ports/_shared/upstream/mekanism`.

Files read (analysis only; the Rust is a clean-room rewrite):

| Area | Files |
|---|---|
| smelter | `common/tile/machine/TileEntityEnergizedSmelter.java`, `common/tile/prefab/TileEntityElectricMachine.java`, `common/tile/prefab/TileEntityProgressMachine.java`, `common/tile/prefab/TileEntityRecipeMachine.java`, `src/api/java/mekanism/api/recipes/cache/{CachedRecipe,CachedRecipeHelper,OneInputCachedRecipe}.java` |
| enrichment chamber (slice 2) | `common/tile/machine/TileEntityEnrichmentChamber.java` (same electric/progress/recipe prefabs), `src/api/java/mekanism/api/recipes/inputs/InputHelper.java`, `src/api/java/mekanism/api/recipes/outputs/OutputHelper.java`, `src/api/java/mekanism/api/recipes/cache/{CachedRecipe,CachedRecipeHelper}.java` |
| energy of a machine | `common/capabilities/energy/MachineEnergyContainer.java`, `common/block/attribute/AttributeEnergy.java`, `common/config/{UsageConfig,StorageConfig}.java` |
| energy cube | `common/tile/TileEntityEnergyCube.java`, `common/block/BlockEnergyCube.java`, `common/capabilities/energy/EnergyCubeEnergyContainer.java`, `common/tier/EnergyCubeTier.java` |
| cable | `common/content/network/transmitter/UniversalCable.java`, `common/tile/transmitter/TileEntityUniversalCable.java`, `common/tier/CableTier.java`, `common/util/CableUtils.java`, `common/content/network/EnergyNetwork.java` |
| recipes | `common/recipe/MekanismRecipeType.java` (`getRecipesUncached`, the `SMELTING` branch), the jar's `data/mekanism/recipe/**` |

## Slice-1 behaviour (upstream truth)

### Energized Smelter

- **Energy**: usage **50 FE/t** (`UsageConfig.energizedSmelter` default 50),
  buffer **20 000 FE** (`StorageConfig.energizedSmelter`). The buffer's default
  when unset is `usage × 20 × 20` ticks = two operations, which is exactly
  20 000. Input only: a machine's `EnergyContainer` is built with
  `notExternal` as the extract predicate, so nothing can pull energy *out* of
  a machine; insertion is allowed on every side.
- **Speed**: `BASE_TICKS_REQUIRED = 10 * 20 = 200` ticks per operation
  (`TileEntityElectricMachine`). One operation therefore costs
  `200 × 50 = 10 000 FE`.
- **Progress semantics** (`CachedRecipe.process` + `CachedRecipeHelper`):
  - energy is spent **every tick while operating** (`useEnergy(operations)`),
    and one `operatingTicks` is added per tick;
  - at `operatingTicks >= 200` the machine consumes **one input item** and
    produces the recipe output, then resets progress to 0
    (`finishProcessing` → `inputHandler.use` + `outputHandler.handleOutput`);
  - **no energy** (`stored / 50 == 0`): `NOT_ENOUGH_ENERGY`, operations 0 →
    the machine goes inactive and **progress is held** (it is not reset);
  - **no/wrong input** (`mismatchedRecipe()` → operations −1): progress **is
    reset to 0**;
  - **output full**: `NOT_ENOUGH_OUTPUT_SPACE`, operations 0 → progress is
    held, no energy spent (energy is only taken when operations > 0);
  - the input is read with `getRecipeInput` each tick (the ingredient's stack
    size does not matter; one item is consumed per operation).
- **Output**: one `ItemStack` — a stack of the recipe's result, so the count
  is whatever the recipe says (1 for every vanilla smelting recipe).
- **Recipes**: the smelter's recipe type is `mekanism:smelting`, but
  `MekanismRecipeType.getRecipesUncached` **adds every `minecraft:smelting`
  recipe** to that type (synthesised as a one-item-to-item recipe), so the
  Energized Smelter smelts anything a furnace can. The jar ships **no**
  `"type": "mekanism:smelting"` files (verified: 0 matches in
  `data/mekanism/recipe/**`), so in ATM10 its recipe set is exactly the
  furnace smelting set of the pack. The port therefore carries the **26.3
  vanilla smelting table** (73 recipe files, generated into
  `mixins/mekanism/smelting.rs` by `tools/gen_smelting.py`).
- **Slots**: input, output and an energy slot (a battery item charges the
  machine). The energy slot is out of slice 1 (no item energy in the ATM10
  jar's basic tier here); the menu is the vanilla furnace menu with 3 slots:
  slot 0 input, slot 1 unused (was the fuel slot), slot 2 output.

### Basic Energy Cube

- **Storage 4 000 000 FE**, **output/transfer 4 000 FE/t** (`EnergyCubeTier`
  `BASIC(4_000_000, 4_000)`). Both insert and extract are unlimited from
  automation (the tier rate only limits the internal item-charging path), so
  the port caps one call at the tier rate (4 000) — a documented deviation
  that keeps the buffer inside `i32` and matches the cable tier.
- The cube **does not tick**: it is storage only.
- Right-click opens a GUI (energy gauge); the port exposes the amount through
  `/mekanism info` instead (no custom screen in slice 1).

### Basic Universal Cable

- A cable is a **buffered transmitter**: `BasicEnergyContainer` with capacity
  = `CableTier.BASIC.getCableCapacity()` = **8 000**; the buffer is the
  cable's throughput. `pullFromAcceptors` pulls up to
  `min(capacity, needed)` from every side whose connection type is PULL, and
  `EnergyNetwork` emits the buffer to the non-pull acceptors each tick.
- Cables connect to each other into one network whose capacity is the sum of
  the cable buffers, so a line relays up to 8 000 FE/t per cable and the
  network's stored energy is shared.
- The port keeps a **per-cable 8 000 buffer** and, on the cable's own block
  entity tick, **pulls from every adjacent energy provider and pushes into
  every adjacent acceptor** (both capped by the 8 000 buffer). A line of
  cables therefore relays hop by hop (up to N hops per tick when the game
  ticks them in order), not as one instantaneous network — a documented
  deviation from the shared `EnergyNetwork` buffer.
- Redstone control, side configuration, connection visuals and the
  `Configurator` are out of slice 1.

### Enrichment Chamber (slice 2)

- **Energy**: usage **50 FE/t** (`UsageConfig.enrichmentChamber`, default 50),
  buffer **20 000 FE** (`StorageConfig.enrichmentChamber`, default 20 000).
  Like every machine its `EnergyContainer` only takes energy (the extract
  predicate is `notExternal`), so nothing can pull energy out of it.
- **Speed**: `BASE_TICKS_REQUIRED = 10 * 20 = 200` ticks
  (`TileEntityElectricMachine`, the same base the smelter uses), so one
  operation costs `200 × 50 = 10 000 FE`.
- **Recipe type**: `mekanism:enriching` (`MekanismRecipeType.ENRICHING`,
  `ItemStackToItemStackRecipe`), **not** the smelter's type and with no
  vanilla recipes added (unlike `SMELTING`, `getRecipesUncached` adds nothing
  to `ENRICHING`).
- **Input count matters**: `InputHelper.use(recipeInput, operations)` shrinks
  the slot by `recipeInput.getCount() * operations`, and
  `calculateOperationsCanSupport` divides the stack by that count; too few
  items is `NOT_ENOUGH_INPUT`, which **resets the progress** (it does not hold
  it). The raw-ore recipes take **3** items per operation, everything else 1.
- **Progress/output semantics**: the same `CachedRecipe` code path as the
  smelter (energy per operating tick, one `operatingTicks` step, consume and
  produce at 200; hold on no energy or no output room; reset on a mismatched
  recipe or too few items).
- **Recipes** (jar `data/mekanism/recipe/`, all `mekanism:enriching`):
  - ore → dust: `processing/<material>/dust/from_ore.json`, input
    `#c:ores/<material>` ×1, output **2** dust (iron, gold, copper; also
    osmium/tin/lead/uranium, whose items this slice does not register);
  - raw ore: `from_raw_ore.json`, `#c:raw_materials/<material>` ×**3** →
    **4** dust;
  - raw block: `from_raw_block.json`, `#c:storage_blocks/raw_<material>` ×1 →
    **12** dust;
  - vanilla-item enrichings: `#c:ores/coal` → 2 coal, diamond → 2,
    emerald → 2, lapis → 12, quartz → 2, redstone → 12,
    `#c:ores/netherite_scrap` (ancient debris) → 2 scrap;
  - conversions with only vanilla items: `#c:gravels` → flint,
    `#c:sands` → gravel, `#c:gunpowders` → flint (the port carries these
    three as well, since both sides exist).
  - **Tags**: the pack's `c:` tags hold modded members the port does not
    register, so `tools/gen_enriching.py` expands each tag to the members
    26.3 has and drops a row whose output the port does not register
    (dusts/dyes/`enriched_*`/`salt`/`string`/`hdpe`).
- **Dusts → ingots**: the jar's `processing/<material>/ingot/from_dust_smelting.json`
  are plain `minecraft:smelting` recipes (dust → 1 ingot, 200 ticks), so they
  belong to the smelter's recipe set, not the chamber's; the port adds the
  three it carries (iron, gold, copper) to its smelting lookup.
- **Block**: `BlockFactoryMachine`, hardness 3.5 / resistance 16 (the machine
  family's numbers the smelter also uses), the upstream `facing`/`active`
  states (the port's barrel template has `facing`/`open`, so it keeps the
  idle front — the same deviation the smelter records), and the carried
  `machine` model with the chamber's own textures (front, front_active, left,
  right, back, top, bottom).
- **Menu**: the same three slots as the smelter (input 0, unused 1, output 2)
  shown through the vanilla furnace menu; upstream's own screen is out of
  scope.

## Horizon API at the pin (corrects ARCHITECTURE.md §3's note)

`ARCHITECTURE.md` §3 records "there is no FE hook yet". At the campaign pin
`1d63e0a5` that is **no longer true** — API **142**
(`docs/native-mods.md:1194`) adds:

- `native_block_entity_energy(block, &mut Option<ModEnergyBuffer>)` — declare a
  block entity's energy buffer (`capacity`, `max_insert`, `max_extract`,
  insert/extract faces); the amount is the int **`energy`** in the entity's
  saved compound, exactly the "energy in the BE's NBT" shape the architecture
  ruling wants, so the amount survives a save and the block's own tick reads
  it;
- `native_energy(dimension, pos, face)`, `native_energy_insert(pos, face,
  amount, simulate)`, `native_energy_extract(dimension, pos, face, amount,
  simulate)`, `native_move_energy(...)` (`crates/horizon-client/src/game/
  machines/native_energy.rs`).

The port uses it for the buffer declaration and for the cable's cross-block
moves; the cable's own buffer is read and written through the `&mut
ModBlockEntity` its tick receives, because the engine takes a ticking block
entity out of the world (`native_energy_at` returns `None` for it,
`native_energy.rs:71-79`). This is a *capability the campaign note did not
know about*, not a change of shape: the energy still lives in the block
entity's NBT and the cable still balances it on its own tick.

## Next slices (planned gap list)

1. **Ore processing chain (the rest of the tier)**: the **Enrichment
   Chamber** (slice 2) and the **Crusher** (slice 3, `mekanism:crushing`, the
   same `ItemStackToItemStackRecipe` machine shape) are done; next are the
   Purification Chamber (needs oxygen, a gas system), Ore Washer (needs water)
   and Chemical Injection Chamber (gases). The gas-based ones need a gas
   capability that does not exist at the pin.
2. **Factories** (`basic_smelting_factory` etc.): the same recipe in parallel,
   hearts of the `TileEntityFactory` family; needs multi-operation-per-tick
   (`operationsPerTick`) and a larger energy buffer.
3. **Tiers**: advanced/elite/ultimate energy cubes and cables (same code,
   different constants — a tier table), the creative cube.
4. **Config and upgrades**: `config/mekanism/…` usage/storage values,
   speed/energy upgrades (`ticksRequired`, `energyPerTick` multipliers),
   redstone control, side configuration, the Configurator, ejector.
5. **GUI/screen**: Mekanism's own machine GUIs, energy gauges, the cube's
   screen; the pin's vanilla furnace menu stands in for the smelter.
6. **Machines that need new capabilities**: Metallurgic Infuser, Thermal
   Distillation, Electrolytic Separator, Digital Miner (requires a gas
   capability and a mod-defined resource type).
7. **Deep tier**: jetpack/armour (`Mekanism Tools`), teleporters, the
   Digital Miner's renderer, the fusion reactor (Mekanism Generators),
   multiblocks; all out of scope here.
8. **Item energy**: the energy tablet and the machine energy slot need
   `native_item_energy_buffer` (API 307) plus a battery item.

## Slice boundary (what this port ships, slices 1–2)

- `.hmod`: `mekanism:energized_smelter`, `mekanism:enrichment_chamber`,
  `mekanism:basic_energy_cube`, `mekanism:basic_universal_cable` and the
  `mekanism:dust_{iron,gold,copper}` items — block/item declarations, carried
  models/textures/blockstates/lang, the vanilla-shaped self-drop loot tables,
  the upstream crafting recipes (carried as-is; their Mekanism ingredients do
  not exist yet, recorded as a gap), a creative tab.
- `.hnmod`: the four block entities, both machines' energy + progress + slots
  and furnace menu, the cube's 4 M FE buffer, the cable's 8 000 FE/t transfer,
  and `/mekanism info|energy|progress`.
- Unit-tested pure core: `process_tick` (the shared machine operation),
  `cable_plan`, energy clamping, the smelting and enriching tables.
