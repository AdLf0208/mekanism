#!/usr/bin/env python3
"""Generate the Chemical Oxidizer's recipe table in `src/runtime.rs`.

The jar's `mekanism:oxidizing` recipes are data. One is sulfur dust -> 100 units
of sulfur dioxide (`data/mekanism/recipe/oxidizing/sulfur_dioxide.json`), and it
is the head of the acid chain this port carries. The rest name items this port
does not register (bio fuel, enriched metals, coal dust, ...) and are skipped
and printed rather than carried as dead rows.

A row is `(input item, items per operation, output chemical, output amount)`.
The gas is made at the operation's finish (upstream's `OneInputCachedRecipe`),
not per tick: 1 sulfur dust -> 100 SO2 over 100 ticks for 20,000 FE.

Usage:
  python3 tools/gen_oxidizing.py [--jar PATH] [--check] [--rows]

`--check` re-derives the table and fails when the committed bytes no longer
match the jar; `--rows` prints it as JSON for the in-game chain leg.
"""
import argparse
import json
import re
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
OUT = ROOT / "src" / "runtime.rs"
BEGIN = "    // BEGIN GENERATED OXIDIZING TABLE (tools/gen_oxidizing.py)\n"
END = "    // END GENERATED OXIDIZING TABLE\n"

TAGS: dict[str, list[str]] = {
    "c:dusts/sulfur": ["mekanism:dust_sulfur"],
}


def known_chemicals() -> set[str]:
    text = OUT.read_text()
    block = re.search(
        r"pub const CHEMICALS: \[&str; \d+\] = \[(.*?)\];", text, re.DOTALL
    )
    if block is None:
        raise SystemExit("the CHEMICALS table could not be found in src/runtime.rs")
    return set(re.findall(r'"([^"]+)"', block.group(1)))


def our_items() -> set[str]:
    return {
        f"mekanism:{path.stem}"
        for path in (ROOT / "data" / "mekanism" / "horizon" / "item").glob("*.json")
    }


def rows(jar: Path) -> tuple[list[tuple[str, int, str, int]], int, list[str]]:
    table: list[tuple[str, int, str, int]] = []
    recipes = 0
    skipped: list[str] = []
    chemicals = known_chemicals()
    items = our_items()
    with zipfile.ZipFile(jar) as archive:
        for name in sorted(archive.namelist()):
            if not name.startswith("data/mekanism/recipe/") or not name.endswith(".json"):
                continue
            data = json.loads(archive.read(name))
            if data.get("type") != "mekanism:oxidizing":
                continue
            recipes += 1
            output = data["output"]["id"]
            if output not in chemicals:
                skipped.append(f"{name}: {output} is not a chemical this port knows")
                continue
            ingredient = data.get("item_input", data.get("input"))
            inputs = (
                [ingredient["item"]]
                if "item" in ingredient
                else TAGS.get(ingredient.get("tag", ""), [])
            )
            if not inputs:
                skipped.append(f"{name}: no member of {ingredient.get('tag', ingredient)}")
                continue
            for item in inputs:
                if item not in items:
                    skipped.append(f"{name}: {item} is not an item this port registers")
                    continue
                table.append(
                    (
                        item,
                        int(ingredient.get("count", 1)),
                        output,
                        int(data["output"]["amount"]),
                    )
                )
    return (sorted(table), recipes, skipped)


def render(jar: Path) -> str:
    table, recipes, skipped = rows(jar)
    header = (
        "    // Generated from the jar's `mekanism:oxidizing` recipes by\n"
        "    // `tools/gen_oxidizing.py`; do not edit by hand. Each row is\n"
        "    // `(input item, items per operation, output chemical, output amount)`.\n"
        "    // The gas is made at the operation's finish, not per tick.\n"
        f"    // Source recipes: {recipes}. Rows: {len(table)}. Skipped: {len(skipped)}.\n"
        "    static TABLE: &[Row] = &[\n"
    )
    body = "".join(
        f'        ("{item}", {count}, "{output}", {amount}),\n'
        for item, count, output, amount in table
    )
    return header + body + "    ];\n"


def replace(text: str, block: str) -> str:
    start = text.index(BEGIN)
    end = text.index(END)
    return text[: start + len(BEGIN)] + block + text[end:]


def committed_rows(text: str) -> list[tuple[str, int, str, int]]:
    block = text[text.index(BEGIN) + len(BEGIN) : text.index(END)]
    return [
        (item, int(count), output, int(amount))
        for item, count, output, amount in re.findall(
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
                        "input": item,
                        "count": count,
                        "output": output,
                        "output_amount": amount,
                    }
                    for item, count, output, amount in table
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
                f"{len(table)} from the {recipes} oxidizing recipes"
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
