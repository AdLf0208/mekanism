#!/usr/bin/env python3
"""Carry the four metals' storage-block and nugget family from the jar.

Slice 14 is the family the port's own gap list named after slice 13: the four
ingots (lead, osmium, tin, uranium) have no storage block, no nugget and none of
the jar's crafting recipes that turn one into the other. The jar holds, per
metal:

- `assets/mekanism/models/block/storage/<m>.json`       (a `cube_all` model)
- `assets/mekanism/blockstates/block_<m>.json`
- `assets/mekanism/models/item/{block,nugget}_<m>.json`
- `assets/mekanism/textures/block/block_<m>.png`
- `assets/mekanism/textures/item/nugget_<m>.png`
- `data/c/tags/item/{ingots,nuggets,storage_blocks}/<m>.json`  (one value each:
  the port's own item, so carrying them is exact)
- `data/mekanism/recipe/processing/<m>/storage_blocks/from_ingots.json`
- `data/mekanism/recipe/processing/<m>/ingot/from_block.json`
- `data/mekanism/recipe/processing/<m>/ingot/from_nuggets.json`
- `data/mekanism/recipe/processing/<m>/nugget/from_ingot.json`

The recipes are the jar's, translated to 26.3's ingredient dialect only (the
slice-3 defect: 1.21.1's `{"item": …}` / `{"tag": …}` objects are rejected by
26.3's shaped parser, which takes a bare id or `#tag`). Nothing about the shapes
is invented: `from_ingots` and `from_nuggets` are the jar's 3x3 with the centre
cell naming the mod's own item and the other eight the `c:` tag, and `from_block`
and `from_ingot` are the jar's shapeless one-ingredient rows. The jar has **no**
`nugget/from_block` and **no** `dust/from_block` row for these metals; neither is
created here (see notes.md).

Authored, not carried (and so not written by this tool): the block and item
declarations under `data/mekanism/horizon/`, the loot tables, the lang keys, the
creative tab entries and the block's three vanilla tag memberships.

Usage:
  python3 tools/carry_storage.py [--jar PATH] [--check]
  python3 tools/carry_storage.py --rows          # the in-game crafting rows

`--check` re-derives every file and fails when the committed bytes differ (the
slice's red before the family exists, its green after).
`--rows` reads the **committed** recipe files and the committed `c:` tags, so the
in-game check crafts exactly what the package ships.
"""
import argparse
import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"

# The four metals slice 12 registered ingots for. Iron, gold and copper are
# vanilla items/blocks in this port, so the family here is their own four.
MATERIALS = ["lead", "osmium", "tin", "uranium"]
# The jar's item tags this family needs: the two the recipes name, and the
# block's own.
TAG_KINDS = ["ingots", "nuggets", "storage_blocks"]


def pretty(value) -> str:
    return json.dumps(value, indent=2, sort_keys=True) + "\n"


def compact(value) -> str:
    return json.dumps(value, separators=(",", ":"), sort_keys=True) + "\n"


def ingredient(value) -> str:
    """A 1.21.1 ingredient object as 26.3's string form: a bare id or `#tag`."""
    if isinstance(value, str):
        return value
    if "tag" in value:
        return "#" + value["tag"]
    return value["item"]


def recipe(jar: zipfile.ZipFile, path: str) -> str:
    src = json.loads(jar.read(f"data/mekanism/recipe/{path}"))
    kind = src["type"]
    out = {"type": kind, "category": src.get("category", "misc")}
    if kind == "minecraft:crafting_shaped":
        out["pattern"] = src["pattern"]
        out["key"] = {k: ingredient(v) for k, v in src["key"].items()}
    elif kind == "minecraft:crafting_shapeless":
        out["ingredients"] = [ingredient(v) for v in src["ingredients"]]
    else:
        raise SystemExit(f"{path}: unexpected recipe type {kind}")
    out["result"] = src["result"]
    return pretty(out)


def derive(jar: zipfile.ZipFile) -> dict[str, tuple[str, bytes]]:
    """target path -> ("carried" | "translated", bytes) for every file."""
    out: dict[str, tuple[str, bytes]] = {}
    for m in MATERIALS:
        for asset in (
            f"models/block/storage/{m}.json",
            f"blockstates/block_{m}.json",
            f"models/item/block_{m}.json",
            f"models/item/nugget_{m}.json",
        ):
            out[f"assets/mekanism/{asset}"] = (
                "carried",
                compact(json.loads(jar.read(f"assets/mekanism/{asset}"))).encode(),
            )
        for texture in (f"block/block_{m}.png", f"item/nugget_{m}.png"):
            out[f"assets/mekanism/textures/{texture}"] = (
                "carried",
                jar.read(f"assets/mekanism/textures/{texture}"),
            )
        for kind in TAG_KINDS:
            out[f"data/c/tags/item/{kind}/{m}.json"] = (
                "carried",
                compact(json.loads(jar.read(f"data/c/tags/item/{kind}/{m}.json"))).encode(),
            )
        for path in (
            f"processing/{m}/storage_blocks/from_ingots.json",
            f"processing/{m}/ingot/from_block.json",
            f"processing/{m}/ingot/from_nuggets.json",
            f"processing/{m}/nugget/from_ingot.json",
        ):
            out[f"data/mekanism/recipe/{path}"] = ("translated", recipe(jar, path).encode())
    return out


def expand(value: str, tags: dict[str, list[str]]) -> list[str]:
    """One ingredient as the items it matches: a tag's members, or itself."""
    if value.startswith("#"):
        return tags.get(value[1:], [])
    return [value]


def rows() -> list[dict]:
    """What the in-game check crafts, read back from the committed package."""
    tags = {
        f"c:{kind}/{m}": json.loads(
            (ROOT / f"data/c/tags/item/{kind}/{m}.json").read_text()
        )["values"]
        for m in MATERIALS
        for kind in TAG_KINDS
    }
    out = []
    for m in MATERIALS:
        for path, label in (
            (f"processing/{m}/storage_blocks/from_ingots.json", f"block_{m} from 9 ingot_{m}"),
            (f"processing/{m}/ingot/from_block.json", f"ingot_{m} x9 from block_{m}"),
            (f"processing/{m}/ingot/from_nuggets.json", f"ingot_{m} from 9 nugget_{m}"),
            (f"processing/{m}/nugget/from_ingot.json", f"nugget_{m} x9 from ingot_{m}"),
        ):
            doc = json.loads((ROOT / f"data/mekanism/recipe/{path}").read_text())
            grid: dict[int, str] = {}
            if doc["type"] == "minecraft:crafting_shaped":
                # Row-major: the 3x3 grid, every cell a key of the jar's pattern.
                for y, line in enumerate(doc["pattern"]):
                    for x, c in enumerate(line):
                        if c == " ":
                            continue
                        members = expand(doc["key"][c], tags)
                        if len(members) != 1:
                            raise SystemExit(
                                f"{path}: {c} matches {members}, so the row cannot name one item"
                            )
                        grid[y * 3 + x] = members[0]
            else:
                names = [expand(i, tags) for i in doc["ingredients"]]
                if len(names) != 1 or len(names[0]) != 1:
                    raise SystemExit(f"{path}: shapeless row is not one item: {names}")
                # A single shapeless ingredient goes in the grid's centre.
                grid[4] = names[0][0]
            result = doc["result"]
            out.append(
                {
                    "label": label,
                    "grid": {str(slot): item for slot, item in sorted(grid.items())},
                    "want": {"item": result["id"], "count": result.get("count", 1)},
                    "consumed": next(iter(grid.values())),
                }
            )
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, default=DEFAULT_JAR)
    parser.add_argument("--check", action="store_true")
    parser.add_argument("--rows", action="store_true")
    args = parser.parse_args()
    if args.rows:
        print(json.dumps(rows(), indent=2))
        return 0
    with zipfile.ZipFile(args.jar) as jar:
        files = derive(jar)
    stale, missing = [], []
    for name, (_, data) in sorted(files.items()):
        target = ROOT / name
        if target.exists() and target.read_bytes() == data:
            continue
        if target.exists():
            stale.append(name)
        else:
            missing.append(name)
        if not args.check:
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
    if args.check:
        if missing or stale:
            if missing:
                print(f"{len(missing)} file(s) the jar carries are not in the package:")
                for name in missing:
                    print(f"  missing {name}")
            if stale:
                print(f"{len(stale)} carried file(s) differ from the jar:")
                for name in stale:
                    print(f"  differs {name}")
            return 1
        print(f"{len(files)} carried files match the jar")
        return 0
    print(f"carried {len(files)} files from {args.jar.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
