#!/usr/bin/env python3
"""Generate `src/runtime.rs` from the prepared 26.3 vanilla data.

Mekanism's Energized Smelter smelts every `minecraft:smelting` recipe (it adds
them to its own `mekanism:smelting` type in
`MekanismRecipeType.getRecipesUncached`), so the port's recipe table is
vanilla's own data, expanded over item tags. Nothing is invented: every pair
below is a `data/minecraft/recipe/*.json` file of the 26.3 resources.

Usage:
  python3 tools/gen_smelting.py [--resources DIR] [--check]
  python3 tools/gen_smelting.py --rows            # the 163 vanilla rows as JSON
  python3 tools/gen_smelting.py --mekanism-rows   # the jar's own dust rows as JSON

`--check` reads the rows back out of the committed file (whatever rustfmt did to
their layout) and fails if they differ from the 26.3 recipes. `--rows` and
`--mekanism-rows` are what the in-game sweep (`tools/smelt_sweep.py`) drives real
smelters with, so the scenario and the tables cannot drift apart.
"""
import argparse
import json
import os
import re
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_RESOURCES = Path(os.environ.get("HORIZON_RES", str(Path.home() / ".local/horizon-client/26.3")))
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
OUT = ROOT / "src" / "runtime.rs"
BEGIN = "    // BEGIN GENERATED SMELTING TABLE (tools/gen_smelting.py)\n"
END = "    // END GENERATED SMELTING TABLE\n"

HEADER = """    // Generated from the prepared 26.3 resources by
    // `tools/gen_smelting.py`; do not edit by hand. Mekanism adds every
    // `minecraft:smelting` recipe to its own smelting type
    // (`MekanismRecipeType.getRecipesUncached`), so the smelter smelts exactly
    // what a furnace smelts. Each row is `(input item, output item, count)`,
    // sorted by input; a tag ingredient is expanded to its member items.
    // Source recipes: {recipes}. Rows: {rows}.
    static TABLE: &[(&str, &str, u32)] = &[
"""


def tag_members(resources: Path, tag: str, seen: set[str]) -> list[str]:
    if tag in seen:
        return []
    seen.add(tag)
    namespace, path = tag.split(":", 1)
    file = resources / "data" / namespace / "tags" / "item" / f"{path}.json"
    if not file.is_file():
        return []
    data = json.loads(file.read_text())
    out: list[str] = []
    for value in data.get("values", []):
        name = value if isinstance(value, str) else value.get("id", "")
        if not name:
            continue
        if name.startswith("#"):
            out.extend(tag_members(resources, name[1:], seen))
        else:
            out.append(name if ":" in name else "minecraft:" + name)
    return out


def ingredient_items(resources: Path, ingredient) -> list[str]:
    entries = ingredient if isinstance(ingredient, list) else [ingredient]
    out: list[str] = []
    for entry in entries:
        if isinstance(entry, str):
            if entry.startswith("#"):
                out.extend(tag_members(resources, entry[1:], set()))
            else:
                out.append(entry if ":" in entry else "minecraft:" + entry)
    return out


def rows(resources: Path) -> list[tuple[str, str, int]]:
    recipe_dir = resources / "data" / "minecraft" / "recipe"
    table: dict[tuple[str, str], int] = {}
    count = 0
    for path in sorted(recipe_dir.glob("*.json")):
        data = json.loads(path.read_text())
        if data.get("type") != "minecraft:smelting":
            continue
        count += 1
        result = data["result"]
        output = result["id"]
        amount = int(result.get("count", 1))
        for item in ingredient_items(resources, data["ingredient"]):
            key = (item, output)
            # A tag can name the same item twice; the larger count wins, as a
            # later recipe of the same pair would in the manager.
            table[key] = max(table.get(key, 0), amount)
    return sorted((item, output, amount) for (item, output), amount in table.items()), count


def render(resources: Path) -> str:
    table, count = rows(resources)
    body = "".join(f'        ("{i}", "{o}", {c}),\n' for i, o, c in table)
    return HEADER.format(recipes=count, rows=len(table)) + body + "    ];\n"


def registered_items() -> set[str]:
    """The non-vanilla items this port registers: one file per id under `horizon/item/`."""
    item_dir = ROOT / "data" / "mekanism" / "horizon" / "item"
    return {f"mekanism:{path.stem}" for path in item_dir.glob("*.json")}


def mekanism_rows(jar: Path) -> list[tuple[str, str, int]]:
    """The jar's own `minecraft:smelting` recipes whose both ends this port has.

    `data/mekanism/recipe/processing/<material>/ingot/from_dust_smelting.json` is
    a vanilla-type recipe (one dust, 200 ticks) that Mekanism adds to its own
    smelting type, so the Energized Smelter smelts it. A row whose ingredient or
    result the port does not register is dropped, exactly as `gen_crushing.py`
    does; bronze, for instance, has no dust or ingot here.
    """
    known = registered_items()
    table: dict[str, tuple[str, int]] = {}
    with zipfile.ZipFile(jar) as archive:
        for name in sorted(archive.namelist()):
            if not name.startswith("data/mekanism/recipe/processing/"):
                continue
            if not name.endswith("/ingot/from_dust_smelting.json"):
                continue
            data = json.loads(archive.read(name))
            if data.get("type") != "minecraft:smelting":
                continue
            item = data["ingredient"].get("item", "")
            result = data["result"]
            output = result["id"]
            if item not in known:
                continue
            if not output.startswith("minecraft:") and output not in known:
                continue
            table[item] = (output, int(result.get("count", 1)))
    return sorted((item, output, count) for item, (output, count) in table.items())


def replace(text: str, block: str) -> str:
    start = text.index(BEGIN)
    end = text.index(END)
    return text[: start + len(BEGIN)] + block + text[end:]


def committed_rows(text: str) -> list[tuple[str, str, int]]:
    """The rows `TABLE` holds, however rustfmt wrapped them."""
    block = text[text.index(BEGIN) + len(BEGIN) : text.index(END)]
    return [
        (item, output, int(count))
        for item, output, count in re.findall(
            r'"([^"]+)"\s*,\s*"([^"]+)"\s*,\s*(\d+)\s*\)?\s*,', block
        )
    ]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--resources", type=Path, default=DEFAULT_RESOURCES)
    parser.add_argument("--jar", type=Path, default=DEFAULT_JAR)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--rows", action="store_true", help="print the vanilla rows as JSON")
    parser.add_argument(
        "--mekanism-rows", action="store_true", help="print the jar's own dust rows as JSON"
    )
    args = parser.parse_args()
    if args.rows or args.mekanism_rows:
        if args.mekanism_rows:
            table = mekanism_rows(args.jar)
        else:
            table, _ = rows(args.resources)
        print(json.dumps(
            [{"input": i, "output": o, "count": c} for i, o, c in table],
            indent=1,
        ))
        return 0
    text = OUT.read_text()
    expected, count = rows(args.resources)
    if args.check:
        committed = committed_rows(text)
        if committed != expected:
            print(
                f"{OUT} is stale: {len(committed)} rows committed, {len(expected)} "
                f"from the {count} 26.3 smelting recipes"
            )
            return 1
        print(f"{OUT} is up to date ({len(expected)} rows from {count} recipes)")
        return 0
    OUT.write_text(replace(text, render(args.resources)))
    print(f"updated {OUT} ({len(text)} bytes)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
