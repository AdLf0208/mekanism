#!/usr/bin/env python3
"""Generate the Chemical Dissolution Chamber's recipe table in `src/runtime.rs`.

Mekanism's `mekanism:dissolution` recipes are data:
`data/mekanism/recipe/processing/<material>/slurry/dirty/from_{ore,raw_ore,
raw_block}.json`, one material family per `PrimaryResource`. Every one of the
jar's 21 rows is `per_tick_usage: true`, so `chemical_input.amount` is spent
**each operating tick** (the machine takes 100 ticks, so an ore costs 100 units
of acid, not 1). A row is
`(input item, items per operation, acid units per tick, dirty slurry, slurry
units per operation)`:

- the item input is a **tag** (`c:ores/<m>`, `c:raw_materials/<m>`,
  `c:storage_blocks/raw_<m>`); `TAGS` expands it to the members 26.3 has, which
  for an ore is the stone and the deepslate form;
- a row whose output slurry this port does not know (`runtime::SLURRIES`) is
  skipped and printed rather than carried as a dead row, which is every
  lead/osmium/tin/uranium row: 26.3 has no such ore.

Usage:
  python3 tools/gen_dissolving.py [--jar PATH] [--check] [--rows]

`--check` reads the committed rows back (whatever rustfmt did to their layout)
and fails when they no longer match the jar. `--rows` prints the table as JSON
for the in-game check (`tools/dissolve_check.py`), so the scenario and the
table cannot drift apart.
"""
import argparse
import json
import re
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
OUT = ROOT / "src" / "runtime.rs"
BEGIN = "    // BEGIN GENERATED DISSOLVING TABLE (tools/gen_dissolving.py)\n"
END = "    // END GENERATED DISSOLVING TABLE\n"

# The `c:` tags the jar's dissolution recipes use, expanded to their 26.3
# members: the pack's tags also hold modded ores (`alltheores:iron_ore`…) and
# Mekanism's own items, which this port does not register. The ore expansion is
# the same one `gen_enriching.py` and `gen_injecting.py` use.
TAGS: dict[str, list[str]] = {
    "c:ores/copper": ["minecraft:copper_ore", "minecraft:deepslate_copper_ore"],
    "c:ores/gold": ["minecraft:gold_ore", "minecraft:deepslate_gold_ore"],
    "c:ores/iron": ["minecraft:iron_ore", "minecraft:deepslate_iron_ore"],
    "c:ores/lead": [],
    "c:ores/osmium": [],
    "c:ores/tin": [],
    "c:ores/uranium": [],
    "c:raw_materials/copper": ["minecraft:raw_copper"],
    "c:raw_materials/gold": ["minecraft:raw_gold"],
    "c:raw_materials/iron": ["minecraft:raw_iron"],
    "c:raw_materials/lead": [],
    "c:raw_materials/osmium": [],
    "c:raw_materials/tin": [],
    "c:raw_materials/uranium": [],
    "c:storage_blocks/raw_copper": ["minecraft:raw_copper_block"],
    "c:storage_blocks/raw_gold": ["minecraft:raw_gold_block"],
    "c:storage_blocks/raw_iron": ["minecraft:raw_iron_block"],
    "c:storage_blocks/raw_lead": [],
    "c:storage_blocks/raw_osmium": [],
    "c:storage_blocks/raw_tin": [],
    "c:storage_blocks/raw_uranium": [],
}

# The slurries this port knows (`runtime::SLURRIES`), kept in step with the
# table by `--check` (a row whose output is outside it is skipped and printed).
SLURRIES = {
    "copper",
    "gold",
    "iron",
    "lead",
    "osmium",
    "tin",
    "uranium",
}

# The acid every row names (`chemical_input.chemical`).
ACID = "mekanism:sulfuric_acid"


def item_inputs(ingredient: dict) -> list[str]:
    if "item" in ingredient:
        return [ingredient["item"]]
    return TAGS.get(ingredient.get("tag", ""), [])


def rows(jar: Path) -> tuple[list[tuple[str, int, int, str, int]], int, list[str]]:
    """The table, how many dissolution recipes the jar holds, and the skipped ones."""
    table: list[tuple[str, int, int, str, int]] = []
    recipes = 0
    skipped: list[str] = []
    with zipfile.ZipFile(jar) as archive:
        for name in sorted(archive.namelist()):
            if not name.startswith("data/mekanism/recipe/") or not name.endswith(".json"):
                continue
            data = json.loads(archive.read(name))
            if data.get("type") != "mekanism:dissolution":
                continue
            recipes += 1
            acid = data["chemical_input"]["chemical"]
            if acid != ACID:
                skipped.append(f"{name}: {acid} is not the acid this port knows")
                continue
            if not data.get("per_tick_usage"):
                skipped.append(f"{name}: not a per-tick-usage row")
                continue
            output = data["output"]["id"]
            material = output.rsplit("_", 1)[-1]
            if not output.startswith("mekanism:dirty_") or material not in SLURRIES:
                skipped.append(f"{name}: {output} is not a slurry this port knows")
                continue
            inputs = item_inputs(data["item_input"])
            if not inputs:
                skipped.append(
                    f"{name}: no 26.3 member of {data['item_input'].get('tag', data['item_input'])}"
                )
                continue
            count = int(data["item_input"].get("count", 1))
            per_tick = int(data["chemical_input"]["amount"])
            amount = int(data["output"]["amount"])
            for item in inputs:
                table.append((item, count, per_tick, output, amount))
    return (sorted(table), recipes, skipped)


def render(jar: Path) -> str:
    table, recipes, skipped = rows(jar)
    header = (
        "    // Generated from the jar's `mekanism:dissolution` recipes by\n"
        "    // `tools/gen_dissolving.py`; do not edit by hand. Each row is\n"
        "    // `(input item, items per operation, acid units per tick, dirty\n"
        "    // slurry, slurry units per operation)`; every row is per-tick usage,\n"
        "    // so the acid is spent over the operation's 100 ticks, and a row whose\n"
        "    // ore or slurry this port does not know is left out.\n"
        f"    // Source recipes: {recipes}. Rows: {len(table)}. Skipped: {len(skipped)}.\n"
        "    static TABLE: &[Row] = &[\n"
    )
    body = "".join(
        f'        ("{item}", {count}, {per_tick}, "{output}", {amount}),\n'
        for item, count, per_tick, output, amount in table
    )
    return header + body + "    ];\n"


def replace(text: str, block: str) -> str:
    start = text.index(BEGIN)
    end = text.index(END)
    return text[: start + len(BEGIN)] + block + text[end:]


def committed_rows(text: str) -> list[tuple[str, int, int, str, int]]:
    block = text[text.index(BEGIN) + len(BEGIN) : text.index(END)]
    # rustfmt lays a row out over one line or over five, so the separator is any
    # whitespace and a trailing comma is optional.
    return [
        (item, int(count), int(per_tick), output, int(amount))
        for item, count, per_tick, output, amount in re.findall(
            r'\(\s*"([^"]+)"\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*"([^"]+)"\s*,\s*(\d+)\s*,?\s*\)',
            block,
            re.DOTALL,
        )
    ]


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
                        "count": count,
                        "acid_per_tick": per_tick,
                        "output": output,
                        "output_amount": amount,
                    }
                    for item, count, per_tick, output, amount in table
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
                f"from the {recipes} dissolution recipes"
            )
            return 1
        print(
            f"{OUT} is up to date ({len(table)} rows from {recipes} recipes, "
            f"{len(skipped)} skipped)"
        )
        return 0
    OUT.write_text(replace(text, render(args.jar)))
    print(f"updated {OUT}: {len(table)} rows from {recipes} recipes, {len(skipped)} skipped")
    for line in skipped:
        print(f"  skipped {line}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
