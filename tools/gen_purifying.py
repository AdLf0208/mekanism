#!/usr/bin/env python3
"""Generate the Purification Chamber's recipe table in `src/runtime.rs`.

Mekanism's `mekanism:purifying` recipes are data:
`data/mekanism/recipe/processing/<material>/clump/{from_ore,from_raw_ore,
from_raw_block,from_shard}.json`. Every ingredient is written as a NeoForge `c:`
tag or as one item id, and a tag's members are the pack's; the port can only
ever see the members 26.3 has, so `ITEM_TAGS` below names exactly those. An
output the port does not register (a clump of a material outside the slice,
another mod's item) drops its row instead of producing an item that does not
exist.

A row is `(input item, items consumed, accepted chemicals, per-tick units, output
item, output count)` — the same shape the injection chamber's table has, because
the two machines run the same `ItemStackChemicalToItemStack` recipe type:

- the **chemical** side is a list because the jar may write one id or a chemical
  tag; every purifying recipe names `mekanism:oxygen`, which is the list's only
  member here.
- `per-tick units` is the recipe's `chemical_input.amount` under
  `per_tick_usage: true` (every purifying recipe carries it). Upstream spends it
  per operating tick through `ChemicalUsageMultiplier.constantUse`, so one
  operation takes `per_tick × 200` units — 1 for an ore/raw/shard row, 2 for a
  raw block.

Usage:
  python3 tools/gen_purifying.py [--jar PATH] [--check] [--rows]

`--check` reads the committed rows back (whatever rustfmt did to their layout)
and fails when they no longer match the jar. `--rows` prints the table as JSON
for the in-game sweep (`tools/gas_sweep.py`), so the scenario and the table
cannot drift apart.
"""
import argparse
import json
import re
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
OUT = ROOT / "src" / "runtime.rs"
BEGIN = "    // BEGIN GENERATED PURIFYING TABLE (tools/gen_purifying.py)\n"
END = "    // END GENERATED PURIFYING TABLE\n"

# The item tags the jar's purifying recipes use, expanded to their 26.3 members:
# the pack's tags also hold modded ores (`alltheores:iron_ore`, ...) and
# Mekanism's own items, which this port does not register. A tag with no member
# here (`c:ores/{lead,osmium,tin,uranium}` and their raw/shard forms) is not
# listed, so its rows are skipped and printed.
ITEM_TAGS: dict[str, list[str]] = {
    "c:ores/copper": ["minecraft:copper_ore", "minecraft:deepslate_copper_ore"],
    "c:ores/gold": ["minecraft:gold_ore", "minecraft:deepslate_gold_ore"],
    "c:ores/iron": ["minecraft:iron_ore", "minecraft:deepslate_iron_ore"],
    "c:raw_materials/copper": ["minecraft:raw_copper"],
    "c:raw_materials/gold": ["minecraft:raw_gold"],
    "c:raw_materials/iron": ["minecraft:raw_iron"],
    "c:storage_blocks/raw_copper": ["minecraft:raw_copper_block"],
    "c:storage_blocks/raw_gold": ["minecraft:raw_gold_block"],
    "c:storage_blocks/raw_iron": ["minecraft:raw_iron_block"],
    "c:shards/copper": ["mekanism:shard_copper"],
    "c:shards/gold": ["mekanism:shard_gold"],
    "c:shards/iron": ["mekanism:shard_iron"],
}

# The chemical tags the jar's recipes use, expanded to the chemicals this port
# knows (`runtime::CHEMICALS`). A chemical tag is one row naming every member,
# not one row per member: a tank of either gas feeds the same operation.
CHEMICAL_TAGS: dict[str, list[str]] = {}

# The non-vanilla items this port registers (the slice-5 shards, the slice-6
# clumps and dirty dusts, and the three dusts of slices 1-3).
REGISTERED = [
    "mekanism:clump_copper",
    "mekanism:clump_gold",
    "mekanism:clump_iron",
    "mekanism:dirty_dust_copper",
    "mekanism:dirty_dust_gold",
    "mekanism:dirty_dust_iron",
    "mekanism:dust_copper",
    "mekanism:dust_gold",
    "mekanism:dust_iron",
    "mekanism:shard_copper",
    "mekanism:shard_gold",
    "mekanism:shard_iron",
]


def registered_items() -> set[str]:
    """The non-vanilla items this port registers."""
    return set(REGISTERED)


def item_inputs(ingredient: dict) -> list[str]:
    if "item" in ingredient:
        return [ingredient["item"]]
    return ITEM_TAGS.get(ingredient.get("tag", ""), [])


def chemical_inputs(ingredient: dict) -> list[str]:
    if "chemical" in ingredient:
        return [ingredient["chemical"]]
    return CHEMICAL_TAGS.get(ingredient.get("tag", ""), [])


def rows(jar: Path) -> tuple[list[tuple[str, int, list[str], int, str, int]], int, list[str]]:
    """The table, how many purifying recipes the jar holds, and the skipped ones."""
    table: dict[str, tuple[int, list[str], int, str, int]] = {}
    recipes = 0
    skipped: list[str] = []
    known_outputs = registered_items()
    with zipfile.ZipFile(jar) as archive:
        for name in sorted(archive.namelist()):
            if not name.startswith("data/mekanism/recipe/") or not name.endswith(".json"):
                continue
            data = json.loads(archive.read(name))
            if data.get("type") != "mekanism:purifying":
                continue
            recipes += 1
            chemicals = chemical_inputs(data["chemical_input"])
            if not chemicals:
                skipped.append(f"{name}: no chemical this port knows in {data['chemical_input']}")
                continue
            inputs = item_inputs(data["item_input"])
            if not inputs:
                skipped.append(f"{name}: no 26.3 member of {data['item_input']}")
                continue
            output = data["output"]["id"]
            if not output.startswith("minecraft:") and output not in known_outputs:
                skipped.append(f"{name}: output {output} is not registered")
                continue
            row = (
                int(data["item_input"].get("count", 1)),
                chemicals,
                int(data["chemical_input"]["amount"]),
                output,
                int(data["output"].get("count", 1)),
            )
            for item in inputs:
                if item in table and table[item] != row:
                    raise ValueError(f"{item} got two purifying recipes: {table[item]} and {row}")
                table[item] = row
    return (
        sorted((item, *rest) for item, rest in table.items()),
        recipes,
        skipped,
    )


def render(jar: Path) -> str:
    table, recipes, skipped = rows(jar)
    header = (
        "    // Generated from the jar's `mekanism:purifying` recipes by\n"
        "    // `tools/gen_purifying.py`; do not edit by hand. Each row is\n"
        "    // `(input item, items consumed, accepted chemicals, per-tick units,\n"
        "    // output item, output count)`; an item tag is expanded to the members\n"
        "    // 26.3 has and a chemical tag to the chemicals this port knows (see the\n"
        "    // tool's ITEM_TAGS/CHEMICAL_TAGS), and a recipe whose output the port\n"
        "    // does not register is left out.\n"
        f"    // Source recipes: {recipes}. Rows: {len(table)}. Skipped: {len(skipped)}.\n"
        "    static TABLE: &[Row] = &[\n"
    )
    body = "".join(
        f'        ("{item}", {consumed}, &[{", ".join(chr(34) + c + chr(34) for c in chemicals)}],'
        f' {per_tick}, "{output}", {count}),\n'
        for item, consumed, chemicals, per_tick, output, count in table
    )
    return header + body + "    ];\n"


def replace(text: str, block: str) -> str:
    start = text.index(BEGIN)
    end = text.index(END)
    return text[: start + len(BEGIN)] + block + text[end:]


def committed_rows(text: str) -> list[tuple[str, int, list[str], int, str, int]]:
    block = text[text.index(BEGIN) + len(BEGIN) : text.index(END)]
    found = []
    for body in re.findall(r'\(\s*"([^"]+)"\s*,(.*?)\),\s*\n', block, re.DOTALL):
        item, rest = body
        consumed, chemicals, per_tick, output, count = re.match(
            r'\s*(\d+)\s*,\s*&\[(.*?)\]\s*,\s*(\d+)\s*,\s*"([^"]+)"\s*,\s*(\d+)', rest, re.DOTALL
        ).groups()
        found.append(
            (
                item,
                int(consumed),
                [c.strip().strip('"') for c in chemicals.split(",") if c.strip()],
                int(per_tick),
                output,
                int(count),
            )
        )
    return found


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, default=DEFAULT_JAR)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--rows", action="store_true")
    args = parser.parse_args()
    table, recipes, skipped = rows(args.jar)
    if args.rows:
        print(
            json.dumps(
                [
                    {
                        "input": item,
                        "consumed": consumed,
                        "chemicals": chemicals,
                        "per_tick": per_tick,
                        "output": output,
                        "count": count,
                    }
                    for item, consumed, chemicals, per_tick, output, count in table
                ],
                indent=1,
            )
        )
        return 0
    text = OUT.read_text()
    committed = committed_rows(text)
    if args.check:
        if committed != table:
            print(
                f"{OUT} is stale: {len(committed)} rows committed, {len(table)} "
                f"from the {recipes} purifying recipes"
            )
            return 1
        print(f"{OUT} is up to date ({len(table)} rows from {recipes} recipes, {len(skipped)} skipped)")
        return 0
    OUT.write_text(replace(text, render(args.jar)))
    print(f"updated {OUT}: {len(table)} rows from {recipes} recipes, {len(skipped)} skipped")
    for line in skipped:
        print(f"  skipped {line}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
