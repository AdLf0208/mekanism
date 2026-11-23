#!/usr/bin/env python3
"""Carry the ore-processing shards' art from the jar, reproducibly.

A shard is one item texture and one item model, both the jar's own bytes:

- `assets/mekanism/textures/item/shard_<material>.png`;
- `assets/mekanism/models/item/shard_<material>.json` (the jar's
  `minecraft:item/generated` model, which names the texture above).

Slice 5 carried the iron, gold and copper shards by hand; slice 11 adds lead,
osmium, tin and uranium, and this tool re-derives all **seven** so a hand edit to
either half is caught instead of shipped. The item declarations
(`data/mekanism/horizon/item/shard_<material>.json`) are authored, not carried:
the jar's shard is a plain `Item` on the same `minecraft:gunpowder` template the
port uses for every item it registers.

The licence is MIT, read from the jar's `META-INF/neoforge.mods.toml`, so the
bytes travel unchanged in the package.

Usage:
  python3 tools/carry_shards.py [--jar PATH] [--check]

`--check` re-derives every file and fails when the committed bytes differ.
"""
import argparse
import json
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_JAR = ROOT.parent / "_shared" / "carry-work" / "mekanism" / "Mekanism-1.21.1-10.7.19.85.jar"
ASSETS = ROOT / "assets" / "mekanism"

# The seven materials whose shards this port registers: the ores the injecting
# table finishes (vanilla's three) and the four the jar's own `c:ores/*` rows
# cannot reach here because 26.3 has no such ore — their shards still exist as
# the crystal leg's product.
SHARDS = ["iron", "gold", "copper", "lead", "osmium", "tin", "uranium"]


def compact(value) -> str:
    return json.dumps(value, separators=(",", ":"), sort_keys=True) + "\n"


def derive(jar: zipfile.ZipFile) -> dict[str, bytes]:
    out: dict[str, bytes] = {}
    for material in SHARDS:
        out[f"textures/item/shard_{material}.png"] = jar.read(
            f"assets/mekanism/textures/item/shard_{material}.png"
        )
        model = f"models/item/shard_{material}.json"
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
