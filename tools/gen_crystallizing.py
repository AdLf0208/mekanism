#!/usr/bin/env python3
"""Generate the Chemical Crystallizer's recipe table in `src/runtime.rs`.

Mekanism's `mekanism:crystallizing` recipes are data: one
`data/mekanism/recipe/processing/<material>/crystal/from_slurry.json` per
`PrimaryResource`, plus the two non-slurry rows the jar ships
(`crystallizing/lithium.json`, `crystallizing/salt.json`). A row is
`(clean slurry, units, crystal item, count)`:

- the input is a **chemical** (`mekanism:clean_<material>`) and it is consumed
  **when the operation finishes** (`OneInputCachedRecipe.finishProcessing` calls
  `inputHandler.use`), so the amount is per operation, not per tick;
- a row whose crystal item this port does not register (the four materials it
  has no item for, and the two non-slurry rows) is skipped and printed rather
  than carried as a dead row — the same rule the washing table follows.

Usage:
  python3 tools/gen_crystallizing.py [--jar PATH] [--check] [--rows]

`--check` reads the committed rows back (whatever rustfmt did to their layout)
and fails when they no longer match the jar. `--rows` prints the table as JSON
for the in-game sweep (`tools/crystal_sweep.py`), so the scenario and the table
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
BEGIN = "    // BEGIN GENERATED CRYSTALLIZING TABLE (tools/gen_crystallizing.py)\n"
END = "    // END GENERATED CRYSTALLIZING TABLE\n"

# The crystals this port registers as items (`data/mekanism/horizon/item/
# crystal_<material>.json`): exactly the seven slurries the washer can make
# (`mekanism:washing`'s rows), so no washer row dead-ends at the crystallizer.
# The jar also crystallizes lithium, brine and antimatter; none of those items
# exists in this port, so their three rows are skipped.
CRYSTALS = {"copper", "gold", "iron", "lead", "osmium", "tin", "uranium"}


def rows(jar: Path) -> tuple[list[tuple[str, int, str, int]], int, list[str]]:
    """The table, how many crystallizing recipes the jar holds, and the skipped ones."""
    table: list[tuple[str, int, str, int]] = []
    recipes = 0
    skipped: list[str] = []
    with zipfile.ZipFile(jar) as archive:
        for name in sorted(archive.namelist()):
            if not name.startswith("data/mekanism/recipe/") or not name.endswith(".json"):
                continue
            data = json.loads(archive.read(name))
            if data.get("type") != "mekanism:crystallizing":
                continue
            recipes += 1
            chemical = data["input"]["chemical"]
            output = data["output"]["id"]
            if not chemical.startswith("mekanism:clean_"):
                skipped.append(f"{name}: {chemical} is not a clean slurry")
                continue
            if not output.startswith("mekanism:crystal_"):
                skipped.append(f"{name}: {output} is not a crystal item")
                continue
            material = output.rsplit("_", 1)[-1]
            if chemical != f"mekanism:clean_{material}":
                skipped.append(f"{name}: {chemical} -> {output} is not a slurry/crystal pair")
                continue
            if material not in CRYSTALS:
                skipped.append(f"{name}: {material} is not a crystal this port registers")
                continue
            table.append(
                (
                    chemical,
                    int(data["input"]["amount"]),
                    output,
                    int(data["output"]["count"]),
                )
            )
    return (sorted(table), recipes, skipped)


def render(jar: Path) -> str:
    table, recipes, skipped = rows(jar)
    header = (
        "    // Generated from the jar's `mekanism:crystallizing` recipes by\n"
        "    // `tools/gen_crystallizing.py`; do not edit by hand. Each row is\n"
        "    // `(clean slurry, its units, crystal item, count)`, the amount one\n"
        "    // **operation** takes (it is consumed when the operation finishes, not\n"
        "    // per tick), and a recipe whose crystal item this port does not\n"
        "    // register is left out.\n"
        f"    // Source recipes: {recipes}. Rows: {len(table)}. Skipped: {len(skipped)}.\n"
        "    static TABLE: &[Row] = &[\n"
    )
    body = "".join(
        f'        ("{chemical}", {amount}, "{item}", {count}),\n'
        for chemical, amount, item, count in table
    )
    return header + body + "    ];\n"


def replace(text: str, block: str) -> str:
    start = text.index(BEGIN)
    end = text.index(END)
    return text[: start + len(BEGIN)] + block + text[end:]


def committed_rows(text: str) -> list[tuple[str, int, str, int]]:
    block = text[text.index(BEGIN) + len(BEGIN) : text.index(END)]
    # rustfmt lays a row out over one line or over four, so the separator is any
    # whitespace and a trailing comma is optional.
    return [
        (chemical, int(amount), item, int(count))
        for chemical, amount, item, count in re.findall(
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
                    {
                        "slurry": chemical,
                        "amount": amount,
                        "crystal": item,
                        "count": count,
                    }
                    for chemical, amount, item, count in table
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
                f"from the {recipes} crystallizing recipes"
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
