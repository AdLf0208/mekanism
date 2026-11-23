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

1. **The four missing crystals** (`crystal_{lead,osmium,tin,uranium}`): the washer already makes their
   clean slurries and `gen_crystallizing.py` drops those four rows because the items do not exist. Four
   items (jar textures) and the table's `--check` closes the washer → crystallizer → enricher chain for
   every slurry this port can make. Smallest next slice with a user-visible gain.
2. **Ore-processing chain, remaining**: the **Osmium Compressor** (needs osmium and a refined-ingot
   chain), the **Chemical Dissolution Chamber** (needs sulfuric acid + an ore-shard family) and the
   **Chemical Oxidizer** (item → gas). The dissolution chamber is the one the brief allowed as a
   stretch and is the next real machine; it needs `mekanism:sulfuric_acid` (already a registered
   chemical) plus a per-material ore → slurry table.
3. **Factories, higher tiers and the rest of the family**: advanced/elite/ultimate factories, the
   creative cube, the advanced cables' tiers.
4. **Tiers**: advanced/elite/ultimate energy cubes and cables (same code, different constants — a tier
   table).
5. **Config and upgrades**: `config/mekanism/…` usage/storage values, speed/energy upgrades
   (`ticksRequired`, `energyPerTick` multipliers), redstone control, side configuration, the
   Configurator, ejector.
6. **GUI/screen**: Mekanism's own machine GUIs, energy gauges; the pin's vanilla furnace menu stands in
   for the item machines.
7. **Machines that need new capabilities or resources**: Metallurgic Infuser, Thermal Distillation,
   Electrolytic Separator, Digital Miner.
8. **Deep tier**: jetpack/armour (`Mekanism Tools`), teleporters, the Digital Miner's renderer, the
   fusion reactor (Mekanism Generators), multiblocks; all out of scope here.
9. **Item energy**: the energy tablet and the machine energy slot need `native_item_energy_buffer`
   (API 307) plus a battery item.

Done since this list was written: the Purification Chamber (slice 6), the five basic factories
(slice 7), the Chemical Washer (slice 8) and the Chemical Crystallizer (slice 9).

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

## Slice 4 — the chemical capability (upstream analysis)

Read for this slice (analysis only; the Rust is a clean-room rewrite):

| Area | Files |
|---|---|
| the tank | `common/tile/TileEntityChemicalTank.java`, `common/tier/ChemicalTankTier.java`, `common/capabilities/chemical/ChemicalTankChemicalTank.java`, `common/registries/MekanismBlockTypes.java` (`createChemicalTank`, `createMachine`) |
| the chemical registry | `common/registries/MekanismChemicals.java` (the 32 named chemicals), `common/ChemicalConstants.java` |
| the stack's saved shape | `src/api/java/mekanism/api/chemical/ChemicalStack.java` (`MAP_CODEC`: `{id, amount}`), `common/attachments/containers/chemical/AttachedChemicals.java` (`mekanism:chemicals`, a list of tanks) |
| the item | `common/item/ItemGaugeDropper.java`, `common/config/GearConfig.java` (`gaugeDropperCapacity`, `gaugeDroppedTransferRate`), `common/attachments/containers/chemical/ComponentBackedChemicalTank.java` |
| the capability | `common/capabilities/Capabilities.java` (`Capabilities.CHEMICAL`, block and item), `src/api/java/mekanism/api/chemical/IChemicalHandler.java` |

Numbers carried: tank **64,000** units (`ChemicalTankTier.BASIC`: 64 × `BUCKET_VOLUME`) and
**1,000/t** out (the port ships no ejector, so the tier's output rate is not used yet);
gauge dropper **16,000** capacity and **250** per call (`GearConfig` defaults); tank block
hardness **3.5** / resistance **16** with **no `active` and no `facing` state**
(`createMachine` minus `AttributeStateActive`); the tank's shape is a **custom inset**
(`BlockShapes.CHEMICAL_TANK`) the port does not carry.

Where the rewrite differs (and why):

- upstream's tanks are `IChemicalTank` objects inside the block entity with a holder per
  side; the port keeps the one tank in the block entity's **saved compound** and answers the
  capability from it, which is the SDK's own fixture shape (`test:chemical`) and survives a
  save because `merge_data` marks the chunk;
- the gauge dropper's payload is upstream's **attachment** (`mekanism:chemicals`, a list of
  stacks) with a fluid tank beside it; the port stores one chemical in the **item component**
  `mekanism:chemical` as `{id, amount}` (the stack's own codec shape) — one tank, no fluid
  side, which is this slice's scope;
- the chemicals are ids only: upstream registers each as a `Chemical` with a texture, a tint
  and (for gases) a bucket-like item; the port carries the names for validation and ships no
  chemical item.

Refinement of "Next slices" 1 and 6: the **gas capability they were blocked on now exists**
(`mekanism:chemical`), so the Purification Chamber, Ore Washer and Chemical Injection Chamber
are unblocked on the capability and blocked only on their own machines and on the chemicals as
items; the Metallurgic Infuser and Electrolytic Separator likewise.

## Slice 5 — the Chemical Injection Chamber (upstream analysis)

Read for this slice (analysis only; the Rust is a clean-room rewrite):

| Area | Files |
|---|---|
| the machine | `common/tile/machine/TileEntityChemicalInjectionChamber.java`, `common/tile/prefab/TileEntityAdvancedElectricMachine.java` |
| the per-tick gas | `src/api/java/mekanism/api/recipes/cache/ItemStackConstantChemicalToObjectCachedRecipe.java` (`ChemicalUsageMultiplier.constantUse`), `src/api/java/mekanism/api/recipes/inputs/InputHelper.java` (`getConstantInputHandler`, `ChemicalInputHandler`) |
| the recipe | `src/api/java/mekanism/api/recipes/ItemStackChemicalToItemStackRecipe.java` (`per_tick_usage`), the jar's `data/mekanism/recipe/injecting/**` and `processing/<material>/shard/**` |
| the numbers | `common/config/UsageConfig.java` (`chemicalInjectionChamber`, `400L`), `common/config/StorageConfig.java` (`160_000L, 1`) |
| the block | `common/registries/MekanismBlockTypes.java` (`createMachine`), `assets/mekanism/blockstates/chemical_injection_chamber.json` |

Numbers carried: **400 FE/t** usage, **160,000 FE** storage, the shared **200**-tick operation
(**80,000 FE** each), and the machine's own **210**-unit gas tank (`MAX_GAS`). The gas is spent
**per tick** — `recipe.chemical_input.amount × 1` each tick for 200 ticks — because the jar's
constant-usage multiplier starts at `baseTotalUsage (= ticks required) / remaining ticks = 1` and
that multiplier is then multiplied by the recipe's amount. A gas the recipe does not take, or less
than one tick of it, **holds** the progress (the constant-usage handler overrides `resetProgress` to
`updateOperations(0)`); it does not reset it.

Where the rewrite differs (and why):

- upstream's tank is filled by automation **pushing** into the chemical capability and the machine
  only runs from its buffer; the port keeps the same 210-unit insert-only buffer and lets the machine
  **pull** one tick's gas from an adjacent handler through `mekanism:chemical` — the slice is the
  capability's consumer, and pulling makes the draw exactly `200 × per_tick`;
- `randomizedConsumption` (upstream default `true`, a statistical gas usage) is not modelled: the
  port has no config and spends the deterministic `constantUse` amount — same expected cost;
- the block is the barrel template's `facing`×`open` rather than `facing`×`active` (the other
  machines' recorded deviation); the `_active` model and textures are carried, unused.

Recipe table: the jar's **91** injecting recipes, **72** carried — an item tag expands per member
and a chemical tag is one row naming every member (`mekanism:water_vapor` = water vapour **or**
steam). The **22** skipped are printed by `tools/gen_injecting.py`: `c:bricks/normal`,
`c:gunpowders` (its output `mekanism:dust_sulfur` is not registered), `c:crystals/*`, the
lead/osmium/tin/uranium ore and raw tags, the plutonium pellet, and the `from_crystal` rows — every
one an item or an output this port does not register.

*(Slice 11 later carried the seven `from_crystal` rows: once the port registered the crystals, the
`c:crystals/<material>` tag has this port's `mekanism:crystal_<material>` as its 26.3 member, and the
four missing shards joined iron/gold/copper. The table is **79 rows of 91** now; the 15 skipped are
the two no-member tags, the twelve lead/osmium/tin/uranium ore/raw rows and the plutonium pellet.)*

The slice closes the "injection chamber" half of *Next slices* 1; the purification chamber and ore
washer remain, and the shards → clumps → dirty dust → dust path from here needs exactly those two.

## Slice 8 — the Chemical Washer (upstream analysis)

Read for this slice from the upstream **tag `v1.21.1-10.7.19.85`** (commit
`a00109e4856fd38b9c5b3dd7f22ce4a59cd65a80`), analysis only — the Rust is a clean-room rewrite, and
the jar's own data (`data/mekanism/recipe/processing/*/slurry/clean.json`) is the recipe source:

| Area | Files |
|---|---|
| the machine | `common/tile/machine/TileEntityChemicalWasher.java` |
| the recipe | `src/api/java/mekanism/api/recipes/FluidChemicalToChemicalRecipe.java`, `src/api/java/mekanism/api/recipes/cache/TwoInputCachedRecipe.java`, `CachedRecipe.java`, `CachedRecipeHelper.java`, `inputs/InputHelper.java` |
| the numbers | `common/config/UsageConfig.java` (`chemicalWasher`, `200L`), `common/config/StorageConfig.java` (`chemicalWasher`, `80_000L, 1`) |
| the slurries | `common/registries/MekanismChemicals.java` (`PROCESSED_RESOURCES` → `registerSlurry`), `common/registration/impl/ChemicalDeferredRegister.java`, `common/resource/PrimaryResource.java` |
| the block | `common/registries/MekanismBlockTypes.java`, `assets/mekanism/blockstates/chemical_washer.json`, `assets/mekanism/models/block/chemical_washer.json` |

Numbers carried: usage **200 FE/t**, storage **80,000 FE**, the shared **200**-tick operation
(**40,000 FE**), the water tank **10,000 mB** (`MAX_FLUID = 10 * BUCKET_VOLUME`, `BasicFluidTank.input`
— fill only, and only fluids some washing recipe takes, which is water), and the two slurry tanks of
**10,000** each (`MAX_SLURRY`, `inputModern` in / `output` out). The jar's seven washing rows are
**5 mB of water + 1 unit of a dirty slurry → 1 unit of its clean slurry** for copper, gold, iron,
lead, osmium, tin and uranium; the pack's CraftTweaker script only changes the *uranium* row to 10 mB,
which the port does not apply (no script support), so uranium is 5 mB here — a recorded deviation.

Where the rewrite differs (and why):

- upstream's washer is filled by automation pushing into its tanks; the port ships **no pipes**, so
  the machine **pulls** one operation's dirty slurry from an adjacent `mekanism:chemical` handler —
  the same recorded deviation the injection and purification chambers carry, and it draws only when it
  could complete an operation (the slice-7 gas-sip defect, guarded from the start);
- the water is the **game's own** `horizon:fluids` tank (`ModFluidTanks` + the entity's `fluids`
  list), not a port-defined key: `FluidResource` is one of the SDK's shared resource types, so the
  capability is answerable across packages and the e2e's read names `the game`;
- the four upstream slots (a fluid-container slot, its output, a slurry-output slot and an energy
  slot) are not carried: the tanks are the interface, and `/mekanism fluid read|insert`,
  `/mekanism chemical insert|drain-tank` and `/mekanism info` are the harness's path;
- both inputs are consumed at the operation's **finish** (`TwoInputCachedRecipe` overrides no
  `useResources`), which the port keeps and the persistence phase pins (a mid-operation washer still
  holds its water and slurry);
- the block's model and seven textures are the jar's **own** (28 elements, `cutout`); the jar's block
  has no `active` state at all, so the barrel template's unused `open` state is the only stand-in.

## Slice 9 — the Chemical Crystallizer (upstream analysis)

Read for this slice from the upstream **tag `v1.21.1-10.7.19.85`** (commit
`a00109e4856fd38b9c5b3dd7f22ce4a59cd65a80`), analysis only — the Rust is a clean-room rewrite, and the
jar's own data (`data/mekanism/recipe/processing/*/crystal/from_slurry.json` and
`recipe/crystallizing/*.json`) is the recipe source:

| Area | Files |
|---|---|
| the machine | `common/tile/machine/TileEntityChemicalCrystallizer.java`, `common/tile/prefab/TileEntityProgressMachine.java`, `TileEntityRecipeMachine.java` |
| the recipe | `src/api/java/mekanism/api/recipes/ChemicalCrystallizerRecipe.java`, `recipes/BasicChemicalCrystallizerRecipe.java`, `recipes/cache/OneInputCachedRecipe.java` (`crystallizing`), `recipes/cache/CachedRecipe.java`, `recipes/inputs/InputHelper.java` |
| the numbers | `common/config/UsageConfig.java:68` (`chemicalCrystallizer`, `400L`), `common/config/StorageConfig.java:86` (`160_000L, 1`), `TileEntityChemicalCrystallizer.java:61-62` (`MAX_CHEMICAL`, `BASE_TICKS_REQUIRED`) |
| the tank | `BasicChemicalTank.inputModern(MAX_CHEMICAL, this::containsRecipe, …)` (`:94`) — the predicate that refuses a chemical with no row |
| the block | `common/registries/MekanismBlockTypes.java` (`createMachine`), `assets/mekanism/blockstates/chemical_crystallizer.json` (`facing` only), `assets/mekanism/models/block/chemical_crystallizer.json` (`neoforge:composite`) |

Numbers carried: usage **400 FE/t**, storage **160,000 FE**, the shared **200**-tick operation
(**80,000 FE**, so the buffer holds exactly two), the tank **10,000** units
(`MAX_CHEMICAL = 10 * BUCKET_VOLUME`, insert-only, predicate `containsRecipe`), and the three
`processing/<material>/crystal/from_slurry.json` rows this port can finish —
**200 units of `clean_<m>` → 1 `crystal_<m>`** for copper, gold and iron, the slurry taken at the
operation's finish (`OneInputCachedRecipe.finishProcessing` → `inputHandler.use`).

The jar's ten `mekanism:crystallizing` recipes are: the three ore slurries above, `clean_{lead,osmium,
tin,uranium}` (dropped: those crystal items are not registered), `brine` → `salt` and `lithium` →
`dust_lithium` (dropped: not registered), and `antimatter` → `pellet_antimatter` (dropped). That is
the gap the next slice can close for four of them.

Where the rewrite differs (and why):

- upstream fills the tank through automation and has a **chemical-container input slot**; the port
  ships no pipes and no container slot, so the machine **pulls one operation's slurry** from an
  adjacent `mekanism:chemical` handler — and only when it could finish that operation (the crystal
  fits the output slot), the slice-7 gas-sip guard;
- the energy is the port's `block_energy` (insert-only faces) rather than
  `MachineEnergyContainer.input`, and the item/chemical/energy slot trio is not carried;
- the jar's model is a `neoforge:composite` (cutout + translucent children) and is flattened to one
  `cutout` model by `tools/carry_crystallizer.py`; the glass therefore draws cutout;
- the jar's block has `facing` only (no `active`), so the barrel template's unused `open` state stands
  in and nothing is lost visually beyond the template's shape;
- the shaped recipe uses vanilla stand-ins (obsidian, amethyst, redstone, iron block), the precedent
  of the fifteen earlier ones, because the real ingredients are not registered.

## Slice 10 — the four missing crystals (upstream analysis)

Read for this slice from the upstream **tag `v1.21.1-10.7.19.85`** (commit
`a00109e4856fd38b9c5b3dd7f22ce4a59cd65a80`), analysis only — the Rust stays a clean-room rewrite and
the jar's own data is the recipe source:

| Area | Files |
|---|---|
| the four recipes | `data/mekanism/recipe/processing/{lead,osmium,tin,uranium}/crystal/from_slurry.json` |
| the four items' art | `assets/mekanism/models/item/crystal_<m>.json`, `assets/mekanism/textures/item/crystal_<m>.png` |
| the four names | `assets/mekanism/lang/en_us.json` (`item.mekanism.crystal_<m>`) |
| the tags they belong to | `data/c/tags/item/crystals/<m>.json` (not carried: this port declares no `c:` item tags) |

Numbers carried: `{"amount": 200, "chemical": "mekanism:clean_<m>"}` → `{"count": 1, "id":
"mekanism:crystal_<m>"}` for each of the four, i.e. exactly slice 9's row shape. With them the jar's
ten `mekanism:crystallizing` recipes split as seven carried slurry rows and three dropped non-slurry
rows (`lithium` → `dust_lithium`, `brine` → `salt`, `antimatter` → `pellet_antimatter`), which is
`gen_crystallizing.py`'s `10 / 7 / 3` header.

Where the rewrite differs (and why):

- the crystals are **plain items** (the `minecraft:gunpowder` template, stack 64) with the jar's own
  textures/models — no `mekanism:crystal` component and no custom behaviour, the same deviation
  slice 9 recorded for iron/gold/copper;
- the port does not declare the `c:crystals/<m>` item tags the jar ships, so other mods cannot ask
  for "any lead crystal" by tag here; a later slice would add them with the dusts;
- the enricher leg (`crystal_<m>` → `shard_<m>`, and later `dust_<m>`) is **not** opened, because
  `mekanism:shard_{lead,osmium,tin,uranium}` and `mekanism:dust_{lead,osmium,tin,uranium}` are not
  registered; `gen_enriching.py` keeps dropping those rows (gap 1 after slice 10).

## Slice 11 — the crystal → shard leg (upstream analysis)

Read for this slice from the upstream **tag `v1.21.1-10.7.19.85`** (commit
`a00109e4856fd38b9c5b3dd7f22ce4a59cd65a80`) and from the installed jar
`Mekanism-1.21.1-10.7.19.85.jar` (SHA-1 `b78945c40cfe7640408f3fd1e44da385a8c8b805`) — analysis
only; the Rust is a clean-room rewrite and the recipe source is the jar's data:

- `data/mekanism/recipe/processing/{copper,gold,iron,lead,osmium,tin,uranium}/shard/from_crystal.json`
  — `type: mekanism:injecting`, `item_input: {count: 1, tag: c:crystals/<material>}`,
  `chemical_input: {amount: 1, chemical: mekanism:hydrogen_chloride}`, `per_tick_usage: true`,
  `output: {count: 1, id: mekanism:shard_<material>}`. Seven files, one per washer material — the
  leg upstream runs after the crystallizer (`shard → clump → dirty dust → dust → ingot`);
- `data/c/tags/item/crystals/<material>.json` and `data/mekanism/tags/item/crystals.json` — the tag
  the recipe names; the jar's members are its own crystals;
- `assets/mekanism/{textures/item,models/item}/shard_<material>.*` — the seven shard items' art;
- `item.mekanism.shard_<material>` in `assets/mekanism/lang/en_us.json`;
- `data/mekanism/recipe/processing/<material>/ingot/from_dust_smelting.json` — where the chain would
  end (`mekanism:ingot_lead`, `ingot_osmium`, `ingot_tin`, `ingot_uranium`), which is why the
  optional dust items do not close the leg here.

Behaviour extracted: the operation is the shared `ItemStackChemicalToItemStack` machine — 400 FE/t ×
200 ticks = 80,000 FE, one unit of gas **per tick** through `ChemicalUsageMultiplier.constantUse`
(200 units an operation), the item consumed when the operation finishes, one output stack of one.
No new machine, state or predicate: the crystal is an ordinary input item with a row.

Where the rewrite differs (and why):

- `c:crystals/<material>` is expanded by `tools/gen_injecting.py` to this port's
  `mekanism:crystal_<material>` — the only crystal item the game has here — and the port still does
  not declare the jar's item tags (recorded as a gap in `notes.md`); the machine's table is Rust, so
  gameplay does not depend on the tag;
- the four new shards are plain items on the `minecraft:gunpowder` template (stack 64), the same
  deviation every item in this port records, with `tools/carry_shards.py` carrying their art
  reproducibly;
- the 12 ore/raw injecting rows for lead, osmium, tin and uranium stay out: 26.3 has no such ore.
