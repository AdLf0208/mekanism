// The pure core of the Mekanism slices 1-7 port (clean-room rewrite).
//
// Four machines from Mekanism 10.7.19.85 (MIT), all inside this module's own
// recipe-driven core where they share one:
//
// - the **Energized Smelter** (50 FE/t, 200 ticks per operation, one input item
//   → the recipe's output) and the **Enrichment Chamber** (the same operation,
//   the `mekanism:enriching` table: one ore → two dust);
// - the **Basic Energy Cube** (4,000,000 FE, 4,000 FE/t);
// - the **Basic Universal Cable** (8,000 FE buffer = its throughput per tick).
//
// Nothing here touches the game: the block entities in `support.rs` read their
// saved NBT, call these functions and write the result back.
//
// Values and semantics come from the upstream tag
// `v1.21.1-10.7.19.85`; see `FINDINGS.md` for the files read.

/// Ticks one Energized Smelter operation takes (`BASE_TICKS_REQUIRED`).
pub const SMELTER_TICKS: i32 = 200;

/// Energy one operating tick costs (`UsageConfig.energizedSmelter`).
pub const SMELTER_USAGE: i32 = 50;

/// The Energized Smelter's buffer (`StorageConfig.energizedSmelter`, two
/// operations' worth).
pub const SMELTER_CAPACITY: i32 = 20_000;

/// Ticks one Enrichment Chamber operation takes (`BASE_TICKS_REQUIRED`, the
/// same electric-machine base as the smelter's).
pub const CHAMBER_TICKS: i32 = 200;

/// Energy one Enrichment Chamber operating tick costs
/// (`UsageConfig.enrichmentChamber`).
pub const CHAMBER_USAGE: i32 = 50;

/// The Enrichment Chamber's buffer (`StorageConfig.enrichmentChamber`).
pub const CHAMBER_CAPACITY: i32 = 20_000;

/// How long one recipe operation takes and what one operating tick costs.
///
/// Upstream keeps these per machine (`BASE_TICKS_REQUIRED` and each machine's
/// `UsageConfig` entry), so the port does too: the two are equal today and
/// may not stay equal.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Operation {
    /// Ticks an operation takes.
    pub ticks: i32,
    /// Energy an operating tick costs, in FE.
    pub usage: i32,
}

/// The Energized Smelter's operation.
pub const SMELTER_OPERATION: Operation = Operation {
    ticks: SMELTER_TICKS,
    usage: SMELTER_USAGE,
};

/// The Enrichment Chamber's operation.
pub const CHAMBER_OPERATION: Operation = Operation {
    ticks: CHAMBER_TICKS,
    usage: CHAMBER_USAGE,
};

/// Ticks one Crusher operation takes (`BASE_TICKS_REQUIRED`, the same
/// electric-machine base as the two machines above).
pub const CRUSHER_TICKS: i32 = 200;

/// Energy one Crusher operating tick costs (`UsageConfig.crusher`).
pub const CRUSHER_USAGE: i32 = 50;

/// The Crusher's buffer (`StorageConfig.crusher`).
pub const CRUSHER_CAPACITY: i32 = 20_000;

/// The Crusher's operation.
pub const CRUSHER_OPERATION: Operation = Operation {
    ticks: CRUSHER_TICKS,
    usage: CRUSHER_USAGE,
};

/// Ticks one Chemical Injection Chamber operation takes
/// (`TileEntityAdvancedElectricMachine.BASE_TICKS_REQUIRED`).
pub const INJECTION_TICKS: i32 = 200;

/// Energy one Chemical Injection Chamber operating tick costs
/// (`UsageConfig.chemicalInjectionChamber`, default 400).
pub const INJECTION_USAGE: i32 = 400;

/// The Chemical Injection Chamber's energy buffer
/// (`StorageConfig.chemicalInjectionChamber`, default 160,000).
pub const INJECTION_CAPACITY: i32 = 160_000;

/// The Chemical Injection Chamber's own gas tank
/// (`TileEntityAdvancedElectricMachine.MAX_GAS`): one operation of a one-unit
/// recipe, and no more — the machine is a consumer, not a reservoir.
pub const INJECTION_TANK: u32 = 210;

/// The Chemical Injection Chamber's operation: the jar's 400 FE/t over the
/// shared 200 ticks, so one operation costs 80,000 FE.
pub const INJECTION_OPERATION: Operation = Operation {
    ticks: INJECTION_TICKS,
    usage: INJECTION_USAGE,
};

/// Ticks one Purification Chamber operation takes
/// (`TileEntityAdvancedElectricMachine.BASE_TICKS_REQUIRED`, the same base the
/// injection chamber uses).
pub const PURIFY_TICKS: i32 = 200;

/// Energy one Purification Chamber operating tick costs
/// (`UsageConfig.purificationChamber`, default 200).
pub const PURIFY_USAGE: i32 = 200;

/// The Purification Chamber's energy buffer
/// (`StorageConfig.purificationChamber`, default 80,000).
pub const PURIFY_CAPACITY: i32 = 80_000;

/// The Purification Chamber's own gas tank
/// (`TileEntityAdvancedElectricMachine.MAX_GAS`), the same 210-unit buffer the
/// injection chamber has.
pub const PURIFY_TANK: u32 = 210;

/// The Purification Chamber's operation: the jar's 200 FE/t over the shared 200
/// ticks, so one operation costs 40,000 FE.
pub const PURIFY_OPERATION: Operation = Operation {
    ticks: PURIFY_TICKS,
    usage: PURIFY_USAGE,
};

/// How many processes one Basic factory runs (`FactoryTier.BASIC.processes`).
///
/// Upstream's tier table is `BASIC 3`, `ADVANCED 5`, `ELITE 7`, `ULTIMATE 9`;
/// this slice ships the basic tier, which is the only one whose blocks the port
/// registers.
pub const FACTORY_PROCESSES: usize = 3;

/// Ticks one factory operation takes (`TileEntityFactory.BASE_TICKS_REQUIRED`,
/// the same `10 * TICKS_PER_SECOND` the single machines use).
pub const FACTORY_TICKS: i32 = 200;

/// A Basic factory's buffer: `Factory.setMachineData` takes half the base
/// machine's `StorageConfig` value — but never less than one operating tick's
/// usage — and multiplies it by the tier's process count, so a smelting factory
/// holds `20,000 / 2 × 3 = 30,000` FE.
#[must_use]
pub const fn factory_capacity(base_storage: i32, usage: i32, processes: usize) -> i32 {
    // Upstream multiplies by the double 0.5; every default storage is even, so
    // integer halving is the same number.
    let half = base_storage / 2;
    let per_process = if half > usage { half } else { usage };
    per_process * processes as i32
}

/// A gas factory's one tank: `MAX_GAS` per process
/// (`BasicChemicalTank.createModern(capacity * tier.processes)`).
#[must_use]
pub const fn factory_tank(base_tank: u32, processes: usize) -> u32 {
    base_tank * processes as u32
}

/// The five basic factories' buffers, each from its base machine's numbers.
pub const SMELTING_FACTORY_CAPACITY: i32 =
    factory_capacity(SMELTER_CAPACITY, SMELTER_USAGE, FACTORY_PROCESSES);
/// The Basic Enriching Factory's buffer (see [`SMELTING_FACTORY_CAPACITY`]).
pub const ENRICHING_FACTORY_CAPACITY: i32 =
    factory_capacity(CHAMBER_CAPACITY, CHAMBER_USAGE, FACTORY_PROCESSES);
/// The Basic Crushing Factory's buffer (see [`SMELTING_FACTORY_CAPACITY`]).
pub const CRUSHING_FACTORY_CAPACITY: i32 =
    factory_capacity(CRUSHER_CAPACITY, CRUSHER_USAGE, FACTORY_PROCESSES);
/// The Basic Injecting Factory's buffer (see [`SMELTING_FACTORY_CAPACITY`]).
pub const INJECTING_FACTORY_CAPACITY: i32 =
    factory_capacity(INJECTION_CAPACITY, INJECTION_USAGE, FACTORY_PROCESSES);
/// The Basic Purifying Factory's buffer (see [`SMELTING_FACTORY_CAPACITY`]).
pub const PURIFYING_FACTORY_CAPACITY: i32 =
    factory_capacity(PURIFY_CAPACITY, PURIFY_USAGE, FACTORY_PROCESSES);

/// The Basic Injecting Factory's tank: the injection chamber's 210 per process.
pub const INJECTING_FACTORY_TANK: u32 = factory_tank(INJECTION_TANK, FACTORY_PROCESSES);

/// The Basic Purifying Factory's tank: the purification chamber's 210 per
/// process.
pub const PURIFYING_FACTORY_TANK: u32 = factory_tank(PURIFY_TANK, FACTORY_PROCESSES);

/// One Basic Energy Cube's storage (`EnergyCubeTier.BASIC`).
pub const CUBE_CAPACITY: i32 = 4_000_000;

/// The Basic Energy Cube's tier transfer rate, the most one call moves.
pub const CUBE_RATE: i32 = 4_000;

/// One Basic Universal Cable's buffer, which is also its throughput per tick.
pub const CABLE_CAPACITY: i32 = 8_000;

/// The Energized Smelter's recipe table: every `minecraft:smelting` recipe of
/// the prepared 26.3 resources, expanded over item tags by
/// `tools/gen_smelting.py` (upstream adds them all to its own smelting type).
pub mod smelting {
    use super::ItemAmount;

    // BEGIN GENERATED SMELTING TABLE (tools/gen_smelting.py)
    // Generated from the prepared 26.3 resources by
    // `tools/gen_smelting.py`; do not edit by hand. Mekanism adds every
    // `minecraft:smelting` recipe to its own smelting type
    // (`MekanismRecipeType.getRecipesUncached`), so the smelter smelts exactly
    // what a furnace smelts. Each row is `(input item, output item, count)`,
    // sorted by input; a tag ingredient is expanded to its member items.
    // Source recipes: 73. Rows: 163.
    static TABLE: &[(&str, &str, u32)] = &[
        ("minecraft:acacia_leaves", "minecraft:leaf_litter", 1),
        ("minecraft:acacia_log", "minecraft:charcoal", 1),
        ("minecraft:acacia_wood", "minecraft:charcoal", 1),
        ("minecraft:ancient_debris", "minecraft:netherite_scrap", 1),
        ("minecraft:azalea_leaves", "minecraft:leaf_litter", 1),
        ("minecraft:basalt", "minecraft:smooth_basalt", 1),
        ("minecraft:beef", "minecraft:cooked_beef", 1),
        ("minecraft:birch_leaves", "minecraft:leaf_litter", 1),
        ("minecraft:birch_log", "minecraft:charcoal", 1),
        ("minecraft:birch_wood", "minecraft:charcoal", 1),
        (
            "minecraft:black_terracotta",
            "minecraft:black_glazed_terracotta",
            1,
        ),
        (
            "minecraft:blue_terracotta",
            "minecraft:blue_glazed_terracotta",
            1,
        ),
        (
            "minecraft:brown_terracotta",
            "minecraft:brown_glazed_terracotta",
            1,
        ),
        ("minecraft:cactus", "minecraft:green_dye", 1),
        ("minecraft:chainmail_boots", "minecraft:iron_nugget", 1),
        ("minecraft:chainmail_chestplate", "minecraft:iron_nugget", 1),
        ("minecraft:chainmail_helmet", "minecraft:iron_nugget", 1),
        ("minecraft:chainmail_leggings", "minecraft:iron_nugget", 1),
        ("minecraft:cherry_leaves", "minecraft:leaf_litter", 1),
        ("minecraft:cherry_log", "minecraft:charcoal", 1),
        ("minecraft:cherry_wood", "minecraft:charcoal", 1),
        ("minecraft:chicken", "minecraft:cooked_chicken", 1),
        ("minecraft:chorus_fruit", "minecraft:popped_chorus_fruit", 1),
        ("minecraft:clay", "minecraft:terracotta", 1),
        ("minecraft:clay_ball", "minecraft:brick", 1),
        ("minecraft:coal_ore", "minecraft:coal", 1),
        ("minecraft:cobbled_deepslate", "minecraft:deepslate", 1),
        ("minecraft:cobblestone", "minecraft:stone", 1),
        ("minecraft:cod", "minecraft:cooked_cod", 1),
        ("minecraft:copper_axe", "minecraft:copper_nugget", 1),
        ("minecraft:copper_boots", "minecraft:copper_nugget", 1),
        ("minecraft:copper_chestplate", "minecraft:copper_nugget", 1),
        ("minecraft:copper_helmet", "minecraft:copper_nugget", 1),
        ("minecraft:copper_hoe", "minecraft:copper_nugget", 1),
        ("minecraft:copper_horse_armor", "minecraft:copper_nugget", 1),
        ("minecraft:copper_leggings", "minecraft:copper_nugget", 1),
        (
            "minecraft:copper_nautilus_armor",
            "minecraft:copper_nugget",
            1,
        ),
        ("minecraft:copper_ore", "minecraft:copper_ingot", 1),
        ("minecraft:copper_pickaxe", "minecraft:copper_nugget", 1),
        ("minecraft:copper_shovel", "minecraft:copper_nugget", 1),
        ("minecraft:copper_spear", "minecraft:copper_nugget", 1),
        ("minecraft:copper_sword", "minecraft:copper_nugget", 1),
        (
            "minecraft:cyan_terracotta",
            "minecraft:cyan_glazed_terracotta",
            1,
        ),
        ("minecraft:dark_oak_leaves", "minecraft:leaf_litter", 1),
        ("minecraft:dark_oak_log", "minecraft:charcoal", 1),
        ("minecraft:dark_oak_wood", "minecraft:charcoal", 1),
        (
            "minecraft:deepslate_bricks",
            "minecraft:cracked_deepslate_bricks",
            1,
        ),
        ("minecraft:deepslate_coal_ore", "minecraft:coal", 1),
        (
            "minecraft:deepslate_copper_ore",
            "minecraft:copper_ingot",
            1,
        ),
        ("minecraft:deepslate_diamond_ore", "minecraft:diamond", 1),
        ("minecraft:deepslate_emerald_ore", "minecraft:emerald", 1),
        ("minecraft:deepslate_gold_ore", "minecraft:gold_ingot", 1),
        ("minecraft:deepslate_iron_ore", "minecraft:iron_ingot", 1),
        ("minecraft:deepslate_lapis_ore", "minecraft:lapis_lazuli", 1),
        ("minecraft:deepslate_redstone_ore", "minecraft:redstone", 1),
        (
            "minecraft:deepslate_tiles",
            "minecraft:cracked_deepslate_tiles",
            1,
        ),
        ("minecraft:diamond_ore", "minecraft:diamond", 1),
        ("minecraft:emerald_ore", "minecraft:emerald", 1),
        (
            "minecraft:flowering_azalea_leaves",
            "minecraft:leaf_litter",
            1,
        ),
        ("minecraft:gold_ore", "minecraft:gold_ingot", 1),
        ("minecraft:golden_axe", "minecraft:gold_nugget", 1),
        ("minecraft:golden_boots", "minecraft:gold_nugget", 1),
        ("minecraft:golden_chestplate", "minecraft:gold_nugget", 1),
        ("minecraft:golden_helmet", "minecraft:gold_nugget", 1),
        ("minecraft:golden_hoe", "minecraft:gold_nugget", 1),
        ("minecraft:golden_horse_armor", "minecraft:gold_nugget", 1),
        ("minecraft:golden_leggings", "minecraft:gold_nugget", 1),
        (
            "minecraft:golden_nautilus_armor",
            "minecraft:gold_nugget",
            1,
        ),
        ("minecraft:golden_pickaxe", "minecraft:gold_nugget", 1),
        ("minecraft:golden_shovel", "minecraft:gold_nugget", 1),
        ("minecraft:golden_spear", "minecraft:gold_nugget", 1),
        ("minecraft:golden_sword", "minecraft:gold_nugget", 1),
        (
            "minecraft:gray_terracotta",
            "minecraft:gray_glazed_terracotta",
            1,
        ),
        (
            "minecraft:green_terracotta",
            "minecraft:green_glazed_terracotta",
            1,
        ),
        ("minecraft:iron_axe", "minecraft:iron_nugget", 1),
        ("minecraft:iron_boots", "minecraft:iron_nugget", 1),
        ("minecraft:iron_chestplate", "minecraft:iron_nugget", 1),
        ("minecraft:iron_helmet", "minecraft:iron_nugget", 1),
        ("minecraft:iron_hoe", "minecraft:iron_nugget", 1),
        ("minecraft:iron_horse_armor", "minecraft:iron_nugget", 1),
        ("minecraft:iron_leggings", "minecraft:iron_nugget", 1),
        ("minecraft:iron_nautilus_armor", "minecraft:iron_nugget", 1),
        ("minecraft:iron_ore", "minecraft:iron_ingot", 1),
        ("minecraft:iron_pickaxe", "minecraft:iron_nugget", 1),
        ("minecraft:iron_shovel", "minecraft:iron_nugget", 1),
        ("minecraft:iron_spear", "minecraft:iron_nugget", 1),
        ("minecraft:iron_sword", "minecraft:iron_nugget", 1),
        ("minecraft:jungle_leaves", "minecraft:leaf_litter", 1),
        ("minecraft:jungle_log", "minecraft:charcoal", 1),
        ("minecraft:jungle_wood", "minecraft:charcoal", 1),
        ("minecraft:kelp", "minecraft:dried_kelp", 1),
        ("minecraft:lapis_ore", "minecraft:lapis_lazuli", 1),
        (
            "minecraft:light_blue_terracotta",
            "minecraft:light_blue_glazed_terracotta",
            1,
        ),
        (
            "minecraft:light_gray_terracotta",
            "minecraft:light_gray_glazed_terracotta",
            1,
        ),
        (
            "minecraft:lime_terracotta",
            "minecraft:lime_glazed_terracotta",
            1,
        ),
        (
            "minecraft:magenta_terracotta",
            "minecraft:magenta_glazed_terracotta",
            1,
        ),
        ("minecraft:mangrove_leaves", "minecraft:leaf_litter", 1),
        ("minecraft:mangrove_log", "minecraft:charcoal", 1),
        ("minecraft:mangrove_wood", "minecraft:charcoal", 1),
        ("minecraft:mutton", "minecraft:cooked_mutton", 1),
        (
            "minecraft:nether_bricks",
            "minecraft:cracked_nether_bricks",
            1,
        ),
        ("minecraft:nether_gold_ore", "minecraft:gold_ingot", 1),
        ("minecraft:nether_quartz_ore", "minecraft:quartz", 1),
        ("minecraft:netherrack", "minecraft:nether_brick", 1),
        ("minecraft:oak_leaves", "minecraft:leaf_litter", 1),
        ("minecraft:oak_log", "minecraft:charcoal", 1),
        ("minecraft:oak_wood", "minecraft:charcoal", 1),
        ("minecraft:orange_poplar_leaves", "minecraft:leaf_litter", 1),
        (
            "minecraft:orange_terracotta",
            "minecraft:orange_glazed_terracotta",
            1,
        ),
        ("minecraft:pale_oak_leaves", "minecraft:leaf_litter", 1),
        ("minecraft:pale_oak_log", "minecraft:charcoal", 1),
        ("minecraft:pale_oak_wood", "minecraft:charcoal", 1),
        (
            "minecraft:pink_terracotta",
            "minecraft:pink_glazed_terracotta",
            1,
        ),
        (
            "minecraft:polished_blackstone_bricks",
            "minecraft:cracked_polished_blackstone_bricks",
            1,
        ),
        ("minecraft:poplar_log", "minecraft:charcoal", 1),
        ("minecraft:poplar_wood", "minecraft:charcoal", 1),
        ("minecraft:porkchop", "minecraft:cooked_porkchop", 1),
        ("minecraft:potato", "minecraft:baked_potato", 1),
        (
            "minecraft:purple_terracotta",
            "minecraft:purple_glazed_terracotta",
            1,
        ),
        ("minecraft:quartz_block", "minecraft:smooth_quartz", 1),
        ("minecraft:rabbit", "minecraft:cooked_rabbit", 1),
        ("minecraft:raw_copper", "minecraft:copper_ingot", 1),
        ("minecraft:raw_gold", "minecraft:gold_ingot", 1),
        ("minecraft:raw_iron", "minecraft:iron_ingot", 1),
        ("minecraft:red_poplar_leaves", "minecraft:leaf_litter", 1),
        ("minecraft:red_sand", "minecraft:glass", 1),
        (
            "minecraft:red_sandstone",
            "minecraft:smooth_red_sandstone",
            1,
        ),
        (
            "minecraft:red_terracotta",
            "minecraft:red_glazed_terracotta",
            1,
        ),
        ("minecraft:redstone_ore", "minecraft:redstone", 1),
        ("minecraft:resin_clump", "minecraft:resin_brick", 1),
        ("minecraft:salmon", "minecraft:cooked_salmon", 1),
        ("minecraft:sand", "minecraft:glass", 1),
        ("minecraft:sandstone", "minecraft:smooth_sandstone", 1),
        ("minecraft:sea_pickle", "minecraft:lime_dye", 1),
        ("minecraft:spruce_leaves", "minecraft:leaf_litter", 1),
        ("minecraft:spruce_log", "minecraft:charcoal", 1),
        ("minecraft:spruce_wood", "minecraft:charcoal", 1),
        ("minecraft:stone", "minecraft:smooth_stone", 1),
        (
            "minecraft:stone_bricks",
            "minecraft:cracked_stone_bricks",
            1,
        ),
        ("minecraft:stripped_acacia_log", "minecraft:charcoal", 1),
        ("minecraft:stripped_acacia_wood", "minecraft:charcoal", 1),
        ("minecraft:stripped_birch_log", "minecraft:charcoal", 1),
        ("minecraft:stripped_birch_wood", "minecraft:charcoal", 1),
        ("minecraft:stripped_cherry_log", "minecraft:charcoal", 1),
        ("minecraft:stripped_cherry_wood", "minecraft:charcoal", 1),
        ("minecraft:stripped_dark_oak_log", "minecraft:charcoal", 1),
        ("minecraft:stripped_dark_oak_wood", "minecraft:charcoal", 1),
        ("minecraft:stripped_jungle_log", "minecraft:charcoal", 1),
        ("minecraft:stripped_jungle_wood", "minecraft:charcoal", 1),
        ("minecraft:stripped_mangrove_log", "minecraft:charcoal", 1),
        ("minecraft:stripped_mangrove_wood", "minecraft:charcoal", 1),
        ("minecraft:stripped_oak_log", "minecraft:charcoal", 1),
        ("minecraft:stripped_oak_wood", "minecraft:charcoal", 1),
        ("minecraft:stripped_pale_oak_log", "minecraft:charcoal", 1),
        ("minecraft:stripped_pale_oak_wood", "minecraft:charcoal", 1),
        ("minecraft:stripped_poplar_log", "minecraft:charcoal", 1),
        ("minecraft:stripped_poplar_wood", "minecraft:charcoal", 1),
        ("minecraft:stripped_spruce_log", "minecraft:charcoal", 1),
        ("minecraft:stripped_spruce_wood", "minecraft:charcoal", 1),
        ("minecraft:wet_sponge", "minecraft:sponge", 1),
        (
            "minecraft:white_terracotta",
            "minecraft:white_glazed_terracotta",
            1,
        ),
        ("minecraft:yellow_poplar_leaves", "minecraft:leaf_litter", 1),
        (
            "minecraft:yellow_terracotta",
            "minecraft:yellow_glazed_terracotta",
            1,
        ),
    ];
    // END GENERATED SMELTING TABLE

    /// The jar's own `minecraft:smelting` recipes: Mekanism's dusts into their
    /// ingots (`data/mekanism/recipe/processing/<material>/ingot/
    /// from_dust_smelting.json`, one item in, one ingot out, 200 ticks). They are
    /// vanilla-type recipes of the pack, so the Energized Smelter smelts them; the
    /// port carries the dusts of slices 1-3 (into vanilla's ingots) and slice
    /// 12's lead, osmium, tin and uranium (into this port's own ingots).
    static MEKANISM_ROWS: &[(&str, &str, u32)] = &[
        ("mekanism:dust_copper", "minecraft:copper_ingot", 1),
        ("mekanism:dust_gold", "minecraft:gold_ingot", 1),
        ("mekanism:dust_iron", "minecraft:iron_ingot", 1),
        ("mekanism:dust_lead", "mekanism:ingot_lead", 1),
        ("mekanism:dust_osmium", "mekanism:ingot_osmium", 1),
        ("mekanism:dust_tin", "mekanism:ingot_tin", 1),
        ("mekanism:dust_uranium", "mekanism:ingot_uranium", 1),
    ];

    /// How many of the jar's own `minecraft:smelting` rows the table carries
    /// (the tests' size check).
    #[cfg(test)]
    #[must_use]
    pub fn mekanism_rows() -> usize {
        MEKANISM_ROWS.len()
    }

    /// The result of smelting `item`, `None` when nothing smelts it.
    ///
    /// Every vanilla smelting recipe takes one item. The jar's own
    /// `minecraft:smelting` recipes (`processing/<material>/ingot/
    /// from_dust_smelting.json`) are part of the same recipe type in the pack,
    /// so [`MEKANISM_ROWS`] adds the dusts this port registers.
    #[must_use]
    pub fn lookup(item: &str) -> Option<super::MachineRecipe> {
        let one = |output: &str, count: u32| {
            Some(super::MachineRecipe {
                input: 1,
                output: ItemAmount::new(output, count),
            })
        };
        if let Ok(index) = TABLE.binary_search_by(|row| row.0.cmp(item)) {
            return one(TABLE[index].1, TABLE[index].2);
        }
        MEKANISM_ROWS
            .iter()
            .find(|row| row.0 == item)
            .and_then(|row| one(row.1, row.2))
    }

    /// The vanilla rows themselves, so a test can assert every one of them is
    /// reachable through [`lookup`] and fix the count e2e phase T must sweep.
    #[cfg(test)]
    #[must_use]
    pub fn rows() -> &'static [(&'static str, &'static str, u32)] {
        TABLE
    }

    /// How many recipe rows the table holds (the tests' size check).
    #[cfg(test)]
    #[must_use]
    pub fn len() -> usize {
        TABLE.len()
    }
}

/// The Enrichment Chamber's recipe table: the jar's `mekanism:enriching`
/// recipes over the items this port registers, generated by
/// `tools/gen_enriching.py` from `Mekanism-1.21.1-10.7.19.85.jar`.
pub mod enriching {
    use super::{ItemAmount, MachineRecipe};

    // BEGIN GENERATED ENRICHING TABLE (tools/gen_enriching.py)
    // Generated from the jar's `mekanism:enriching` recipes by
    // `tools/gen_enriching.py`; do not edit by hand. Each row is
    // `(input item, items consumed, output item, output count)`; a `c:`
    // tag is expanded to the members 26.3 has (see the tool's TAGS), and
    // a recipe whose output the port does not register is left out.
    // Source recipes: 322. Rows: 136. Skipped: 195.
    static TABLE: &[(&str, u32, &str, u32)] = &[
        ("mekanism:dirty_dust_copper", 1, "mekanism:dust_copper", 1),
        ("mekanism:dirty_dust_gold", 1, "mekanism:dust_gold", 1),
        ("mekanism:dirty_dust_iron", 1, "mekanism:dust_iron", 1),
        ("mekanism:dirty_dust_lead", 1, "mekanism:dust_lead", 1),
        ("mekanism:dirty_dust_osmium", 1, "mekanism:dust_osmium", 1),
        ("mekanism:dirty_dust_tin", 1, "mekanism:dust_tin", 1),
        ("mekanism:dirty_dust_uranium", 1, "mekanism:dust_uranium", 1),
        ("minecraft:allium", 1, "minecraft:magenta_dye", 2),
        (
            "minecraft:ancient_debris",
            1,
            "minecraft:netherite_scrap",
            2,
        ),
        ("minecraft:andesite", 1, "minecraft:polished_andesite", 1),
        (
            "minecraft:andesite_slab",
            1,
            "minecraft:polished_andesite_slab",
            1,
        ),
        (
            "minecraft:andesite_stairs",
            1,
            "minecraft:polished_andesite_stairs",
            1,
        ),
        ("minecraft:blue_orchid", 1, "minecraft:light_blue_dye", 2),
        ("minecraft:cactus", 1, "minecraft:green_dye", 2),
        (
            "minecraft:chiseled_deepslate",
            1,
            "minecraft:cracked_deepslate_tiles",
            1,
        ),
        ("minecraft:chiseled_tuff", 1, "minecraft:tuff", 1),
        ("minecraft:clay", 1, "minecraft:clay_ball", 4),
        ("minecraft:coal_ore", 1, "minecraft:coal", 2),
        (
            "minecraft:cobbled_deepslate_slab",
            1,
            "minecraft:polished_deepslate_slab",
            1,
        ),
        (
            "minecraft:cobbled_deepslate_stairs",
            1,
            "minecraft:polished_deepslate_stairs",
            1,
        ),
        (
            "minecraft:cobbled_deepslate_wall",
            1,
            "minecraft:polished_deepslate_wall",
            1,
        ),
        ("minecraft:cobweb", 1, "minecraft:string", 9),
        ("minecraft:cocoa_beans", 1, "minecraft:brown_dye", 2),
        ("minecraft:copper_ore", 1, "mekanism:dust_copper", 2),
        (
            "minecraft:cracked_deepslate_bricks",
            1,
            "minecraft:deepslate_bricks",
            1,
        ),
        (
            "minecraft:cracked_deepslate_tiles",
            1,
            "minecraft:deepslate_tiles",
            1,
        ),
        (
            "minecraft:cracked_nether_bricks",
            1,
            "minecraft:nether_bricks",
            1,
        ),
        (
            "minecraft:cracked_polished_blackstone_bricks",
            1,
            "minecraft:polished_blackstone_bricks",
            1,
        ),
        (
            "minecraft:cracked_stone_bricks",
            1,
            "minecraft:stone_bricks",
            1,
        ),
        ("minecraft:dandelion", 1, "minecraft:yellow_dye", 2),
        ("minecraft:deepslate", 1, "minecraft:polished_deepslate", 1),
        ("minecraft:deepslate_coal_ore", 1, "minecraft:coal", 2),
        (
            "minecraft:deepslate_copper_ore",
            1,
            "mekanism:dust_copper",
            2,
        ),
        ("minecraft:deepslate_diamond_ore", 1, "minecraft:diamond", 2),
        ("minecraft:deepslate_emerald_ore", 1, "minecraft:emerald", 2),
        ("minecraft:deepslate_gold_ore", 1, "mekanism:dust_gold", 2),
        ("minecraft:deepslate_iron_ore", 1, "mekanism:dust_iron", 2),
        (
            "minecraft:deepslate_lapis_ore",
            1,
            "minecraft:lapis_lazuli",
            12,
        ),
        (
            "minecraft:deepslate_redstone_ore",
            1,
            "minecraft:redstone",
            12,
        ),
        (
            "minecraft:deepslate_tile_slab",
            1,
            "minecraft:deepslate_brick_slab",
            1,
        ),
        (
            "minecraft:deepslate_tile_stairs",
            1,
            "minecraft:deepslate_brick_stairs",
            1,
        ),
        (
            "minecraft:deepslate_tile_wall",
            1,
            "minecraft:deepslate_brick_wall",
            1,
        ),
        (
            "minecraft:deepslate_tiles",
            1,
            "minecraft:cracked_deepslate_bricks",
            1,
        ),
        ("minecraft:diamond_ore", 1, "minecraft:diamond", 2),
        ("minecraft:diorite", 1, "minecraft:polished_diorite", 1),
        (
            "minecraft:diorite_slab",
            1,
            "minecraft:polished_diorite_slab",
            1,
        ),
        (
            "minecraft:diorite_stairs",
            1,
            "minecraft:polished_diorite_stairs",
            1,
        ),
        ("minecraft:emerald_ore", 1, "minecraft:emerald", 2),
        (
            "minecraft:exposed_chiseled_copper",
            1,
            "minecraft:chiseled_copper",
            1,
        ),
        ("minecraft:exposed_copper", 1, "minecraft:copper_block", 1),
        (
            "minecraft:exposed_copper_bulb",
            1,
            "minecraft:copper_bulb",
            1,
        ),
        (
            "minecraft:exposed_copper_door",
            1,
            "minecraft:copper_door",
            1,
        ),
        (
            "minecraft:exposed_copper_grate",
            1,
            "minecraft:copper_grate",
            1,
        ),
        (
            "minecraft:exposed_copper_trapdoor",
            1,
            "minecraft:copper_trapdoor",
            1,
        ),
        ("minecraft:exposed_cut_copper", 1, "minecraft:cut_copper", 1),
        (
            "minecraft:exposed_cut_copper_slab",
            1,
            "minecraft:cut_copper_slab",
            1,
        ),
        (
            "minecraft:exposed_cut_copper_stairs",
            1,
            "minecraft:cut_copper_stairs",
            1,
        ),
        ("minecraft:glowstone", 1, "minecraft:glowstone_dust", 4),
        ("minecraft:gold_ore", 1, "mekanism:dust_gold", 2),
        ("minecraft:granite", 1, "minecraft:polished_granite", 1),
        (
            "minecraft:granite_slab",
            1,
            "minecraft:polished_granite_slab",
            1,
        ),
        (
            "minecraft:granite_stairs",
            1,
            "minecraft:polished_granite_stairs",
            1,
        ),
        ("minecraft:gravel", 1, "minecraft:flint", 1),
        ("minecraft:gunpowder", 1, "minecraft:flint", 1),
        ("minecraft:iron_ore", 1, "mekanism:dust_iron", 2),
        ("minecraft:lapis_ore", 1, "minecraft:lapis_lazuli", 12),
        ("minecraft:lilac", 1, "minecraft:magenta_dye", 4),
        ("minecraft:mossy_cobblestone", 1, "minecraft:cobblestone", 1),
        (
            "minecraft:mossy_cobblestone_slab",
            1,
            "minecraft:cobblestone_slab",
            1,
        ),
        (
            "minecraft:mossy_cobblestone_stairs",
            1,
            "minecraft:cobblestone_stairs",
            1,
        ),
        (
            "minecraft:mossy_cobblestone_wall",
            1,
            "minecraft:cobblestone_wall",
            1,
        ),
        (
            "minecraft:mossy_stone_brick_slab",
            1,
            "minecraft:stone_brick_slab",
            1,
        ),
        (
            "minecraft:mossy_stone_brick_stairs",
            1,
            "minecraft:stone_brick_stairs",
            1,
        ),
        (
            "minecraft:mossy_stone_brick_wall",
            1,
            "minecraft:stone_brick_wall",
            1,
        ),
        (
            "minecraft:mossy_stone_bricks",
            1,
            "minecraft:stone_bricks",
            1,
        ),
        ("minecraft:mud", 1, "minecraft:clay", 1),
        (
            "minecraft:nether_bricks",
            1,
            "minecraft:chiseled_nether_bricks",
            1,
        ),
        ("minecraft:nether_quartz_ore", 1, "minecraft:quartz", 2),
        (
            "minecraft:oxidized_chiseled_copper",
            1,
            "minecraft:weathered_chiseled_copper",
            1,
        ),
        (
            "minecraft:oxidized_copper",
            1,
            "minecraft:weathered_copper",
            1,
        ),
        (
            "minecraft:oxidized_copper_bulb",
            1,
            "minecraft:weathered_copper_bulb",
            1,
        ),
        (
            "minecraft:oxidized_copper_door",
            1,
            "minecraft:weathered_copper_door",
            1,
        ),
        (
            "minecraft:oxidized_copper_grate",
            1,
            "minecraft:weathered_copper_grate",
            1,
        ),
        (
            "minecraft:oxidized_copper_trapdoor",
            1,
            "minecraft:weathered_copper_trapdoor",
            1,
        ),
        (
            "minecraft:oxidized_cut_copper",
            1,
            "minecraft:weathered_cut_copper",
            1,
        ),
        (
            "minecraft:oxidized_cut_copper_slab",
            1,
            "minecraft:weathered_cut_copper_slab",
            1,
        ),
        (
            "minecraft:oxidized_cut_copper_stairs",
            1,
            "minecraft:weathered_cut_copper_stairs",
            1,
        ),
        ("minecraft:packed_mud", 1, "minecraft:mud_bricks", 1),
        ("minecraft:peony", 1, "minecraft:pink_dye", 4),
        ("minecraft:pitcher_plant", 1, "minecraft:cyan_dye", 4),
        (
            "minecraft:polished_blackstone",
            1,
            "minecraft:cracked_polished_blackstone_bricks",
            1,
        ),
        (
            "minecraft:polished_blackstone_bricks",
            1,
            "minecraft:chiseled_polished_blackstone",
            1,
        ),
        (
            "minecraft:polished_deepslate",
            1,
            "minecraft:chiseled_deepslate",
            1,
        ),
        (
            "minecraft:polished_deepslate_slab",
            1,
            "minecraft:deepslate_tile_slab",
            1,
        ),
        (
            "minecraft:polished_deepslate_stairs",
            1,
            "minecraft:deepslate_tile_stairs",
            1,
        ),
        (
            "minecraft:polished_deepslate_wall",
            1,
            "minecraft:deepslate_tile_wall",
            1,
        ),
        ("minecraft:polished_tuff", 1, "minecraft:tuff_bricks", 1),
        (
            "minecraft:polished_tuff_slab",
            1,
            "minecraft:tuff_brick_slab",
            1,
        ),
        (
            "minecraft:polished_tuff_stairs",
            1,
            "minecraft:tuff_brick_stairs",
            1,
        ),
        (
            "minecraft:polished_tuff_wall",
            1,
            "minecraft:tuff_brick_wall",
            1,
        ),
        ("minecraft:purpur_block", 1, "minecraft:purpur_pillar", 1),
        ("minecraft:raw_copper", 3, "mekanism:dust_copper", 4),
        ("minecraft:raw_copper_block", 1, "mekanism:dust_copper", 12),
        ("minecraft:raw_gold", 3, "mekanism:dust_gold", 4),
        ("minecraft:raw_gold_block", 1, "mekanism:dust_gold", 12),
        ("minecraft:raw_iron", 3, "mekanism:dust_iron", 4),
        ("minecraft:raw_iron_block", 1, "mekanism:dust_iron", 12),
        ("minecraft:red_sand", 1, "minecraft:gravel", 1),
        ("minecraft:redstone_ore", 1, "minecraft:redstone", 12),
        ("minecraft:rose_bush", 1, "minecraft:red_dye", 4),
        ("minecraft:sand", 1, "minecraft:gravel", 1),
        ("minecraft:sea_pickle", 1, "minecraft:lime_dye", 2),
        ("minecraft:smooth_quartz", 1, "minecraft:quartz_block", 1),
        (
            "minecraft:smooth_quartz_slab",
            1,
            "minecraft:quartz_slab",
            1,
        ),
        (
            "minecraft:smooth_quartz_stairs",
            1,
            "minecraft:quartz_stairs",
            1,
        ),
        ("minecraft:soul_sand", 1, "minecraft:soul_soil", 1),
        ("minecraft:stone", 1, "minecraft:cracked_stone_bricks", 1),
        (
            "minecraft:stone_bricks",
            1,
            "minecraft:chiseled_stone_bricks",
            1,
        ),
        ("minecraft:sunflower", 1, "minecraft:yellow_dye", 4),
        ("minecraft:tuff", 1, "minecraft:polished_tuff", 1),
        ("minecraft:tuff_brick_slab", 1, "minecraft:tuff_slab", 1),
        ("minecraft:tuff_brick_stairs", 1, "minecraft:tuff_stairs", 1),
        ("minecraft:tuff_brick_wall", 1, "minecraft:tuff_wall", 1),
        ("minecraft:tuff_bricks", 1, "minecraft:chiseled_tuff", 1),
        ("minecraft:tuff_slab", 1, "minecraft:polished_tuff_slab", 1),
        (
            "minecraft:tuff_stairs",
            1,
            "minecraft:polished_tuff_stairs",
            1,
        ),
        ("minecraft:tuff_wall", 1, "minecraft:polished_tuff_wall", 1),
        (
            "minecraft:weathered_chiseled_copper",
            1,
            "minecraft:exposed_chiseled_copper",
            1,
        ),
        (
            "minecraft:weathered_copper",
            1,
            "minecraft:exposed_copper",
            1,
        ),
        (
            "minecraft:weathered_copper_bulb",
            1,
            "minecraft:exposed_copper_bulb",
            1,
        ),
        (
            "minecraft:weathered_copper_door",
            1,
            "minecraft:exposed_copper_door",
            1,
        ),
        (
            "minecraft:weathered_copper_grate",
            1,
            "minecraft:exposed_copper_grate",
            1,
        ),
        (
            "minecraft:weathered_copper_trapdoor",
            1,
            "minecraft:exposed_copper_trapdoor",
            1,
        ),
        (
            "minecraft:weathered_cut_copper",
            1,
            "minecraft:exposed_cut_copper",
            1,
        ),
        (
            "minecraft:weathered_cut_copper_slab",
            1,
            "minecraft:exposed_cut_copper_slab",
            1,
        ),
        (
            "minecraft:weathered_cut_copper_stairs",
            1,
            "minecraft:exposed_cut_copper_stairs",
            1,
        ),
    ];
    // END GENERATED ENRICHING TABLE

    /// The enrichment of `item`, `None` when nothing enriches it.
    #[must_use]
    pub fn lookup(item: &str) -> Option<MachineRecipe> {
        TABLE
            .binary_search_by(|row| row.0.cmp(item))
            .ok()
            .map(|index| MachineRecipe {
                input: TABLE[index].1,
                output: ItemAmount::new(TABLE[index].2, TABLE[index].3),
            })
    }

    /// How many recipe rows the table holds (the tests' size check).
    #[cfg(test)]
    #[must_use]
    pub fn len() -> usize {
        TABLE.len()
    }
}

/// The Crusher's recipe table: the jar's `mekanism:crushing` recipes over the
/// items this slice has, generated by `tools/gen_crushing.py`.
pub mod crushing {
    use super::{ItemAmount, MachineRecipe};

    // BEGIN GENERATED CRUSHING TABLE (tools/gen_crushing.py)
    // Generated from the jar's `mekanism:crushing` recipes by
    // `tools/gen_crushing.py`; do not edit by hand. Each row is
    // `(input item, items consumed, output item, output count)`; a `c:`
    // tag is expanded to the members 26.3 has (see the tool's TAGS), and
    // a recipe whose output the port does not register is left out.
    // Source recipes: 313. Rows: 156. Skipped: 179.
    static TABLE: &[(&str, u32, &str, u32)] = &[
        ("mekanism:clump_copper", 1, "mekanism:dirty_dust_copper", 1),
        ("mekanism:clump_gold", 1, "mekanism:dirty_dust_gold", 1),
        ("mekanism:clump_iron", 1, "mekanism:dirty_dust_iron", 1),
        ("mekanism:clump_lead", 1, "mekanism:dirty_dust_lead", 1),
        ("mekanism:clump_osmium", 1, "mekanism:dirty_dust_osmium", 1),
        ("mekanism:clump_tin", 1, "mekanism:dirty_dust_tin", 1),
        (
            "mekanism:clump_uranium",
            1,
            "mekanism:dirty_dust_uranium",
            1,
        ),
        ("mekanism:ingot_lead", 1, "mekanism:dust_lead", 1),
        ("mekanism:ingot_osmium", 1, "mekanism:dust_osmium", 1),
        ("mekanism:ingot_tin", 1, "mekanism:dust_tin", 1),
        ("mekanism:ingot_uranium", 1, "mekanism:dust_uranium", 1),
        ("minecraft:black_wool", 1, "minecraft:string", 4),
        ("minecraft:blaze_rod", 1, "minecraft:blaze_powder", 4),
        ("minecraft:blue_wool", 1, "minecraft:string", 4),
        ("minecraft:bone", 1, "minecraft:bone_meal", 6),
        ("minecraft:bone_block", 1, "minecraft:bone_meal", 9),
        ("minecraft:breeze_rod", 1, "minecraft:wind_charge", 6),
        ("minecraft:brown_wool", 1, "minecraft:string", 4),
        ("minecraft:chiseled_deepslate", 1, "minecraft:deepslate", 1),
        (
            "minecraft:chiseled_nether_bricks",
            1,
            "minecraft:nether_bricks",
            1,
        ),
        (
            "minecraft:chiseled_polished_blackstone",
            1,
            "minecraft:polished_blackstone_bricks",
            1,
        ),
        (
            "minecraft:chiseled_quartz_block",
            1,
            "minecraft:quartz_pillar",
            1,
        ),
        (
            "minecraft:chiseled_red_sandstone",
            1,
            "minecraft:red_sand",
            2,
        ),
        ("minecraft:chiseled_sandstone", 1, "minecraft:sand", 2),
        (
            "minecraft:chiseled_stone_bricks",
            1,
            "minecraft:stone_bricks",
            1,
        ),
        ("minecraft:chiseled_tuff", 1, "minecraft:tuff_bricks", 1),
        ("minecraft:cobblestone", 1, "minecraft:gravel", 1),
        ("minecraft:copper_ingot", 1, "mekanism:dust_copper", 1),
        (
            "minecraft:cracked_deepslate_bricks",
            1,
            "minecraft:deepslate_tiles",
            1,
        ),
        (
            "minecraft:cracked_deepslate_tiles",
            1,
            "minecraft:chiseled_deepslate",
            1,
        ),
        (
            "minecraft:cracked_polished_blackstone_bricks",
            1,
            "minecraft:polished_blackstone",
            1,
        ),
        ("minecraft:cracked_stone_bricks", 1, "minecraft:stone", 1),
        ("minecraft:cut_red_sandstone", 1, "minecraft:red_sand", 2),
        ("minecraft:cut_sandstone", 1, "minecraft:sand", 2),
        ("minecraft:cyan_wool", 1, "minecraft:string", 4),
        ("minecraft:deepslate", 1, "minecraft:cobbled_deepslate", 1),
        (
            "minecraft:deepslate_brick_slab",
            1,
            "minecraft:deepslate_tile_slab",
            1,
        ),
        (
            "minecraft:deepslate_brick_stairs",
            1,
            "minecraft:deepslate_tile_stairs",
            1,
        ),
        (
            "minecraft:deepslate_brick_wall",
            1,
            "minecraft:deepslate_tile_wall",
            1,
        ),
        (
            "minecraft:deepslate_bricks",
            1,
            "minecraft:cracked_deepslate_bricks",
            1,
        ),
        (
            "minecraft:deepslate_tile_slab",
            1,
            "minecraft:cobbled_deepslate_slab",
            1,
        ),
        (
            "minecraft:deepslate_tile_stairs",
            1,
            "minecraft:cobbled_deepslate_stairs",
            1,
        ),
        (
            "minecraft:deepslate_tile_wall",
            1,
            "minecraft:cobbled_deepslate_wall",
            1,
        ),
        (
            "minecraft:deepslate_tiles",
            1,
            "minecraft:cracked_deepslate_tiles",
            1,
        ),
        (
            "minecraft:dripstone_block",
            1,
            "minecraft:pointed_dripstone",
            4,
        ),
        ("minecraft:flint", 1, "minecraft:gunpowder", 1),
        ("minecraft:gold_ingot", 1, "mekanism:dust_gold", 1),
        ("minecraft:gravel", 1, "minecraft:sand", 1),
        ("minecraft:gray_wool", 1, "minecraft:string", 4),
        ("minecraft:green_wool", 1, "minecraft:string", 4),
        ("minecraft:honeycomb_block", 1, "minecraft:honeycomb", 4),
        ("minecraft:iron_ingot", 1, "mekanism:dust_iron", 1),
        ("minecraft:light_blue_wool", 1, "minecraft:string", 4),
        ("minecraft:light_gray_wool", 1, "minecraft:string", 4),
        ("minecraft:lime_wool", 1, "minecraft:string", 4),
        ("minecraft:magenta_wool", 1, "minecraft:string", 4),
        ("minecraft:mud_bricks", 1, "minecraft:packed_mud", 1),
        ("minecraft:music_disc_5", 1, "minecraft:disc_fragment_5", 9),
        (
            "minecraft:nether_bricks",
            1,
            "minecraft:cracked_nether_bricks",
            1,
        ),
        ("minecraft:orange_wool", 1, "minecraft:string", 4),
        ("minecraft:pink_wool", 1, "minecraft:string", 4),
        ("minecraft:polished_andesite", 1, "minecraft:andesite", 1),
        (
            "minecraft:polished_andesite_slab",
            1,
            "minecraft:andesite_slab",
            1,
        ),
        (
            "minecraft:polished_andesite_stairs",
            1,
            "minecraft:andesite_stairs",
            1,
        ),
        ("minecraft:polished_basalt", 1, "minecraft:basalt", 1),
        (
            "minecraft:polished_blackstone",
            1,
            "minecraft:blackstone",
            1,
        ),
        (
            "minecraft:polished_blackstone_bricks",
            1,
            "minecraft:cracked_polished_blackstone_bricks",
            1,
        ),
        (
            "minecraft:polished_blackstone_slab",
            1,
            "minecraft:blackstone_slab",
            1,
        ),
        (
            "minecraft:polished_blackstone_stairs",
            1,
            "minecraft:blackstone_stairs",
            1,
        ),
        (
            "minecraft:polished_blackstone_wall",
            1,
            "minecraft:blackstone_wall",
            1,
        ),
        (
            "minecraft:polished_deepslate",
            1,
            "minecraft:deepslate_bricks",
            1,
        ),
        (
            "minecraft:polished_deepslate_slab",
            1,
            "minecraft:deepslate_brick_slab",
            1,
        ),
        (
            "minecraft:polished_deepslate_stairs",
            1,
            "minecraft:deepslate_brick_stairs",
            1,
        ),
        (
            "minecraft:polished_deepslate_wall",
            1,
            "minecraft:deepslate_brick_wall",
            1,
        ),
        ("minecraft:polished_diorite", 1, "minecraft:diorite", 1),
        (
            "minecraft:polished_diorite_slab",
            1,
            "minecraft:diorite_slab",
            1,
        ),
        (
            "minecraft:polished_diorite_stairs",
            1,
            "minecraft:diorite_stairs",
            1,
        ),
        ("minecraft:polished_granite", 1, "minecraft:granite", 1),
        (
            "minecraft:polished_granite_slab",
            1,
            "minecraft:granite_slab",
            1,
        ),
        (
            "minecraft:polished_granite_stairs",
            1,
            "minecraft:granite_stairs",
            1,
        ),
        ("minecraft:polished_tuff", 1, "minecraft:tuff", 1),
        ("minecraft:polished_tuff_slab", 1, "minecraft:tuff_slab", 1),
        (
            "minecraft:polished_tuff_stairs",
            1,
            "minecraft:tuff_stairs",
            1,
        ),
        ("minecraft:polished_tuff_wall", 1, "minecraft:tuff_wall", 1),
        ("minecraft:prismarine", 1, "minecraft:prismarine_shard", 4),
        (
            "minecraft:prismarine_brick_slab",
            2,
            "minecraft:prismarine_shard",
            9,
        ),
        (
            "minecraft:prismarine_brick_stairs",
            1,
            "minecraft:prismarine_shard",
            9,
        ),
        (
            "minecraft:prismarine_bricks",
            1,
            "minecraft:prismarine_shard",
            9,
        ),
        (
            "minecraft:prismarine_slab",
            1,
            "minecraft:prismarine_shard",
            2,
        ),
        (
            "minecraft:prismarine_stairs",
            1,
            "minecraft:prismarine_shard",
            4,
        ),
        (
            "minecraft:prismarine_wall",
            1,
            "minecraft:prismarine_shard",
            4,
        ),
        ("minecraft:purple_wool", 1, "minecraft:string", 4),
        ("minecraft:purpur_pillar", 1, "minecraft:purpur_block", 1),
        ("minecraft:quartz_block", 1, "minecraft:smooth_quartz", 1),
        (
            "minecraft:quartz_bricks",
            1,
            "minecraft:chiseled_quartz_block",
            1,
        ),
        ("minecraft:quartz_pillar", 1, "minecraft:quartz_block", 1),
        (
            "minecraft:quartz_slab",
            1,
            "minecraft:smooth_quartz_slab",
            1,
        ),
        (
            "minecraft:quartz_stairs",
            1,
            "minecraft:smooth_quartz_stairs",
            1,
        ),
        ("minecraft:red_sandstone", 1, "minecraft:red_sand", 2),
        ("minecraft:red_wool", 1, "minecraft:string", 4),
        ("minecraft:sandstone", 1, "minecraft:sand", 2),
        ("minecraft:smooth_basalt", 1, "minecraft:basalt", 1),
        ("minecraft:smooth_quartz", 1, "minecraft:quartz_bricks", 1),
        ("minecraft:smooth_red_sandstone", 1, "minecraft:red_sand", 2),
        ("minecraft:smooth_sandstone", 1, "minecraft:sand", 2),
        ("minecraft:soul_soil", 1, "minecraft:soul_sand", 1),
        ("minecraft:stone", 1, "minecraft:cobblestone", 1),
        (
            "minecraft:stone_bricks",
            1,
            "minecraft:cracked_stone_bricks",
            1,
        ),
        ("minecraft:stone_slab", 1, "minecraft:cobblestone_slab", 1),
        (
            "minecraft:stone_stairs",
            1,
            "minecraft:cobblestone_stairs",
            1,
        ),
        ("minecraft:tuff", 1, "minecraft:chiseled_tuff", 1),
        (
            "minecraft:tuff_brick_slab",
            1,
            "minecraft:polished_tuff_slab",
            1,
        ),
        (
            "minecraft:tuff_brick_stairs",
            1,
            "minecraft:polished_tuff_stairs",
            1,
        ),
        (
            "minecraft:tuff_brick_wall",
            1,
            "minecraft:polished_tuff_wall",
            1,
        ),
        ("minecraft:tuff_bricks", 1, "minecraft:polished_tuff", 1),
        ("minecraft:tuff_slab", 1, "minecraft:tuff_brick_slab", 1),
        ("minecraft:tuff_stairs", 1, "minecraft:tuff_brick_stairs", 1),
        ("minecraft:tuff_wall", 1, "minecraft:tuff_brick_wall", 1),
        (
            "minecraft:waxed_chiseled_copper",
            1,
            "minecraft:chiseled_copper",
            1,
        ),
        (
            "minecraft:waxed_copper_block",
            1,
            "minecraft:copper_block",
            1,
        ),
        ("minecraft:waxed_copper_bulb", 1, "minecraft:copper_bulb", 1),
        ("minecraft:waxed_copper_door", 1, "minecraft:copper_door", 1),
        (
            "minecraft:waxed_copper_grate",
            1,
            "minecraft:copper_grate",
            1,
        ),
        (
            "minecraft:waxed_copper_trapdoor",
            1,
            "minecraft:copper_trapdoor",
            1,
        ),
        ("minecraft:waxed_cut_copper", 1, "minecraft:cut_copper", 1),
        (
            "minecraft:waxed_cut_copper_slab",
            1,
            "minecraft:cut_copper_slab",
            1,
        ),
        (
            "minecraft:waxed_cut_copper_stairs",
            1,
            "minecraft:cut_copper_stairs",
            1,
        ),
        (
            "minecraft:waxed_exposed_chiseled_copper",
            1,
            "minecraft:exposed_chiseled_copper",
            1,
        ),
        (
            "minecraft:waxed_exposed_copper",
            1,
            "minecraft:exposed_copper",
            1,
        ),
        (
            "minecraft:waxed_exposed_copper_bulb",
            1,
            "minecraft:exposed_copper_bulb",
            1,
        ),
        (
            "minecraft:waxed_exposed_copper_door",
            1,
            "minecraft:exposed_copper_door",
            1,
        ),
        (
            "minecraft:waxed_exposed_copper_grate",
            1,
            "minecraft:exposed_copper_grate",
            1,
        ),
        (
            "minecraft:waxed_exposed_copper_trapdoor",
            1,
            "minecraft:exposed_copper_trapdoor",
            1,
        ),
        (
            "minecraft:waxed_exposed_cut_copper",
            1,
            "minecraft:exposed_cut_copper",
            1,
        ),
        (
            "minecraft:waxed_exposed_cut_copper_slab",
            1,
            "minecraft:exposed_cut_copper_slab",
            1,
        ),
        (
            "minecraft:waxed_exposed_cut_copper_stairs",
            1,
            "minecraft:exposed_cut_copper_stairs",
            1,
        ),
        (
            "minecraft:waxed_oxidized_chiseled_copper",
            1,
            "minecraft:oxidized_chiseled_copper",
            1,
        ),
        (
            "minecraft:waxed_oxidized_copper",
            1,
            "minecraft:oxidized_copper",
            1,
        ),
        (
            "minecraft:waxed_oxidized_copper_bulb",
            1,
            "minecraft:oxidized_copper_bulb",
            1,
        ),
        (
            "minecraft:waxed_oxidized_copper_door",
            1,
            "minecraft:oxidized_copper_door",
            1,
        ),
        (
            "minecraft:waxed_oxidized_copper_grate",
            1,
            "minecraft:oxidized_copper_grate",
            1,
        ),
        (
            "minecraft:waxed_oxidized_copper_trapdoor",
            1,
            "minecraft:oxidized_copper_trapdoor",
            1,
        ),
        (
            "minecraft:waxed_oxidized_cut_copper",
            1,
            "minecraft:oxidized_cut_copper",
            1,
        ),
        (
            "minecraft:waxed_oxidized_cut_copper_slab",
            1,
            "minecraft:oxidized_cut_copper_slab",
            1,
        ),
        (
            "minecraft:waxed_oxidized_cut_copper_stairs",
            1,
            "minecraft:oxidized_cut_copper_stairs",
            1,
        ),
        (
            "minecraft:waxed_weathered_chiseled_copper",
            1,
            "minecraft:weathered_chiseled_copper",
            1,
        ),
        (
            "minecraft:waxed_weathered_copper",
            1,
            "minecraft:weathered_copper",
            1,
        ),
        (
            "minecraft:waxed_weathered_copper_bulb",
            1,
            "minecraft:weathered_copper_bulb",
            1,
        ),
        (
            "minecraft:waxed_weathered_copper_door",
            1,
            "minecraft:weathered_copper_door",
            1,
        ),
        (
            "minecraft:waxed_weathered_copper_grate",
            1,
            "minecraft:weathered_copper_grate",
            1,
        ),
        (
            "minecraft:waxed_weathered_copper_trapdoor",
            1,
            "minecraft:weathered_copper_trapdoor",
            1,
        ),
        (
            "minecraft:waxed_weathered_cut_copper",
            1,
            "minecraft:weathered_cut_copper",
            1,
        ),
        (
            "minecraft:waxed_weathered_cut_copper_slab",
            1,
            "minecraft:weathered_cut_copper_slab",
            1,
        ),
        (
            "minecraft:waxed_weathered_cut_copper_stairs",
            1,
            "minecraft:weathered_cut_copper_stairs",
            1,
        ),
        ("minecraft:white_wool", 1, "minecraft:string", 4),
        ("minecraft:yellow_wool", 1, "minecraft:string", 4),
    ];
    // END GENERATED CRUSHING TABLE

    /// What crushing `item` makes, `None` when the crusher has no recipe for it.
    #[must_use]
    pub fn lookup(item: &str) -> Option<MachineRecipe> {
        TABLE
            .binary_search_by(|row| row.0.cmp(item))
            .ok()
            .map(|index| MachineRecipe {
                input: TABLE[index].1,
                output: ItemAmount::new(TABLE[index].2, TABLE[index].3),
            })
    }

    /// How many recipe rows the table holds (the tests' size check).
    #[cfg(test)]
    #[must_use]
    pub fn len() -> usize {
        TABLE.len()
    }
}

/// An amount of one item as the pure core sees it: an id and a count.
///
/// Not `Stack`: the mod API has a `Stack` of its own, and `cargo horizon-mod`
/// records a package's own struct literal against the SDK type of the same name,
/// which makes `uses.json` claim a dependency the port does not have (and `fits`
/// then fails a build the package really fits).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ItemAmount {
    /// The item id (`minecraft:iron_ingot`).
    pub item: String,
    /// How many of it.
    pub count: u32,
}

impl ItemAmount {
    /// A stack of `count` of `item`.
    #[must_use]
    pub fn new(item: &str, count: u32) -> Self {
        Self {
            item: item.to_owned(),
            count,
        }
    }

    /// Whether the stack holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count == 0 || self.item.is_empty()
    }
}

/// A machine's saved state: its energy and its progress in ticks.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Machine {
    /// Stored energy in FE.
    pub energy: i32,
    /// Ticks of the operation in progress, `0..operation.ticks`.
    pub progress: i32,
}

/// Why a tick did nothing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IdleReason {
    /// The input slot is empty (the progress resets).
    NoInput,
    /// The input has no recipe (the progress resets).
    NoRecipe,
    /// The input stack is smaller than the recipe needs (the progress
    /// resets; upstream's `NOT_ENOUGH_INPUT`).
    NotEnoughInput,
    /// Less than one tick's energy is stored (the progress is held).
    NoEnergy,
    /// The tank holds none of the chemical the recipe takes, or less than one
    /// operating tick of it (the progress is held: upstream's constant-usage
    /// handler pauses rather than resetting).
    NoChemical,
    /// The fluid tank holds none of the fluid the recipe takes, another fluid,
    /// or less than one operation needs (the progress **resets**: upstream's
    /// `mismatchedRecipe` for an empty tank and `NOT_ENOUGH_SECONDARY_INPUT`
    /// for a short one both go through `resetProgress`).
    NoFluid,
    /// The output slot has no room for the recipe's result (held).
    NoOutputSpace,
    /// The crystallizer's tank holds none of the clean slurry a row takes (the
    /// progress **resets**: upstream's `mismatchedRecipe` for an empty tank).
    NoChemicalInput,
}

/// What one tick of a machine did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// A tick of the operation was made; energy was spent.
    Working,
    /// The operation finished: the recipe's input was consumed and its result
    /// landed in the output slot.
    Produced,
    /// Nothing happened.
    Idle(IdleReason),
}

/// One recipe as the machines run it: how many input items one operation
/// consumes and what it produces.
///
/// Not `Recipe`, for the same reason as [`ItemAmount`]: the SDK has a `Recipe`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MachineRecipe {
    /// Items taken from the input slot when the operation finishes.
    pub input: u32,
    /// The result, a stack of the recipe's item and count.
    pub output: ItemAmount,
}

/// `energy` clamped into `0..=capacity`, as a buffer reads its saved int.
#[must_use]
pub fn clamp_energy(energy: i32, capacity: i32) -> i32 {
    energy.clamp(0, capacity.max(0))
}

/// How many of a recipe's output item the output slot still takes: the whole
/// stack when the slot is empty, the room left when it holds the same item, and
/// none when it holds another.
#[must_use]
pub fn room_for(output: &ItemAmount, recipe_output: &ItemAmount, max_stack: u32) -> u32 {
    if output.is_empty() {
        max_stack
    } else if output.item == recipe_output.item {
        max_stack.saturating_sub(output.count)
    } else {
        0
    }
}

/// One tick of a recipe machine (`CachedRecipe.process`).
///
/// `lookup` answers the recipe for an input item id, `None` when nothing
/// makes it. `max_stack` is the output item's stack size. Each operating tick
/// costs `operation.usage` energy and adds one progress tick; on
/// `operation.ticks` the recipe's input count is consumed and its result goes
/// into the output. No energy or no output room holds the progress; an empty
/// input, an input without a recipe or too few items resets it (upstream's
/// `mismatchedRecipe` and `NOT_ENOUGH_INPUT`).
pub fn process_tick(
    machine: &mut Machine,
    operation: Operation,
    input: &mut ItemAmount,
    output: &mut ItemAmount,
    max_stack: u32,
    lookup: &dyn Fn(&str) -> Option<MachineRecipe>,
) -> Outcome {
    if input.is_empty() {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NoInput);
    }
    let Some(recipe) = lookup(&input.item).filter(|recipe| !recipe.output.is_empty()) else {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NoRecipe);
    };
    if input.count < recipe.input.max(1) {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NotEnoughInput);
    }
    if room_for(output, &recipe.output, max_stack) < recipe.output.count {
        return Outcome::Idle(IdleReason::NoOutputSpace);
    }
    if machine.energy < operation.usage {
        return Outcome::Idle(IdleReason::NoEnergy);
    }
    machine.energy -= operation.usage;
    machine.progress += 1;
    if machine.progress < operation.ticks {
        return Outcome::Working;
    }
    machine.progress = 0;
    input.count -= recipe.input.max(1);
    if output.is_empty() {
        *output = recipe.output;
    } else {
        output.count += recipe.output.count;
    }
    Outcome::Produced
}

/// One side of a cable as the pure planner needs it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Endpoint {
    /// The face, 0..6 (`Direction`: down, up, north, south, west, east).
    pub face: u8,
    /// The block there takes energy from this side.
    pub can_insert: bool,
    /// The block there gives energy through this side.
    pub can_extract: bool,
    /// How much more energy that block takes.
    pub space: i32,
    /// How much energy that block holds.
    pub stored: i32,
}

/// One energy move the cable makes after its tick.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Move {
    /// Take `amount` from the block at `face` into the cable.
    Pull { face: u8, amount: i32 },
    /// Give `amount` from the cable to the block at `face`.
    Push { face: u8, amount: i32 },
}

/// What a cable does in one tick: its moves, in order, and its energy after.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Plan {
    /// The moves, pull first (face order), then push (face order).
    pub moves: Vec<Move>,
    /// The cable's energy once every move ran.
    pub energy_after: i32,
}

impl Plan {
    /// The energy taken from the neighbours.
    #[must_use]
    pub fn pulled(&self) -> i32 {
        self.moves
            .iter()
            .map(|m| match m {
                Move::Pull { amount, .. } => *amount,
                Move::Push { .. } => 0,
            })
            .sum()
    }

    /// The energy given to the neighbours.
    #[must_use]
    pub fn pushed(&self) -> i32 {
        self.moves
            .iter()
            .map(|m| match m {
                Move::Push { amount, .. } => *amount,
                Move::Pull { .. } => 0,
            })
            .sum()
    }
}

/// Plan one Basic Universal Cable's tick: fill its buffer from the providers
/// around it, then empty it into the acceptors, never moving more than the
/// buffer ([`CABLE_CAPACITY`]) in or out.
///
/// Upstream's transmitter pulls from its PULL sides into a buffer of the
/// cable's capacity and the network emits that buffer each tick; the port does
/// the same for one cable. Faces are visited in ascending order so the plan is
/// deterministic.
#[must_use]
pub fn cable_plan(energy: i32, capacity: i32, neighbours: &[Endpoint]) -> Plan {
    let capacity = capacity.max(0);
    let mut current = clamp_energy(energy, capacity);
    let mut moves = Vec::new();
    for endpoint in neighbours {
        if !endpoint.can_extract || endpoint.stored <= 0 {
            continue;
        }
        // The neighbour's buffer, drained at most into the cable's free room.
        let room = capacity - current;
        let (taken, _) = energy_after_extract(endpoint.stored, room, room);
        if taken > 0 {
            current += taken;
            moves.push(Move::Pull {
                face: endpoint.face,
                amount: taken,
            });
        }
    }
    for endpoint in neighbours {
        if !endpoint.can_insert || endpoint.space <= 0 || current <= 0 {
            continue;
        }
        // The neighbour's empty room, filled from at most what the cable holds.
        let (given, _) = energy_after_insert(0, endpoint.space, endpoint.space, current);
        if given > 0 {
            current -= given;
            moves.push(Move::Push {
                face: endpoint.face,
                amount: given,
            });
        }
    }
    Plan {
        moves,
        energy_after: current,
    }
}

/// The most a planned move may really move: a pull is capped by the cable's
/// free room, a push by what the cable holds *now* (a neighbour may have given
/// less than the plan expected). Without this a short pull still pushes the
/// planned amount and makes energy out of nothing.
#[must_use]
pub fn cable_move_limit(change: Move, energy: i32, capacity: i32) -> i32 {
    let energy = energy.max(0);
    match change {
        Move::Pull { amount, .. } => amount.min(capacity.saturating_sub(energy).max(0)),
        Move::Push { amount, .. } => amount.min(energy),
    }
}

/// `SimpleEnergyHandler.insert`: how much a buffer holding `energy` of at most
/// `capacity` takes from `offered` when one call moves at most `rate`.
/// Answers `(accepted, energy after)`.
#[must_use]
pub fn energy_after_insert(energy: i32, capacity: i32, rate: i32, offered: i32) -> (i32, i32) {
    let energy = clamp_energy(energy, capacity);
    let accepted = capacity
        .saturating_sub(energy)
        .min(offered)
        .min(rate)
        .max(0);
    (accepted, energy.saturating_add(accepted))
}

/// `SimpleEnergyHandler.extract`: how much a buffer holding `energy` gives of
/// `asked` when one call moves at most `rate`. Answers `(taken, energy after)`.
#[must_use]
pub fn energy_after_extract(energy: i32, rate: i32, asked: i32) -> (i32, i32) {
    let taken = energy.min(asked).min(rate).max(0);
    (taken, energy.saturating_sub(taken))
}

/// How far a player without operator permission may be from a machine to read it.
pub const INSPECT_RANGE: f64 = 8.0;

/// `Commands.LEVEL_ADMINS`: the permission level that bypasses the reach check.
pub const LEVEL_ADMINS: u8 = 2;

/// Whether a caller may read a machine's contents: an operator, or a player in
/// the same level within [`INSPECT_RANGE`] of the block.
pub fn may_inspect(op: bool, same_level: bool, distance: f64) -> bool {
    op || (same_level && distance <= INSPECT_RANGE)
}

// ---------------------------------------------------------------------------
// Slice 4: the chemical capability — one tank, one item, one id table
// ---------------------------------------------------------------------------

/// The empty chemical's registry id (`ChemicalStack.EMPTY`): what a drained tank
/// and an empty gauge dropper name, as upstream serialises an empty stack.
pub const EMPTY_CHEMICAL: &str = "mekanism:empty";

/// The chemicals Mekanism registers by name (`MekanismChemicals`, tag
/// v1.21.1-10.7.19.85), sorted. Names only: this slice carries no chemical item,
/// and the pigment/slurry families are later slices.
pub const CHEMICALS: [&str; 32] = [
    "mekanism:antimatter",
    "mekanism:bio",
    "mekanism:brine",
    "mekanism:carbon",
    "mekanism:chlorine",
    "mekanism:diamond",
    "mekanism:ethene",
    "mekanism:fissile_fuel",
    "mekanism:fungi",
    "mekanism:gold",
    "mekanism:hydrofluoric_acid",
    "mekanism:hydrogen",
    "mekanism:hydrogen_chloride",
    "mekanism:lithium",
    "mekanism:nuclear_waste",
    "mekanism:osmium",
    "mekanism:oxygen",
    "mekanism:plutonium",
    "mekanism:polonium",
    "mekanism:redstone",
    "mekanism:refined_obsidian",
    "mekanism:sodium",
    "mekanism:spent_nuclear_waste",
    "mekanism:steam",
    "mekanism:sulfur_dioxide",
    "mekanism:sulfur_trioxide",
    "mekanism:sulfuric_acid",
    "mekanism:superheated_sodium",
    "mekanism:tin",
    "mekanism:uranium_hexafluoride",
    "mekanism:uranium_oxide",
    "mekanism:water_vapor",
];

/// The id of a chemical Mekanism registers, or `None` for one it does not (and
/// for the empty chemical, which is not a chemical a tank takes).
///
/// Both registries this port knows: [`CHEMICALS`] (the gases and infuse types)
/// and [`SLURRIES`] (the ore-processing slurries), which upstream registers in
/// the same chemical registry (`MekanismChemicals.PROCESSED_RESOURCES`).
#[must_use]
pub fn known_chemical(id: &str) -> Option<&'static str> {
    known_slurry(id).or_else(|| CHEMICALS.iter().copied().find(|known| *known == id))
}

/// Mekanism's processed slurries (`ChemicalDeferredRegister.registerSlurry`):
/// a dirty and a clean one per `PrimaryResource`, as ids of the same chemical
/// registry a gas is (`MekanismChemicals.PROCESSED_RESOURCES`). The Chemical
/// Washer is what turns one into the other; they ship no item, a tank carries
/// them.
pub const SLURRIES: [&str; 14] = [
    "mekanism:clean_copper",
    "mekanism:clean_gold",
    "mekanism:clean_iron",
    "mekanism:clean_lead",
    "mekanism:clean_osmium",
    "mekanism:clean_tin",
    "mekanism:clean_uranium",
    "mekanism:dirty_copper",
    "mekanism:dirty_gold",
    "mekanism:dirty_iron",
    "mekanism:dirty_lead",
    "mekanism:dirty_osmium",
    "mekanism:dirty_tin",
    "mekanism:dirty_uranium",
];

/// The id of a slurry this port knows, `None` for anything else.
#[must_use]
pub fn known_slurry(id: &str) -> Option<&'static str> {
    SLURRIES.iter().copied().find(|known| *known == id)
}

/// What the Basic Chemical Tank's one tank holds (`ChemicalTankTier.BASIC`: 64
/// buckets, a chemical's amount being a millibucket as a fluid's is).
pub const TANK_CAPACITY: u32 = 64_000;

/// What the Gauge Dropper holds (`GearConfig.gaugeDropperCapacity`).
pub const DROPPER_CAPACITY: u32 = 16_000;

/// How much one Gauge Dropper call moves (`GearConfig.gaugeDroppedTransferRate`).
pub const DROPPER_TRANSFER: u32 = 250;

/// One tank of one chemical: what the block entity saves in its compound
/// (`chemical` and `amount`) and what the item keeps in its component.
///
/// `amount == 0` is an empty tank whatever `chemical` names, as an empty
/// `ChemicalStack` carries its id with amount 0.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Tank {
    /// The chemical it holds, `None` when it holds none.
    pub chemical: Option<String>,
    /// How much of it.
    pub amount: u32,
}

impl Tank {
    /// A tank holding `amount` of `chemical` (`amount` 0 for an empty one).
    #[must_use]
    pub fn new(chemical: &str, amount: u32) -> Self {
        if amount == 0 {
            Self::default()
        } else {
            Self {
                chemical: Some(chemical.to_owned()),
                amount,
            }
        }
    }

    /// The chemical and its amount, `None` for an empty tank.
    #[must_use]
    pub fn held(&self) -> Option<(&str, u32)> {
        match (&self.chemical, self.amount) {
            (Some(chemical), amount) if amount > 0 => Some((chemical, amount)),
            _ => None,
        }
    }

    /// Takes `amount` out of the tank and answers how much came out; a tank that
    /// runs dry is empty (`default`), not a chemical at 0.
    pub fn spend(&mut self, amount: u32) -> u32 {
        let came = amount.min(self.amount);
        self.amount -= came;
        if self.amount == 0 {
            *self = Self::default();
        }
        came
    }
}

/// Why a chemical did not go into a tank.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// The id is not one of [`CHEMICALS`].
    UnknownChemical(String),
}

/// How much of `amount` one Gauge Dropper call moves: at most its transfer rate.
#[must_use]
pub fn dropper_move(amount: u32) -> u32 {
    amount.min(DROPPER_TRANSFER)
}

/// Puts up to `amount` of `chemical` into `tank`, up to `capacity`: how much
/// went in.
///
/// A different chemical than the one held (or than a partly filled tank's) is
/// refused with 0, as `ChemicalTank.insert` does; an id outside [`CHEMICALS`] is
/// refused by name.
///
/// # Errors
/// [`Refusal::UnknownChemical`] when `chemical` is not a chemical Mekanism
/// registers.
pub fn insert_chemical(
    tank: &mut Tank,
    chemical: &str,
    amount: u32,
    capacity: u32,
) -> Result<u32, Refusal> {
    let Some(canonical) = known_chemical(chemical) else {
        return Err(Refusal::UnknownChemical(chemical.to_owned()));
    };
    if tank.held().is_some_and(|(held, _)| held != canonical) {
        return Ok(0);
    }
    let held = tank.amount.min(capacity);
    let went = amount.min(capacity.saturating_sub(held));
    if went > 0 {
        *tank = Tank::new(canonical, held + went);
    }
    Ok(went)
}

/// Takes up to `amount` of `chemical` out of `tank`: how much came out; nothing
/// of another chemical than the one it holds, and nothing from an empty tank.
pub fn extract_chemical(tank: &mut Tank, chemical: &str, amount: u32) -> u32 {
    if tank.held().is_none_or(|(held, _)| held != chemical) {
        return 0;
    }
    tank.spend(amount)
}

/// One tank of one fluid, in millibuckets: what the washer's water tank holds.
///
/// The shape is [`Tank`]'s (an id and an amount, `amount == 0` empty whatever
/// the id), kept apart because a fluid is not a chemical: the two capabilities
/// are different tables and neither id is valid in the other's tank.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FluidTank {
    /// The fluid it holds (`minecraft:water`), `None` when it holds none.
    pub fluid: Option<String>,
    /// How many millibuckets of it (a bucket is a thousand).
    pub amount: u32,
}

impl FluidTank {
    /// A tank holding `amount` millibuckets of `fluid` (`amount` 0 for empty).
    #[must_use]
    pub fn new(fluid: &str, amount: u32) -> Self {
        if amount == 0 {
            Self::default()
        } else {
            Self {
                fluid: Some(fluid.to_owned()),
                amount,
            }
        }
    }

    /// The fluid and its amount, `None` for an empty tank.
    #[must_use]
    pub fn held(&self) -> Option<(&str, u32)> {
        match (&self.fluid, self.amount) {
            (Some(fluid), amount) if amount > 0 => Some((fluid, amount)),
            _ => None,
        }
    }

    /// Takes `amount` millibuckets out of the tank and answers how much came
    /// out; a tank that runs dry is empty, not a fluid at 0.
    pub fn spend(&mut self, amount: u32) -> u32 {
        let came = amount.min(self.amount);
        self.amount -= came;
        if self.amount == 0 {
            *self = Self::default();
        }
        came
    }
}

/// Puts up to `amount` millibuckets of `fluid` into `tank`, up to `capacity`:
/// how much went in.
///
/// Only [`WATER`] goes in: upstream's tank predicate is
/// `containsRecipeAB(fluid, ...)`, and every washing recipe takes water, so a
/// lava (or any other fluid) offer moves nothing. A different fluid than the one
/// held is refused whole, as `BasicFluidTank.insert` tests the held stack first.
#[must_use]
pub fn insert_fluid(tank: &mut FluidTank, fluid: &str, amount: u32, capacity: u32) -> u32 {
    if fluid != WATER {
        return 0;
    }
    if tank.held().is_some_and(|(held, _)| held != WATER) {
        return 0;
    }
    let held = tank.amount.min(capacity);
    let went = amount.min(capacity.saturating_sub(held));
    if went > 0 {
        *tank = FluidTank::new(WATER, held + went);
    }
    went
}

// ---------------------------------------------------------------------------
// Slice 5: the Chemical Injection Chamber — a consumer of the capability
// ---------------------------------------------------------------------------

/// One gas-machine recipe as the machine runs it: the items one operation
/// consumes, the chemicals its gas input accepts (the jar writes either one id
/// or a chemical tag), how much of that gas **one operating tick** takes, and
/// what it produces. The Chemical Injection Chamber and the Purification
/// Chamber run the same `ItemStackChemicalToItemStack` recipe type, so they
/// share this shape.
///
/// `per_tick` is the jar's `chemical_input.amount` under `per_tick_usage: true`.
/// Upstream spends it through `ChemicalUsageMultiplier.constantUse`, so a whole
/// operation takes `per_tick × 200` units.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GasRecipe {
    /// Items taken from the input slot when the operation finishes.
    pub input: u32,
    /// The chemicals the gas input accepts (a tag expands to its members).
    pub chemical: &'static [&'static str],
    /// Units of it one operating tick spends.
    pub per_tick: u32,
    /// The result, a stack of the recipe's item and count.
    pub output: ItemAmount,
}

impl GasRecipe {
    /// Whether the recipe takes `chemical`.
    #[must_use]
    pub fn accepts(&self, chemical: &str) -> bool {
        self.chemical.contains(&chemical)
    }
}

/// The jar's `mekanism:injecting` recipes over the items and chemicals this port
/// registers, generated into `src/runtime.rs` by `tools/gen_injecting.py`.
pub mod injecting {
    use super::{GasRecipe, ItemAmount};

    /// How many rows the table holds.
    pub const ROW_COUNT: usize = TABLE.len();

    /// The recipe for `item`, whatever the machine's tank holds.
    #[must_use]
    pub fn recipe_for(item: &str) -> Option<GasRecipe> {
        TABLE.iter().find(|row| row.0 == item).map(row_recipe)
    }

    /// The recipe for `item`, only when it takes `chemical`.
    #[must_use]
    pub fn lookup(item: &str, chemical: &str) -> Option<GasRecipe> {
        recipe_for(item).filter(|recipe| recipe.accepts(chemical))
    }

    fn row_recipe(row: &Row) -> GasRecipe {
        GasRecipe {
            input: row.1,
            chemical: row.2,
            per_tick: row.3,
            output: ItemAmount::new(row.4, row.5),
        }
    }

    /// `(input item, items consumed, accepted chemicals, per-tick units, output
    /// item, output count)`.
    type Row = (
        &'static str,
        u32,
        &'static [&'static str],
        u32,
        &'static str,
        u32,
    );

    // BEGIN GENERATED INJECTING TABLE (tools/gen_injecting.py)
    // Generated from the jar's `mekanism:injecting` recipes by
    // `tools/gen_injecting.py`; do not edit by hand. Each row is
    // `(input item, items consumed, accepted chemicals, per-tick units,
    // output item, output count)`; an item tag is expanded to the members
    // 26.3 has and a chemical tag to the chemicals this port knows (see the
    // tool's ITEM_TAGS/CHEMICAL_TAGS), and a recipe whose output the port
    // does not register is left out.
    // Source recipes: 91. Rows: 79. Skipped: 15.
    static TABLE: &[Row] = &[
        (
            "mekanism:crystal_copper",
            1,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_copper",
            1,
        ),
        (
            "mekanism:crystal_gold",
            1,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_gold",
            1,
        ),
        (
            "mekanism:crystal_iron",
            1,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_iron",
            1,
        ),
        (
            "mekanism:crystal_lead",
            1,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_lead",
            1,
        ),
        (
            "mekanism:crystal_osmium",
            1,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_osmium",
            1,
        ),
        (
            "mekanism:crystal_tin",
            1,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_tin",
            1,
        ),
        (
            "mekanism:crystal_uranium",
            1,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_uranium",
            1,
        ),
        (
            "minecraft:black_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:black_concrete",
            1,
        ),
        (
            "minecraft:blue_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:blue_concrete",
            1,
        ),
        (
            "minecraft:brown_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:brown_concrete",
            1,
        ),
        (
            "minecraft:chiseled_copper",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:exposed_chiseled_copper",
            1,
        ),
        (
            "minecraft:copper_block",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:exposed_copper",
            1,
        ),
        (
            "minecraft:copper_bulb",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:exposed_copper_bulb",
            1,
        ),
        (
            "minecraft:copper_door",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:exposed_copper_door",
            1,
        ),
        (
            "minecraft:copper_grate",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:exposed_copper_grate",
            1,
        ),
        (
            "minecraft:copper_ore",
            1,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_copper",
            4,
        ),
        (
            "minecraft:copper_trapdoor",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:exposed_copper_trapdoor",
            1,
        ),
        (
            "minecraft:cut_copper",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:exposed_cut_copper",
            1,
        ),
        (
            "minecraft:cut_copper_slab",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:exposed_cut_copper_slab",
            1,
        ),
        (
            "minecraft:cut_copper_stairs",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:exposed_cut_copper_stairs",
            1,
        ),
        (
            "minecraft:cyan_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:cyan_concrete",
            1,
        ),
        (
            "minecraft:dead_brain_coral",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            3,
            "minecraft:brain_coral",
            1,
        ),
        (
            "minecraft:dead_brain_coral_block",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            5,
            "minecraft:brain_coral_block",
            1,
        ),
        (
            "minecraft:dead_brain_coral_fan",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            3,
            "minecraft:brain_coral_fan",
            1,
        ),
        (
            "minecraft:dead_bubble_coral",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            3,
            "minecraft:bubble_coral",
            1,
        ),
        (
            "minecraft:dead_bubble_coral_block",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            5,
            "minecraft:bubble_coral_block",
            1,
        ),
        (
            "minecraft:dead_bubble_coral_fan",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            3,
            "minecraft:bubble_coral_fan",
            1,
        ),
        (
            "minecraft:dead_fire_coral",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            3,
            "minecraft:fire_coral",
            1,
        ),
        (
            "minecraft:dead_fire_coral_block",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            5,
            "minecraft:fire_coral_block",
            1,
        ),
        (
            "minecraft:dead_fire_coral_fan",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            3,
            "minecraft:fire_coral_fan",
            1,
        ),
        (
            "minecraft:dead_horn_coral",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            3,
            "minecraft:horn_coral",
            1,
        ),
        (
            "minecraft:dead_horn_coral_block",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            5,
            "minecraft:horn_coral_block",
            1,
        ),
        (
            "minecraft:dead_horn_coral_fan",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            3,
            "minecraft:horn_coral_fan",
            1,
        ),
        (
            "minecraft:dead_tube_coral",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            3,
            "minecraft:tube_coral",
            1,
        ),
        (
            "minecraft:dead_tube_coral_block",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            5,
            "minecraft:tube_coral_block",
            1,
        ),
        (
            "minecraft:dead_tube_coral_fan",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            3,
            "minecraft:tube_coral_fan",
            1,
        ),
        (
            "minecraft:deepslate_copper_ore",
            1,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_copper",
            4,
        ),
        (
            "minecraft:deepslate_gold_ore",
            1,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_gold",
            4,
        ),
        (
            "minecraft:deepslate_iron_ore",
            1,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_iron",
            4,
        ),
        (
            "minecraft:dirt",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:mud",
            1,
        ),
        (
            "minecraft:exposed_chiseled_copper",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:weathered_chiseled_copper",
            1,
        ),
        (
            "minecraft:exposed_copper",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:weathered_copper",
            1,
        ),
        (
            "minecraft:exposed_copper_bulb",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:weathered_copper_bulb",
            1,
        ),
        (
            "minecraft:exposed_copper_door",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:weathered_copper_door",
            1,
        ),
        (
            "minecraft:exposed_copper_grate",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:weathered_copper_grate",
            1,
        ),
        (
            "minecraft:exposed_copper_trapdoor",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:weathered_copper_trapdoor",
            1,
        ),
        (
            "minecraft:exposed_cut_copper",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:weathered_cut_copper",
            1,
        ),
        (
            "minecraft:exposed_cut_copper_slab",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:weathered_cut_copper_slab",
            1,
        ),
        (
            "minecraft:exposed_cut_copper_stairs",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:weathered_cut_copper_stairs",
            1,
        ),
        (
            "minecraft:gold_ore",
            1,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_gold",
            4,
        ),
        (
            "minecraft:gray_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:gray_concrete",
            1,
        ),
        (
            "minecraft:green_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:green_concrete",
            1,
        ),
        (
            "minecraft:iron_ore",
            1,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_iron",
            4,
        ),
        (
            "minecraft:light_blue_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:light_blue_concrete",
            1,
        ),
        (
            "minecraft:light_gray_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:light_gray_concrete",
            1,
        ),
        (
            "minecraft:lime_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:lime_concrete",
            1,
        ),
        (
            "minecraft:magenta_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:magenta_concrete",
            1,
        ),
        (
            "minecraft:orange_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:orange_concrete",
            1,
        ),
        (
            "minecraft:pink_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:pink_concrete",
            1,
        ),
        (
            "minecraft:purple_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:purple_concrete",
            1,
        ),
        (
            "minecraft:raw_copper",
            3,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_copper",
            8,
        ),
        (
            "minecraft:raw_copper_block",
            1,
            &["mekanism:hydrogen_chloride"],
            2,
            "mekanism:shard_copper",
            24,
        ),
        (
            "minecraft:raw_gold",
            3,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_gold",
            8,
        ),
        (
            "minecraft:raw_gold_block",
            1,
            &["mekanism:hydrogen_chloride"],
            2,
            "mekanism:shard_gold",
            24,
        ),
        (
            "minecraft:raw_iron",
            3,
            &["mekanism:hydrogen_chloride"],
            1,
            "mekanism:shard_iron",
            8,
        ),
        (
            "minecraft:raw_iron_block",
            1,
            &["mekanism:hydrogen_chloride"],
            2,
            "mekanism:shard_iron",
            24,
        ),
        (
            "minecraft:red_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:red_concrete",
            1,
        ),
        (
            "minecraft:terracotta",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:clay",
            1,
        ),
        (
            "minecraft:weathered_chiseled_copper",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:oxidized_chiseled_copper",
            1,
        ),
        (
            "minecraft:weathered_copper",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:oxidized_copper",
            1,
        ),
        (
            "minecraft:weathered_copper_bulb",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:oxidized_copper_bulb",
            1,
        ),
        (
            "minecraft:weathered_copper_door",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:oxidized_copper_door",
            1,
        ),
        (
            "minecraft:weathered_copper_grate",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:oxidized_copper_grate",
            1,
        ),
        (
            "minecraft:weathered_copper_trapdoor",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:oxidized_copper_trapdoor",
            1,
        ),
        (
            "minecraft:weathered_cut_copper",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:oxidized_cut_copper",
            1,
        ),
        (
            "minecraft:weathered_cut_copper_slab",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:oxidized_cut_copper_slab",
            1,
        ),
        (
            "minecraft:weathered_cut_copper_stairs",
            1,
            &["mekanism:oxygen"],
            1,
            "minecraft:oxidized_cut_copper_stairs",
            1,
        ),
        (
            "minecraft:white_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:white_concrete",
            1,
        ),
        (
            "minecraft:yellow_concrete_powder",
            1,
            &["mekanism:water_vapor", "mekanism:steam"],
            1,
            "minecraft:yellow_concrete",
            1,
        ),
    ];
    // END GENERATED INJECTING TABLE
}

/// The jar's `mekanism:purifying` recipes over the items this port registers,
/// generated into `src/runtime.rs` by `tools/gen_purifying.py`.
pub mod purifying {
    use super::{GasRecipe, ItemAmount};

    /// How many rows the table holds.
    pub const ROW_COUNT: usize = TABLE.len();

    /// The recipe for `item`, whatever the machine's tank holds.
    #[must_use]
    pub fn recipe_for(item: &str) -> Option<GasRecipe> {
        TABLE.iter().find(|row| row.0 == item).map(row_recipe)
    }

    /// The recipe for `item`, only when it takes `chemical`.
    #[must_use]
    pub fn lookup(item: &str, chemical: &str) -> Option<GasRecipe> {
        recipe_for(item).filter(|recipe| recipe.accepts(chemical))
    }

    fn row_recipe(row: &Row) -> GasRecipe {
        GasRecipe {
            input: row.1,
            chemical: row.2,
            per_tick: row.3,
            output: ItemAmount::new(row.4, row.5),
        }
    }

    /// `(input item, items consumed, accepted chemicals, per-tick units, output
    /// item, output count)`.
    type Row = (
        &'static str,
        u32,
        &'static [&'static str],
        u32,
        &'static str,
        u32,
    );

    // BEGIN GENERATED PURIFYING TABLE (tools/gen_purifying.py)
    // Generated from the jar's `mekanism:purifying` recipes by
    // `tools/gen_purifying.py`; do not edit by hand. Each row is
    // `(input item, items consumed, accepted chemicals, per-tick units,
    // output item, output count)`; an item tag is expanded to the members
    // 26.3 has and a chemical tag to the chemicals this port knows (see the
    // tool's ITEM_TAGS/CHEMICAL_TAGS), and a recipe whose output the port
    // does not register is left out.
    // Source recipes: 28. Rows: 15. Skipped: 16.
    static TABLE: &[Row] = &[
        (
            "mekanism:shard_copper",
            1,
            &["mekanism:oxygen"],
            1,
            "mekanism:clump_copper",
            1,
        ),
        (
            "mekanism:shard_gold",
            1,
            &["mekanism:oxygen"],
            1,
            "mekanism:clump_gold",
            1,
        ),
        (
            "mekanism:shard_iron",
            1,
            &["mekanism:oxygen"],
            1,
            "mekanism:clump_iron",
            1,
        ),
        (
            "minecraft:copper_ore",
            1,
            &["mekanism:oxygen"],
            1,
            "mekanism:clump_copper",
            3,
        ),
        (
            "minecraft:deepslate_copper_ore",
            1,
            &["mekanism:oxygen"],
            1,
            "mekanism:clump_copper",
            3,
        ),
        (
            "minecraft:deepslate_gold_ore",
            1,
            &["mekanism:oxygen"],
            1,
            "mekanism:clump_gold",
            3,
        ),
        (
            "minecraft:deepslate_iron_ore",
            1,
            &["mekanism:oxygen"],
            1,
            "mekanism:clump_iron",
            3,
        ),
        (
            "minecraft:gold_ore",
            1,
            &["mekanism:oxygen"],
            1,
            "mekanism:clump_gold",
            3,
        ),
        (
            "minecraft:iron_ore",
            1,
            &["mekanism:oxygen"],
            1,
            "mekanism:clump_iron",
            3,
        ),
        (
            "minecraft:raw_copper",
            1,
            &["mekanism:oxygen"],
            1,
            "mekanism:clump_copper",
            2,
        ),
        (
            "minecraft:raw_copper_block",
            1,
            &["mekanism:oxygen"],
            2,
            "mekanism:clump_copper",
            18,
        ),
        (
            "minecraft:raw_gold",
            1,
            &["mekanism:oxygen"],
            1,
            "mekanism:clump_gold",
            2,
        ),
        (
            "minecraft:raw_gold_block",
            1,
            &["mekanism:oxygen"],
            2,
            "mekanism:clump_gold",
            18,
        ),
        (
            "minecraft:raw_iron",
            1,
            &["mekanism:oxygen"],
            1,
            "mekanism:clump_iron",
            2,
        ),
        (
            "minecraft:raw_iron_block",
            1,
            &["mekanism:oxygen"],
            2,
            "mekanism:clump_iron",
            18,
        ),
    ];
    // END GENERATED PURIFYING TABLE
}

/// The gas the machine pulls from a neighbour for the tick it is about to run:
/// the chemical to ask for and how much of it, or `None` when its own tank
/// already holds a tick's worth or a chemical the recipe does not take.
///
/// Exactly one tick's worth is asked for, so a neighbour loses the gas the
/// operation spends and no more: the machine's tank is a buffer, not a
/// reservoir.
#[must_use]
pub fn gas_pull(recipe: &GasRecipe, tank: &Tank) -> Option<(&'static str, u32)> {
    let chemical = match tank.held() {
        Some((held, _)) if recipe.accepts(held) => {
            *recipe.chemical.iter().find(|it| **it == held)?
        }
        Some(_) => return None,
        None => recipe.chemical.first().copied()?,
    };
    let want = recipe.per_tick.saturating_sub(tank.amount);
    (want > 0).then_some((chemical, want))
}

/// One tick of a gas machine — the Chemical Injection Chamber or the
/// Purification Chamber (`TileEntityAdvancedElectricMachine` through
/// `ItemStackConstantChemicalToObjectCachedRecipe`); the two differ only in
/// their operation's constants and their recipe table.
///
/// `lookup` answers the recipe for an input item id. Each operating tick costs
/// `operation.usage` energy **and** `recipe.per_tick` units of the gas, and adds
/// one progress tick; on `operation.ticks` the recipe's input count is consumed
/// and its result goes into the output. A tank of another chemical, or one with
/// less than a tick's gas, **holds** the progress and spends nothing (upstream's
/// constant-usage handler pauses instead of resetting); so do no energy and no
/// output room. An empty input, an input without a recipe or too few items reset
/// it, exactly as the other machines do.
pub fn process_gas_tick(
    machine: &mut Machine,
    operation: Operation,
    tank: &mut Tank,
    input: &mut ItemAmount,
    output: &mut ItemAmount,
    max_stack: u32,
    lookup: &dyn Fn(&str) -> Option<GasRecipe>,
) -> Outcome {
    if input.is_empty() {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NoInput);
    }
    let Some(recipe) = lookup(&input.item).filter(|recipe| !recipe.output.is_empty()) else {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NoRecipe);
    };
    if input.count < recipe.input.max(1) {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NotEnoughInput);
    }
    if tank
        .held()
        .is_none_or(|(held, stored)| !recipe.accepts(held) || stored < recipe.per_tick)
    {
        return Outcome::Idle(IdleReason::NoChemical);
    }
    if room_for(output, &recipe.output, max_stack) < recipe.output.count {
        return Outcome::Idle(IdleReason::NoOutputSpace);
    }
    if machine.energy < operation.usage {
        return Outcome::Idle(IdleReason::NoEnergy);
    }
    machine.energy -= operation.usage;
    machine.progress += 1;
    tank.spend(recipe.per_tick);
    if machine.progress < operation.ticks {
        return Outcome::Working;
    }
    machine.progress = 0;
    input.count -= recipe.input.max(1);
    if output.is_empty() {
        *output = recipe.output;
    } else {
        output.count += recipe.output.count;
    }
    Outcome::Produced
}

// ---------------------------------------------------------------------------
// Slice 8: the Chemical Washer — a fluid beside the chemical inputs
// ---------------------------------------------------------------------------

/// The fluid every washing recipe takes: `minecraft:water`, as the jar's
/// `fluid_input` tag names it.
pub const WATER: &str = "minecraft:water";

/// The washer's usage in FE per tick (`UsageConfig.chemicalWasher`, `200L`).
pub const WASHER_USAGE: i32 = 200;

/// The washer's buffer in FE (`StorageConfig.chemicalWasher`, `80_000L, 1`).
pub const WASHER_CAPACITY: i32 = 80_000;

/// Ticks one washing operation takes (`TileEntityRecipeMachine`'s
/// `BASE_TICKS_REQUIRED`), so one operation costs 200 x 200 = 40,000 FE.
pub const WASHER_TICKS: i32 = 200;

/// The washer's one operation: 200 ticks at 200 FE/t.
pub const WASHER_OPERATION: Operation = Operation {
    ticks: WASHER_TICKS,
    usage: WASHER_USAGE,
};

/// The washer's water tank, in millibuckets (`MAX_FLUID = 10 *
/// BUCKET_VOLUME`). Water is what every washing recipe takes, so the fluid side
/// of the machine is one tank of it.
pub const WASHER_WATER: u32 = 10_000;

/// Each of the washer's two slurry tanks, in millibuckets (`MAX_SLURRY = 10 *
/// BUCKET_VOLUME`): the dirty one it takes and the clean one it makes.
pub const WASHER_SLURRY: u32 = 10_000;

/// One `mekanism:washing` recipe as the washer runs it: the fluid and its
/// amount one operation consumes, the dirty slurry and its amount, and the clean
/// slurry it makes. `FluidChemicalToChemicalRecipe`.
///
/// Both inputs are consumed **when the operation finishes**
/// (`TwoInputCachedRecipe` overrides no `useResources`), unlike the gas
/// machines' per-tick chemical, so one operation takes exactly one row's worth.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WashRecipe {
    /// The fluid the recipe takes ([`WATER`]).
    pub fluid: &'static str,
    /// Millibuckets of it one operation consumes.
    pub fluid_amount: u32,
    /// The dirty slurry the recipe takes.
    pub chemical: &'static str,
    /// Units of it one operation consumes.
    pub chemical_amount: u32,
    /// The clean slurry it makes.
    pub output: &'static str,
    /// Units of it one operation makes.
    pub output_amount: u32,
}

/// The jar's `mekanism:washing` recipes, generated into `src/runtime.rs` by
/// `tools/gen_washing.py`.
pub mod washing {
    use super::WashRecipe;

    /// How many rows the table holds.
    pub const ROW_COUNT: usize = TABLE.len();

    /// The recipe whose dirty slurry is `chemical`, `None` when no row takes it.
    #[must_use]
    pub fn recipe_for(chemical: &str) -> Option<WashRecipe> {
        TABLE
            .iter()
            .find(|row| row.1 == chemical)
            .map(|row| WashRecipe {
                fluid: row.0,
                fluid_amount: row.2,
                chemical: row.1,
                chemical_amount: row.3,
                output: row.4,
                output_amount: row.5,
            })
    }

    /// `(fluid, dirty slurry, fluid millibuckets, slurry units, clean slurry,
    /// clean units)`.
    type Row = (&'static str, &'static str, u32, u32, &'static str, u32);

    // BEGIN GENERATED WASHING TABLE (tools/gen_washing.py)
    // Generated from the jar's `mekanism:washing` recipes by
    // `tools/gen_washing.py`; do not edit by hand. Each row is
    // `(fluid, dirty slurry, fluid millibuckets, slurry units, clean
    // slurry, clean units)`, the amounts one **operation** takes and makes
    // (both inputs are consumed when it finishes, not per tick), and a
    // recipe whose slurry ids this port does not know is left out.
    // Source recipes: 7. Rows: 7. Skipped: 0.
    static TABLE: &[Row] = &[
        (
            "minecraft:water",
            "mekanism:dirty_copper",
            5,
            1,
            "mekanism:clean_copper",
            1,
        ),
        (
            "minecraft:water",
            "mekanism:dirty_gold",
            5,
            1,
            "mekanism:clean_gold",
            1,
        ),
        (
            "minecraft:water",
            "mekanism:dirty_iron",
            5,
            1,
            "mekanism:clean_iron",
            1,
        ),
        (
            "minecraft:water",
            "mekanism:dirty_lead",
            5,
            1,
            "mekanism:clean_lead",
            1,
        ),
        (
            "minecraft:water",
            "mekanism:dirty_osmium",
            5,
            1,
            "mekanism:clean_osmium",
            1,
        ),
        (
            "minecraft:water",
            "mekanism:dirty_tin",
            5,
            1,
            "mekanism:clean_tin",
            1,
        ),
        (
            "minecraft:water",
            "mekanism:dirty_uranium",
            5,
            1,
            "mekanism:clean_uranium",
            1,
        ),
    ];
    // END GENERATED WASHING TABLE
}

/// One tick of the Chemical Washer (`TileEntityChemicalWasher` through
/// `TwoInputCachedRecipe.fluidChemicalToChemical`).
///
/// `lookup` answers the recipe for a dirty slurry. Each operating tick costs
/// `operation.usage` energy and adds one progress tick; on `operation.ticks` the
/// recipe's fluid and slurry are consumed and its clean slurry goes into the
/// output tank. The fluid side is the **secondary** input: an empty water tank
/// (or one of another fluid) is upstream's `mismatchedRecipe`, and less water
/// than one operation needs is `NOT_ENOUGH_SECONDARY_INPUT` — both reset the
/// progress, as the item machines reset theirs. Too little dirty slurry resets
/// it too. No energy and no output room **hold** it.
pub fn process_wash_tick(
    machine: &mut Machine,
    operation: Operation,
    water: &mut FluidTank,
    slurry_in: &mut Tank,
    slurry_out: &mut Tank,
    slurry_capacity: u32,
    lookup: &dyn Fn(&str) -> Option<WashRecipe>,
) -> Outcome {
    let Some((dirty, dirty_amount)) = slurry_in.held() else {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NoInput);
    };
    let Some(recipe) = lookup(dirty).filter(|recipe| recipe.output_amount > 0) else {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NoRecipe);
    };
    match water.held() {
        Some((fluid, amount)) if fluid == recipe.fluid && amount >= recipe.fluid_amount => {}
        _ => {
            machine.progress = 0;
            return Outcome::Idle(IdleReason::NoFluid);
        }
    }
    if dirty_amount < recipe.chemical_amount {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NotEnoughInput);
    }
    if room_in_tank(
        slurry_out,
        recipe.output,
        recipe.output_amount,
        slurry_capacity,
    ) < recipe.output_amount
    {
        return Outcome::Idle(IdleReason::NoOutputSpace);
    }
    if machine.energy < operation.usage {
        return Outcome::Idle(IdleReason::NoEnergy);
    }
    machine.energy -= operation.usage;
    machine.progress += 1;
    if machine.progress < operation.ticks {
        return Outcome::Working;
    }
    machine.progress = 0;
    water.spend(recipe.fluid_amount);
    slurry_in.spend(recipe.chemical_amount);
    let held = slurry_out.amount.min(slurry_capacity);
    *slurry_out = Tank::new(recipe.output, held + recipe.output_amount);
    Outcome::Produced
}

/// How much of a recipe's clean slurry an output tank still takes: its whole
/// capacity when empty, the room left when it holds the same chemical, and none
/// when it holds another (upstream's `NOT_ENOUGH_OUTPUT_SPACE`).
#[must_use]
pub fn room_in_tank(output: &Tank, recipe_output: &str, recipe_amount: u32, capacity: u32) -> u32 {
    let room = match output.held() {
        None => capacity,
        Some((held, _)) if held == recipe_output => capacity.saturating_sub(output.amount),
        Some(_) => 0,
    };
    // One operation must fit at all: a tank smaller than a row's output takes
    // nothing, which is the same answer as a full one.
    if room >= recipe_amount { room } else { 0 }
}

/// The dirty slurry the washer pulls from a neighbour for the operation it is
/// about to run: which slurry and how much, or `None` when its own tank already
/// holds the operation's worth (or holds a slurry it cannot use).
///
/// One **operation's** amount is asked for, not one tick's: this machine consumes
/// its chemical when the operation finishes, unlike the gas machines' per-tick
/// gas. The port's recorded deviation for the gas machines applies here: with no
/// pipes, a machine takes from the handler beside it what it is about to spend.
#[must_use]
pub fn washer_pull(recipe: &WashRecipe, tank: &Tank) -> Option<(&'static str, u32)> {
    match tank.held() {
        Some((held, stored)) if held == recipe.chemical => {
            let want = recipe.chemical_amount.saturating_sub(stored);
            (want > 0).then_some((recipe.chemical, want))
        }
        Some(_) => None,
        None => Some((recipe.chemical, recipe.chemical_amount)),
    }
}

// ---------------------------------------------------------------------------
// Slice 9: the Chemical Crystallizer — a clean slurry into a crystal
// ---------------------------------------------------------------------------

/// The crystallizer's usage in FE per tick (`UsageConfig.chemicalCrystallizer`,
/// `400L`). Not the washer's 200: the two machines are different rows of the
/// jar's config, and the crystallizer is the dearer one.
pub const CRYSTALLIZER_USAGE: i32 = 400;

/// The crystallizer's buffer in FE (`StorageConfig.chemicalCrystallizer`,
/// `160_000L, 1`).
pub const CRYSTALLIZER_CAPACITY: i32 = 160_000;

/// Ticks one crystallizing operation takes
/// (`TileEntityChemicalCrystallizer.BASE_TICKS_REQUIRED` =
/// `10 * SharedConstants.TICKS_PER_SECOND`), so one operation costs
/// 200 x 400 = 80,000 FE and the 160,000 buffer holds exactly two.
pub const CRYSTALLIZER_TICKS: i32 = 200;

/// The crystallizer's one operation: 200 ticks at 400 FE/t.
pub const CRYSTALLIZER_OPERATION: Operation = Operation {
    ticks: CRYSTALLIZER_TICKS,
    usage: CRYSTALLIZER_USAGE,
};

/// The crystallizer's own tank, in units (`MAX_CHEMICAL = 10L *
/// FluidType.BUCKET_VOLUME`). It takes a clean slurry and gives none back
/// (upstream's `BasicChemicalTank.inputModern`).
pub const CRYSTALLIZER_TANK: u32 = 10_000;

/// One `mekanism:crystallizing` recipe as the crystallizer runs it: the clean
/// slurry and its amount one operation consumes, and the crystal it makes.
/// `ChemicalCrystallizerRecipe` over `BasicChemicalCrystallizerRecipe`.
///
/// The input is consumed **when the operation finishes**
/// (`OneInputCachedRecipe.finishProcessing` calls `inputHandler.use`), so one
/// operation takes exactly one row's worth — not a per-tick draw like the gas
/// machines' gas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CrystallizeRecipe {
    /// The clean slurry the recipe takes.
    pub chemical: &'static str,
    /// Units of it one operation consumes.
    pub amount: u32,
    /// The crystal item it makes.
    pub output: &'static str,
    /// How many of it one operation makes.
    pub output_count: u32,
}

/// The jar's `mekanism:crystallizing` recipes this port can run, generated into
/// `src/runtime.rs` by `tools/gen_crystallizing.py`.
pub mod crystallizing {
    use super::{CrystallizeRecipe, ItemAmount, room_for};

    /// How many rows the table holds.
    pub const ROW_COUNT: usize = TABLE.len();

    /// The recipe whose clean slurry is `chemical`, `None` when no row takes it.
    #[must_use]
    pub fn recipe_for(chemical: &str) -> Option<CrystallizeRecipe> {
        TABLE
            .iter()
            .find(|row| row.0 == chemical)
            .map(|row| CrystallizeRecipe {
                chemical: row.0,
                amount: row.1,
                output: row.2,
                output_count: row.3,
            })
    }

    /// Whether the machine's tank takes `chemical`: upstream's tank predicate is
    /// `containsRecipe`, so a chemical with no row (a dirty slurry, a gas, water)
    /// is refused by name rather than stored and found useless later.
    #[must_use]
    pub fn accepts(chemical: &str) -> bool {
        recipe_for(chemical).is_some()
    }

    /// The clean slurries the table takes, in table order — the order an empty
    /// tank asks a neighbour for them.
    pub fn slurries() -> impl Iterator<Item = &'static str> {
        TABLE.iter().map(|row| row.0)
    }

    /// The slurries an **empty** tank asks a neighbour for: only those whose
    /// crystal the output slot could take, in table order.
    ///
    /// The room check belongs to a candidate, not to the loop. A slot holding
    /// one material's crystals (`crystal_iron`) takes `clean_iron` and reports
    /// no other row, so the tank does not stop at the table's first entry
    /// (`clean_copper`) and starve; a full stack, or a slot of another item,
    /// takes nothing, so a machine that could not finish an operation never
    /// siphons its neighbour.
    pub fn slurries_that_fit(
        output: &ItemAmount,
        max_stack: u32,
    ) -> impl Iterator<Item = &'static str> {
        TABLE.iter().filter_map(move |row| {
            (room_for(output, &ItemAmount::new(row.2, row.3), max_stack) >= row.3).then_some(row.0)
        })
    }

    /// `(clean slurry, its units, crystal item, count)`.
    type Row = (&'static str, u32, &'static str, u32);

    // BEGIN GENERATED CRYSTALLIZING TABLE (tools/gen_crystallizing.py)
    // Generated from the jar's `mekanism:crystallizing` recipes by
    // `tools/gen_crystallizing.py`; do not edit by hand. Each row is
    // `(clean slurry, its units, crystal item, count)`, the amount one
    // **operation** takes (it is consumed when the operation finishes, not
    // per tick), and a recipe whose crystal item this port does not
    // register is left out.
    // Source recipes: 10. Rows: 7. Skipped: 3.
    static TABLE: &[Row] = &[
        ("mekanism:clean_copper", 200, "mekanism:crystal_copper", 1),
        ("mekanism:clean_gold", 200, "mekanism:crystal_gold", 1),
        ("mekanism:clean_iron", 200, "mekanism:crystal_iron", 1),
        ("mekanism:clean_lead", 200, "mekanism:crystal_lead", 1),
        ("mekanism:clean_osmium", 200, "mekanism:crystal_osmium", 1),
        ("mekanism:clean_tin", 200, "mekanism:crystal_tin", 1),
        ("mekanism:clean_uranium", 200, "mekanism:crystal_uranium", 1),
    ];
    // END GENERATED CRYSTALLIZING TABLE
}

/// One tick of the Chemical Crystallizer (`TileEntityChemicalCrystallizer`
/// through `OneInputCachedRecipe.crystallizing`).
///
/// `lookup` answers the recipe for the tank's clean slurry. Each operating tick
/// costs `operation.usage` energy and adds one progress tick; on
/// `operation.ticks` the recipe's slurry is consumed and its crystal goes into
/// the output slot. An empty tank, a chemical with no row and too little slurry
/// are upstream's `mismatchedRecipe`/`NOT_ENOUGH_INPUT`: they **reset** the
/// progress, as the item machines' bad input does. No energy and no output room
/// **hold** it.
pub fn process_crystal_tick(
    machine: &mut Machine,
    operation: Operation,
    tank: &mut Tank,
    output: &mut ItemAmount,
    max_stack: u32,
    lookup: &dyn Fn(&str) -> Option<CrystallizeRecipe>,
) -> Outcome {
    let Some((chemical, amount)) = tank.held() else {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NoChemicalInput);
    };
    let Some(recipe) = lookup(chemical).filter(|recipe| recipe.output_count > 0) else {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NoRecipe);
    };
    if amount < recipe.amount {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NotEnoughInput);
    }
    let result = ItemAmount::new(recipe.output, recipe.output_count);
    if room_for(output, &result, max_stack) < recipe.output_count {
        return Outcome::Idle(IdleReason::NoOutputSpace);
    }
    if machine.energy < operation.usage {
        return Outcome::Idle(IdleReason::NoEnergy);
    }
    machine.energy -= operation.usage;
    machine.progress += 1;
    if machine.progress < operation.ticks {
        return Outcome::Working;
    }
    machine.progress = 0;
    tank.spend(recipe.amount);
    if output.is_empty() {
        *output = result;
    } else {
        output.count += recipe.output_count;
    }
    Outcome::Produced
}

/// The clean slurry the crystallizer pulls from a neighbour for the operation it
/// is about to run: which slurry and how much, or `None` when its own tank
/// already holds the operation's worth (or holds a chemical with a row that is
/// not this recipe's — the tank never mixes).
///
/// One **operation's** amount is asked for, not one tick's: this machine consumes
/// its slurry when the operation finishes. The port's recorded deviation for the
/// gas machines applies here: with no pipes, a machine takes from the handler
/// beside it what it is about to spend.
#[must_use]
pub fn crystallizer_pull(recipe: &CrystallizeRecipe, tank: &Tank) -> Option<(&'static str, u32)> {
    match tank.held() {
        Some((held, stored)) if held == recipe.chemical => {
            let want = recipe.amount.saturating_sub(stored);
            (want > 0).then_some((recipe.chemical, want))
        }
        Some(_) => None,
        None => Some((recipe.chemical, recipe.amount)),
    }
}

// ---------------------------------------------------------------------------
// Slice 15: the Chemical Dissolution Chamber — an ore and acid into a dirty slurry
// ---------------------------------------------------------------------------

/// The acid every dissolution recipe takes (`chemical_input.chemical` in the
/// jar's `processing/<material>/slurry/dirty/*.json`).
pub const SULFURIC_ACID: &str = "mekanism:sulfuric_acid";

/// The chamber's usage in FE per tick (`UsageConfig.chemicalDissolutionChamber`,
/// `400L`).
pub const DISSOLUTION_USAGE: i32 = 400;

/// The chamber's buffer in FE (`StorageConfig.chemicalDissolutionChamber`,
/// `160_000L, 1`).
pub const DISSOLUTION_CAPACITY: i32 = 160_000;

/// Ticks one operation takes (`TileEntityChemicalDissolutionChamber.BASE_TICKS_REQUIRED`
/// = `5 * SharedConstants.TICKS_PER_SECOND`), so one operation costs
/// 100 x 400 = 40,000 FE.
pub const DISSOLUTION_TICKS: i32 = 100;

/// The chamber's one operation: 100 ticks at 400 FE/t.
pub const DISSOLUTION_OPERATION: Operation = Operation {
    ticks: DISSOLUTION_TICKS,
    usage: DISSOLUTION_USAGE,
};

/// Each of the chamber's two tanks in millibuckets
/// (`TileEntityChemicalDissolutionChamber.MAX_CHEMICAL = 10 * BUCKET_VOLUME`):
/// the acid it takes and the slurry it makes.
pub const DISSOLUTION_TANK: u32 = 10_000;

/// One `mekanism:dissolution` recipe as the chamber runs it: the items one
/// operation consumes, the acid it spends **per tick** (every row in the jar is
/// `per_tick_usage: true`), and the dirty slurry it makes.
///
/// Not `Recipe`, for the same reason as [`ItemAmount`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DissolveRecipe {
    /// Items taken from the input slot when the operation finishes.
    pub input: u32,
    /// Units of [`SULFURIC_ACID`] one operating tick spends.
    pub acid_per_tick: u32,
    /// The dirty slurry it makes (`mekanism:dirty_<material>`).
    pub output: &'static str,
    /// Units of it one operation makes.
    pub output_amount: u32,
}

/// The jar's `mekanism:dissolution` recipes over the items and slurries this
/// port registers, generated into `src/runtime.rs` by `tools/gen_dissolving.py`.
pub mod dissolving {
    use super::DissolveRecipe;

    /// How many rows the table holds.
    pub const ROW_COUNT: usize = TABLE.len();

    /// The recipe for `item`, `None` when nothing dissolves it.
    ///
    /// Upstream looks a dissolution recipe up by the item input; the acid is
    /// the same chemical in every row, so an item names at most one row.
    #[must_use]
    pub fn recipe_for(item: &str) -> Option<DissolveRecipe> {
        TABLE.iter().find(|row| row.0 == item).map(recipe)
    }

    /// One table row as the machine's recipe.
    fn recipe(row: &Row) -> DissolveRecipe {
        DissolveRecipe {
            input: row.1,
            acid_per_tick: row.2,
            output: row.3,
            output_amount: row.4,
        }
    }

    /// The table's rows, so a test can assert every one is reachable and the
    /// sweep can drive the machine from the committed data.
    #[must_use]
    pub fn rows() -> &'static [Row] {
        TABLE
    }

    /// `(input item, items per operation, acid units per tick, dirty slurry,
    /// slurry units per operation)`.
    pub type Row = (&'static str, u32, u32, &'static str, u32);

    // BEGIN GENERATED DISSOLVING TABLE (tools/gen_dissolving.py)
    // Generated from the jar's `mekanism:dissolution` recipes by
    // `tools/gen_dissolving.py`; do not edit by hand. Each row is
    // `(input item, items per operation, acid units per tick, dirty
    // slurry, slurry units per operation)`; every row is per-tick usage,
    // so the acid is spent over the operation's 100 ticks, and a row whose
    // ore or slurry this port does not know is left out.
    // Source recipes: 23. Rows: 12. Skipped: 14.
    static TABLE: &[Row] = &[
        ("minecraft:copper_ore", 1, 1, "mekanism:dirty_copper", 1000),
        (
            "minecraft:deepslate_copper_ore",
            1,
            1,
            "mekanism:dirty_copper",
            1000,
        ),
        (
            "minecraft:deepslate_gold_ore",
            1,
            1,
            "mekanism:dirty_gold",
            1000,
        ),
        (
            "minecraft:deepslate_iron_ore",
            1,
            1,
            "mekanism:dirty_iron",
            1000,
        ),
        ("minecraft:gold_ore", 1, 1, "mekanism:dirty_gold", 1000),
        ("minecraft:iron_ore", 1, 1, "mekanism:dirty_iron", 1000),
        ("minecraft:raw_copper", 3, 1, "mekanism:dirty_copper", 2000),
        (
            "minecraft:raw_copper_block",
            1,
            2,
            "mekanism:dirty_copper",
            6000,
        ),
        ("minecraft:raw_gold", 3, 1, "mekanism:dirty_gold", 2000),
        (
            "minecraft:raw_gold_block",
            1,
            2,
            "mekanism:dirty_gold",
            6000,
        ),
        ("minecraft:raw_iron", 3, 1, "mekanism:dirty_iron", 2000),
        (
            "minecraft:raw_iron_block",
            1,
            2,
            "mekanism:dirty_iron",
            6000,
        ),
    ];
    // END GENERATED DISSOLVING TABLE
}

/// One tick of the Chemical Dissolution Chamber
/// (`TileEntityChemicalDissolutionChamber` through
/// `ItemStackConstantChemicalToObjectCachedRecipe`): an item in the input slot
/// and sulfuric acid in the chamber's own tank become a dirty slurry in its
/// output tank.
///
/// Each operating tick costs `operation.usage` energy **and**
/// `recipe.acid_per_tick` acid and adds one progress tick; on `operation.ticks`
/// the recipe's item count is consumed and its slurry lands in the output tank.
/// A tank of another chemical, or one with less than a tick's acid, **holds**
/// the progress (upstream's constant-usage handler pauses rather than
/// resetting); so do no energy and no room for the whole batch in the output
/// tank. An empty input, an input without a recipe or too few items reset it.
///
/// # Panics
/// When the recipe's output cannot be named as a chemical: every table row is
/// generated from the jar's own register of slurries, so this is a table bug.
pub fn process_dissolve_tick(
    machine: &mut Machine,
    operation: Operation,
    acid: &mut Tank,
    slurry: &mut Tank,
    input: &mut ItemAmount,
    lookup: &dyn Fn(&str) -> Option<DissolveRecipe>,
) -> Outcome {
    if input.is_empty() {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NoInput);
    }
    let Some(recipe) = lookup(&input.item) else {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NoRecipe);
    };
    if input.count < recipe.input.max(1) {
        machine.progress = 0;
        return Outcome::Idle(IdleReason::NotEnoughInput);
    }
    if acid
        .held()
        .is_none_or(|(held, stored)| held != SULFURIC_ACID || stored < recipe.acid_per_tick)
    {
        return Outcome::Idle(IdleReason::NoChemical);
    }
    if room_in_tank(
        slurry,
        recipe.output,
        recipe.output_amount,
        DISSOLUTION_TANK,
    ) < recipe.output_amount
    {
        return Outcome::Idle(IdleReason::NoOutputSpace);
    }
    if machine.energy < operation.usage {
        return Outcome::Idle(IdleReason::NoEnergy);
    }
    machine.energy -= operation.usage;
    machine.progress += 1;
    acid.spend(recipe.acid_per_tick);
    if machine.progress < operation.ticks {
        return Outcome::Working;
    }
    machine.progress = 0;
    input.count -= recipe.input.max(1);
    // The room check above guarantees the whole batch fits; a table row naming
    // a chemical the port does not know is a generator bug, not input.
    insert_chemical(
        slurry,
        recipe.output,
        recipe.output_amount,
        DISSOLUTION_TANK,
    )
    .expect("a generated dissolution row names a registered slurry");
    Outcome::Produced
}

/// The acid the chamber pulls from a neighbour for the tick it is about to run:
/// the chemical to ask for and how much, or `None` when its own tank already
/// holds a tick's worth or holds a chemical the recipe does not take.
///
/// Exactly one tick's worth is asked for, so a neighbour loses the acid the
/// operation spends and no more: the chamber's tank is a buffer, not a
/// reservoir (the gas machines' recorded deviation: the port ships no pipes).
#[must_use]
pub fn dissolve_pull(recipe: &DissolveRecipe, acid: &Tank) -> Option<(&'static str, u32)> {
    if acid.held().is_some_and(|(held, _)| held != SULFURIC_ACID) {
        return None;
    }
    let want = recipe.acid_per_tick.saturating_sub(acid.amount);
    (want > 0).then_some((SULFURIC_ACID, want))
}

// ---------------------------------------------------------------------------
// The basic factories: the same operations, run side by side
// ---------------------------------------------------------------------------

/// One process of a factory: its own progress in ticks, its own input slot and
/// its own output slot (`FactoryInputInventorySlot`/`OutputInventorySlot` at
/// `TileEntityItemToItemFactory`).
///
/// The energy buffer and (for a gas factory) the tank are **not** here: one
/// factory has one of each, and every process draws from it.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct FactoryProcess {
    /// Ticks of this process's operation in progress, `0..operation.ticks`.
    pub progress: i32,
    /// The items in this process's input slot.
    pub input: ItemAmount,
    /// The stack in this process's output slot.
    pub output: ItemAmount,
}

/// How many operations one process completes in a tick
/// (`MekanismUtils.getOperationsPerTick`).
///
/// Upstream keeps `operationsPerTick` on the factory (default 1) and raises it
/// only for a speed-upgraded machine whose operation takes less than a tick, so
/// that it still finishes in one tick. `ticks_tenths` is the effective operation
/// length × 10 (2,000 at the unupgraded 200 ticks) and `default_operations` the
/// process's baseline (1); tenths are enough for every multiplier this port
/// could carry, and it carries no upgrade system, so the answer is always the
/// baseline.
#[must_use]
pub const fn operations_per_tick(ticks_tenths: i32, default_operations: i32) -> i32 {
    if ticks_tenths >= 10 {
        return default_operations;
    }
    let divisor = if ticks_tenths < 1 { 1 } else { ticks_tenths };
    let operations = default_operations * 10 / divisor;
    if operations < 1 { 1 } else { operations }
}

/// One tick of an item factory — smelting, enriching or crushing.
///
/// Every process runs the shared operation over **one** energy buffer: a factory
/// with three working processes spends three times what one machine would
/// (`energy_per_tick × processes`), and a process with no energy of its own left
/// in the buffer holds its progress while the ones before it keep working.
/// `limits[i]` is process `i`'s output stack size, `operations` is
/// [`operations_per_tick`]'s answer.
///
/// # Panics
/// When `limits` and `processes` differ in length: one stack limit per process
/// is the caller's invariant.
pub fn process_factory_tick(
    energy: &mut i32,
    processes: &mut [FactoryProcess],
    operation: Operation,
    limits: &[u32],
    operations: i32,
    lookup: &dyn Fn(&str) -> Option<MachineRecipe>,
) -> Vec<Outcome> {
    assert_eq!(
        limits.len(),
        processes.len(),
        "one stack limit per factory process"
    );
    let mut outcomes = Vec::with_capacity(processes.len());
    for (process, limit) in processes.iter_mut().zip(limits) {
        let mut outcome = Outcome::Idle(IdleReason::NoInput);
        for _ in 0..operations.max(1) {
            let mut machine = Machine {
                energy: *energy,
                progress: process.progress,
            };
            outcome = process_tick(
                &mut machine,
                operation,
                &mut process.input,
                &mut process.output,
                *limit,
                lookup,
            );
            *energy = machine.energy;
            process.progress = machine.progress;
            if !matches!(outcome, Outcome::Produced) {
                break;
            }
        }
        outcomes.push(outcome);
    }
    outcomes
}

/// One tick of a gas factory — the Basic Injecting and Basic Purifying
/// factories: the item factory's rules over one energy buffer **and** one tank,
/// so every working process draws its own tick's gas
/// (`3 × per_tick` for three processes) and a foreign or short gas holds them
/// all.
///
/// # Panics
/// When `limits` and `processes` differ in length, as
/// [`process_factory_tick`] does.
pub fn process_gas_factory_tick(
    energy: &mut i32,
    tank: &mut Tank,
    processes: &mut [FactoryProcess],
    operation: Operation,
    limits: &[u32],
    operations: i32,
    lookup: &dyn Fn(&str) -> Option<GasRecipe>,
) -> Vec<Outcome> {
    assert_eq!(
        limits.len(),
        processes.len(),
        "one stack limit per factory process"
    );
    let mut outcomes = Vec::with_capacity(processes.len());
    for (process, limit) in processes.iter_mut().zip(limits) {
        let mut outcome = Outcome::Idle(IdleReason::NoInput);
        for _ in 0..operations.max(1) {
            let mut machine = Machine {
                energy: *energy,
                progress: process.progress,
            };
            outcome = process_gas_tick(
                &mut machine,
                operation,
                tank,
                &mut process.input,
                &mut process.output,
                *limit,
                lookup,
            );
            *energy = machine.energy;
            process.progress = machine.progress;
            if !matches!(outcome, Outcome::Produced) {
                break;
            }
        }
        outcomes.push(outcome);
    }
    outcomes
}

/// How much gas a gas factory's processes need for this tick: the sum of the
/// per-tick amounts of the processes that can work, less what the shared tank
/// already holds.
///
/// `energy` is the shared buffer's amount and `operation` the process's cost,
/// so only the processes this tick can actually pay for are counted, in the same
/// order [`process_gas_factory_tick`] runs them — a factory with no energy does
/// not draw a neighbour's gas. Answers `None` when no process can work (an empty
/// input, no recipe, a full output slot, too little energy), when the tank
/// already holds the tick's gas, and when it holds a chemical no working recipe
/// takes.
#[must_use]
pub fn factory_gas_want(
    energy: i32,
    operation: Operation,
    tank: &Tank,
    processes: &[FactoryProcess],
    limits: &[u32],
    lookup: &dyn Fn(&str) -> Option<GasRecipe>,
) -> Option<(&'static str, u32)> {
    let mut chosen: Option<&'static str> = None;
    let mut needed = 0u32;
    let mut affordable = energy;
    for (process, limit) in processes.iter().zip(limits) {
        if affordable < operation.usage {
            break;
        }
        if process.input.is_empty() {
            continue;
        }
        let Some(recipe) = lookup(&process.input.item) else {
            continue;
        };
        if room_for(&process.output, &recipe.output, *limit) < recipe.output.count {
            continue;
        }
        // One tank holds one chemical: the one it already holds when a recipe
        // takes it, otherwise the recipe's own first id.
        let chemical = match tank.held() {
            Some((held, _)) => match recipe.chemical.iter().find(|id| **id == held) {
                Some(id) => *id,
                None => continue,
            },
            None => match recipe.chemical.first() {
                Some(id) => *id,
                None => continue,
            },
        };
        if chosen.is_some_and(|already| already != chemical) {
            continue;
        }
        chosen = Some(chemical);
        needed = needed.saturating_add(recipe.per_tick);
        affordable -= operation.usage;
    }
    let chemical = chosen?;
    let have = match tank.held() {
        Some((held, amount)) if held == chemical => amount,
        _ => 0,
    };
    let want = needed.saturating_sub(have);
    (want > 0).then_some((chemical, want))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(item: &str, count: u32) -> ItemAmount {
        ItemAmount::new(item, count)
    }

    fn one(item: &str, count: u32) -> MachineRecipe {
        MachineRecipe {
            input: 1,
            output: stack(item, count),
        }
    }

    /// The Energized Smelter's own operation, so these tests read as the
    /// machine they check; `process_tick` is the shared core.
    fn smelt_tick(
        machine: &mut Machine,
        input: &mut ItemAmount,
        output: &mut ItemAmount,
        max_stack: u32,
        lookup: &dyn Fn(&str) -> Option<MachineRecipe>,
    ) -> Outcome {
        process_tick(machine, SMELTER_OPERATION, input, output, max_stack, lookup)
    }

    fn iron_ingot(_input: &str) -> Option<MachineRecipe> {
        Some(one("minecraft:iron_ingot", 1))
    }

    fn nothing(_input: &str) -> Option<MachineRecipe> {
        None
    }

    /// Upstream spends 50 FE each operating tick and finishes after 200 ticks,
    /// consuming one input item and producing the output there and then.
    #[test]
    fn one_operation_takes_two_hundred_ticks_and_ten_thousand_energy() {
        let mut smelter = Machine {
            energy: SMELTER_CAPACITY,
            progress: 0,
        };
        let mut input = stack("minecraft:iron_ore", 2);
        let mut output = ItemAmount::default();
        for _ in 0..(SMELTER_TICKS - 1) {
            assert_eq!(
                smelt_tick(&mut smelter, &mut input, &mut output, 64, &iron_ingot),
                Outcome::Working
            );
        }
        assert_eq!(smelter.progress, SMELTER_TICKS - 1);
        assert_eq!(
            smelter.energy,
            SMELTER_CAPACITY - (SMELTER_TICKS - 1) * SMELTER_USAGE
        );
        assert_eq!(
            input.count, 2,
            "the input is consumed when the operation finishes"
        );
        assert!(output.is_empty(), "no output before the last tick");

        assert_eq!(
            smelt_tick(&mut smelter, &mut input, &mut output, 64, &iron_ingot),
            Outcome::Produced
        );
        assert_eq!(smelter.progress, 0);
        assert_eq!(
            smelter.energy,
            SMELTER_CAPACITY - SMELTER_TICKS * SMELTER_USAGE
        );
        assert_eq!(input.count, 1);
        assert_eq!(output, stack("minecraft:iron_ingot", 1));
    }

    /// One operation costs exactly 10,000 FE, so a full buffer smelts two.
    #[test]
    fn a_full_buffer_smelts_exactly_two_operations() {
        let mut smelter = Machine {
            energy: SMELTER_CAPACITY,
            progress: 0,
        };
        let mut input = stack("minecraft:iron_ore", 3);
        let mut output = ItemAmount::default();
        for _ in 0..(SMELTER_TICKS * 3) {
            let _ = smelt_tick(&mut smelter, &mut input, &mut output, 64, &iron_ingot);
        }
        assert_eq!(output.count, 2);
        assert_eq!(input.count, 1);
        assert_eq!(smelter.energy, 0);
        assert_eq!(smelter.progress, 0);
    }

    /// No energy holds the progress (upstream only resets on a bad recipe).
    #[test]
    fn no_energy_holds_the_progress_and_spends_nothing() {
        let mut smelter = Machine {
            energy: SMELTER_USAGE - 1,
            progress: 137,
        };
        let mut input = stack("minecraft:iron_ore", 1);
        let mut output = ItemAmount::default();
        assert_eq!(
            smelt_tick(&mut smelter, &mut input, &mut output, 64, &iron_ingot),
            Outcome::Idle(IdleReason::NoEnergy)
        );
        assert_eq!(smelter.progress, 137);
        assert_eq!(smelter.energy, SMELTER_USAGE - 1);
        assert_eq!(input.count, 1);
        assert!(output.is_empty());
    }

    #[test]
    fn an_empty_input_resets_the_progress() {
        let mut smelter = Machine {
            energy: SMELTER_CAPACITY,
            progress: 137,
        };
        let mut input = ItemAmount::default();
        let mut output = ItemAmount::default();
        assert_eq!(
            smelt_tick(&mut smelter, &mut input, &mut output, 64, &iron_ingot),
            Outcome::Idle(IdleReason::NoInput)
        );
        assert_eq!(smelter.progress, 0);
    }

    #[test]
    fn an_item_without_a_recipe_resets_the_progress() {
        let mut smelter = Machine {
            energy: SMELTER_CAPACITY,
            progress: 137,
        };
        let mut input = stack("minecraft:diamond", 1);
        let mut output = ItemAmount::default();
        assert_eq!(
            smelt_tick(&mut smelter, &mut input, &mut output, 64, &nothing),
            Outcome::Idle(IdleReason::NoRecipe)
        );
        assert_eq!(smelter.progress, 0);
        assert_eq!(smelter.energy, SMELTER_CAPACITY);
    }

    #[test]
    fn a_full_output_holds_the_progress_and_spends_nothing() {
        let mut smelter = Machine {
            energy: SMELTER_CAPACITY,
            progress: 50,
        };
        let mut input = stack("minecraft:iron_ore", 1);
        let mut output = stack("minecraft:iron_ingot", 64);
        assert_eq!(
            smelt_tick(&mut smelter, &mut input, &mut output, 64, &iron_ingot),
            Outcome::Idle(IdleReason::NoOutputSpace)
        );
        assert_eq!(smelter.progress, 50);
        assert_eq!(smelter.energy, SMELTER_CAPACITY);
        assert_eq!(output.count, 64);
    }

    #[test]
    fn an_output_of_another_item_leaves_no_room() {
        let mut smelter = Machine {
            energy: SMELTER_CAPACITY,
            progress: 0,
        };
        let mut input = stack("minecraft:iron_ore", 1);
        let mut output = stack("minecraft:diamond", 1);
        assert_eq!(
            smelt_tick(&mut smelter, &mut input, &mut output, 64, &iron_ingot),
            Outcome::Idle(IdleReason::NoOutputSpace)
        );
        assert_eq!(output, stack("minecraft:diamond", 1));
    }

    /// The output is a stack: the recipe's count lands in the slot (iron ore
    /// smelts one, but the path supports any count the recipe names).
    #[test]
    fn a_recipe_with_a_count_of_two_produces_two_items() {
        fn two(_input: &str) -> Option<MachineRecipe> {
            Some(MachineRecipe {
                input: 1,
                output: stack("minecraft:iron_ingot", 2),
            })
        }
        let mut smelter = Machine {
            energy: SMELTER_CAPACITY,
            progress: SMELTER_TICKS - 1,
        };
        let mut input = stack("minecraft:iron_ore", 1);
        let mut output = ItemAmount::default();
        assert_eq!(
            smelt_tick(&mut smelter, &mut input, &mut output, 64, &two),
            Outcome::Produced
        );
        assert_eq!(output, stack("minecraft:iron_ingot", 2));
    }

    #[test]
    fn a_second_operation_stacks_onto_the_same_output() {
        let mut smelter = Machine {
            energy: SMELTER_CAPACITY,
            progress: SMELTER_TICKS - 1,
        };
        let mut input = stack("minecraft:iron_ore", 1);
        let mut output = stack("minecraft:iron_ingot", 5);
        assert_eq!(
            smelt_tick(&mut smelter, &mut input, &mut output, 64, &iron_ingot),
            Outcome::Produced
        );
        assert_eq!(output, stack("minecraft:iron_ingot", 6));
    }

    #[test]
    fn a_two_count_output_that_would_overflow_is_refused() {
        fn two(_input: &str) -> Option<MachineRecipe> {
            Some(MachineRecipe {
                input: 1,
                output: stack("minecraft:iron_ingot", 2),
            })
        }
        let mut smelter = Machine {
            energy: SMELTER_CAPACITY,
            progress: 0,
        };
        let mut input = stack("minecraft:iron_ore", 1);
        let mut output = stack("minecraft:iron_ingot", 63);
        assert_eq!(
            smelt_tick(&mut smelter, &mut input, &mut output, 64, &two),
            Outcome::Idle(IdleReason::NoOutputSpace)
        );
        assert_eq!(output.count, 63);
    }

    #[test]
    fn the_smelting_table_has_the_vanilla_recipes() {
        assert_eq!(
            smelting::lookup("minecraft:iron_ore"),
            Some(one("minecraft:iron_ingot", 1))
        );
        assert_eq!(
            smelting::lookup("minecraft:deepslate_iron_ore"),
            Some(one("minecraft:iron_ingot", 1))
        );
        assert_eq!(
            smelting::lookup("minecraft:sand"),
            Some(one("minecraft:glass", 1))
        );
        assert_eq!(smelting::lookup("minecraft:diamond"), None);
        assert_eq!(smelting::lookup("mekanism:does_not_exist"), None);
        // A tag ingredient is expanded: sand smelts through
        // `#minecraft:smelts_to_glass`, and so do its sibling members.
        assert_eq!(
            smelting::lookup("minecraft:red_sand"),
            Some(one("minecraft:glass", 1))
        );
        assert!(smelting::len() >= 73, "the table holds every expanded row");
    }

    #[test]
    fn energy_is_clamped_to_the_buffer() {
        assert_eq!(clamp_energy(-5, SMELTER_CAPACITY), 0);
        assert_eq!(
            clamp_energy(SMELTER_CAPACITY + 1, SMELTER_CAPACITY),
            SMELTER_CAPACITY
        );
        assert_eq!(clamp_energy(7, SMELTER_CAPACITY), 7);
        assert_eq!(clamp_energy(9, 0), 0);
    }

    #[test]
    fn the_cube_takes_at_most_its_rate_per_call() {
        let (accepted, after) = energy_after_insert(0, CUBE_CAPACITY, CUBE_RATE, 10_000);
        assert_eq!(accepted, CUBE_RATE);
        assert_eq!(after, CUBE_RATE);
    }

    #[test]
    fn the_cube_takes_only_its_remaining_room() {
        let (accepted, after) =
            energy_after_insert(CUBE_CAPACITY - 100, CUBE_CAPACITY, CUBE_RATE, 9_000);
        assert_eq!(accepted, 100);
        assert_eq!(after, CUBE_CAPACITY);
    }

    #[test]
    fn a_full_cube_takes_nothing() {
        let (accepted, after) = energy_after_insert(CUBE_CAPACITY, CUBE_CAPACITY, CUBE_RATE, 1);
        assert_eq!(accepted, 0);
        assert_eq!(after, CUBE_CAPACITY);
    }

    #[test]
    fn the_cube_gives_at_most_its_rate_and_what_it_holds() {
        assert_eq!(
            energy_after_extract(9_000, CUBE_RATE, 8_000),
            (CUBE_RATE, 5_000)
        );
        assert_eq!(energy_after_extract(100, CUBE_RATE, 8_000), (100, 0));
        assert_eq!(energy_after_extract(100, CUBE_RATE, 0), (0, 100));
    }

    fn endpoint(
        face: u8,
        can_insert: bool,
        can_extract: bool,
        space: i32,
        stored: i32,
    ) -> Endpoint {
        Endpoint {
            face,
            can_insert,
            can_extract,
            space,
            stored,
        }
    }

    /// The cable fills itself from a provider and empties into a sink, within
    /// one 8,000 FE buffer.
    #[test]
    fn a_cable_pulls_from_a_provider_and_pushes_to_a_sink() {
        let neighbours = [
            endpoint(0, true, false, 4_000, 0),
            endpoint(1, false, true, 0, 10_000),
        ];
        let plan = cable_plan(0, CABLE_CAPACITY, &neighbours);
        assert_eq!(plan.pulled(), 8_000);
        assert_eq!(plan.pushed(), 4_000);
        assert_eq!(plan.energy_after, 4_000);
        assert_eq!(
            plan.moves,
            vec![
                Move::Pull {
                    face: 1,
                    amount: 8_000
                },
                Move::Push {
                    face: 0,
                    amount: 4_000
                },
            ]
        );
    }

    #[test]
    fn a_cable_never_moves_more_than_its_buffer() {
        let neighbours = [
            endpoint(0, true, false, 100_000, 0),
            endpoint(1, true, false, 100_000, 0),
            endpoint(2, true, false, 100_000, 0),
            endpoint(3, false, true, 0, 100_000),
            endpoint(4, false, true, 0, 100_000),
        ];
        let plan = cable_plan(0, CABLE_CAPACITY, &neighbours);
        assert_eq!(plan.pulled(), CABLE_CAPACITY);
        assert_eq!(plan.pushed(), CABLE_CAPACITY);
        assert_eq!(plan.energy_after, 0);
    }

    #[test]
    fn a_cable_pushes_only_what_it_holds_and_no_more() {
        let neighbours = [endpoint(0, true, false, 8_000, 0)];
        let plan = cable_plan(1_000, CABLE_CAPACITY, &neighbours);
        assert_eq!(plan.pulled(), 0);
        assert_eq!(plan.pushed(), 1_000);
        assert_eq!(plan.energy_after, 0);
    }

    #[test]
    fn a_cable_skips_faces_that_cannot_move() {
        let neighbours = [
            endpoint(0, true, false, 0, 0),
            endpoint(1, false, true, 0, 0),
            endpoint(2, false, false, 4_000, 4_000),
        ];
        let plan = cable_plan(500, CABLE_CAPACITY, &neighbours);
        assert!(plan.moves.is_empty());
        assert_eq!(plan.energy_after, 500);
    }

    #[test]
    fn a_cable_with_no_neighbours_keeps_its_energy() {
        let plan = cable_plan(3_000, CABLE_CAPACITY, &[]);
        assert!(plan.moves.is_empty());
        assert_eq!(plan.energy_after, 3_000);
    }

    /// No energy is created or destroyed: the cable's final amount is its
    /// initial one plus what it pulled minus what it pushed.
    #[test]
    fn the_cable_plan_conserves_energy() {
        let neighbours = [
            endpoint(0, true, false, 5_000, 0),
            endpoint(1, false, true, 0, 6_000),
            endpoint(2, false, true, 0, 100),
            endpoint(3, true, false, 1, 0),
        ];
        let plan = cable_plan(7_000, CABLE_CAPACITY, &neighbours);
        assert_eq!(plan.energy_after, 7_000 + plan.pulled() - plan.pushed());
        assert!(plan.energy_after >= 0);
        assert!(plan.energy_after <= CABLE_CAPACITY);
        assert!(plan.pushed() <= 7_000 + plan.pulled());
    }

    /// The bug the e2e caught: a pull that moves less than planned must not
    /// let the push move the planned amount.
    #[test]
    fn a_move_is_capped_by_what_the_cable_really_holds() {
        let push = Move::Push {
            face: 4,
            amount: 8_000,
        };
        assert_eq!(cable_move_limit(push, 4_000, CABLE_CAPACITY), 4_000);
        assert_eq!(cable_move_limit(push, 0, CABLE_CAPACITY), 0);
        let pull = Move::Pull {
            face: 5,
            amount: 8_000,
        };
        assert_eq!(cable_move_limit(pull, 6_000, CABLE_CAPACITY), 2_000);
        assert_eq!(cable_move_limit(pull, 0, CABLE_CAPACITY), 8_000);
    }

    #[test]
    fn the_cable_pull_is_capped_by_its_room() {
        let neighbours = [endpoint(0, false, true, 0, 9_000)];
        let plan = cable_plan(7_500, CABLE_CAPACITY, &neighbours);
        assert_eq!(plan.pulled(), 500);
        assert_eq!(plan.energy_after, CABLE_CAPACITY);
    }

    /// `#c:ores/iron` is expanded to the vanilla iron ores, and the jar's
    /// `processing/iron/dust/from_ore.json` makes two dust of one ore.
    #[test]
    fn the_enriching_table_turns_an_ore_into_two_dust() {
        for (ore, dust) in [
            ("minecraft:iron_ore", "mekanism:dust_iron"),
            ("minecraft:deepslate_iron_ore", "mekanism:dust_iron"),
            ("minecraft:gold_ore", "mekanism:dust_gold"),
            ("minecraft:deepslate_gold_ore", "mekanism:dust_gold"),
            ("minecraft:copper_ore", "mekanism:dust_copper"),
            ("minecraft:deepslate_copper_ore", "mekanism:dust_copper"),
        ] {
            assert_eq!(enriching::lookup(ore), Some(one(dust, 2)), "{ore}");
        }
    }

    /// The raw forms: three raw ore make four dust, a raw block twelve.
    #[test]
    fn the_enriching_table_expands_raw_ore_and_raw_blocks() {
        for (raw, dust) in [
            ("minecraft:raw_iron", "mekanism:dust_iron"),
            ("minecraft:raw_gold", "mekanism:dust_gold"),
            ("minecraft:raw_copper", "mekanism:dust_copper"),
        ] {
            assert_eq!(
                enriching::lookup(raw),
                Some(MachineRecipe {
                    input: 3,
                    output: stack(dust, 4),
                }),
                "{raw}"
            );
            let block = format!("{raw}_block");
            assert_eq!(
                enriching::lookup(&block),
                Some(MachineRecipe {
                    input: 1,
                    output: stack(dust, 12),
                }),
                "{block}"
            );
        }
    }

    /// The jar's ore recipes for the vanilla materials that do not need a
    /// Mekanism item: gems, lapis, quartz, redstone and ancient debris.
    #[test]
    fn the_enriching_table_holds_the_vanilla_gems_and_ores() {
        for (ore, item, count) in [
            ("minecraft:coal_ore", "minecraft:coal", 2),
            ("minecraft:deepslate_coal_ore", "minecraft:coal", 2),
            ("minecraft:diamond_ore", "minecraft:diamond", 2),
            ("minecraft:deepslate_diamond_ore", "minecraft:diamond", 2),
            ("minecraft:emerald_ore", "minecraft:emerald", 2),
            ("minecraft:deepslate_emerald_ore", "minecraft:emerald", 2),
            ("minecraft:lapis_ore", "minecraft:lapis_lazuli", 12),
            (
                "minecraft:deepslate_lapis_ore",
                "minecraft:lapis_lazuli",
                12,
            ),
            ("minecraft:nether_quartz_ore", "minecraft:quartz", 2),
            ("minecraft:redstone_ore", "minecraft:redstone", 12),
            ("minecraft:deepslate_redstone_ore", "minecraft:redstone", 12),
            ("minecraft:ancient_debris", "minecraft:netherite_scrap", 2),
        ] {
            assert_eq!(enriching::lookup(ore), Some(one(item, count)), "{ore}");
        }
    }

    #[test]
    fn the_enriching_table_has_no_recipe_for_a_gem_or_a_food() {
        assert_eq!(enriching::lookup("minecraft:diamond"), None);
        assert_eq!(enriching::lookup("minecraft:iron_ingot"), None);
        assert_eq!(enriching::lookup("minecraft:bread"), None);
        assert_eq!(enriching::lookup("mekanism:does_not_exist"), None);
        // The jar's 322 enriching recipes over the items this port registers.
        assert!(
            enriching::len() >= 100,
            "the table holds the resolvable rows"
        );
    }

    /// Upstream's Enrichment Chamber: `UsageConfig.enrichmentChamber` 50 FE/t,
    /// `StorageConfig.enrichmentChamber` 20,000 FE, `BASE_TICKS_REQUIRED` 200.
    #[test]
    fn the_chamber_uses_the_upstream_numbers() {
        assert_eq!(CHAMBER_TICKS, 200);
        assert_eq!(CHAMBER_USAGE, 50);
        assert_eq!(CHAMBER_CAPACITY, 20_000);
        assert_eq!(
            CHAMBER_OPERATION,
            Operation {
                ticks: CHAMBER_TICKS,
                usage: CHAMBER_USAGE
            }
        );
    }

    /// One ore, 200 ticks, 2 dust, 10,000 FE - the same shape as the smelter's
    /// operation with the enrichment recipe table.
    #[test]
    fn an_enriching_operation_takes_two_hundred_ticks_and_two_dust() {
        let mut machine = Machine {
            energy: CHAMBER_CAPACITY,
            progress: 0,
        };
        let mut input = stack("minecraft:iron_ore", 2);
        let mut output = ItemAmount::default();
        for _ in 0..(CHAMBER_TICKS - 1) {
            assert_eq!(
                process_tick(
                    &mut machine,
                    CHAMBER_OPERATION,
                    &mut input,
                    &mut output,
                    64,
                    &enriching::lookup
                ),
                Outcome::Working
            );
        }
        assert!(output.is_empty(), "no dust before the last tick");
        assert_eq!(input.count, 2, "the ore is consumed at the finish");
        assert_eq!(
            process_tick(
                &mut machine,
                CHAMBER_OPERATION,
                &mut input,
                &mut output,
                64,
                &enriching::lookup
            ),
            Outcome::Produced
        );
        assert_eq!(output, stack("mekanism:dust_iron", 2));
        assert_eq!(input.count, 1);
        assert_eq!(
            machine.energy,
            CHAMBER_CAPACITY - CHAMBER_TICKS * CHAMBER_USAGE
        );
    }

    /// A raw-ore recipe takes its whole input count per operation.
    #[test]
    fn an_enriching_operation_consumes_three_raw_ore() {
        let mut machine = Machine {
            energy: CHAMBER_CAPACITY,
            progress: 0,
        };
        let mut input = stack("minecraft:raw_iron", 3);
        let mut output = ItemAmount::default();
        for _ in 0..CHAMBER_TICKS {
            let _ = process_tick(
                &mut machine,
                CHAMBER_OPERATION,
                &mut input,
                &mut output,
                64,
                &enriching::lookup,
            );
        }
        assert_eq!(output, stack("mekanism:dust_iron", 4));
        assert_eq!(input.count, 0);
    }

    /// Fewer items than the recipe needs is upstream's `NOT_ENOUGH_INPUT`: the
    /// progress resets and no energy is spent.
    #[test]
    fn too_few_raw_ore_resets_the_progress_and_spends_nothing() {
        let mut machine = Machine {
            energy: CHAMBER_CAPACITY,
            progress: 137,
        };
        let mut input = stack("minecraft:raw_iron", 2);
        let mut output = ItemAmount::default();
        assert_eq!(
            process_tick(
                &mut machine,
                CHAMBER_OPERATION,
                &mut input,
                &mut output,
                64,
                &enriching::lookup
            ),
            Outcome::Idle(IdleReason::NotEnoughInput)
        );
        assert_eq!(machine.progress, 0);
        assert_eq!(machine.energy, CHAMBER_CAPACITY);
        assert_eq!(input.count, 2);
        assert!(output.is_empty());
    }

    /// The jar's own `minecraft:smelting` recipes turn Mekanism's dusts into
    /// ingots, so the Energized Smelter completes the chain.
    #[test]
    fn the_smelter_smelts_mekanism_dust_into_its_ingot() {
        assert_eq!(
            smelting::lookup("mekanism:dust_iron"),
            Some(one("minecraft:iron_ingot", 1))
        );
        assert_eq!(
            smelting::lookup("mekanism:dust_gold"),
            Some(one("minecraft:gold_ingot", 1))
        );
        assert_eq!(
            smelting::lookup("mekanism:dust_copper"),
            Some(one("minecraft:copper_ingot", 1))
        );
    }

    /// Slice 12's four metals run the whole chain the jar declares, one row per
    /// family: `processing/<m>/dirty_dust/from_clump.json` (`mekanism:crushing`,
    /// one clump into one dirty dust), `processing/<m>/dust/from_dirty_dust.json`
    /// (`mekanism:enriching`, one dirty dust into one dust) and
    /// `processing/<m>/ingot/from_dust_smelting.json` (a `minecraft:smelting`
    /// recipe the Energized Smelter takes, one dust into one ingot). The four
    /// rows are the tool's `c:clumps/<m>` and `c:dirty_dusts/<m>` expansions; the
    /// row counts pin the generators' output.
    #[test]
    fn the_ore_chain_crushes_every_clump_into_its_dirty_dust() {
        for material in ["lead", "osmium", "tin", "uranium"] {
            let clump = format!("mekanism:clump_{material}");
            let dirty = format!("mekanism:dirty_dust_{material}");
            assert_eq!(crushing::lookup(&clump), Some(one(&dirty, 1)), "{clump}");
        }
        assert_eq!(
            crushing::len(),
            156,
            "the rows the generator keeps from the jar's 313 crushing recipes \
             (152 through slice 12, plus slice 13's four c:ingots rows)"
        );
    }

    #[test]
    fn the_ore_chain_enriches_every_dirty_dust_into_its_dust() {
        for material in ["lead", "osmium", "tin", "uranium"] {
            let dirty = format!("mekanism:dirty_dust_{material}");
            let dust = format!("mekanism:dust_{material}");
            assert_eq!(enriching::lookup(&dirty), Some(one(&dust, 1)), "{dirty}");
        }
        assert_eq!(
            enriching::len(),
            136,
            "the rows the generator keeps from the jar's 322 enriching recipes"
        );
    }

    /// Slice 13: the jar's `processing/<material>/dust/from_ingot.json` rows, the
    /// `c:ingots/<m>` tag expanded to the four ingots slice 12 registered — an
    /// ingot crushes back into its dust the way copper, gold and iron already do.
    #[test]
    fn the_crusher_turns_every_registered_ingot_into_its_dust() {
        for material in ["lead", "osmium", "tin", "uranium"] {
            let ingot = format!("mekanism:ingot_{material}");
            let dust = format!("mekanism:dust_{material}");
            assert_eq!(crushing::lookup(&ingot), Some(one(&dust, 1)), "{ingot}");
        }
        // The three vanilla-metal `from_ingot` rows the table has carried since
        // slice 3: the same recipe family, so they are asserted beside them.
        for (ingot, dust) in [
            ("minecraft:iron_ingot", "mekanism:dust_iron"),
            ("minecraft:gold_ingot", "mekanism:dust_gold"),
            ("minecraft:copper_ingot", "mekanism:dust_copper"),
        ] {
            assert_eq!(crushing::lookup(ingot), Some(one(dust, 1)), "{ingot}");
        }
    }

    /// Slice 13's other half: the Energized Smelter's table is *the whole* 26.3
    /// vanilla smelting table plus the jar's own dust rows, and every row is
    /// reachable through the binary search the machine calls — the unit half of
    /// e2e phase T, which smelts exactly these row counts in real smelters.
    #[test]
    fn the_smelting_table_covers_every_carried_smelting_row() {
        let rows = smelting::rows();
        for &(input, output, count) in rows {
            assert_eq!(smelting::lookup(input), Some(one(output, count)), "{input}");
        }
        assert_eq!(
            rows.len(),
            163,
            "every row the 26.3 data's smelting recipes expand to"
        );
        assert_eq!(
            rows.len() + smelting::mekanism_rows(),
            170,
            "163 vanilla rows plus the jar's 7 dust -> ingot rows"
        );
    }

    #[test]
    fn the_ore_chain_smelts_every_dust_into_its_ingot() {
        for material in ["lead", "osmium", "tin", "uranium"] {
            let dust = format!("mekanism:dust_{material}");
            let ingot = format!("mekanism:ingot_{material}");
            assert_eq!(smelting::lookup(&dust), Some(one(&ingot, 1)), "{dust}");
        }
        assert_eq!(
            smelting::mekanism_rows(),
            7,
            "three vanilla-metal dusts and slice 12's four"
        );
    }

    /// `crushing/cobblestone_to_gravel.json`, whose ingredient is the
    /// `c:cobblestones/normal` tag.
    #[test]
    fn the_crushing_table_turns_cobblestone_into_gravel() {
        assert_eq!(
            crushing::lookup("minecraft:cobblestone"),
            Some(one("minecraft:gravel", 1))
        );
        assert_eq!(
            crushing::lookup("minecraft:gravel"),
            Some(one("minecraft:sand", 1))
        );
    }

    /// `crushing/wool_to_string.json`: one wool of any colour, four string.
    #[test]
    fn the_crushing_table_turns_every_wool_into_four_string() {
        for wool in [
            "minecraft:white_wool",
            "minecraft:red_wool",
            "minecraft:light_blue_wool",
            "minecraft:black_wool",
        ] {
            assert_eq!(
                crushing::lookup(wool),
                Some(one("minecraft:string", 4)),
                "{wool}"
            );
        }
    }

    /// `processing/<material>/dust/from_ingot.json`: the crusher is where the
    /// port's three dusts can also be made from ingots.
    #[test]
    fn the_crushing_table_grinds_the_three_ingots_into_their_dust() {
        for (ingot, dust) in [
            ("minecraft:iron_ingot", "mekanism:dust_iron"),
            ("minecraft:gold_ingot", "mekanism:dust_gold"),
            ("minecraft:copper_ingot", "mekanism:dust_copper"),
        ] {
            assert_eq!(crushing::lookup(ingot), Some(one(dust, 1)), "{ingot}");
        }
    }

    /// The jar's dewax family: a waxed block crushes back to the plain one.
    #[test]
    fn the_crushing_table_holds_the_waxed_copper_conversions() {
        for (waxed, plain) in [
            ("minecraft:waxed_copper_block", "minecraft:copper_block"),
            ("minecraft:waxed_exposed_copper", "minecraft:exposed_copper"),
            (
                "minecraft:waxed_oxidized_cut_copper_stairs",
                "minecraft:oxidized_cut_copper_stairs",
            ),
        ] {
            assert_eq!(crushing::lookup(waxed), Some(one(plain, 1)), "{waxed}");
        }
    }

    /// Some recipes take more than one input item: the jar's
    /// `prismarine/brick_slabs_to_tile.json` takes two slabs and makes nine
    /// shards.
    #[test]
    fn the_crushing_table_consumes_the_input_count_the_jar_declares() {
        assert_eq!(
            crushing::lookup("minecraft:prismarine_brick_slab"),
            Some(MachineRecipe {
                input: 2,
                output: stack("minecraft:prismarine_shard", 9),
            })
        );
        assert_eq!(
            crushing::lookup("minecraft:bone"),
            Some(one("minecraft:bone_meal", 6)),
            "bone is one input and six output"
        );
    }

    /// A row whose output the port does not register is left out: crushing a
    /// diamond makes `mekanism:dust_diamond` upstream, which this port has no
    /// item for, so the table must not answer a recipe the machine could not
    /// finish.
    #[test]
    fn the_crushing_table_has_no_recipe_for_what_the_port_cannot_make() {
        assert_eq!(crushing::lookup("minecraft:diamond"), None);
        assert_eq!(crushing::lookup("minecraft:emerald"), None);
        assert_eq!(crushing::lookup("minecraft:charcoal"), None);
        assert_eq!(crushing::lookup("minecraft:dirt"), None);
        assert_eq!(crushing::lookup("mekanism:does_not_exist"), None);
        // The jar's `mekanism:crushing` recipes over the items this slice has.
        assert!(
            crushing::len() >= 130,
            "the table holds the resolvable rows"
        );
    }

    /// Upstream's Crusher: `UsageConfig.crusher` 50 FE/t,
    /// `StorageConfig.crusher` 20,000 FE, `BASE_TICKS_REQUIRED` 200.
    #[test]
    fn the_crusher_uses_the_upstream_numbers() {
        assert_eq!(CRUSHER_TICKS, 200);
        assert_eq!(CRUSHER_USAGE, 50);
        assert_eq!(CRUSHER_CAPACITY, 20_000);
        assert_eq!(
            CRUSHER_OPERATION,
            Operation {
                ticks: CRUSHER_TICKS,
                usage: CRUSHER_USAGE
            }
        );
    }

    /// One cobblestone in, one gravel out, and the operation costs exactly
    /// 10,000 FE (200 ticks x 50 FE/t).
    #[test]
    fn a_crushing_operation_takes_two_hundred_ticks_and_ten_thousand_energy() {
        let mut crusher = Machine {
            energy: 10_000,
            progress: 0,
        };
        let mut input = stack("minecraft:cobblestone", 2);
        let mut output = ItemAmount::default();
        for _ in 0..(CRUSHER_TICKS - 1) {
            assert_eq!(
                process_tick(
                    &mut crusher,
                    CRUSHER_OPERATION,
                    &mut input,
                    &mut output,
                    64,
                    &crushing::lookup
                ),
                Outcome::Working
            );
        }
        assert!(output.is_empty(), "no gravel before the last tick");
        assert_eq!(input.count, 2, "the cobblestone is consumed at the finish");
        assert_eq!(
            process_tick(
                &mut crusher,
                CRUSHER_OPERATION,
                &mut input,
                &mut output,
                64,
                &crushing::lookup
            ),
            Outcome::Produced
        );
        assert_eq!(crusher.energy, 0, "one operation costs exactly 10,000 FE");
        assert_eq!(input.count, 1);
        assert_eq!(output, stack("minecraft:gravel", 1));
    }

    #[test]
    fn a_readout_needs_an_operator_reach_or_the_same_level() {
        assert!(may_inspect(true, false, f64::MAX), "an operator");
        assert!(may_inspect(false, true, 3.0), "a player in reach");
        assert!(!may_inspect(false, true, 8.5), "out of reach");
        assert!(!may_inspect(false, false, 1.0), "another level");
    }
}

#[cfg(test)]
mod chemical_tests {
    use super::*;

    /// A tank takes a chemical Mekanism registers, up to its capacity: 64 buckets
    /// for the Basic Chemical Tank (`ChemicalTankTier.BASIC`).
    #[test]
    fn a_tank_takes_a_known_chemical_up_to_its_capacity() {
        let mut tank = Tank::default();
        assert_eq!(
            insert_chemical(&mut tank, "mekanism:hydrogen", 4_000, TANK_CAPACITY),
            Ok(4_000),
            "a fresh tank takes what is put in it"
        );
        assert_eq!(
            tank.held(),
            Some(("mekanism:hydrogen", 4_000)),
            "and holds it under the chemical's own id"
        );
        assert_eq!(
            insert_chemical(&mut tank, "mekanism:hydrogen", 999_999, TANK_CAPACITY),
            Ok(TANK_CAPACITY - 4_000),
            "a second insert is capped by the room left, not by the amount asked"
        );
        assert_eq!(tank.amount, TANK_CAPACITY, "the tank is full");
        assert_eq!(
            insert_chemical(&mut tank, "mekanism:hydrogen", 1, TANK_CAPACITY),
            Ok(0),
            "a full tank takes nothing more"
        );
        assert_eq!(
            insert_chemical(&mut tank, "mekanism:hydrogen", 0, TANK_CAPACITY),
            Ok(0),
            "and an empty offer moves nothing"
        );
    }

    /// An id Mekanism does not register is refused **by name**, and the tank is
    /// untouched: the id is the mod's own registry, not a free string.
    #[test]
    fn an_unknown_chemical_is_refused_by_name() {
        let mut tank = Tank::new("mekanism:hydrogen", 4_000);
        assert_eq!(
            insert_chemical(&mut tank, "mekanism:unobtainium", 500, TANK_CAPACITY),
            Err(Refusal::UnknownChemical("mekanism:unobtainium".to_owned())),
            "the refusal names the id that was asked for"
        );
        assert_eq!(
            tank,
            Tank::new("mekanism:hydrogen", 4_000),
            "a refused insert leaves the tank exactly as it was"
        );
        assert_eq!(
            known_chemical("mekanism:empty"),
            None,
            "the empty chemical is not a chemical a tank takes"
        );
        assert_eq!(
            known_chemical("minecraft:water"),
            None,
            "a vanilla id is not a chemical"
        );
        assert_eq!(
            known_chemical("mekanism:hydrogen"),
            Some("mekanism:hydrogen"),
            "a registered one is its own id"
        );
    }

    /// One tank holds one chemical: a second is refused whole, as
    /// `ChemicalTank.insert` tests the held stack first.
    #[test]
    fn a_tank_holding_one_chemical_refuses_another() {
        let mut tank = Tank::new("mekanism:hydrogen", 4_000);
        assert_eq!(
            insert_chemical(&mut tank, "mekanism:oxygen", 1_000, TANK_CAPACITY),
            Ok(0),
            "oxygen does not go into a tank of hydrogen"
        );
        assert_eq!(tank, Tank::new("mekanism:hydrogen", 4_000));
        assert_eq!(
            extract_chemical(&mut tank, "mekanism:oxygen", 1_000),
            0,
            "and no oxygen comes out of it either"
        );
        assert_eq!(tank, Tank::new("mekanism:hydrogen", 4_000));
    }

    /// A partial extract leaves the rest, and the tank empties to nothing rather
    /// than to a zero-amount chemical.
    #[test]
    fn extracting_part_leaves_the_rest_and_never_takes_another() {
        let mut tank = Tank::new("mekanism:hydrogen", 4_000);
        assert_eq!(
            extract_chemical(&mut tank, "mekanism:hydrogen", 250),
            250,
            "a gauge dropper takes its transfer rate"
        );
        assert_eq!(tank.held(), Some(("mekanism:hydrogen", 3_750)));
        assert_eq!(
            extract_chemical(&mut tank, "mekanism:hydrogen", 999_999),
            3_750,
            "a bigger ask takes only what is there"
        );
        assert_eq!(tank, Tank::default(), "an emptied tank holds nothing");
        assert_eq!(
            extract_chemical(&mut tank, "mekanism:hydrogen", 1),
            0,
            "and gives nothing"
        );
        assert_eq!(Tank::new("mekanism:hydrogen", 0), Tank::default());
    }

    /// The id table is Mekanism's own: every entry namespaced, sorted, and the
    /// empty chemical outside it.
    #[test]
    fn the_chemical_table_is_mekanism_s_named_chemicals() {
        assert_eq!(
            CHEMICALS.len(),
            32,
            "MekanismChemicals registers 32 named ones"
        );
        assert!(
            CHEMICALS.windows(2).all(|pair| pair[0] < pair[1]),
            "the table is sorted and has no duplicates"
        );
        for id in CHEMICALS {
            assert!(
                id.starts_with("mekanism:") && id.len() > "mekanism:".len(),
                "{id} is not a namespaced mekanism id"
            );
            assert_eq!(known_chemical(id), Some(id));
        }
        assert!(!CHEMICALS.contains(&EMPTY_CHEMICAL));
    }

    /// The gauge dropper moves at most its transfer rate per call, whatever is
    /// asked, and holds 16 buckets.
    #[test]
    fn the_gauge_dropper_moves_at_most_its_transfer_rate() {
        assert_eq!(dropper_move(1_000), DROPPER_TRANSFER);
        assert_eq!(dropper_move(10), 10);
        assert_eq!(dropper_move(0), 0);
        let mut dropper = Tank::default();
        assert_eq!(
            insert_chemical(&mut dropper, "mekanism:hydrogen", 1_000, DROPPER_CAPACITY),
            Ok(1_000),
            "one insert of 1,000 fits the dropper"
        );
        assert_eq!(
            insert_chemical(&mut dropper, "mekanism:hydrogen", 99_999, DROPPER_CAPACITY),
            Ok(DROPPER_CAPACITY - 1_000),
            "and its capacity is 16,000"
        );
    }
}

#[cfg(test)]
mod injection_tests {
    use super::*;

    /// A charged Chemical Injection Chamber with raw ore in its input and the
    /// gas its recipe needs in its own tank.
    fn injection(gas: u32) -> (Machine, Tank, ItemAmount, ItemAmount) {
        (
            Machine {
                energy: INJECTION_CAPACITY,
                progress: 0,
            },
            Tank::new("mekanism:hydrogen_chloride", gas),
            ItemAmount::new("minecraft:raw_iron", 3),
            ItemAmount::default(),
        )
    }

    /// The table is the jar's `mekanism:injecting` data over the items and
    /// chemicals this port registers: 91 recipes in the jar, 72 rows here (a
    /// chemical tag is one row naming every chemical it accepts, an item tag one
    /// row per member).
    #[test]
    fn the_injection_table_is_the_jar_s_recipes_over_this_port_s_items() {
        let shards = injecting::recipe_for("minecraft:raw_iron").expect("raw iron is injected");
        assert_eq!(shards.input, 3, "from_raw_ore takes three raw ore");
        assert_eq!(shards.output, ItemAmount::new("mekanism:shard_iron", 8));
        assert_eq!(shards.per_tick, 1, "the jar's per_tick_usage amount");
        assert!(shards.accepts("mekanism:hydrogen_chloride"));
        assert!(!shards.accepts("mekanism:oxygen"));
        let ore = injecting::recipe_for("minecraft:deepslate_iron_ore").expect("an iron ore");
        assert_eq!(ore.output, ItemAmount::new("mekanism:shard_iron", 4));
        // The water-vapour family accepts the jar's tag, both of its members.
        let mud = injecting::recipe_for("minecraft:dirt").expect("dirt is injected");
        assert_eq!(mud.output, ItemAmount::new("minecraft:mud", 1));
        assert!(mud.accepts("mekanism:water_vapor") && mud.accepts("mekanism:steam"));
        assert!(!mud.accepts("mekanism:hydrogen_chloride"));
        // The jar's oxygen half (copper oxidising) is carried as it is.
        let oxidised = injecting::recipe_for("minecraft:copper_block").expect("copper oxidises");
        assert_eq!(
            oxidised.output,
            ItemAmount::new("minecraft:exposed_copper", 1)
        );
        assert!(oxidised.accepts("mekanism:oxygen"));
        assert_eq!(
            injecting::ROW_COUNT,
            79,
            "the rows the generator keeps from the jar's 91 injecting recipes"
        );
    }

    /// The crystal → shard leg: every crystal this port's crystallizer makes —
    /// the same seven materials the washer's slurries name — has the jar's
    /// `data/mekanism/recipe/processing/<material>/shard/from_crystal.json` row:
    /// one crystal and one unit of hydrogen chloride per operating tick become
    /// one shard of that material. So no crystal the port ships dead-ends at the
    /// injection chamber, exactly as no clean slurry dead-ends at the
    /// crystallizer.
    #[test]
    fn every_crystal_the_crystallizer_makes_injects_into_its_material_s_shard() {
        let materials = ["copper", "gold", "iron", "lead", "osmium", "tin", "uranium"];
        assert_eq!(
            materials.len(),
            crystallizing::ROW_COUNT,
            "the jar crystallizes exactly the seven materials this port carries"
        );
        for material in materials {
            let crystal = format!("mekanism:crystal_{material}");
            let recipe = injecting::recipe_for(&crystal)
                .unwrap_or_else(|| panic!("no injecting row for {crystal}"));
            assert_eq!(
                recipe,
                GasRecipe {
                    input: 1,
                    chemical: &["mekanism:hydrogen_chloride"],
                    per_tick: 1,
                    output: ItemAmount::new(&format!("mekanism:shard_{material}"), 1),
                },
                "{crystal} must inject into one shard for one HCl a tick"
            );
        }
    }

    /// One tick of the operation: the gas the recipe declares is taken per tick,
    /// one progress tick is added, and 400 FE (the jar's
    /// `UsageConfig.chemicalInjectionChamber`) is spent.
    #[test]
    fn one_injection_tick_spends_fe_one_progress_and_the_tick_s_gas() {
        let (mut machine, mut tank, mut input, mut output) = injection(200);
        let outcome = process_gas_tick(
            &mut machine,
            INJECTION_OPERATION,
            &mut tank,
            &mut input,
            &mut output,
            64,
            &injecting::recipe_for,
        );
        assert_eq!(outcome, Outcome::Working);
        assert_eq!(machine.progress, 1);
        assert_eq!(
            machine.energy,
            INJECTION_CAPACITY - INJECTION_USAGE,
            "400 FE for one tick"
        );
        assert_eq!(tank.held(), Some(("mekanism:hydrogen_chloride", 199)));
        assert_eq!(
            input.count, 3,
            "the input is consumed when the operation ends"
        );
        assert!(output.is_empty());
        assert_eq!(
            INJECTION_TANK, 210,
            "TileEntityAdvancedElectricMachine.MAX_GAS"
        );
    }

    /// A missing gas, or a gas the recipe does not take, holds the progress and
    /// spends nothing: upstream's constant-usage handler pauses instead of
    /// resetting, and no energy is taken when no operation runs.
    #[test]
    fn a_missing_or_wrong_gas_holds_the_progress_and_spends_nothing() {
        for gas in [Tank::default(), Tank::new("mekanism:oxygen", 4_000)] {
            let (mut machine, mut input, mut output) = {
                let (machine, _tank, input, output) = injection(0);
                (machine, input, output)
            };
            machine.progress = 41;
            let mut tank = gas.clone();
            let outcome = process_gas_tick(
                &mut machine,
                INJECTION_OPERATION,
                &mut tank,
                &mut input,
                &mut output,
                64,
                &injecting::recipe_for,
            );
            assert_eq!(outcome, Outcome::Idle(IdleReason::NoChemical), "{gas:?}");
            assert_eq!(
                machine,
                Machine {
                    energy: INJECTION_CAPACITY,
                    progress: 41
                }
            );
            assert_eq!(tank, gas, "a held tick moves no gas");
        }
    }

    /// A whole operation: 200 ticks, 80,000 FE, 200 units of the gas, three raw
    /// ore in and the jar's eight shards out.
    #[test]
    fn an_injection_operation_consumes_its_input_and_yields_the_jar_s_stack() {
        let (mut machine, mut tank, mut input, mut output) = injection(200);
        let mut last = Outcome::Working;
        for _ in 0..INJECTION_TICKS {
            last = process_gas_tick(
                &mut machine,
                INJECTION_OPERATION,
                &mut tank,
                &mut input,
                &mut output,
                64,
                &injecting::recipe_for,
            );
        }
        assert_eq!(last, Outcome::Produced);
        assert_eq!(machine.progress, 0, "a finished operation starts over");
        assert_eq!(
            machine.energy,
            INJECTION_CAPACITY - 80_000,
            "200 ticks x 400 FE"
        );
        assert_eq!(tank, Tank::default(), "all 200 units went into the shards");
        assert!(
            input.is_empty(),
            "the operation consumed its three raw iron"
        );
        assert_eq!(output, ItemAmount::new("mekanism:shard_iron", 8));
    }

    /// The gas is spent every tick: a tank with half an operation's gas runs
    /// half the operation and then holds, with the FE it did not spend intact.
    #[test]
    fn the_gas_runs_out_mid_operation_and_the_machine_holds() {
        let (mut machine, mut tank, mut input, mut output) = injection(100);
        let mut last = Outcome::Working;
        for _ in 0..INJECTION_TICKS {
            last = process_gas_tick(
                &mut machine,
                INJECTION_OPERATION,
                &mut tank,
                &mut input,
                &mut output,
                64,
                &injecting::recipe_for,
            );
        }
        assert_eq!(
            last,
            Outcome::Idle(IdleReason::NoChemical),
            "the gas ran out at tick 100, so the last tick had nothing to burn"
        );
        assert_eq!(machine.progress, 100, "the progress is held, not reset");
        assert_eq!(
            machine.energy,
            INJECTION_CAPACITY - 40_000,
            "100 ticks of 400"
        );
        assert_eq!(tank, Tank::default());
        assert!(output.is_empty(), "no operation finished");
        assert_eq!(input.count, 3, "and no input was consumed");
    }

    /// The machine asks a neighbour for exactly the gas its next tick spends:
    /// nothing when its tank already holds a tick's worth, and never more than
    /// that, so a whole operation draws 200 units and leaves none behind.
    #[test]
    fn the_top_up_asks_for_exactly_the_tick_s_gas() {
        let recipe = injecting::recipe_for("minecraft:raw_iron").expect("a recipe");
        assert_eq!(
            gas_pull(&recipe, &Tank::default()),
            Some(("mekanism:hydrogen_chloride", 1)),
            "an empty tank asks for one tick's gas"
        );
        assert_eq!(
            gas_pull(&recipe, &Tank::new("mekanism:hydrogen_chloride", 1)),
            None,
            "a tank holding the tick's gas asks for nothing"
        );
        assert_eq!(
            gas_pull(&recipe, &Tank::new("mekanism:hydrogen_chloride", 200)),
            None
        );
        assert_eq!(
            gas_pull(&recipe, &Tank::new("mekanism:oxygen", 4_000)),
            None,
            "a tank of another chemical is not topped up with this one"
        );
        let coral = injecting::recipe_for("minecraft:dead_tube_coral").expect("a coral recipe");
        assert_eq!(coral.per_tick, 3);
        assert_eq!(
            gas_pull(&coral, &Tank::new("mekanism:steam", 1)),
            Some(("mekanism:steam", 2)),
            "a tag's second member is pulled under its own name"
        );
        assert_eq!(
            gas_pull(&coral, &Tank::default()),
            Some(("mekanism:water_vapor", 3)),
            "an empty tank asks for the tag's first chemical"
        );
    }

    /// The machine is the jar's: 400 FE/t, a 160,000 FE buffer, a 210-unit gas
    /// buffer, and the shared 200-tick operation.
    #[test]
    fn the_injection_machine_is_the_jar_s_energy_and_gas() {
        assert_eq!(INJECTION_USAGE, 400, "UsageConfig.chemicalInjectionChamber");
        assert_eq!(
            INJECTION_CAPACITY, 160_000,
            "StorageConfig.chemicalInjectionChamber"
        );
        assert_eq!(INJECTION_TICKS, 200, "TileEntityAdvancedElectricMachine");
        assert_eq!(INJECTION_OPERATION.usage, INJECTION_USAGE);
        assert_eq!(INJECTION_OPERATION.ticks, INJECTION_TICKS);
        assert_eq!(
            room_for(
                &ItemAmount::default(),
                &ItemAmount::new("mekanism:shard_iron", 8),
                64
            ),
            64,
            "an empty output takes the stack"
        );
    }
}

#[cfg(test)]
mod purification_tests {
    use super::*;

    /// A charged Purification Chamber with one iron ore in its input and oxygen
    /// in its own tank: the slice's headline machine.
    fn chamber(oxygen: u32) -> (Machine, Tank, ItemAmount, ItemAmount) {
        (
            Machine {
                energy: PURIFY_CAPACITY,
                progress: 0,
            },
            Tank::new("mekanism:oxygen", oxygen),
            ItemAmount::new("minecraft:iron_ore", 1),
            ItemAmount::default(),
        )
    }

    /// The table is the jar's `mekanism:purifying` data over the items this port
    /// registers: 28 recipes in the jar, 15 rows here (four materials' clump
    /// recipes name items or tags this port has; an item tag is one row per
    /// 26.3 member).
    #[test]
    fn the_purifying_table_is_the_jar_s_clump_recipes() {
        let ore = purifying::recipe_for("minecraft:iron_ore").expect("an iron ore clumps");
        assert_eq!(ore.input, 1);
        assert_eq!(ore.output, ItemAmount::new("mekanism:clump_iron", 3));
        assert_eq!(ore.per_tick, 1, "the jar's per_tick_usage amount");
        assert!(ore.accepts("mekanism:oxygen"));
        assert!(
            !ore.accepts("mekanism:hydrogen_chloride"),
            "hydrogen chloride is the injection chamber's gas"
        );
        let raw = purifying::recipe_for("minecraft:raw_gold").expect("raw gold clumps");
        assert_eq!(raw.output, ItemAmount::new("mekanism:clump_gold", 2));
        let block = purifying::recipe_for("minecraft:raw_copper_block").expect("a raw block");
        assert_eq!(block.output, ItemAmount::new("mekanism:clump_copper", 18));
        assert_eq!(block.per_tick, 2, "from_raw_block's chemical_input.amount");
        let shard = purifying::recipe_for("mekanism:shard_iron").expect("a shard clumps");
        assert_eq!(shard.output, ItemAmount::new("mekanism:clump_iron", 1));
        assert_eq!(
            purifying::ROW_COUNT,
            15,
            "the rows the generator keeps from the jar's 28 purifying recipes"
        );
    }

    /// One tick of the operation: the recipe's oxygen is taken per tick, one
    /// progress tick is added, and 200 FE (`UsageConfig.purificationChamber`) is
    /// spent.
    #[test]
    fn one_purification_tick_spends_fe_one_progress_and_the_tick_s_oxygen() {
        let (mut machine, mut tank, mut input, mut output) = chamber(200);
        let outcome = process_gas_tick(
            &mut machine,
            PURIFY_OPERATION,
            &mut tank,
            &mut input,
            &mut output,
            64,
            &purifying::recipe_for,
        );
        assert_eq!(outcome, Outcome::Working);
        assert_eq!(machine.progress, 1);
        assert_eq!(
            machine.energy,
            PURIFY_CAPACITY - PURIFY_USAGE,
            "200 FE for one tick"
        );
        assert_eq!(tank.held(), Some(("mekanism:oxygen", 199)));
        assert_eq!(
            input.count, 1,
            "the input is consumed when the operation ends"
        );
        assert!(output.is_empty());
        assert_eq!(
            PURIFY_TANK, 210,
            "TileEntityAdvancedElectricMachine.MAX_GAS"
        );
    }

    /// A whole operation: 200 ticks, 40,000 FE (half the buffer), 200 units of
    /// oxygen, one iron ore in and the jar's three clumps out.
    #[test]
    fn a_purification_operation_costs_forty_thousand_fe_and_two_hundred_oxygen() {
        let (mut machine, mut tank, mut input, mut output) = chamber(200);
        let mut last = Outcome::Working;
        for _ in 0..PURIFY_TICKS {
            last = process_gas_tick(
                &mut machine,
                PURIFY_OPERATION,
                &mut tank,
                &mut input,
                &mut output,
                64,
                &purifying::recipe_for,
            );
        }
        assert_eq!(last, Outcome::Produced);
        assert_eq!(machine.progress, 0, "a finished operation starts over");
        assert_eq!(
            machine.energy,
            PURIFY_CAPACITY - 40_000,
            "200 ticks x 200 FE"
        );
        assert_eq!(tank, Tank::default(), "all 200 units went into the clumps");
        assert!(input.is_empty(), "the operation consumed its iron ore");
        assert_eq!(output, ItemAmount::new("mekanism:clump_iron", 3));
    }

    /// The purification chamber takes oxygen and nothing else: a tank of the
    /// injection chamber's hydrogen chloride holds the progress and spends
    /// nothing, exactly as an empty tank does.
    #[test]
    fn a_purification_chamber_takes_no_other_gas() {
        for tank in [
            Tank::default(),
            Tank::new("mekanism:hydrogen_chloride", 4_000),
        ] {
            let (mut machine, mut input, mut output) = {
                let (machine, _tank, input, output) = chamber(0);
                (machine, input, output)
            };
            machine.progress = 41;
            let mut tank = tank.clone();
            let outcome = process_gas_tick(
                &mut machine,
                PURIFY_OPERATION,
                &mut tank,
                &mut input,
                &mut output,
                64,
                &purifying::recipe_for,
            );
            assert_eq!(outcome, Outcome::Idle(IdleReason::NoChemical), "{tank:?}");
            assert_eq!(
                machine,
                Machine {
                    energy: PURIFY_CAPACITY,
                    progress: 41
                }
            );
            assert_eq!(tank, tank.clone(), "a held tick moves no gas");
        }
        assert_eq!(
            gas_pull(
                &purifying::recipe_for("minecraft:iron_ore").expect("a recipe"),
                &Tank::default()
            ),
            Some(("mekanism:oxygen", 1)),
            "the machine asks for one tick's oxygen"
        );
    }

    /// The jar's ore-processing chain, row by row: a clump crushes into a dirty
    /// dust, and a dirty dust enriches into a dust. Upstream's
    /// `from_dirty_dust.json` is `mekanism:enriching`, so the purification
    /// chamber has no row for a dirty dust — the port does not invent one.
    #[test]
    fn the_chain_rows_are_the_jar_s_clump_to_dirty_dust_and_dirty_dust_to_dust() {
        assert_eq!(
            crushing::lookup("mekanism:clump_iron"),
            Some(MachineRecipe {
                input: 1,
                output: ItemAmount::new("mekanism:dirty_dust_iron", 1),
            })
        );
        assert_eq!(
            enriching::lookup("mekanism:dirty_dust_iron"),
            Some(MachineRecipe {
                input: 1,
                output: ItemAmount::new("mekanism:dust_iron", 1),
            })
        );
        assert_eq!(
            purifying::recipe_for("mekanism:dirty_dust_iron"),
            None,
            "from_dirty_dust is an enriching recipe, not a purifying one"
        );
    }
}

#[cfg(test)]
mod factory_tests {
    use super::*;

    /// One factory process holding `count` of `item`, with an empty output.
    fn process_with(item: &str, count: u32) -> FactoryProcess {
        FactoryProcess {
            progress: 0,
            input: ItemAmount::new(item, count),
            output: ItemAmount::default(),
        }
    }

    /// A basic factory with an ore in every one of its three processes.
    fn three_ore() -> Vec<FactoryProcess> {
        vec![process_with("minecraft:iron_ore", 1); 3]
    }

    /// Run a whole operation's worth of ticks of the item core and answer how
    /// many operations completed.
    fn run_item(energy: &mut i32, processes: &mut [FactoryProcess], operation: Operation) -> usize {
        let limits = [64; FACTORY_PROCESSES];
        let operations = operations_per_tick(operation.ticks * 10, 1);
        let mut produced = 0;
        for _ in 0..operation.ticks {
            produced += process_factory_tick(
                energy,
                processes,
                operation,
                &limits,
                operations,
                &smelting::lookup,
            )
            .iter()
            .filter(|outcome| **outcome == Outcome::Produced)
            .count();
        }
        produced
    }

    /// The same over the gas core, with the purifying table.
    fn run_gas(energy: &mut i32, tank: &mut Tank, processes: &mut [FactoryProcess]) -> usize {
        let operation = PURIFY_OPERATION;
        let limits = [64; FACTORY_PROCESSES];
        let operations = operations_per_tick(operation.ticks * 10, 1);
        let mut produced = 0;
        for _ in 0..operation.ticks {
            produced += process_gas_factory_tick(
                energy,
                tank,
                processes,
                operation,
                &limits,
                operations,
                &purifying::recipe_for,
            )
            .iter()
            .filter(|outcome| **outcome == Outcome::Produced)
            .count();
        }
        produced
    }

    /// `FactoryTier.BASIC` is three processes, and `Factory.setMachineData`
    /// gives the factory half its base machine's storage per process — but never
    /// less than one tick's usage per process.
    #[test]
    fn a_basic_factory_holds_half_its_base_machines_buffer_per_process() {
        assert_eq!(FACTORY_PROCESSES, 3, "FactoryTier.BASIC.processes");
        assert_eq!(
            SMELTING_FACTORY_CAPACITY,
            factory_capacity(SMELTER_CAPACITY, SMELTER_USAGE, FACTORY_PROCESSES)
        );
        assert_eq!(SMELTING_FACTORY_CAPACITY, 30_000, "20,000 / 2 x 3");
        assert_eq!(ENRICHING_FACTORY_CAPACITY, 30_000);
        assert_eq!(CRUSHING_FACTORY_CAPACITY, 30_000);
        assert_eq!(INJECTING_FACTORY_CAPACITY, 240_000, "160,000 / 2 x 3");
        assert_eq!(PURIFYING_FACTORY_CAPACITY, 120_000, "80,000 / 2 x 3");
        assert_eq!(
            factory_capacity(100, 50, 3),
            150,
            "the floor is one tick's usage per process"
        );
    }

    /// The gas factories' one tank is `MAX_GAS` per process: 630 for a basic
    /// factory, not the 210 a single machine carries.
    #[test]
    fn a_basic_factory_has_a_tank_of_one_machine_s_gas_per_process() {
        assert_eq!(FACTORY_TICKS, 200, "TileEntityFactory.BASE_TICKS_REQUIRED");
        assert_eq!(factory_tank(PURIFY_TANK, FACTORY_PROCESSES), 630);
        assert_eq!(PURIFYING_FACTORY_TANK, 630);
        assert_eq!(INJECTING_FACTORY_TANK, 630);
        assert_eq!(
            PURIFYING_FACTORY_TANK,
            INJECTION_TANK * FACTORY_PROCESSES as u32
        );
    }

    /// `MekanismUtils.getOperationsPerTick`: a whole-tick operation is one per
    /// tick; a sub-tick one (only a speed-upgraded factory) runs more.
    #[test]
    fn a_whole_tick_operation_is_one_operation_per_tick() {
        assert_eq!(operations_per_tick(FACTORY_TICKS * 10, 1), 1);
        assert_eq!(
            operations_per_tick(10, 1),
            1,
            "exactly one tick is still one"
        );
        assert_eq!(operations_per_tick(5, 1), 2, "half a tick runs two");
        assert_eq!(operations_per_tick(4, 1), 2, "0.4 ticks runs 2, truncated");
        assert_eq!(operations_per_tick(0, 1), 10, "a tenth of a tick runs ten");
    }

    /// Three ore, one per process: three operations complete in one 200-tick
    /// window, not one after another, and the shared buffer pays for all three.
    #[test]
    fn three_ore_in_three_processes_make_three_operations_side_by_side() {
        let mut energy = SMELTING_FACTORY_CAPACITY;
        let mut processes = three_ore();
        assert_eq!(
            run_item(&mut energy, &mut processes, SMELTER_OPERATION),
            3,
            "one operation per process, all in the same 200 ticks"
        );
        assert_eq!(
            processes.iter().map(|p| p.output.count).sum::<u32>(),
            3,
            "one ingot each, not three in one slot"
        );
        assert!(
            processes
                .iter()
                .all(|p| p.input.is_empty() && p.progress == 0),
            "every process consumed its ore and reset its progress"
        );
        assert_eq!(energy, 0, "three operations at 10,000 FE each");
    }

    /// One process working costs one operation, not the whole factory: the other
    /// two idle processes must not be charged.
    #[test]
    fn one_process_working_costs_one_operation_not_the_whole_factory() {
        let mut energy = SMELTING_FACTORY_CAPACITY;
        let mut processes = vec![
            process_with("minecraft:iron_ore", 1),
            FactoryProcess::default(),
            FactoryProcess::default(),
        ];
        assert_eq!(run_item(&mut energy, &mut processes, SMELTER_OPERATION), 1);
        assert_eq!(processes[0].output.count, 1);
        assert_eq!(processes[1].output, ItemAmount::default());
        assert_eq!(processes[2].output, ItemAmount::default());
        assert_eq!(energy, 20_000, "10,000 FE, not the factory's 30,000");
    }

    /// The buffer is shared and each working process draws its own tick: 40,000
    /// FE at 3 x 200 FE/t buys exactly 200 progress ticks in total, so no
    /// operation completes. A factory that charged once per tick would have
    /// finished all three.
    #[test]
    fn the_shared_buffer_pays_for_every_working_process() {
        let mut energy = 40_000;
        let mut tank = Tank::new("mekanism:oxygen", 4_000);
        let mut processes = three_ore();
        assert_eq!(run_gas(&mut energy, &mut tank, &mut processes), 0);
        assert!(
            processes.iter().all(|p| p.output.is_empty()),
            "nothing completes on a third of the energy"
        );
        assert_eq!(
            processes.iter().map(|p| p.progress).sum::<i32>(),
            200,
            "40,000 FE = 200 progress ticks at 200 FE/t, not 600"
        );
        assert_eq!(energy, 0);
        assert_eq!(
            tank.held(),
            Some(("mekanism:oxygen", 3_800)),
            "and only the ticks that ran drew gas"
        );
    }

    /// An empty process resets its own progress and leaves its neighbours alone.
    #[test]
    fn an_empty_process_resets_while_its_neighbours_keep_working() {
        let mut energy = SMELTING_FACTORY_CAPACITY;
        let mut processes = vec![
            process_with("minecraft:iron_ore", 1),
            FactoryProcess::default(),
            process_with("minecraft:iron_ore", 1),
        ];
        assert_eq!(run_item(&mut energy, &mut processes, SMELTER_OPERATION), 2);
        assert_eq!(processes[1].progress, 0, "the empty process resets");
        assert_eq!(processes[0].output.count, 1);
        assert_eq!(processes[2].output.count, 1);
        assert_eq!(energy, 10_000, "two operations at 10,000 FE each");
    }

    /// The gas factory draws one tick of the gas for every working process: three
    /// iron ore in a basic purifying factory spend 3 x 200 = 600 units, not 200.
    #[test]
    fn a_gas_factory_spends_one_tick_of_each_process_s_gas() {
        let mut energy = PURIFYING_FACTORY_CAPACITY;
        let mut tank = Tank::new("mekanism:oxygen", 4_000);
        let mut processes = three_ore();
        assert_eq!(run_gas(&mut energy, &mut tank, &mut processes), 3);
        assert_eq!(
            processes.iter().map(|p| p.output.count).sum::<u32>(),
            9,
            "three clumps per ore, three processes"
        );
        assert_eq!(
            tank.held(),
            Some(("mekanism:oxygen", 3_400)),
            "4,000 - 3 x 200"
        );
        assert_eq!(energy, 0, "3 x 40,000 FE");
    }

    /// How much gas the factory's working processes need this tick: the sum of
    /// their per-tick amounts, less what the shared tank already holds. An idle
    /// process asks for nothing, and a tank of another gas asks for nothing.
    #[test]
    fn a_gas_factory_asks_for_one_tick_of_gas_per_working_process() {
        let empty = Tank::default();
        let processes = three_ore();
        let op = PURIFY_OPERATION;
        assert_eq!(
            factory_gas_want(
                PURIFYING_FACTORY_CAPACITY,
                op,
                &empty,
                &processes,
                &[64; 3],
                &purifying::recipe_for
            ),
            Some(("mekanism:oxygen", 3)),
            "three ore, three units this tick"
        );
        let one = Tank::new("mekanism:oxygen", 1);
        assert_eq!(
            factory_gas_want(
                PURIFYING_FACTORY_CAPACITY,
                op,
                &one,
                &processes,
                &[64; 3],
                &purifying::recipe_for
            ),
            Some(("mekanism:oxygen", 2)),
            "the tank's own unit counts toward the three"
        );
        let full = Tank::new("mekanism:oxygen", 3);
        assert_eq!(
            factory_gas_want(
                PURIFYING_FACTORY_CAPACITY,
                op,
                &full,
                &processes,
                &[64; 3],
                &purifying::recipe_for
            ),
            None,
            "a tank that already holds the tick's gas is not drawn from"
        );
        let idle = [
            process_with("minecraft:iron_ore", 1),
            FactoryProcess::default(),
            FactoryProcess::default(),
        ];
        assert_eq!(
            factory_gas_want(
                PURIFYING_FACTORY_CAPACITY,
                op,
                &empty,
                &idle,
                &[64; 3],
                &purifying::recipe_for
            ),
            Some(("mekanism:oxygen", 1)),
            "an empty process asks for nothing"
        );
        let foreign = Tank::new("mekanism:hydrogen_chloride", 4_000);
        assert_eq!(
            factory_gas_want(
                PURIFYING_FACTORY_CAPACITY,
                op,
                &foreign,
                &processes,
                &[64; 3],
                &purifying::recipe_for
            ),
            None,
            "a tank of a gas the recipe does not take is not drawn from"
        );
        // The buffer pays the processes in order, as the tick does: with one
        // tick's energy only the first process is paid for, and with none
        // nothing is drawn at all (the defect the 0.2.6 in-game run found: an
        // unpowered factory sipped its neighbour's gas).
        assert_eq!(
            factory_gas_want(
                PURIFY_USAGE,
                op,
                &empty,
                &processes,
                &[64; 3],
                &purifying::recipe_for
            ),
            Some(("mekanism:oxygen", 1)),
            "one tick's energy buys one process's gas"
        );
        assert_eq!(
            factory_gas_want(0, op, &empty, &processes, &[64; 3], &purifying::recipe_for),
            None,
            "no energy, no gas"
        );
    }

    /// A gas the recipe does not take holds every process and draws nothing.
    #[test]
    fn a_gas_factory_with_a_foreign_gas_holds_every_process() {
        let mut energy = PURIFYING_FACTORY_CAPACITY;
        let mut tank = Tank::new("mekanism:hydrogen_chloride", 4_000);
        let mut processes = three_ore();
        assert_eq!(run_gas(&mut energy, &mut tank, &mut processes), 0);
        assert!(processes.iter().all(|p| p.progress == 0));
        assert_eq!(tank.held(), Some(("mekanism:hydrogen_chloride", 4_000)));
        assert_eq!(energy, PURIFYING_FACTORY_CAPACITY);
    }

    /// Each process owns its output slot, so one full slot blocks only itself.
    #[test]
    fn a_full_output_slot_blocks_only_its_own_process() {
        let mut energy = SMELTING_FACTORY_CAPACITY;
        let mut processes = three_ore();
        processes[1].output = ItemAmount::new("minecraft:cobblestone", 64);
        assert_eq!(run_item(&mut energy, &mut processes, SMELTER_OPERATION), 2);
        assert_eq!(processes[1].progress, 0, "the blocked process stays held");
        assert_eq!(processes[1].output.item, "minecraft:cobblestone");
        assert_eq!(processes[0].output.count, 1);
        assert_eq!(processes[2].output.count, 1);
        assert_eq!(energy, 10_000);
    }
}

// ---------------------------------------------------------------------------
// Slice 8: the Chemical Washer — a fluid input beside the chemical ones
// ---------------------------------------------------------------------------

#[cfg(test)]
mod washer_tests {
    use super::*;

    /// The washer's numbers are the jar's own (`UsageConfig.chemicalWasher`,
    /// `StorageConfig.chemicalWasher`, `MAX_FLUID`/`MAX_SLURRY`), not carried
    /// over from another machine.
    #[test]
    fn the_washer_is_the_jar_s_200_fe_a_tick_and_its_three_tanks() {
        assert_eq!(WASHER_USAGE, 200, "UsageConfig.chemicalWasher");
        assert_eq!(WASHER_CAPACITY, 80_000, "StorageConfig.chemicalWasher");
        assert_eq!(
            WASHER_TICKS, 200,
            "TileEntityRecipeMachine BASE_TICKS_REQUIRED"
        );
        assert_eq!(WASHER_OPERATION.usage, 200);
        assert_eq!(WASHER_OPERATION.ticks, 200);
        assert_eq!(WASHER_WATER, 10_000, "MAX_FLUID = 10 x BUCKET_VOLUME");
        assert_eq!(WASHER_SLURRY, 10_000, "MAX_SLURRY = 10 x BUCKET_VOLUME");
        assert_eq!(WATER, "minecraft:water");
    }

    /// Slurries are chemicals of Mekanism's own registry
    /// (`MekanismChemicals.PROCESSED_RESOURCES` → `registerSlurry`): fourteen
    /// ids, `dirty_` and `clean_` for the seven `PrimaryResource`s, and a tank
    /// therefore takes them.
    #[test]
    fn the_slurry_table_is_mekanism_s_fourteen_chemicals() {
        assert_eq!(SLURRIES.len(), 14, "seven materials x a dirty and a clean");
        assert!(
            SLURRIES.windows(2).all(|pair| pair[0] < pair[1]),
            "the table is sorted and has no duplicates"
        );
        for id in SLURRIES {
            assert!(
                id.starts_with("mekanism:dirty_") || id.starts_with("mekanism:clean_"),
                "{id} is not a slurry id"
            );
            assert_eq!(known_slurry(id), Some(id));
            assert_eq!(
                known_chemical(id),
                Some(id),
                "a tank takes a slurry: it is a chemical of the same registry"
            );
        }
        assert_eq!(
            known_slurry("mekanism:hydrogen"),
            None,
            "a gas is not a slurry"
        );
        assert_eq!(
            known_chemical(WATER),
            None,
            "water is a fluid, not a chemical"
        );
    }

    /// Every washing row is the jar's: 5 mB of water and one unit of the
    /// material's dirty slurry make one unit of its clean slurry.
    #[test]
    fn every_washing_row_is_the_jar_s() {
        assert_eq!(washing::ROW_COUNT, 7, "one clean.json per PrimaryResource");
        for material in ["copper", "gold", "iron", "lead", "osmium", "tin", "uranium"] {
            let dirty = format!("mekanism:dirty_{material}");
            let clean = format!("mekanism:clean_{material}");
            let recipe =
                washing::recipe_for(&dirty).unwrap_or_else(|| panic!("no row for {dirty}"));
            assert_eq!(recipe.fluid, WATER);
            assert_eq!(recipe.fluid_amount, 5, "the jar's fluid_input amount");
            assert_eq!(recipe.chemical, dirty);
            assert_eq!(recipe.chemical_amount, 1);
            assert_eq!(recipe.output, clean);
            assert_eq!(recipe.output_amount, 1);
            assert_eq!(
                washing::recipe_for(&clean),
                None,
                "a clean slurry is not an input: the jar has no row for one"
            );
        }
        assert_eq!(washing::recipe_for("mekanism:hydrogen"), None);
    }

    /// One operation at the jar's price: 200 ticks x 200 FE, 5 mB of water and
    /// one unit of dirty slurry in, one unit of clean slurry out — and the
    /// three tanks each move by exactly that.
    #[test]
    fn one_washing_operation_costs_one_operation_s_energy_water_and_slurry() {
        let mut machine = Machine {
            energy: WASHER_CAPACITY,
            progress: 0,
        };
        let mut water = FluidTank::new(WATER, WASHER_WATER);
        let mut dirty = Tank::new("mekanism:dirty_iron", 1_000);
        let mut clean = Tank::default();
        let mut outcomes = Vec::new();
        for _ in 0..WASHER_TICKS {
            outcomes.push(process_wash_tick(
                &mut machine,
                WASHER_OPERATION,
                &mut water,
                &mut dirty,
                &mut clean,
                WASHER_SLURRY,
                &washing::recipe_for,
            ));
        }
        assert!(
            outcomes[..199]
                .iter()
                .all(|outcome| *outcome == Outcome::Working),
            "every tick but the last only works"
        );
        assert_eq!(*outcomes.last().unwrap(), Outcome::Produced);
        assert_eq!(
            machine.energy,
            WASHER_CAPACITY - 40_000,
            "200 ticks x 200 FE"
        );
        assert_eq!(machine.progress, 0, "the operation finished");
        assert_eq!(
            water.held(),
            Some((WATER, WASHER_WATER - 5)),
            "5 mB of water an operation, not 5 a tick"
        );
        assert_eq!(dirty.held(), Some(("mekanism:dirty_iron", 999)));
        assert_eq!(clean.held(), Some(("mekanism:clean_iron", 1)));
    }

    /// No water (or another fluid in the tank) is upstream's mismatched recipe:
    /// the progress resets and nothing at all is spent.
    #[test]
    fn a_washer_with_no_water_resets_and_spends_nothing() {
        for (fluid, amount) in [(None, 0u32), (Some("minecraft:lava"), 10_000)] {
            let mut machine = Machine {
                energy: WASHER_CAPACITY,
                progress: 60,
            };
            let mut water = FluidTank {
                fluid: fluid.map(str::to_owned),
                amount,
            };
            let mut dirty = Tank::new("mekanism:dirty_iron", 1_000);
            let mut clean = Tank::default();
            assert_eq!(
                process_wash_tick(
                    &mut machine,
                    WASHER_OPERATION,
                    &mut water,
                    &mut dirty,
                    &mut clean,
                    WASHER_SLURRY,
                    &washing::recipe_for,
                ),
                Outcome::Idle(IdleReason::NoFluid)
            );
            assert_eq!(
                machine.progress, 0,
                "a mismatched recipe resets the progress"
            );
            assert_eq!(machine.energy, WASHER_CAPACITY, "and spends no energy");
            assert_eq!(dirty, Tank::new("mekanism:dirty_iron", 1_000));
            assert_eq!(clean, Tank::default());
        }
    }

    /// Too little water for one operation is `NOT_ENOUGH_SECONDARY_INPUT`: it
    /// resets the progress rather than holding it, as the item machines do.
    #[test]
    fn a_washer_short_of_water_resets_too() {
        let mut machine = Machine {
            energy: WASHER_CAPACITY,
            progress: 60,
        };
        let mut water = FluidTank::new(WATER, 4);
        let mut dirty = Tank::new("mekanism:dirty_iron", 1_000);
        let mut clean = Tank::default();
        assert_eq!(
            process_wash_tick(
                &mut machine,
                WASHER_OPERATION,
                &mut water,
                &mut dirty,
                &mut clean,
                WASHER_SLURRY,
                &washing::recipe_for,
            ),
            Outcome::Idle(IdleReason::NoFluid)
        );
        assert_eq!(machine.progress, 0);
        assert_eq!(machine.energy, WASHER_CAPACITY);
        assert_eq!(water, FluidTank::new(WATER, 4), "not a drop is taken");
    }

    /// A tank of clean slurry is an input with no recipe: the progress resets,
    /// and neither the water nor the energy moves.
    #[test]
    fn a_washer_with_a_clean_slurry_has_no_recipe() {
        let mut machine = Machine {
            energy: WASHER_CAPACITY,
            progress: 60,
        };
        let mut water = FluidTank::new(WATER, WASHER_WATER);
        let mut clean_in = Tank::new("mekanism:clean_iron", 1_000);
        let mut clean_out = Tank::default();
        assert_eq!(
            process_wash_tick(
                &mut machine,
                WASHER_OPERATION,
                &mut water,
                &mut clean_in,
                &mut clean_out,
                WASHER_SLURRY,
                &washing::recipe_for,
            ),
            Outcome::Idle(IdleReason::NoRecipe)
        );
        assert_eq!(machine.progress, 0);
        assert_eq!(machine.energy, WASHER_CAPACITY);
        assert_eq!(water.held(), Some((WATER, WASHER_WATER)));
    }

    /// No energy holds the progress: nothing is spent, and the tanks stay.
    #[test]
    fn a_washer_without_energy_holds_its_progress() {
        let mut machine = Machine {
            energy: WASHER_USAGE - 1,
            progress: 60,
        };
        let mut water = FluidTank::new(WATER, WASHER_WATER);
        let mut dirty = Tank::new("mekanism:dirty_iron", 1_000);
        let mut clean = Tank::default();
        assert_eq!(
            process_wash_tick(
                &mut machine,
                WASHER_OPERATION,
                &mut water,
                &mut dirty,
                &mut clean,
                WASHER_SLURRY,
                &washing::recipe_for,
            ),
            Outcome::Idle(IdleReason::NoEnergy)
        );
        assert_eq!(machine.progress, 60, "held, not reset");
        assert_eq!(machine.energy, WASHER_USAGE - 1);
        assert_eq!(water.held(), Some((WATER, WASHER_WATER)));
    }

    /// A full output tank (or one holding another chemical) holds the progress
    /// and spends nothing, as `NOT_ENOUGH_OUTPUT_SPACE` does.
    #[test]
    fn a_washer_with_no_output_room_holds_its_progress() {
        for held in ["mekanism:clean_iron", "mekanism:clean_gold"] {
            let mut machine = Machine {
                energy: WASHER_CAPACITY,
                progress: 60,
            };
            let mut water = FluidTank::new(WATER, WASHER_WATER);
            let mut dirty = Tank::new("mekanism:dirty_iron", 1_000);
            let mut clean = Tank::new(held, WASHER_SLURRY);
            assert_eq!(
                process_wash_tick(
                    &mut machine,
                    WASHER_OPERATION,
                    &mut water,
                    &mut dirty,
                    &mut clean,
                    WASHER_SLURRY,
                    &washing::recipe_for,
                ),
                Outcome::Idle(IdleReason::NoOutputSpace),
                "{held}"
            );
            assert_eq!(machine.progress, 60, "{held}: held, not reset");
            assert_eq!(machine.energy, WASHER_CAPACITY);
        }
    }

    /// The water tank takes water and nothing else: upstream's tank predicate is
    /// `containsRecipeAB(fluid, ...)`, and every washing row takes water.
    #[test]
    fn the_water_tank_takes_water_only() {
        let mut water = FluidTank::default();
        assert_eq!(
            insert_fluid(&mut water, WATER, WASHER_WATER + 1, WASHER_WATER),
            WASHER_WATER,
            "capped by the tank, not by the offer"
        );
        assert_eq!(water.held(), Some((WATER, WASHER_WATER)));
        assert_eq!(
            insert_fluid(&mut water, "minecraft:lava", 1_000, WASHER_WATER),
            0,
            "a fluid no washing recipe takes does not go in"
        );
        assert_eq!(
            insert_fluid(&mut water, "mekanism:dirty_iron", 1_000, WASHER_WATER),
            0,
            "a chemical is not a fluid"
        );
        let mut fresh = FluidTank::default();
        assert_eq!(insert_fluid(&mut fresh, WATER, 4, WASHER_WATER), 4);
        assert_eq!(fresh.held(), Some((WATER, 4)));
    }

    /// The washer pulls one **operation's** dirty slurry from a neighbour, and
    /// asks for nothing while its own tank already holds that much (or holds a
    /// slurry no row takes).
    #[test]
    fn the_washer_pulls_one_operation_s_slurry_and_no_more() {
        let recipe = washing::recipe_for("mekanism:dirty_iron").expect("the iron row");
        assert_eq!(
            washer_pull(&recipe, &Tank::default()),
            Some(("mekanism:dirty_iron", 1)),
            "one operation's unit, not one tick's"
        );
        assert_eq!(
            washer_pull(&recipe, &Tank::new("mekanism:dirty_iron", 1)),
            None,
            "a tank with the operation's worth is not topped up"
        );
        assert_eq!(
            washer_pull(&recipe, &Tank::new("mekanism:dirty_iron", 5)),
            None
        );
        assert_eq!(
            washer_pull(&recipe, &Tank::new("mekanism:dirty_gold", 1)),
            None,
            "another slurry is not drawn from either"
        );
    }

    /// The three tanks each keep their own contents: a spent water tank leaves
    /// the slurry tanks alone and the other way round.
    #[test]
    fn the_three_tanks_are_separate() {
        let mut water = FluidTank::new(WATER, 500);
        let mut dirty = Tank::new("mekanism:dirty_iron", 10);
        let mut clean = Tank::new("mekanism:clean_gold", 20);
        let mut machine = Machine {
            energy: WASHER_CAPACITY,
            progress: 0,
        };
        assert_eq!(
            process_wash_tick(
                &mut machine,
                WASHER_OPERATION,
                &mut water,
                &mut dirty,
                &mut clean,
                WASHER_SLURRY,
                &washing::recipe_for,
            ),
            Outcome::Idle(IdleReason::NoOutputSpace),
            "a tank of another clean slurry has no room for this one"
        );
        assert_eq!(water.held(), Some((WATER, 500)));
        assert_eq!(dirty.held(), Some(("mekanism:dirty_iron", 10)));
        assert_eq!(clean.held(), Some(("mekanism:clean_gold", 20)));
    }
}

// ---------------------------------------------------------------------------
// Slice 9: the Chemical Crystallizer — the slurries' third machine
// ---------------------------------------------------------------------------

#[cfg(test)]
mod crystallizer_tests {
    use super::*;

    /// The crystallizer's numbers are the jar's own (`UsageConfig.chemicalCrystallizer`
    /// 400 FE/t, `StorageConfig.chemicalCrystallizer` 160,000 FE,
    /// `TileEntityChemicalCrystallizer.BASE_TICKS_REQUIRED` 200 ticks,
    /// `MAX_CHEMICAL` 10,000 units), not another machine's.
    #[test]
    fn the_crystallizer_is_the_jar_s_400_fe_a_tick_and_its_160_000_buffer() {
        assert_eq!(CRYSTALLIZER_USAGE, 400, "UsageConfig.chemicalCrystallizer");
        assert_eq!(CRYSTALLIZER_CAPACITY, 160_000, "StorageConfig");
        assert_eq!(CRYSTALLIZER_TICKS, 200, "BASE_TICKS_REQUIRED");
        assert_eq!(CRYSTALLIZER_OPERATION.usage, 400);
        assert_eq!(CRYSTALLIZER_OPERATION.ticks, 200);
        assert_eq!(
            CRYSTALLIZER_OPERATION.usage * CRYSTALLIZER_OPERATION.ticks,
            80_000,
            "one operation is 80,000 FE, so the 160,000 buffer holds exactly two"
        );
        assert_eq!(CRYSTALLIZER_TANK, 10_000, "MAX_CHEMICAL = 10 x BUCKET");
    }

    /// The jar's seven carried crystallizing rows: 200 units of a material's
    /// clean slurry make one of its crystals, one operation each. These are
    /// exactly the slurries the washer can make, so no washer row dead-ends.
    #[test]
    fn every_crystallizing_row_is_the_jar_s() {
        assert_eq!(
            crystallizing::ROW_COUNT,
            7,
            "the seven slurries the washer makes, each with its crystal"
        );
        for material in ["copper", "gold", "iron", "lead", "osmium", "tin", "uranium"] {
            let slurry = format!("mekanism:clean_{material}");
            let crystal = format!("mekanism:crystal_{material}");
            let recipe =
                crystallizing::recipe_for(&slurry).unwrap_or_else(|| panic!("no row for {slurry}"));
            assert_eq!(recipe.chemical, slurry);
            assert_eq!(recipe.amount, 200, "the jar's input amount");
            assert_eq!(recipe.output, crystal);
            assert_eq!(recipe.output_count, 1);
        }
        assert_eq!(
            crystallizing::recipe_for("mekanism:dirty_iron"),
            None,
            "a dirty slurry is the washer's input, not the crystallizer's"
        );
        assert_eq!(crystallizing::recipe_for("mekanism:hydrogen"), None);
        assert_eq!(
            crystallizing::slurries().collect::<Vec<_>>(),
            [
                "mekanism:clean_copper",
                "mekanism:clean_gold",
                "mekanism:clean_iron",
                "mekanism:clean_lead",
                "mekanism:clean_osmium",
                "mekanism:clean_tin",
                "mekanism:clean_uranium"
            ],
            "the draw asks for the table's own slurries, in its order"
        );
    }

    /// Slice 10: the washer's last four slurries have their crystals, so the
    /// jar's crystallizing table leaves out only its three non-slurry rows
    /// (lithium, brine and antimatter), whose output items this port has not.
    #[test]
    fn the_four_new_crystals_complete_every_slurry_the_washer_makes() {
        for material in ["lead", "osmium", "tin", "uranium"] {
            let slurry = format!("mekanism:clean_{material}");
            let crystal = format!("mekanism:crystal_{material}");
            let recipe = crystallizing::recipe_for(&slurry)
                .unwrap_or_else(|| panic!("no row for {slurry}: the washer can make it"));
            assert_eq!(recipe.chemical, slurry);
            assert_eq!(recipe.amount, 200, "the jar's input amount");
            assert_eq!(recipe.output, crystal);
            assert_eq!(recipe.output_count, 1);
            assert!(
                crystallizing::accepts(&slurry),
                "{slurry} is a crystallizing input, so the tank takes it"
            );
        }
    }

    /// The crystallizer's tank takes a chemical with a row and refuses the rest
    /// (upstream's `containsRecipe` predicate), so a dirty slurry never enters.
    #[test]
    fn only_a_slurry_with_a_row_enters_the_crystallizer_s_tank() {
        assert!(crystallizing::recipe_for("mekanism:clean_iron").is_some());
        assert!(known_slurry("mekanism:clean_iron").is_some());
        for refused in [
            "mekanism:dirty_iron",
            "mekanism:hydrogen",
            "minecraft:water",
        ] {
            assert_eq!(
                crystallizing::recipe_for(refused),
                None,
                "{refused} has no crystallizing row, so the tank refuses it"
            );
        }
    }

    /// One operation at the jar's price: 200 ticks x 400 FE, 200 units of clean
    /// slurry in, one crystal out — and the tank and the output each move by
    /// exactly that.
    #[test]
    fn one_crystallizing_operation_costs_80_000_fe_and_200_units() {
        let mut machine = Machine {
            energy: CRYSTALLIZER_CAPACITY,
            progress: 0,
        };
        let mut tank = Tank::new("mekanism:clean_iron", 1_000);
        let mut output = ItemAmount::default();
        let mut outcomes = Vec::new();
        for _ in 0..CRYSTALLIZER_TICKS {
            outcomes.push(process_crystal_tick(
                &mut machine,
                CRYSTALLIZER_OPERATION,
                &mut tank,
                &mut output,
                64,
                &crystallizing::recipe_for,
            ));
        }
        assert!(
            outcomes[..199]
                .iter()
                .all(|outcome| *outcome == Outcome::Working),
            "every tick but the last only works"
        );
        assert_eq!(*outcomes.last().unwrap(), Outcome::Produced);
        assert_eq!(
            machine.energy,
            CRYSTALLIZER_CAPACITY - 80_000,
            "200 ticks x 400 FE"
        );
        assert_eq!(machine.progress, 0, "the operation finished");
        assert_eq!(
            tank.held(),
            Some(("mekanism:clean_iron", 800)),
            "200 units an operation, spent at its finish, not 200 a tick"
        );
        assert_eq!(output, ItemAmount::new("mekanism:crystal_iron", 1));
    }

    /// The slurry is spent when the operation finishes, as upstream's
    /// `OneInputCachedRecipe.finishProcessing` does: a mid-operation machine
    /// still holds all 200 units it drew.
    #[test]
    fn the_slurry_is_spent_at_the_finish_not_per_tick() {
        let mut machine = Machine {
            energy: CRYSTALLIZER_CAPACITY,
            progress: 0,
        };
        let mut tank = Tank::new("mekanism:clean_iron", 200);
        let mut output = ItemAmount::default();
        for _ in 0..199 {
            process_crystal_tick(
                &mut machine,
                CRYSTALLIZER_OPERATION,
                &mut tank,
                &mut output,
                64,
                &crystallizing::recipe_for,
            );
        }
        assert_eq!(machine.progress, 199);
        assert_eq!(
            tank.held(),
            Some(("mekanism:clean_iron", 200)),
            "the operation has not finished, so nothing is taken"
        );
        assert!(output.is_empty());
    }

    /// A dirty slurry is a chemical this machine has no row for: the progress
    /// resets and neither the slurry nor the energy moves. This is the
    /// counterfactual the in-game leg repeats.
    #[test]
    fn a_crystallizer_with_a_dirty_slurry_resets_and_spends_nothing() {
        let mut machine = Machine {
            energy: CRYSTALLIZER_CAPACITY,
            progress: 60,
        };
        let mut tank = Tank::new("mekanism:dirty_iron", 1_000);
        let mut output = ItemAmount::default();
        assert_eq!(
            process_crystal_tick(
                &mut machine,
                CRYSTALLIZER_OPERATION,
                &mut tank,
                &mut output,
                64,
                &crystallizing::recipe_for,
            ),
            Outcome::Idle(IdleReason::NoRecipe)
        );
        assert_eq!(machine.progress, 0, "no recipe resets the progress");
        assert_eq!(machine.energy, CRYSTALLIZER_CAPACITY, "and spends nothing");
        assert_eq!(tank, Tank::new("mekanism:dirty_iron", 1_000));
        assert!(output.is_empty());
    }

    /// An empty tank is upstream's `mismatchedRecipe`: the progress resets
    /// rather than holding, as the item machines' empty input does.
    #[test]
    fn an_empty_crystallizer_tank_resets_the_progress() {
        let mut machine = Machine {
            energy: CRYSTALLIZER_CAPACITY,
            progress: 60,
        };
        let mut tank = Tank::default();
        let mut output = ItemAmount::default();
        assert_eq!(
            process_crystal_tick(
                &mut machine,
                CRYSTALLIZER_OPERATION,
                &mut tank,
                &mut output,
                64,
                &crystallizing::recipe_for,
            ),
            Outcome::Idle(IdleReason::NoChemicalInput)
        );
        assert_eq!(machine.progress, 0);
        assert_eq!(machine.energy, CRYSTALLIZER_CAPACITY);
    }

    /// Short of one operation's slurry is `NOT_ENOUGH_INPUT`: it resets too,
    /// and not a unit is taken.
    #[test]
    fn a_crystallizer_short_of_slurry_resets_too() {
        let mut machine = Machine {
            energy: CRYSTALLIZER_CAPACITY,
            progress: 60,
        };
        let mut tank = Tank::new("mekanism:clean_iron", 199);
        let mut output = ItemAmount::default();
        assert_eq!(
            process_crystal_tick(
                &mut machine,
                CRYSTALLIZER_OPERATION,
                &mut tank,
                &mut output,
                64,
                &crystallizing::recipe_for,
            ),
            Outcome::Idle(IdleReason::NotEnoughInput)
        );
        assert_eq!(machine.progress, 0);
        assert_eq!(machine.energy, CRYSTALLIZER_CAPACITY);
        assert_eq!(tank.held(), Some(("mekanism:clean_iron", 199)));
    }

    /// No energy holds the progress (it does not reset it), and no output room
    /// holds it too: the machine has started an operation it cannot finish.
    #[test]
    fn no_energy_and_no_output_room_hold_the_crystallizer() {
        let mut machine = Machine {
            energy: 399,
            progress: 60,
        };
        let mut tank = Tank::new("mekanism:clean_iron", 1_000);
        let mut output = ItemAmount::default();
        assert_eq!(
            process_crystal_tick(
                &mut machine,
                CRYSTALLIZER_OPERATION,
                &mut tank,
                &mut output,
                64,
                &crystallizing::recipe_for,
            ),
            Outcome::Idle(IdleReason::NoEnergy)
        );
        assert_eq!(machine.progress, 60, "a short buffer holds the progress");
        assert_eq!(tank.held(), Some(("mekanism:clean_iron", 1_000)));

        let mut machine = Machine {
            energy: CRYSTALLIZER_CAPACITY,
            progress: 60,
        };
        let mut full = ItemAmount::new("mekanism:crystal_gold", 64);
        assert_eq!(
            process_crystal_tick(
                &mut machine,
                CRYSTALLIZER_OPERATION,
                &mut tank,
                &mut full,
                64,
                &crystallizing::recipe_for,
            ),
            Outcome::Idle(IdleReason::NoOutputSpace),
            "a full stack of another crystal has no room for this one"
        );
        assert_eq!(machine.progress, 60);
        assert_eq!(
            machine.energy, CRYSTALLIZER_CAPACITY,
            "and no energy is spent"
        );
        assert_eq!(full, ItemAmount::new("mekanism:crystal_gold", 64));
    }

    /// The neighbour draw asks for one operation's worth of the slurry it is
    /// about to spend, and takes nothing when the tank already holds it.
    #[test]
    fn the_crystallizer_draws_one_operation_s_slurry() {
        let recipe = crystallizing::recipe_for("mekanism:clean_iron").expect("a clean-iron row");
        assert_eq!(
            crystallizer_pull(&recipe, &Tank::default()),
            Some(("mekanism:clean_iron", 200)),
            "an empty tank asks for a whole operation"
        );
        assert_eq!(
            crystallizer_pull(&recipe, &Tank::new("mekanism:clean_iron", 50)),
            Some(("mekanism:clean_iron", 150)),
            "a part-filled tank is topped up to one operation"
        );
        assert_eq!(
            crystallizer_pull(&recipe, &Tank::new("mekanism:clean_iron", 200)),
            None,
            "a tank that already holds the operation draws nothing"
        );
        assert_eq!(
            crystallizer_pull(&recipe, &Tank::new("mekanism:dirty_iron", 1_000)),
            None,
            "another slurry is not drawn from"
        );
    }

    /// The tank decides what it takes (upstream's `containsRecipe` predicate):
    /// a chemical with no row is refused by name, so a dirty slurry never fills
    /// it, while any carried clean slurry does.
    #[test]
    fn the_crystallizer_s_tank_refuses_a_chemical_without_a_row() {
        assert!(crystallizing::accepts("mekanism:clean_iron"));
        assert!(crystallizing::accepts("mekanism:clean_gold"));
        assert!(crystallizing::accepts("mekanism:clean_copper"));
        assert!(!crystallizing::accepts("mekanism:dirty_iron"));
        assert!(
            crystallizing::accepts("mekanism:clean_lead"),
            "slice 10 carries the lead crystal, so its row exists"
        );
        assert!(!crystallizing::accepts("mekanism:hydrogen"));
        assert!(!crystallizing::accepts("minecraft:water"));
    }

    /// The slurries the tank asks a neighbour for: only those whose crystal the
    /// output slot could take.
    ///
    /// This is the 0.2.8 e2e's N1 stall as a rule. The empty-tank loop used the
    /// room check as a **loop-wide** verdict, so a slot of `crystal_iron`
    /// stopped it at the table's first row (`clean_copper`, a different item)
    /// and the machine never reached `clean_iron`: one operation, then 1,300
    /// ticks of nothing while the neighbour tank stayed at 800. A candidate the
    /// slot cannot take is skipped, not treated as "nothing fits".
    #[test]
    fn a_slot_of_one_crystal_asks_for_that_materials_slurry_and_skips_the_rest() {
        let one_iron = ItemAmount::new("mekanism:crystal_iron", 1);
        assert_eq!(
            crystallizing::slurries_that_fit(&one_iron, 64).collect::<Vec<_>>(),
            vec!["mekanism:clean_iron"],
            "a slot holding crystal_iron takes clean_iron, and reports no other row"
        );
        // Slice 10's four rows sit after iron in the table, so a lead slot is
        // the case that would starve if the room check were loop-wide again.
        assert_eq!(
            crystallizing::slurries_that_fit(&ItemAmount::new("mekanism:crystal_lead", 1), 64)
                .collect::<Vec<_>>(),
            vec!["mekanism:clean_lead"],
            "a slot holding crystal_lead takes clean_lead, past the earlier rows"
        );
        assert_eq!(
            crystallizing::slurries_that_fit(&ItemAmount::default(), 64).count(),
            crystallizing::ROW_COUNT,
            "an empty slot takes any row's crystal"
        );
        assert_eq!(
            crystallizing::slurries_that_fit(&ItemAmount::new("mekanism:crystal_iron", 64), 64)
                .count(),
            0,
            "a full stack takes no more, so a stalled crystallizer siphons nothing"
        );
        assert_eq!(
            crystallizing::slurries_that_fit(&ItemAmount::new("minecraft:diamond", 1), 64).count(),
            0,
            "a slot of another item takes no crystal at all"
        );
    }
}

#[cfg(test)]
mod dissolution_tests {
    use super::*;

    /// The chamber's numbers are the jar's own
    /// (`UsageConfig.chemicalDissolutionChamber` 400 FE/t,
    /// `StorageConfig.chemicalDissolutionChamber` 160,000 FE,
    /// `TileEntityChemicalDissolutionChamber.BASE_TICKS_REQUIRED` 100 ticks,
    /// `MAX_CHEMICAL` 10,000 units), not the shared 200-tick machine's.
    #[test]
    fn the_chamber_is_the_jar_s_400_fe_a_tick_its_160_000_buffer_and_100_ticks() {
        assert_eq!(
            DISSOLUTION_USAGE, 400,
            "UsageConfig.chemicalDissolutionChamber"
        );
        assert_eq!(
            DISSOLUTION_CAPACITY, 160_000,
            "StorageConfig.chemicalDissolutionChamber"
        );
        assert_eq!(
            DISSOLUTION_TICKS, 100,
            "BASE_TICKS_REQUIRED = 5 x TICKS_PER_SECOND"
        );
        assert_eq!(DISSOLUTION_OPERATION.usage, 400);
        assert_eq!(DISSOLUTION_OPERATION.ticks, 100);
        assert_eq!(
            DISSOLUTION_OPERATION.usage * DISSOLUTION_OPERATION.ticks,
            40_000,
            "one operation is 40,000 FE, so the 160,000 buffer holds exactly four"
        );
        assert_eq!(DISSOLUTION_TANK, 10_000, "MAX_CHEMICAL = 10 x BUCKET");
    }

    /// The jar's 21 dissolution recipes over this port's items: twelve carry
    /// (three vanilla ores x their two 26.3 members, and the raw ore / raw block
    /// rows for iron, gold and copper), nine drop with lead, osmium, tin and
    /// uranium, whose ores 26.3 does not have.
    #[test]
    fn every_carried_dissolution_row_is_the_jar_s() {
        assert_eq!(
            dissolving::ROW_COUNT,
            12,
            "6 ore rows (3 ores x 2 vanilla members) + 3 raw ore + 3 raw block"
        );
        for (input, count, acid, output, amount) in [
            ("minecraft:iron_ore", 1, 1, "mekanism:dirty_iron", 1_000),
            (
                "minecraft:deepslate_iron_ore",
                1,
                1,
                "mekanism:dirty_iron",
                1_000,
            ),
            ("minecraft:gold_ore", 1, 1, "mekanism:dirty_gold", 1_000),
            (
                "minecraft:deepslate_gold_ore",
                1,
                1,
                "mekanism:dirty_gold",
                1_000,
            ),
            ("minecraft:copper_ore", 1, 1, "mekanism:dirty_copper", 1_000),
            (
                "minecraft:deepslate_copper_ore",
                1,
                1,
                "mekanism:dirty_copper",
                1_000,
            ),
            ("minecraft:raw_iron", 3, 1, "mekanism:dirty_iron", 2_000),
            ("minecraft:raw_gold", 3, 1, "mekanism:dirty_gold", 2_000),
            ("minecraft:raw_copper", 3, 1, "mekanism:dirty_copper", 2_000),
            (
                "minecraft:raw_iron_block",
                1,
                2,
                "mekanism:dirty_iron",
                6_000,
            ),
            (
                "minecraft:raw_gold_block",
                1,
                2,
                "mekanism:dirty_gold",
                6_000,
            ),
            (
                "minecraft:raw_copper_block",
                1,
                2,
                "mekanism:dirty_copper",
                6_000,
            ),
        ] {
            let recipe = dissolving::recipe_for(input)
                .unwrap_or_else(|| panic!("no dissolution row for {input}"));
            assert_eq!(recipe.input, count, "{input}: the jar's item count");
            assert_eq!(recipe.acid_per_tick, acid, "{input}: acid per tick");
            assert_eq!(recipe.output, output, "{input}: the jar's slurry");
            assert_eq!(
                recipe.output_amount, amount,
                "{input}: slurry per operation"
            );
        }
        for dropped in [
            "minecraft:dirt",
            "mekanism:iron_ore",
            "minecraft:nether_gold_ore",
        ] {
            assert_eq!(
                dissolving::recipe_for(dropped),
                None,
                "{dropped} has no carried dissolution row"
            );
        }
    }

    /// One operation at the jar's price: 100 ticks x 400 FE, 100 units of acid
    /// (1 a tick x 100 ticks), one ore in, 1,000 units of dirty slurry out —
    /// and each of the energy, the acid and the tank moves by exactly that.
    #[test]
    fn one_dissolution_operation_costs_40_000_fe_and_100_acid() {
        let mut machine = Machine {
            energy: DISSOLUTION_CAPACITY,
            progress: 0,
        };
        let mut acid = Tank::new(SULFURIC_ACID, 1_000);
        let mut slurry = Tank::default();
        let mut input = ItemAmount::new("minecraft:iron_ore", 1);
        let mut outcomes = Vec::new();
        for _ in 0..DISSOLUTION_TICKS {
            outcomes.push(process_dissolve_tick(
                &mut machine,
                DISSOLUTION_OPERATION,
                &mut acid,
                &mut slurry,
                &mut input,
                &dissolving::recipe_for,
            ));
        }
        assert!(
            outcomes[..99].iter().all(|o| *o == Outcome::Working),
            "the first 99 ticks are work"
        );
        assert_eq!(outcomes[99], Outcome::Produced);
        assert_eq!(machine.energy, DISSOLUTION_CAPACITY - 40_000);
        assert_eq!(machine.progress, 0);
        assert_eq!(acid.held(), Some((SULFURIC_ACID, 900)), "100 ticks x 1");
        assert_eq!(slurry.held(), Some(("mekanism:dirty_iron", 1_000)));
        assert!(
            input.is_empty(),
            "the ore is consumed when the operation ends"
        );
    }

    /// A chemical the recipe does not take — water in the acid tank — makes no
    /// progress and spends nothing: the tank holds what it held and the buffer
    /// stays full.
    #[test]
    fn a_chemical_the_recipe_does_not_take_holds_the_progress_and_spends_nothing() {
        let mut machine = Machine {
            energy: DISSOLUTION_CAPACITY,
            progress: 0,
        };
        let mut acid = Tank::new("minecraft:water", 1_000);
        let mut slurry = Tank::default();
        let mut input = ItemAmount::new("minecraft:iron_ore", 1);
        for _ in 0..DISSOLUTION_TICKS {
            assert_eq!(
                process_dissolve_tick(
                    &mut machine,
                    DISSOLUTION_OPERATION,
                    &mut acid,
                    &mut slurry,
                    &mut input,
                    &dissolving::recipe_for,
                ),
                Outcome::Idle(IdleReason::NoChemical)
            );
        }
        assert_eq!(machine.energy, DISSOLUTION_CAPACITY, "no FE is spent");
        assert_eq!(machine.progress, 0);
        assert_eq!(acid.amount, 1_000);
        assert!(slurry.held().is_none());
        assert_eq!(input.count, 1);
    }

    /// Too little room in the output tank holds the progress (upstream's
    /// `NOT_ENOUGH_OUTPUT_SPACE`), and so does an input with no row, which
    /// resets it.
    #[test]
    fn a_full_output_tank_holds_and_an_input_without_a_row_resets() {
        let mut machine = Machine {
            energy: DISSOLUTION_CAPACITY,
            progress: 3,
        };
        let mut acid = Tank::new(SULFURIC_ACID, 1_000);
        let mut input = ItemAmount::new("minecraft:iron_ore", 1);
        // 9,500 of another material leaves 500 units of room: less than one row.
        let mut full = Tank::new("mekanism:dirty_gold", 9_500);
        assert_eq!(
            process_dissolve_tick(
                &mut machine,
                DISSOLUTION_OPERATION,
                &mut acid,
                &mut full,
                &mut input,
                &dissolving::recipe_for,
            ),
            Outcome::Idle(IdleReason::NoOutputSpace)
        );
        assert_eq!(machine.progress, 3, "a full tank holds the progress");
        assert_eq!(machine.energy, DISSOLUTION_CAPACITY);

        let mut slurry = Tank::default();
        let mut dirt = ItemAmount::new("minecraft:dirt", 1);
        machine.progress = 5;
        assert_eq!(
            process_dissolve_tick(
                &mut machine,
                DISSOLUTION_OPERATION,
                &mut acid,
                &mut slurry,
                &mut dirt,
                &dissolving::recipe_for,
            ),
            Outcome::Idle(IdleReason::NoRecipe)
        );
        assert_eq!(machine.progress, 0, "no recipe resets the progress");

        // And too few raw ore (the row takes three) resets it as well.
        let mut raw = ItemAmount::new("minecraft:raw_iron", 2);
        machine.progress = 7;
        assert_eq!(
            process_dissolve_tick(
                &mut machine,
                DISSOLUTION_OPERATION,
                &mut acid,
                &mut slurry,
                &mut raw,
                &dissolving::recipe_for,
            ),
            Outcome::Idle(IdleReason::NotEnoughInput)
        );
        assert_eq!(machine.progress, 0);
    }

    /// The acid a tick asks a neighbour for is one tick's worth, never the
    /// operation's: 1 unit for an ore, 2 for a raw block, and nothing when the
    /// chamber's own tank already holds that tick.
    #[test]
    fn the_acid_pull_is_one_tick_s_worth() {
        let ore = dissolving::recipe_for("minecraft:iron_ore").expect("a row");
        let block = dissolving::recipe_for("minecraft:raw_iron_block").expect("a row");
        assert_eq!(
            dissolve_pull(&ore, &Tank::default()),
            Some((SULFURIC_ACID, 1))
        );
        assert_eq!(
            dissolve_pull(&block, &Tank::default()),
            Some((SULFURIC_ACID, 2))
        );
        assert_eq!(
            dissolve_pull(&ore, &Tank::new(SULFURIC_ACID, 1)),
            None,
            "a tank with a tick's acid asks for nothing"
        );
        assert_eq!(
            dissolve_pull(&ore, &Tank::new("minecraft:water", 1_000)),
            None,
            "a tank of another chemical is not topped up"
        );
    }
}
