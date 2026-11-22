#!/usr/bin/env python3
"""Craft every shaped recipe the package carries, in a real crafting table.

Called by `tools/e2e.sh` with the open game's control socket. The harness's
crafting table exposes the 3x3 grid as slots 100-108 (row-major) and the result
as 200 (`ctl screen`). For each recipe this:

1. resets the table and clears the inventory;
2. gives exactly the pattern's ingredients;
3. places them with right-clicks (one item per slot);
4. asserts the result's **id and count**;
5. shift-clicks the result into the inventory and asserts it arrived;
6. asserts the grid is empty and one distinctive ingredient was consumed.

Two deliberately wrong patterns must leave the result empty, so the check cannot
pass on a table that crafts whatever it is handed. Exits 1 on the first
mismatch, printing the observed result.

    python3 tools/craft_check.py SOCK CTL X Y Z
"""
import json
import subprocess
import sys

# The crafting table's grid, row-major, and its result.
GRID = {
    "tl": 100, "tm": 101, "tr": 102,
    "ml": 103, "mm": 104, "mr": 105,
    "bl": 106, "bm": 107, "br": 108,
}
RESULT = 200


def main() -> int:
    sock, ctl, x, y, z = sys.argv[1], sys.argv[2], sys.argv[3], sys.argv[4], sys.argv[5]

    def run(*args):
        out = subprocess.run([ctl, "--socket", sock, *args], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"ctl {' '.join(args)}: {out.stdout}{out.stderr}")

    def wait(ticks=4):
        run("wait", str(ticks))

    def slots():
        screen = run("screen")
        if "slots" not in screen:
            raise SystemExit(f"no menu is open: {screen}")
        return screen["slots"]

    def find(item):
        for slot in slots():
            if slot.get("item") == item:
                return slot["slot"]
        raise SystemExit(f"missing ingredient {item} in the inventory")

    def result():
        stack = [s for s in slots() if s["slot"] == RESULT]
        if not stack or not stack[0].get("item"):
            return None
        return stack[0]

    def grid_is_empty():
        return all(not s.get("item") for s in slots() if 100 <= s["slot"] <= 108)

    def count(item):
        return run("inventory").get("counts", {}).get(item, 0)

    def place(item, targets):
        # One pickup per target: a max-stack-1 ingredient cannot fill two slots
        # from one held stack, while a stack of four can.
        for target in targets:
            slot = find(item)
            run("click-slot", str(slot))
            wait(1)
            run("click-slot", str(target), "right")
            wait(1)
            run("click-slot", str(slot))
            wait(1)

    def setup(ingredients):
        try:
            run("close")
        except SystemExit:
            pass
        wait(4)
        # Replace the table so its block entity, and therefore the grid, is fresh.
        run("setblock", x, y, z, "minecraft:air")
        run("setblock", x, y, z, "minecraft:crafting_table")
        wait(2)
        run("command", f"tp @s {float(x) - 0.5} {y} {float(z) + 0.5}")
        wait(3)
        run("command", "clear @s")
        for item, n in ingredients:
            run("give", item, str(n))
        wait(3)
        run("interact", x, y, z)
        wait(5)

    failures = []
    recipes = [0]

    def check(label, ingredients, placements, want_item, want_count, consumed_item):
        recipes[0] += 1
        setup(ingredients)
        for item, targets in placements:
            place(item, targets)
        got = result()
        print(f"  {label} -> {json.dumps(got)}")
        if not got or got.get("item") != want_item or got.get("count") != want_count:
            grid = [s for s in slots() if 100 <= s["slot"] <= 108]
            print(f"    grid: {json.dumps(grid)}")
            failures.append(f"{label}: expected {want_count} {want_item}, got {got}")
            return
        run("click-slot", str(RESULT), "shift")
        wait(4)
        made = count(want_item)
        if made < want_count:
            failures.append(f"{label}: the crafted stack reached the inventory as {made}")
        if not grid_is_empty():
            failures.append(f"{label}: the grid still holds ingredients after the craft")
        left = count(consumed_item)
        if left != 0:
            failures.append(f"{label}: {consumed_item} was not consumed ({left} left)")
        run("command", "clear @s")
        wait(2)

    def empty_control(label, ingredients, placements):
        setup(ingredients)
        for item, targets in placements:
            place(item, targets)
        got = result()
        print(f"  {label} -> {json.dumps(got)}")
        if got is not None:
            failures.append(f"{label}: the control crafted {got['item']} x{got['count']}")

    def every(*slots):
        return list(slots)

    # --- the five blocks of slices 1-3 ------------------------------------
    check(
        "energized smelter: 4 iron + 2 redstone + 2 glass + iron block",
        [
            ("minecraft:iron_ingot", 4),
            ("minecraft:redstone", 2),
            ("minecraft:glass", 2),
            ("minecraft:iron_block", 1),
        ],
        [
            ("minecraft:iron_ingot", every(100, 102, 106, 108)),
            ("minecraft:redstone", every(101, 107)),
            ("minecraft:glass", every(103, 105)),
            ("minecraft:iron_block", every(104)),
        ],
        "mekanism:energized_smelter", 1, "minecraft:glass",
    )
    check(
        "enrichment chamber: 4 iron + 2 redstone + 2 gold + iron block",
        [
            ("minecraft:iron_ingot", 4),
            ("minecraft:redstone", 2),
            ("minecraft:gold_ingot", 2),
            ("minecraft:iron_block", 1),
        ],
        [
            ("minecraft:iron_ingot", every(100, 102, 106, 108)),
            ("minecraft:redstone", every(101, 107)),
            ("minecraft:gold_ingot", every(103, 105)),
            ("minecraft:iron_block", every(104)),
        ],
        "mekanism:enrichment_chamber", 1, "minecraft:gold_ingot",
    )
    check(
        "crusher: 4 redstone + 2 copper + 2 magma + iron block",
        [
            ("minecraft:redstone", 4),
            ("minecraft:copper_ingot", 2),
            ("minecraft:magma_block", 2),
            ("minecraft:iron_block", 1),
        ],
        [
            ("minecraft:redstone", every(100, 102, 106, 108)),
            ("minecraft:copper_ingot", every(101, 107)),
            ("minecraft:magma_block", every(103, 105)),
            ("minecraft:iron_block", every(104)),
        ],
        "mekanism:crusher", 1, "minecraft:magma_block",
    )
    check(
        "basic energy cube: 4 iron + 2 redstone blocks + 2 copper + iron block",
        [
            ("minecraft:iron_ingot", 4),
            ("minecraft:redstone_block", 2),
            ("minecraft:copper_ingot", 2),
            ("minecraft:iron_block", 1),
        ],
        [
            ("minecraft:iron_ingot", every(100, 102, 106, 108)),
            ("minecraft:redstone_block", every(101, 107)),
            ("minecraft:copper_ingot", every(103, 105)),
            ("minecraft:iron_block", every(104)),
        ],
        "mekanism:basic_energy_cube", 1, "minecraft:redstone_block",
    )
    check(
        "basic chemical tank: 4 iron + 4 glass",
        [
            ("minecraft:iron_ingot", 4),
            ("minecraft:glass", 4),
        ],
        [
            ("minecraft:iron_ingot", every(100, 102, 106, 108)),
            ("minecraft:glass", every(101, 107, 103, 105)),
        ],
        "mekanism:basic_chemical_tank", 1, "minecraft:glass",
    )
    check(
        "gauge dropper: iron + 5 glass panes",
        [
            ("minecraft:iron_ingot", 1),
            ("minecraft:glass_pane", 5),
        ],
        [
            ("minecraft:iron_ingot", every(101)),
            ("minecraft:glass_pane", every(103, 105, 106, 107, 108)),
        ],
        "mekanism:gauge_dropper", 1, "minecraft:glass_pane",
    )
    check(
        "basic universal cable: 2 iron + redstone, eight at a time",
        [("minecraft:iron_ingot", 2), ("minecraft:redstone", 1)],
        [
            ("minecraft:iron_ingot", every(100, 102)),
            ("minecraft:redstone", every(101)),
        ],
        "mekanism:basic_universal_cable", 8, "minecraft:redstone",
    )
    check(
        "chemical injection chamber: 4 gold + 3 iron + iron block + copper block",
        [
            ("minecraft:gold_ingot", 4),
            ("minecraft:iron_ingot", 3),
            ("minecraft:iron_block", 1),
            ("minecraft:copper_block", 1),
        ],
        [
            ("minecraft:gold_ingot", every(100, 102, 106, 108)),
            ("minecraft:iron_ingot", every(101, 103, 105)),
            ("minecraft:iron_block", every(104)),
            ("minecraft:copper_block", every(107)),
        ],
        "mekanism:chemical_injection_chamber", 1, "minecraft:copper_block",
    )
    check(
        "purification chamber: 4 gold + 2 redstone + 2 copper + an enrichment chamber",
        [
            ("minecraft:gold_ingot", 4),
            ("minecraft:redstone", 2),
            ("minecraft:copper_ingot", 2),
            ("mekanism:enrichment_chamber", 1),
        ],
        [
            ("minecraft:gold_ingot", every(100, 102, 106, 108)),
            ("minecraft:redstone", every(101, 107)),
            ("minecraft:copper_ingot", every(103, 105)),
            ("mekanism:enrichment_chamber", every(104)),
        ],
        "mekanism:purification_chamber", 1, "mekanism:enrichment_chamber",
    )

    # --- slice 7: the five basic factories --------------------------------
    for factory, base in (
        ("smelting", "mekanism:energized_smelter"),
        ("enriching", "mekanism:enrichment_chamber"),
        ("crushing", "mekanism:crusher"),
        ("injecting", "mekanism:chemical_injection_chamber"),
        ("purifying", "mekanism:purification_chamber"),
    ):
        check(
            f"basic {factory} factory: 4 gold + 2 redstone + 2 iron + a {base}",
            [
                ("minecraft:gold_ingot", 4),
                ("minecraft:redstone", 2),
                ("minecraft:iron_ingot", 2),
                (base, 1),
            ],
            [
                ("minecraft:gold_ingot", every(100, 102, 106, 108)),
                ("minecraft:redstone", every(101, 107)),
                ("minecraft:iron_ingot", every(103, 105)),
                (base, every(104)),
            ],
            f"mekanism:basic_{factory}_factory", 1, base,
        )

    # --- controls: a wrong ingredient leaves the result empty -------------
    empty_control(
        "smelter with a stick where the glass goes",
        [
            ("minecraft:iron_ingot", 4),
            ("minecraft:redstone", 2),
            ("minecraft:stick", 2),
            ("minecraft:iron_block", 1),
        ],
        [
            ("minecraft:iron_ingot", every(100, 102, 106, 108)),
            ("minecraft:redstone", every(101, 107)),
            ("minecraft:stick", every(103, 105)),
            ("minecraft:iron_block", every(104)),
        ],
    )
    empty_control(
        "chemical tank with a stick where the glass goes",
        [
            ("minecraft:iron_ingot", 4),
            ("minecraft:stick", 4),
        ],
        [
            ("minecraft:iron_ingot", every(100, 102, 106, 108)),
            ("minecraft:stick", every(101, 107, 103, 105)),
        ],
    )
    empty_control(
        "gauge dropper with a dirt where the glass panes go",
        [
            ("minecraft:iron_ingot", 1),
            ("minecraft:dirt", 5),
        ],
        [
            ("minecraft:iron_ingot", every(101)),
            ("minecraft:dirt", every(103, 105, 106, 107, 108)),
        ],
    )
    empty_control(
        "crusher with a stick where the magma goes",
        [
            ("minecraft:redstone", 6),
            ("minecraft:copper_ingot", 2),
            ("minecraft:stick", 2),
            ("minecraft:iron_block", 1),
        ],
        [
            ("minecraft:redstone", every(100, 102, 106, 108)),
            ("minecraft:copper_ingot", every(101, 107)),
            ("minecraft:stick", every(103, 105)),
            ("minecraft:iron_block", every(104)),
        ],
    )
    empty_control(
        "purification chamber with a stick where the enrichment chamber goes",
        [
            ("minecraft:gold_ingot", 4),
            ("minecraft:redstone", 2),
            ("minecraft:copper_ingot", 2),
            ("minecraft:stick", 1),
        ],
        [
            ("minecraft:gold_ingot", every(100, 102, 106, 108)),
            ("minecraft:redstone", every(101, 107)),
            ("minecraft:copper_ingot", every(103, 105)),
            ("minecraft:stick", every(104)),
        ],
    )
    empty_control(
        "basic smelting factory with a stick where the energized smelter goes",
        [
            ("minecraft:gold_ingot", 4),
            ("minecraft:redstone", 2),
            ("minecraft:iron_ingot", 2),
            ("minecraft:stick", 1),
        ],
        [
            ("minecraft:gold_ingot", every(100, 102, 106, 108)),
            ("minecraft:redstone", every(101, 107)),
            ("minecraft:iron_ingot", every(103, 105)),
            ("minecraft:stick", every(104)),
        ],
    )
    empty_control(
        "injection chamber with a stick where the iron block goes",
        [
            ("minecraft:gold_ingot", 4),
            ("minecraft:iron_ingot", 3),
            ("minecraft:stick", 1),
            ("minecraft:copper_block", 1),
        ],
        [
            ("minecraft:gold_ingot", every(100, 102, 106, 108)),
            ("minecraft:iron_ingot", every(101, 103, 105)),
            ("minecraft:stick", every(104)),
            ("minecraft:copper_block", every(107)),
        ],
    )

    run("close")
    wait(3)
    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        return 1
    print(f"recipe PASS: all {recipes[0]} carried shaped recipes craft with their declared id and "
          "count, their ingredients are consumed, and every wrong-ingredient control leaves "
          "the result empty")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
