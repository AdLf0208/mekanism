#!/usr/bin/env python3
"""Carry the Chemical Dissolution Chamber's art from the jar, reproducibly.

The jar's bytes are six files and one is derived from them:

- `assets/mekanism/textures/block/models/chemical_dissolution_chamber_1.png`
  and `_2.png` — the block's own faces;
- `assets/mekanism/textures/block/bottom.png` and `glass.png` — the two shared
  textures its model names (the ports' textures were already carried by the
  washer);
- `assets/mekanism/models/item/chemical_dissolution_chamber.json` — the jar's
  own JSON, re-serialised compactly;
- `assets/mekanism/models/block/chemical_dissolution_chamber.json` — the jar's
  model, **flattened**: upstream ships a `neoforge:composite` of a cutout child
  and a translucent glass one, and the pin's model loader has no composite, so
  this tool emits one model holding both children's elements with
  `render_type: cutout` (the window therefore draws cutout rather than
  translucent — recorded as a deviation in `notes.md`).

The **blockstate** is authored, not carried: the block is the barrel template
(`facing` x `open`), so it needs the twelve variants the washer's has, while the
jar's machine has only `facing` (recorded as a deviation).

The licence is MIT, read from the jar's `META-INF/neoforge.mods.toml`, so the
bytes travel unchanged in the package.

Usage:
  python3 tools/carry_dissolution.py [--jar PATH] [--check]

`--check` re-derives every file and fails when the committed bytes differ, so a
hand edit to carried art is caught instead of shipped.
"""
import argparse
import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
ASSETS = ROOT / "assets" / "mekanism"

TEXTURES = [
    "textures/block/bottom.png",
    "textures/block/glass.png",
    "textures/block/models/chemical_dissolution_chamber_1.png",
    "textures/block/models/chemical_dissolution_chamber_2.png",
]

BLOCK_MODEL = "models/block/chemical_dissolution_chamber.json"
ITEM_MODEL = "models/item/chemical_dissolution_chamber.json"


def compact(value) -> str:
    return json.dumps(value, separators=(",", ":"), sort_keys=True) + "\n"


def flatten_model(jar: zipfile.ZipFile) -> str:
    """One model from the jar's composite: both children's elements, cutout."""
    model = json.loads(jar.read(f"assets/mekanism/{BLOCK_MODEL}"))
    textures: dict[str, str] = {}
    elements: list[dict] = []
    for child in model["children"].values():
        textures.update(child["textures"])
        elements.extend(child["elements"])
    flat = {
        "credit": "Model by CyanideX",
        "render_type": "minecraft:cutout",
        "textures": dict(sorted(textures.items())),
        "elements": elements,
    }
    return compact(flat)


def derive(jar: zipfile.ZipFile) -> dict[str, bytes]:
    out: dict[str, bytes] = {}
    for name in TEXTURES:
        out[name] = jar.read(f"assets/mekanism/{name}")
    out[ITEM_MODEL] = compact(json.loads(jar.read(f"assets/mekanism/{ITEM_MODEL}"))).encode()
    out[BLOCK_MODEL] = flatten_model(jar).encode()
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
