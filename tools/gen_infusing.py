#!/usr/bin/env python3
"""Generate the Chemical Infuser's acid-chain table in `src/runtime.rs`.

The jar's `mekanism:chemical_infusing` recipes are data. Two of them are the acid
chain: oxygen + 2 SO2 -> 2 SO3, and SO3 + water vapour -> sulfuric acid. The
other two (hydrogen chloride, uranium hexafluoride) are real recipes with known
chemicals but no producer/consumer in this slice, so they are skipped and
printed rather than carried as dead rows (the parity rule).

A row is `(left chemical, left amount, right chemical, right amount, output,
output amount)`. The infuser is a one-tick machine: each tick consumes a whole
row and produces its output for 200 FE (`TileEntityRecipeMachine`'s default
`requiredTicks = 1`).

Usage:
  python3 tools/gen_infusing.py [--jar PATH] [--check] [--rows]
"""
import argparse
import json
import re
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
OUT = ROOT / "src" / "runtime.rs"
BEGIN = "    // BEGIN GENERATED INFUSING TABLE (tools/gen_infusing.py)\n"
END = "    // END GENERATED INFUSING TABLE\n"
ACID_CHAIN = {"mekanism:sulfur_trioxide", "mekanism:sulfuric_acid"}


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
            if data.get("type") != "mekanism:chemical_infusing":
                continue
            recipes += 1
            output = data["output"]["id"]
            if output not in ACID_CHAIN:
                skipped.append(f"{name}: {output} is outside the acid chain this slice carries")
                continue
            left = data["left_input"]
            right = data["right_input"]
            if not all(
                part["chemical"] in chemicals for part in (left, right)
            ) or output not in chemicals:
                skipped.append(f"{name}: a chemical this port does not know")
                continue
            table.append(
                (
                    left["chemical"],
                    int(left["amount"]),
                    right["chemical"],
                    int(right["amount"]),
                    output,
                    int(data["output"]["amount"]),
                )
            )
    return (sorted(table), recipes, skipped)


def render(jar: Path) -> str:
    table, recipes, skipped = rows(jar)
    header = (
        "    // Generated from the jar's `mekanism:chemical_infusing` recipes by\n"
        "    // `tools/gen_infusing.py`; do not edit by hand. Each row is\n"
        "    // `(left chemical, left amount, right chemical, right amount,\n"
        "    // output chemical, output amount)`, one operation a tick.\n"
        f"    // Source recipes: {recipes}. Rows: {len(table)}. Skipped: {len(skipped)}.\n"
        "    static TABLE: &[Row] = &[\n"
    )
    body = "".join(
        f'        ("{left}", {la}, "{right}", {ra}, "{output}", {amount}),\n'
        for left, la, right, ra, output, amount in table
    )
    return header + body + "    ];\n"


def replace(text: str, block: str) -> str:
    start = text.index(BEGIN)
    end = text.index(END)
    return text[: start + len(BEGIN)] + block + text[end:]


def committed_rows(text: str) -> list[tuple[str, int, str, int, str, int]]:
    block = text[text.index(BEGIN) + len(BEGIN) : text.index(END)]
    return [
        (left, int(la), right, int(ra), output, int(amount))
        for left, la, right, ra, output, amount in re.findall(
            r'\(\s*"([^"]+)"\s*,\s*(\d+)\s*,\s*"([^"]+)"\s*,\s*(\d+)\s*,\s*"([^"]+)"\s*,\s*(\d+)\s*,?\s*\)',
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
                        "left": left,
                        "left_amount": la,
                        "right": right,
                        "right_amount": ra,
                        "output": output,
                        "output_amount": amount,
                    }
                    for left, la, right, ra, output, amount in table
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
                f"{len(table)} from the {recipes} chemical-infusing recipes"
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
