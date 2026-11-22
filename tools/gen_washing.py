#!/usr/bin/env python3
"""Generate the Chemical Washer's recipe table in `src/runtime.rs`.

Mekanism's `mekanism:washing` recipes are data:
`data/mekanism/recipe/processing/<material>/slurry/clean.json`, one per
`PrimaryResource`. A row is `(fluid, dirty slurry, fluid millibuckets, slurry
units, clean slurry, clean units)`:

- the fluid input is a **tag** (`minecraft:water`); a tank holds a source fluid,
  so the row names `minecraft:water` (the tag's flowing form is never in a tank);
- both inputs are consumed **when the operation finishes**
  (`TwoInputCachedRecipe` overrides no `useResources`), so the numbers are per
  operation, not per tick — unlike the gas machines' `per_tick_usage`;
- a recipe whose chemical ids this port does not know (`runtime::SLURRIES`) is
  skipped and printed rather than carried as a dead row.

Usage:
  python3 tools/gen_washing.py [--jar PATH] [--check] [--rows]

`--check` reads the committed rows back (whatever rustfmt did to their layout)
and fails when they no longer match the jar. `--rows` prints the table as JSON
for the in-game sweep (`tools/wash_sweep.py`), so the scenario and the table
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
BEGIN = "    // BEGIN GENERATED WASHING TABLE (tools/gen_washing.py)\n"
END = "    // END GENERATED WASHING TABLE\n"

# The slurries this port knows (`runtime::SLURRIES`), kept in step with the
# table by `--check` (a row whose ids are outside it is skipped and printed).
SLURRIES = {
    "copper",
    "gold",
    "iron",
    "lead",
    "osmium",
    "tin",
    "uranium",
}

# The fluid tag the jar's washing recipes take, as the tank's fluid: water, and
# only water (a tank holds a source fluid, never `minecraft:flowing_water`).
FLUID_TAGS = {"minecraft:water": "minecraft:water"}


def fluid_input(ingredient: dict) -> str | None:
    if "fluid" in ingredient:
        return ingredient["fluid"]
    return FLUID_TAGS.get(ingredient.get("tag", ""))


def rows(jar: Path) -> tuple[list[tuple[str, str, int, int, str, int]], int, list[str]]:
    """The table, how many washing recipes the jar holds, and the skipped ones."""
    table: list[tuple[str, str, int, int, str, int]] = []
    recipes = 0
    skipped: list[str] = []
    with zipfile.ZipFile(jar) as archive:
        for name in sorted(archive.namelist()):
            if not name.startswith("data/mekanism/recipe/") or not name.endswith(".json"):
                continue
            data = json.loads(archive.read(name))
            if data.get("type") != "mekanism:washing":
                continue
            recipes += 1
            fluid = fluid_input(data["fluid_input"])
            if fluid is None:
                skipped.append(f"{name}: no fluid this port knows in {data['fluid_input']}")
                continue
            dirty = data["chemical_input"]["chemical"]
            clean = data["output"]["id"]
            materials = {dirty.rsplit("_", 1)[-1], clean.rsplit("_", 1)[-1]}
            if len(materials) != 1 or not dirty.startswith("mekanism:dirty_") or not clean.startswith(
                "mekanism:clean_"
            ):
                skipped.append(f"{name}: {dirty} -> {clean} is not a slurry pair")
                continue
            (material,) = materials
            if material not in SLURRIES:
                skipped.append(f"{name}: {material} is not a slurry this port knows")
                continue
            table.append(
                (
                    fluid,
                    dirty,
                    int(data["fluid_input"]["amount"]),
                    int(data["chemical_input"]["amount"]),
                    clean,
                    int(data["output"]["amount"]),
                )
            )
    return (sorted(table), recipes, skipped)


def render(jar: Path) -> str:
    table, recipes, skipped = rows(jar)
    header = (
        "    // Generated from the jar's `mekanism:washing` recipes by\n"
        "    // `tools/gen_washing.py`; do not edit by hand. Each row is\n"
        "    // `(fluid, dirty slurry, fluid millibuckets, slurry units, clean\n"
        "    // slurry, clean units)`, the amounts one **operation** takes and makes\n"
        "    // (both inputs are consumed when it finishes, not per tick), and a\n"
        "    // recipe whose slurry ids this port does not know is left out.\n"
        f"    // Source recipes: {recipes}. Rows: {len(table)}. Skipped: {len(skipped)}.\n"
        "    static TABLE: &[Row] = &[\n"
    )
    body = "".join(
        f'        ("{fluid}", "{dirty}", {fluid_amount}, {chemical_amount}, "{clean}", {clean_amount}),\n'
        for fluid, dirty, fluid_amount, chemical_amount, clean, clean_amount in table
    )
    return header + body + "    ];\n"


def replace(text: str, block: str) -> str:
    start = text.index(BEGIN)
    end = text.index(END)
    return text[: start + len(BEGIN)] + block + text[end:]


def committed_rows(text: str) -> list[tuple[str, str, int, int, str, int]]:
    block = text[text.index(BEGIN) + len(BEGIN) : text.index(END)]
    # rustfmt lays a row out over one line or over six, so the separator is any
    # whitespace and a trailing comma is optional.
    return [
        (fluid, dirty, int(fluid_amount), int(chemical_amount), clean, int(clean_amount))
        for fluid, dirty, fluid_amount, chemical_amount, clean, clean_amount in re.findall(
            r'\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*(\d+)\s*,\s*(\d+)\s*,\s*"([^"]+)"\s*,\s*(\d+)\s*,?\s*\)',
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
                        "fluid": fluid,
                        "dirty": dirty,
                        "water": fluid_amount,
                        "dirty_amount": chemical_amount,
                        "clean": clean,
                        "clean_amount": clean_amount,
                    }
                    for fluid, dirty, fluid_amount, chemical_amount, clean, clean_amount in table
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
                f"from the {recipes} washing recipes"
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
