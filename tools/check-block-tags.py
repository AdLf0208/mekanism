#!/usr/bin/env python3
"""Every declared block needs the tag that makes a pickaxe its correct tool and
a loot table, or a survival break drops nothing.

The engine's `Items::correct_for_drops` asks the held item's tool rules whether
the block is a member of `minecraft:mineable/pickaxe`; `player_destroy_block`
passes `harvest = !requires_tool || correct_for_drops(...)`, so a block declared
`requires_tool: true` that is missing from the tag **breaks without dropping**.
The 0.2.14 e2e caught exactly that for `mekanism:chemical_dissolution_chamber`
(every other machine was already in the tag from its own slice); this tool makes
the class checkable instead of waiting for the break phase.

It also checks each block has its `data/mekanism/loot_table/blocks/<path>.json`,
because the tag alone would let the block break and drop nothing.

Usage:
  python3 tools/check-block-tags.py [--port DIR] [--check-file PATH]

Exit 0 when every declared block is in the tag and has a loot table; exit 1 and
prints the missing names otherwise.
"""
import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
TAG = Path("data/minecraft/tags/block/mineable/pickaxe.json")
LOOT = Path("data/mekanism/loot_table/blocks")


def declared_blocks(port: Path) -> list[str]:
    blocks = []
    for path in sorted((port / "data/mekanism/horizon/block").glob("*.json")):
        data = json.loads(path.read_text())
        if data.get("item"):
            blocks.append(f"mekanism:{path.stem}")
    return blocks


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--port", type=Path, default=ROOT)
    args = parser.parse_args()
    port = args.port
    tag = json.loads((port / TAG).read_text())["values"]
    blocks = declared_blocks(port)
    missing_tag = [block for block in blocks if block not in tag]
    missing_loot = [
        block for block in blocks if not (port / LOOT / f"{block.split(':', 1)[1]}.json").exists()
    ]
    problems = []
    for block in missing_tag:
        problems.append(f"{block} is declared but not in minecraft:mineable/pickaxe")
    for block in missing_loot:
        problems.append(f"{block} has no {LOOT}/{block.split(':', 1)[1]}.json")
    if problems:
        for problem in problems:
            print(f"  {problem}")
        print(f"{len(problems)} problem(s) over {len(blocks)} declared blocks")
        return 1
    print(f"{len(blocks)} declared blocks are in mineable/pickaxe and have a loot table")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
