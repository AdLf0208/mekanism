#!/usr/bin/env python3
"""Generate the Electrolytic Separator's recipe table in `src/runtime.rs`.

The jar's `mekanism:separating` recipes are data. The water row
(`data/mekanism/recipe/separator/water.json`) takes 2 mB of water and makes
2 units of hydrogen plus 1 of oxygen. The brine row names the `c:brine` fluid
tag, which has no member in this game, so it is skipped and printed rather than
carried as a dead row. A row whose `energy_multiplier` is not the default 1 is
skipped too: this port's one operation is the jar's `2 x hydrogen energy
density` FE for one tick, and a multiplier would need its own operation.

Usage:
  python3 tools/gen_separating.py [--jar PATH] [--check] [--rows]
"""
import argparse
import json
import re
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
OUT = ROOT / "src" / "runtime.rs"
BEGIN = "    // BEGIN GENERATED SEPARATING TABLE (tools/gen_separating.py)\n"
END = "    // END GENERATED SEPARATING TABLE\n"

# Only fluids this game actually has. The jar writes a tag for `c:brine`, whose
# `minecraft:water` sibling here is the only separating input that can run.
FLUIDS: dict[str, str] = {
    "minecraft:water": "minecraft:water",
}


def known_chemicals() -> set[str]:
    text = OUT.read_text()
    block = re.search(r"pub const CHEMICALS: \[&str; \d+\] = \[(.*?)\];", text, re.DOTALL)
    if block is None:
        raise SystemExit("the CHEMICALS table could not be found in src/runtime.rs")
    return set(re.findall(r'"([^"]+)"', block.group(1)))


def rows(jar: Path) -> tuple[list[tuple[str, int, str, int, str, int]], int, list[str]]:
    table: list[tuple[str, int, str, int, str, int]] = []
    recipes = 0
    skipped: list[str] = []
    chemicals = known_chemicals()
    with zipfile.ZipFile(jar) as archive:
        for name in sorted(archive.namelist()):
            if not name.startswith("data/mekanism/recipe/") or not name.endswith(".json"):
                continue
            data = json.loads(archive.read(name))
            if data.get("type") != "mekanism:separating":
                continue
            recipes += 1
            multiplier = int(data.get("energy_multiplier", 1))
            if multiplier != 1:
                skipped.append(f"{name}: energy multiplier {multiplier} is not the default 1")
                continue
            ingredient = data["input"]
            fluid = (
                FLUIDS.get(ingredient["item"])
                if "item" in ingredient
                else FLUIDS.get(ingredient.get("tag", ""))
            )
            if fluid is None:
                skipped.append(f"{name}: {ingredient.get('tag', ingredient)} has no fluid in this game")
                continue
            left = data["left_chemical_output"]
            right = data["right_chemical_output"]
            unknown = [
                output["id"]
                for output in (left, right)
                if output["id"] not in chemicals
            ]
            if unknown:
                skipped.append(f"{name}: {', '.join(unknown)} is not a chemical this port knows")
                continue
            table.append(
                (
                    fluid,
                    int(ingredient.get("amount", 1)),
                    left["id"],
                    int(left["amount"]),
                    right["id"],
                    int(right["amount"]),
                )
            )
    return (sorted(table), recipes, skipped)


def render(jar: Path) -> str:
    table, recipes, skipped = rows(jar)
    header = (
        "    // Generated from the jar's `mekanism:separating` recipes by\n"
        "    // `tools/gen_separating.py`; do not edit by hand. Each row is\n"
        "    // `(fluid, mB per operation, left chemical, left amount,\n"
        "    //  right chemical, right amount)`.\n"
        f"    // Source recipes: {recipes}. Rows: {len(table)}. Skipped: {len(skipped)}.\n"
        "    static TABLE: &[Row] = &[\n"
    )
    body = "".join(
        f'        ("{fluid}", {mb}, "{left}", {la}, "{right}", {ra}),\n'
        for fluid, mb, left, la, right, ra in table
    )
    return header + body + "    ];\n"


def replace(text: str, block: str) -> str:
    start = text.index(BEGIN)
    end = text.index(END)
    return text[: start + len(BEGIN)] + block + text[end:]


def committed_rows(text: str) -> list[tuple[str, int, str, int, str, int]]:
    block = text[text.index(BEGIN) + len(BEGIN) : text.index(END)]
    return [
        (fluid, int(mb), left, int(la), right, int(ra))
        for fluid, mb, left, la, right, ra in re.findall(
            r'\(\s*"([^"]+)"\s*,\s*(\d+)\s*,\s*"([^"]+)"\s*,\s*(\d+)\s*,'
            r'\s*"([^"]+)"\s*,\s*(\d+)\s*,?\s*\)',
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
                        "mb": mb,
                        "left": left,
                        "left_amount": la,
                        "right": right,
                        "right_amount": ra,
                    }
                    for fluid, mb, left, la, right, ra in table
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
                f"{len(table)} from the {recipes} separating recipes"
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
