#!/usr/bin/env python3
"""Carry the rotary Condensentrator art from the jar.

Carried byte for byte from `Mekanism-1.21.1-10.7.19.85.jar` (JSON re-serialised
compactly): the machine's own texture, its block and item models. The jar's
blockstate has only four `facing` variants; this port's barrel template has
`facing` x `open`, so the carried model gets the same twelve variants the other
machines' blockstates have. The remaining model textures (`ports`,
`ports_large`, `ports_large_led`, `ports_led`, `bottom`) are already carried by
earlier slices and referenced by their existing ids.

MIT: the jar's own bytes, with the licence notices the package already ships.

Usage:
  python3 tools/carry_condensentrator.py [--jar PATH] [--check]
"""
import argparse
import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
ASSETS = ROOT / "assets" / "mekanism"

TEXTURES = [
    "textures/block/models/rotary_condensentrator.png",
]
MODELS = [
    "models/block/rotary_condensentrator.json",
    "models/item/rotary_condensentrator.json",
]


def compact(value) -> str:
    return json.dumps(value, separators=(",", ":"), sort_keys=True) + "\n"


def blockstate() -> str:
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
            model = {"model": "mekanism:block/rotary_condensentrator"}
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
    out["blockstates/rotary_condensentrator.json"] = blockstate().encode()
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
