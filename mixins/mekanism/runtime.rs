// The pure core of the Mekanism slices 1-2 port (clean-room rewrite).
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
    use super::Stack;

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
    /// port carries the three dusts this slice registers.
    static MEKANISM_ROWS: &[(&str, &str, u32)] = &[
        ("mekanism:dust_copper", "minecraft:copper_ingot", 1),
        ("mekanism:dust_gold", "minecraft:gold_ingot", 1),
        ("mekanism:dust_iron", "minecraft:iron_ingot", 1),
    ];

    /// The result of smelting `item`, `None` when nothing smelts it.
    ///
    /// Every vanilla smelting recipe takes one item. The jar's own
    /// `minecraft:smelting` recipes (`processing/<material>/ingot/
    /// from_dust_smelting.json`) are part of the same recipe type in the pack,
    /// so [`MEKANISM_ROWS`] adds the dusts this port registers.
    #[must_use]
    pub fn lookup(item: &str) -> Option<super::Recipe> {
        let one = |output: &str, count: u32| {
            Some(super::Recipe {
                input: 1,
                output: Stack::new(output, count),
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
    use super::{Recipe, Stack};

    // BEGIN GENERATED ENRICHING TABLE (tools/gen_enriching.py)
    // Generated from the jar's `mekanism:enriching` recipes by
    // `tools/gen_enriching.py`; do not edit by hand. Each row is
    // `(input item, items consumed, output item, output count)`; a `c:`
    // tag is expanded to the members 26.3 has (see the tool's TAGS), and
    // a recipe whose output the port does not register is left out.
    // Source recipes: 322. Rows: 129. Skipped: 202.
    static TABLE: &[(&str, u32, &str, u32)] = &[
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
    pub fn lookup(item: &str) -> Option<Recipe> {
        TABLE
            .binary_search_by(|row| row.0.cmp(item))
            .ok()
            .map(|index| Recipe {
                input: TABLE[index].1,
                output: Stack::new(TABLE[index].2, TABLE[index].3),
            })
    }

    /// How many recipe rows the table holds (the tests' size check).
    #[cfg(test)]
    #[must_use]
    pub fn len() -> usize {
        TABLE.len()
    }
}

/// An item stack as the pure core sees it: an id and a count.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Stack {
    /// The item id (`minecraft:iron_ingot`).
    pub item: String,
    /// How many of it.
    pub count: u32,
}

impl Stack {
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
    /// The output slot has no room for the recipe's result (held).
    NoOutputSpace,
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Recipe {
    /// Items taken from the input slot when the operation finishes.
    pub input: u32,
    /// The result, a stack of the recipe's item and count.
    pub output: Stack,
}

/// `energy` clamped into `0..=capacity`, as a buffer reads its saved int.
#[must_use]
pub fn clamp_energy(energy: i32, capacity: i32) -> i32 {
    energy.clamp(0, capacity.max(0))
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
    input: &mut Stack,
    output: &mut Stack,
    max_stack: u32,
    lookup: &dyn Fn(&str) -> Option<Recipe>,
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
    let room = if output.is_empty() {
        max_stack
    } else if output.item == recipe.output.item {
        max_stack.saturating_sub(output.count)
    } else {
        0
    };
    if room < recipe.output.count {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn stack(item: &str, count: u32) -> Stack {
        Stack::new(item, count)
    }

    fn one(item: &str, count: u32) -> Recipe {
        Recipe {
            input: 1,
            output: stack(item, count),
        }
    }

    /// The Energized Smelter's own operation, so these tests read as the
    /// machine they check; `process_tick` is the shared core.
    fn smelt_tick(
        machine: &mut Machine,
        input: &mut Stack,
        output: &mut Stack,
        max_stack: u32,
        lookup: &dyn Fn(&str) -> Option<Recipe>,
    ) -> Outcome {
        process_tick(machine, SMELTER_OPERATION, input, output, max_stack, lookup)
    }

    fn iron_ingot(_input: &str) -> Option<Recipe> {
        Some(one("minecraft:iron_ingot", 1))
    }

    fn nothing(_input: &str) -> Option<Recipe> {
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
        let mut output = Stack::default();
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
        let mut output = Stack::default();
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
        let mut output = Stack::default();
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
        let mut input = Stack::default();
        let mut output = Stack::default();
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
        let mut output = Stack::default();
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
        fn two(_input: &str) -> Option<Recipe> {
            Some(Recipe {
                input: 1,
                output: stack("minecraft:iron_ingot", 2),
            })
        }
        let mut smelter = Machine {
            energy: SMELTER_CAPACITY,
            progress: SMELTER_TICKS - 1,
        };
        let mut input = stack("minecraft:iron_ore", 1);
        let mut output = Stack::default();
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
        fn two(_input: &str) -> Option<Recipe> {
            Some(Recipe {
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
                Some(Recipe {
                    input: 3,
                    output: stack(dust, 4),
                }),
                "{raw}"
            );
            let block = format!("{raw}_block");
            assert_eq!(
                enriching::lookup(&block),
                Some(Recipe {
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
        let mut output = Stack::default();
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
        let mut output = Stack::default();
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
        let mut output = Stack::default();
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
}
