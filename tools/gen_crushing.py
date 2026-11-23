#!/usr/bin/env python3
"""Generate the Crusher's recipe table in `src/runtime.rs`.

Mekanism's `mekanism:crushing` recipes are data: `data/mekanism/recipe/crushing/**`,
the jar's `processing/<material>/dust/from_ingot.json` (an ingot into its dust) and
`processing/<material>/dirty_dust/from_clump.json` (a clump into dirty dust). Every
ingredient written as a `c:` tag is expanded to the members 26.3 has, from `TAGS`
below; the pack's tags also hold modded items and Mekanism's own items, which this
slice does not register. An output that is neither a vanilla item nor one of the
port's own dusts drops its row, so the machine never answers a recipe it could not
finish.

Rows are `(input item, items consumed per operation, output item, output count)`.

Usage:
  python3 tools/gen_crushing.py [--jar PATH] [--check] [--rows]

`--check` reads the committed rows back (whatever rustfmt did to their layout) and
fails when they no longer match the jar. `--rows` prints the rows as JSON, which
the in-game sweep (`tools/crush_sweep.py`) drives the real machine with, so the
scenario and the table cannot drift apart.
"""
import argparse
import json
import re
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
OUT = ROOT / "src" / "runtime.rs"
BEGIN = "    // BEGIN GENERATED CRUSHING TABLE (tools/gen_crushing.py)\n"
END = "    // END GENERATED CRUSHING TABLE\n"

WOOLS = [
    "white", "orange", "magenta", "light_blue", "yellow", "lime", "pink", "gray",
    "light_gray", "cyan", "purple", "blue", "brown", "green", "red", "black",
]

# The `c:` tags the jar's crushing recipes name, expanded to their 26.3 members.
# `minecraft:` tags are vanilla and are listed too where the jar names one.
TAGS: dict[str, list[str]] = {
    "c:cobblestones/normal": ["minecraft:cobblestone"],
    "c:gravels": ["minecraft:gravel"],
    "c:sandstone/uncolored_blocks": [
        "minecraft:sandstone",
        "minecraft:cut_sandstone",
        "minecraft:chiseled_sandstone",
        "minecraft:smooth_sandstone",
    ],
    "c:sandstone/red_blocks": [
        "minecraft:red_sandstone",
        "minecraft:cut_red_sandstone",
        "minecraft:chiseled_red_sandstone",
        "minecraft:smooth_red_sandstone",
    ],
    "minecraft:wool": [f"minecraft:{colour}_wool" for colour in WOOLS],
    "c:rods/blaze": ["minecraft:blaze_rod"],
    "c:rods/breeze": ["minecraft:breeze_rod"],
    "c:obsidians/normal": ["minecraft:obsidian", "minecraft:crying_obsidian"],
    "c:ingots/iron": ["minecraft:iron_ingot"],
    "c:ingots/gold": ["minecraft:gold_ingot"],
    "c:ingots/copper": ["minecraft:copper_ingot"],
    "c:clumps/copper": ["mekanism:clump_copper"],
    "c:clumps/gold": ["mekanism:clump_gold"],
    "c:clumps/iron": ["mekanism:clump_iron"],
    "c:clumps/lead": ["mekanism:clump_lead"],
    "c:clumps/osmium": ["mekanism:clump_osmium"],
    "c:clumps/tin": ["mekanism:clump_tin"],
    "c:clumps/uranium": ["mekanism:clump_uranium"],
}

# The items the port registers: vanilla's own, plus the three dusts of slices
# 1-3, the clumps and dirty dusts of slice 6 and slice 12's four extra metals
# (lead, osmium, tin, uranium), all declared in
# `data/mekanism/horizon/item/`. The clump rows (clump -> dirty dust) need both
# ends registered: the tag above names the input, this list the output.
DUSTS = [
    "mekanism:dust_copper",
    "mekanism:dust_gold",
    "mekanism:dust_iron",
    "mekanism:dust_lead",
    "mekanism:dust_osmium",
    "mekanism:dust_tin",
    "mekanism:dust_uranium",
]
CLUMPS = [
    "mekanism:clump_copper",
    "mekanism:clump_gold",
    "mekanism:clump_iron",
    "mekanism:clump_lead",
    "mekanism:clump_osmium",
    "mekanism:clump_tin",
    "mekanism:clump_uranium",
]
DIRTY_DUSTS = [
    "mekanism:dirty_dust_copper",
    "mekanism:dirty_dust_gold",
    "mekanism:dirty_dust_iron",
    "mekanism:dirty_dust_lead",
    "mekanism:dirty_dust_osmium",
    "mekanism:dirty_dust_tin",
    "mekanism:dirty_dust_uranium",
]


def registered_items() -> set[str]:
    """The non-vanilla items this port registers (dusts, clumps, dirty dusts)."""
    return set(DUSTS) | set(CLUMPS) | set(DIRTY_DUSTS)


def ingredient_inputs(ingredient: dict) -> list[str]:
    if "item" in ingredient:
        return [ingredient["item"]]
    if ingredient.get("type") == "neoforge:compound":
        # A union of ingredients (`polished_or_smooth_basalt_to_basalt`).
        members: list[str] = []
        for child in ingredient.get("children", []):
            members.extend(ingredient_inputs(child))
        return members
    tag = ingredient.get("tag", "")
    if tag in TAGS:
        return TAGS[tag]
    return []


def rows(jar: Path) -> tuple[list[tuple[str, int, str, int]], int, list[str]]:
    """The table, how many crushing recipes the jar holds, and the skipped ones."""
    table: dict[str, tuple[int, str, int]] = {}
    recipes = 0
    skipped: list[str] = []
    known_outputs = registered_items()
    with zipfile.ZipFile(jar) as archive:
        for name in sorted(archive.namelist()):
            if not name.startswith("data/mekanism/recipe/") or not name.endswith(".json"):
                continue
            data = json.loads(archive.read(name))
            if data.get("type") != "mekanism:crushing":
                continue
            recipes += 1
            if "neoforge:conditions" in data:
                # Another mod's items (AE2, Biomes We've Gone): not this port's.
                skipped.append(f"{name}: another mod's condition")
                continue
            output = data["output"]
            count = int(output.get("count", 1))
            outcome = output["id"]
            inputs = ingredient_inputs(data["input"])
            inputs = [i for i in inputs if i.startswith("minecraft:") or i in known_outputs]
            if not inputs:
                skipped.append(f"{name}: no 26.3 member of {data['input']}")
                continue
            if not outcome.startswith("minecraft:") and outcome not in known_outputs:
                skipped.append(f"{name}: output {outcome} is not registered")
                continue
            consumed = int(data["input"].get("count", 1))
            for item in inputs:
                if item in table and table[item] != (consumed, outcome, count):
                    raise ValueError(
                        f"{item} got two crushing recipes: {table[item]} and "
                        f"{(consumed, outcome, count)}"
                    )
                table[item] = (consumed, outcome, count)
    return (
        sorted((item, consumed, outcome, count) for item, (consumed, outcome, count) in table.items()),
        recipes,
        skipped,
    )


def render(jar: Path) -> str:
    table, recipes, skipped = rows(jar)
    header = (
        "    // Generated from the jar's `mekanism:crushing` recipes by\n"
        "    // `tools/gen_crushing.py`; do not edit by hand. Each row is\n"
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
        (item, int(consumed), outcome, int(count))
        for item, consumed, outcome, count in re.findall(
            r'"([^"]+)"\s*,\s*(\d+)\s*,\s*"([^"]+)"\s*,\s*(\d+)\s*\)?\s*,', block
        )
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, default=DEFAULT_JAR)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--rows", action="store_true", help="print the rows as JSON")
    args = parser.parse_args()
    expected, recipes, skipped = rows(args.jar)
    if args.rows:
        print(json.dumps(
            [{"input": i, "consumed": c, "output": o, "count": n} for i, c, o, n in expected],
            indent=1,
        ))
        return 0
    text = OUT.read_text()
    if args.check:
        committed = committed_rows(text)
        if committed != expected:
            print(
                f"{OUT} is stale: {len(committed)} rows committed, {len(expected)} "
                f"from the {recipes} crushing recipes"
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
