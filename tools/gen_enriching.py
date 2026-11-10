#!/usr/bin/env python3
"""Generate the Enrichment Chamber's recipe table in `mixins/mekanism/runtime.rs`.

Mekanism's `mekanism:enriching` recipes are data: `data/mekanism/recipe/enriching/**`
and `data/mekanism/recipe/processing/<material>/dust/from_{ore,raw_ore,raw_block}.json`
of the pack's jar. Every ingredient is written as a NeoForge `c:` tag, and the
tag members are the pack's items; the port can only ever see the members 26.3
has (vanilla ores and raw forms), so `TAGS` below names exactly those. An
output the port does not register (a Mekanism item outside the slice, another
mod's item) drops its row instead of producing an item that does not exist.

Rows are `(input item, items consumed per operation, output item, output count)`.
An input the jar writes as one item id (`{"item": ...}`) keeps its own count.

Usage:
  python3 tools/gen_enriching.py [--jar PATH] [--check]

`--check` reads the committed rows back (whatever rustfmt did to their layout)
and fails when they no longer match the jar.
"""
import argparse
import io
import json
import re
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
OUT = ROOT / "mixins" / "mekanism" / "runtime.rs"
BEGIN = "    // BEGIN GENERATED ENRICHING TABLE (tools/gen_enriching.py)\n"
END = "    // END GENERATED ENRICHING TABLE\n"

# The `c:` tags the jar's enriching recipes use, expanded to their 26.3
# members: the pack's tags also hold modded ores (`alltheores:iron_ore`...) and
# Mekanism's own items, which this slice does not register. `minecraft:` tags
# are vanilla and are not listed.
TAGS: dict[str, list[str]] = {
    "c:ores/coal": ["minecraft:coal_ore", "minecraft:deepslate_coal_ore"],
    "c:ores/copper": ["minecraft:copper_ore", "minecraft:deepslate_copper_ore"],
    "c:ores/diamond": ["minecraft:diamond_ore", "minecraft:deepslate_diamond_ore"],
    "c:ores/emerald": ["minecraft:emerald_ore", "minecraft:deepslate_emerald_ore"],
    "c:ores/gold": ["minecraft:gold_ore", "minecraft:deepslate_gold_ore"],
    "c:ores/iron": ["minecraft:iron_ore", "minecraft:deepslate_iron_ore"],
    "c:ores/lapis": ["minecraft:lapis_ore", "minecraft:deepslate_lapis_ore"],
    "c:ores/netherite_scrap": ["minecraft:ancient_debris"],
    "c:ores/quartz": ["minecraft:nether_quartz_ore"],
    "c:ores/redstone": ["minecraft:redstone_ore", "minecraft:deepslate_redstone_ore"],
    "c:raw_materials/copper": ["minecraft:raw_copper"],
    "c:raw_materials/gold": ["minecraft:raw_gold"],
    "c:raw_materials/iron": ["minecraft:raw_iron"],
    "c:storage_blocks/raw_copper": ["minecraft:raw_copper_block"],
    "c:storage_blocks/raw_gold": ["minecraft:raw_gold_block"],
    "c:storage_blocks/raw_iron": ["minecraft:raw_iron_block"],
    "c:gravels": ["minecraft:gravel"],
    "c:sands": ["minecraft:sand", "minecraft:red_sand"],
    "c:gunpowders": ["minecraft:gunpowder"],
}

# The items the port registers: vanilla's own, plus the three dusts this slice
# declares in `content/data/mekanism/horizon/item/`.
DUSTS = ["mekanism:dust_copper", "mekanism:dust_gold", "mekanism:dust_iron"]


def registered_items() -> set[str]:
    """The non-vanilla items this slice registers (its three dusts)."""
    return set(DUSTS)


def ingredient_inputs(ingredient: dict) -> list[str]:
    if "item" in ingredient:
        return [ingredient["item"]]
    tag = ingredient.get("tag", "")
    if tag in TAGS:
        return TAGS[tag]
    return []


def rows(jar: Path) -> tuple[list[tuple[str, int, str, int]], int, list[str]]:
    """The table, how many enriching recipes the jar holds, and the skipped ones."""
    table: dict[str, tuple[int, str, int]] = {}
    recipes = 0
    skipped: list[str] = []
    known_outputs = registered_items()
    with zipfile.ZipFile(jar) as archive:
        for name in sorted(archive.namelist()):
            if not name.startswith("data/mekanism/recipe/") or not name.endswith(".json"):
                continue
            data = json.loads(archive.read(name))
            if data.get("type") != "mekanism:enriching":
                continue
            recipes += 1
            output = data["output"]["id"]
            count = int(data["output"].get("count", 1))
            inputs = ingredient_inputs(data["input"])
            inputs = [i for i in inputs if i.startswith("minecraft:") or i in known_outputs]
            if not inputs:
                skipped.append(f"{name}: no 26.3 member of {data['input']}")
                continue
            if not output.startswith("minecraft:") and output not in known_outputs:
                skipped.append(f"{name}: output {output} is not registered")
                continue
            consumed = int(data["input"].get("count", 1))
            for item in inputs:
                if item in table and table[item] != (consumed, output, count):
                    raise ValueError(f"{item} got two enriching recipes: {table[item]} and {(consumed, output, count)}")
                table[item] = (consumed, output, count)
    return (
        sorted((item, consumed, output, count) for item, (consumed, output, count) in table.items()),
        recipes,
        skipped,
    )


def render(jar: Path) -> str:
    table, recipes, skipped = rows(jar)
    header = (
        "    // Generated from the jar's `mekanism:enriching` recipes by\n"
        "    // `tools/gen_enriching.py`; do not edit by hand. Each row is\n"
        "    // `(input item, items consumed, output item, output count)`; a `c:`\n"
        "    // tag is expanded to the members 26.3 has (see the tool's TAGS), and\n"
        "    // a recipe whose output the port does not register is left out.\n"
        f"    // Source recipes: {recipes}. Rows: {len(table)}. Skipped: {len(skipped)}.\n"
        "    static TABLE: &[(&str, u32, &str, u32)] = &[\n"
    )
    body = "".join(f'        ("{i}", {c}, "{o}", {n}),\n' for i, c, o, n in table)
    return header + body + "    ];\n"


def replace(text: str, block: str) -> str:
    start = text.index(BEGIN)
    end = text.index(END)
    return text[: start + len(BEGIN)] + block + text[end:]


def committed_rows(text: str) -> list[tuple[str, int, str, int]]:
    block = text[text.index(BEGIN) + len(BEGIN) : text.index(END)]
    return [
        (item, int(consumed), output, int(count))
        for item, consumed, output, count in re.findall(
            r'"([^"]+)"\s*,\s*(\d+)\s*,\s*"([^"]+)"\s*,\s*(\d+)\s*\)?\s*,', block
        )
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, default=DEFAULT_JAR)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    text = OUT.read_text()
    expected, recipes, skipped = rows(args.jar)
    if args.check:
        committed = committed_rows(text)
        if committed != expected:
            print(
                f"{OUT} is stale: {len(committed)} rows committed, {len(expected)} "
                f"from the {recipes} enriching recipes"
            )
            return 1
        print(f"{OUT} is up to date ({len(expected)} rows from {recipes} recipes, {len(skipped)} skipped)")
        return 0
    OUT.write_text(replace(text, render(args.jar)))
    print(f"updated {OUT}: {len(expected)} rows from {recipes} recipes, {len(skipped)} skipped")
    for line in skipped:
        print(f"  skipped {line}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
