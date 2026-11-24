#!/usr/bin/env python3
"""Generate the rotary Condensentrator's recipe table in `src/runtime.rs`.

The jar's 17 `type: mekanism:rotary` records each carry both halves of one
conversion: a fluid input and a chemical output (decondensentrating), and a
chemical input and a fluid output (condensentrating). The port keeps that shape
as one `(fluid, mB, chemical, units)` row per conversion.

Only fluids this game actually has are carried. The jar writes a `c:` fluid tag
for every row except water vapour (`minecraft:water`); none of those tags has a
member in 26.3, so those 16 rows are skipped and printed rather than carried as
dead data. A row whose two chemical ids are not in this port's 32-name chemical
table is skipped too.

Usage:
  python3 tools/gen_condensentrating.py [--jar PATH] [--check] [--rows]
"""
import argparse
import json
import re
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
OUT = ROOT / "src" / "runtime.rs"
BEGIN = "    // BEGIN GENERATED CONDENSENTRATING TABLE (tools/gen_condensentrating.py)\n"
END = "    // END GENERATED CONDENSENTRATING TABLE\n"

# The only rotary fluid input this game can fill a tank with. The jar's other
# rows name `c:brine`, `c:steam`, `c:hydrogen` ... tags with no fluid member here.
FLUIDS: dict[str, str] = {
    "minecraft:water": "minecraft:water",
}


def known_chemicals() -> set[str]:
    text = OUT.read_text()
    block = re.search(r"pub const CHEMICALS: \[&str; \d+\] = \[(.*?)\];", text, re.DOTALL)
    if block is None:
        raise SystemExit("the CHEMICALS table could not be found in src/runtime.rs")
    return set(re.findall(r'"([^"]+)"', block.group(1)))


def rows(jar: Path) -> tuple[list[tuple[str, int, str, int]], int, list[str]]:
    table: list[tuple[str, int, str, int]] = []
    recipes = 0
    skipped: list[str] = []
    chemicals = known_chemicals()
    with zipfile.ZipFile(jar) as archive:
        for name in sorted(archive.namelist()):
            if not name.startswith("data/mekanism/recipe/rotary/") or not name.endswith(".json"):
                continue
            data = json.loads(archive.read(name))
            if data.get("type") != "mekanism:rotary":
                continue
            recipes += 1
            ingredient = data["fluid_input"]
            tag = ingredient.get("tag") or ingredient.get("item", "")
            fluid = FLUIDS.get(tag)
            if fluid is None:
                skipped.append(f"{name}: {tag} has no fluid in this game")
                continue
            chemical_id = data["chemical_output"]["id"]
            chemical_in = data["chemical_input"]["chemical"]
            if chemical_id not in chemicals or chemical_in not in chemicals:
                skipped.append(f"{name}: {chemical_id}/{chemical_in} is not a chemical this port knows")
                continue
            if chemical_id != chemical_in:
                skipped.append(f"{name}: the two halves name different chemicals")
                continue
            table.append(
                (
                    fluid,
                    int(ingredient.get("amount", 1)),
                    chemical_id,
                    int(data["chemical_output"]["amount"]),
                )
            )
    return (sorted(table), recipes, skipped)


def render(jar: Path) -> str:
    table, recipes, skipped = rows(jar)
    header = (
        "    // Generated from the jar's `mekanism:rotary` recipes by\n"
        "    // `tools/gen_condensentrating.py`; do not edit by hand. Each row is\n"
        "    // `(fluid, mB per operation, chemical, units per operation)`.\n"
        f"    // Source recipes: {recipes}. Rows: {len(table)}. Skipped: {len(skipped)}.\n"
        "    static TABLE: &[Row] = &[\n"
    )
    body = "".join(
        f'        ("{fluid}", {mb}, "{chemical}", {units}),\n'
        for fluid, mb, chemical, units in table
    )
    return header + body + "    ];\n"


def replace(text: str, block: str) -> str:
    start = text.index(BEGIN)
    end = text.index(END)
    return text[: start + len(BEGIN)] + block + text[end:]


def committed_rows(text: str) -> list[tuple[str, int, str, int]]:
    block = text[text.index(BEGIN) + len(BEGIN) : text.index(END)]
    return [
        (fluid, int(mb), chemical, int(units))
        for fluid, mb, chemical, units in re.findall(
            r'\(\s*"([^"]+)"\s*,\s*(\d+)\s*,\s*"([^"]+)"\s*,\s*(\d+)\s*,?\s*\)',
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
                    {"fluid": fluid, "mb": mb, "chemical": chemical, "units": units}
                    for fluid, mb, chemical, units in table
                ],
                indent=1,
            )
        )
        return 0
    text = OUT.read_text()
    if args.check:
        committed = committed_rows(text)
        if committed != table:
            print(
                f"{OUT} is stale: {len(committed)} rows committed, "
                f"{len(table)} from the {recipes} rotary recipes"
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
