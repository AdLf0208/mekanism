#!/usr/bin/env python3
"""Carry the four extra metals' ore-chain art from the jar, reproducibly.

Slice 12 registers the rest of the jar's ore chain for lead, osmium, tin and
uranium: the clump the Purification Chamber makes, the dirty dust the Crusher
makes from it, the dust the Enrichment Chamber makes from that, and the ingot
the Energized Smelter makes from the dust. Each is one item texture and one item
model, both the jar's own bytes:

- `assets/mekanism/textures/item/<kind>_<material>.png`;
- `assets/mekanism/models/item/<kind>_<material>.json` (the jar's
  `minecraft:item/generated` model, which names the texture above).

The item declarations (`data/mekanism/horizon/item/<kind>_<material>.json`) are
authored, not carried: each is a plain `Item` on the same `minecraft:gunpowder`
template the port uses for every item it registers. The three metals the earlier
slices carried by hand (iron, gold, copper) are re-derived too, so a hand edit to
either half is caught instead of shipped; their ingots are vanilla items, so
only the four jar ingots are carried.

The licence is MIT, read from the jar's `META-INF/neoforge.mods.toml`, so the
bytes travel unchanged in the package.

Usage:
  python3 tools/carry_ore_chain.py [--jar PATH] [--check]

`--check` re-derives every file and fails when the committed bytes differ.
"""
import argparse
import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
ASSETS = ROOT / "assets" / "mekanism"

# The materials whose full chain this port ships, and which of the four items
# the jar carries for them. Iron, gold and copper smelt into vanilla ingots, so
# the jar has no `ingot_<m>` for those; the four slice-12 metals have all four.
MATERIALS = ["iron", "gold", "copper", "lead", "osmium", "tin", "uranium"]
KINDS = ["clump", "dirty_dust", "dust", "ingot"]
VANILLA_INGOTS = {"iron", "gold", "copper"}


def carried_items() -> list[str]:
    """The `<kind>_<material>` ids whose art the jar holds for this port."""
    return [
        f"{kind}_{material}"
        for material in MATERIALS
        for kind in KINDS
        if not (kind == "ingot" and material in VANILLA_INGOTS)
    ]


def compact(value) -> str:
    return json.dumps(value, separators=(",", ":"), sort_keys=True) + "\n"


def derive(jar: zipfile.ZipFile) -> dict[str, bytes]:
    out: dict[str, bytes] = {}
    for item in carried_items():
        out[f"textures/item/{item}.png"] = jar.read(
            f"assets/mekanism/textures/item/{item}.png"
        )
        model = f"models/item/{item}.json"
        out[model] = compact(json.loads(jar.read(f"assets/mekanism/{model}"))).encode()
    return out


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--jar", type=Path, default=DEFAULT_JAR)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    with zipfile.ZipFile(args.jar) as jar:
        files = derive(jar)
    stale = []
    for name, data in sorted(files.items()):
        target = ASSETS / name
        if args.check:
            if not target.exists() or target.read_bytes() != data:
                stale.append(name)
            continue
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(data)
    if args.check:
        if stale:
            print(f"{len(stale)} carried file(s) differ from the jar: {', '.join(stale)}")
            return 1
        print(f"{len(files)} carried files match the jar")
        return 0
    print(f"carried {len(files)} files from {args.jar.name}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
