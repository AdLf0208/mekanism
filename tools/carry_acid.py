#!/usr/bin/env python3
"""Carry the Chemical Oxidizer, Chemical Infuser and sulfur dust art from the jar.

Carried byte for byte from `Mekanism-1.21.1-10.7.19.85.jar` (re-serialised
compactly for JSON): the two blocks' own textures, the sulfur dust texture,
their block and item models and the dust's item model. The two blockstates are
**authored**, not carried: the jar's machines have a `facing` state, while the
barrel template this port uses for every machine has `facing` x `open`, so each
needs the same twelve variants the dissolution chamber's blockstate has.

MIT: the jar's own bytes, with the licence notices the package already ships.

Usage:
  python3 tools/carry_acid.py [--jar PATH] [--check]
"""
import argparse
import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
ASSETS = ROOT / "assets" / "mekanism"

TEXTURES = [
    "textures/block/models/chemical_oxidizer.png",
    "textures/block/models/chemical_oxidizer_elements.png",
    "textures/block/models/chemical_infuser.png",
    "textures/item/dust_sulfur.png",
]
MODELS = [
    "models/block/chemical_oxidizer.json",
    "models/block/chemical_infuser.json",
    "models/item/chemical_oxidizer.json",
    "models/item/chemical_infuser.json",
    "models/item/dust_sulfur.json",
]
BLOCKS = ["chemical_oxidizer", "chemical_infuser"]


def compact(value) -> str:
    return json.dumps(value, separators=(",", ":"), sort_keys=True) + "\n"


def blockstate(name: str) -> str:
    """The barrel template's facing x open variants, as the other machines use."""
    variants = {}
    for facing, rot in [
        ("north", None),
        ("south", 180),
        ("east", 90),
        ("west", -90),
        ("up", None),
        ("down", None),
    ]:
        for open_state in ("false", "true"):
            model = {"model": f"mekanism:block/{name}"}
            if facing == "up":
                model["x"] = -90
            elif facing == "down":
                model["x"] = 90
            elif rot is not None:
                model["y"] = rot
            variants[f"facing={facing},open={open_state}"] = model
    return json.dumps({"variants": variants}, indent=2, sort_keys=True) + "\n"


def derive(jar: zipfile.ZipFile) -> dict[str, bytes]:
    out: dict[str, bytes] = {}
    for name in TEXTURES:
        out[name] = jar.read(f"assets/mekanism/{name}")
    for name in MODELS:
        out[name] = compact(json.loads(jar.read(f"assets/mekanism/{name}"))).encode()
    for name in BLOCKS:
        out[f"blockstates/{name}.json"] = blockstate(name).encode()
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
