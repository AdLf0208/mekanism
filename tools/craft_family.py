#!/usr/bin/env python3
"""Craft the storage-block and nugget family in a real crafting table.

Slice 14's family is 16 crafting recipes the jar declares and this port carries:
per metal (lead, osmium, tin, uranium) a 3x3 block from nine ingots, a shapeless
nine ingots from a block, a 3x3 ingot from nine nuggets and a shapeless nine
nuggets from an ingot. `tools/carry_storage.py --rows` reads the **committed**
recipe and tag files and emits the grid each one declares, so this script crafts
exactly what the package ships and cannot drift from it.

Each row: give the grid's items, place them with right-clicks, assert the result
slot's id **and** count, shift-click it into the inventory, then assert the stack
arrived, the grid is empty and the counted ingredient is gone.

Two counterfactuals make it fail-capable:

- `--control`: the same nine-ingot grid with **dirt** in the centre cell must
  leave the result empty (a table that crafts whatever it is handed fails here);
- `SELFTEST`/`--selftest`: the nine-ingot row's result assertion is inverted, so
  the run must fail with `FAIL: … crafted …` (the newest family's own check).

    python3 tools/craft_family.py SOCK CTL ROWS_JSON X Y Z [--selftest]
    python3 tools/craft_family.py SOCK CTL ROWS_JSON X Y Z --control
"""
import json
import os
import subprocess
import sys

GRID = [100, 101, 102, 103, 104, 105, 106, 107, 108]
RESULT = 200
DIRT = "minecraft:dirt"


def main() -> int:
    args = sys.argv[1:]
    if len(args) < 6:
        raise SystemExit(__doc__)
    sock, ctl, rows_path, x, y, z = args[0], args[1], args[2], args[3], args[4], args[5]
    control = "--control" in args
    selftest = "--selftest" in args or os.environ.get("SELFTEST") == "1"
    rows = json.load(open(rows_path))

    def run(*command):
        out = subprocess.run([ctl, "--socket", sock, *command], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"ctl {' '.join(command)}: {out.stdout}{out.stderr}")

    def wait(ticks=2):
        run("wait", str(ticks))

    def slots():
        # The result slot is the crafting table's own (200): without it the
        # reply is the player's inventory menu, where a "null result" would be
        # vacuous, so every read goes through this guard.
        screen = run("screen")
        if "slots" not in screen or not any(s["slot"] == RESULT for s in screen["slots"]):
            raise SystemExit(f"the crafting table is not open: {json.dumps(screen)[:200]}")
        return screen["slots"]

    def grid_cells():
        return {s["slot"]: s for s in slots() if s["slot"] in GRID}

    def result():
        stack = [s for s in slots() if s["slot"] == RESULT]
        return stack[0] if stack and stack[0].get("item") else None

    def count(item):
        return run("inventory").get("counts", {}).get(item, 0)

    def close_table():
        for _ in range(3):
            try:
                run("close")
            except SystemExit:
                pass
            wait(3)
            screen = run("screen")
            if "slots" not in screen or not any(s["slot"] == RESULT for s in screen["slots"]):
                return
        print("  (note: the crafting table did not close)")

    def open_table():
        try:
            run("close")
        except SystemExit:
            pass
        wait(3)
        run("setblock", x, y, z, "minecraft:air")
        run("setblock", x, y, z, "minecraft:crafting_table")
        wait(2)
        run("command", f"tp @s {float(x) - 0.5} {y} {float(z) + 0.5}")
        wait(3)
        run("interact", x, y, z)
        wait(5)

    def place(item, target):
        slot = next((s["slot"] for s in slots() if s.get("item") == item), None)
        if slot is None:
            raise SystemExit(f"missing ingredient {item} in the inventory")
        run("click-slot", str(slot))
        wait(1)
        run("click-slot", str(target), "right")
        wait(1)
        run("click-slot", str(slot))
        wait(1)

    def load(cells):
        """Clear the player, give the grid's items and place them."""
        run("command", "clear @s")
        wait(2)
        totals = {}
        for item in cells.values():
            totals[item] = totals.get(item, 0) + 1
        for item, n in totals.items():
            run("give", item, str(n))
        wait(3)
        for slot, item in sorted(cells.items()):
            place(item, GRID[slot])

    failures = []

    def grid_empty_failure(label):
        held = {s: c.get("item") for s, c in grid_cells().items() if c.get("item")}
        if held:
            failures.append(f"{label}: the grid still holds {held} after the craft")

    def craft(row):
        label, cells = row["label"], {int(k): v for k, v in row["grid"].items()}
        want = row["want"]
        load(cells)
        got = result()
        print(f"  {label} -> {json.dumps(got)}")
        if selftest and want["item"] == "mekanism:block_lead":
            # The flip: the leg asserts this row leaves the result empty, so a
            # real craft is the failure. It stops here — a polluted grid would
            # fail a later row instead, and the log must name the flip's own row.
            if got is not None:
                failures.append(
                    f"selftest: expected no result for {label}, got "
                    f"{got['item']} x{got['count']}"
                )
            else:
                print("  (selftest: the nine-ingot grid left the result empty)")
            close_table()
            return
        if not got or got.get("item") != want["item"] or got.get("count") != want["count"]:
            grid = [c for c in grid_cells().values()]
            print(f"    grid: {json.dumps(grid)}")
            failures.append(f"{label}: expected {want['count']} {want['item']}, got {got}")
            return
        run("click-slot", str(RESULT), "shift")
        wait(4)
        made = count(want["item"])
        if made < want["count"]:
            failures.append(f"{label}: the crafted stack reached the inventory as {made}")
        grid_empty_failure(label)
        left = count(row["consumed"])
        if left != 0:
            failures.append(f"{label}: {row['consumed']} was not consumed ({left} left)")
        run("command", "clear @s")
        wait(2)

    open_table()
    if control:
        # The same 3x3 the first nine-ingot row declares, with dirt where the
        # mod's own ingredient goes: if the key `P` were accepted as anything,
        # this would craft a block out of eight ingots and a dirt block.
        base = next(r for r in rows if r["want"]["item"] == "mekanism:block_lead")
        cells = {int(k): v for k, v in base["grid"].items()}
        # The jar's pattern puts its own item in the centre cell (`#P#`); every
        # other cell is the tag, and the row names one item for all nine, so the
        # centre is slot 4.
        centre = 4 if 4 in cells else next(iter(cells))
        cells[centre] = DIRT
        print(f"== control: dirt in the {base['label']} grid ==")
        # The control opens its own table: a run that reused whatever screen was
        # left open could read the inventory menu and "see" an empty result.
        open_table()
        load(cells)
        got = result()
        print(f"  {base['label']} with dirt in the centre -> {json.dumps(got)}")
        if got is not None:
            failures.append(f"the control crafted {got['item']} x{got['count']} with dirt in the grid")
        if failures:
            for line in failures:
                print(f"FAIL: {line}", file=sys.stderr)
            close_table()
            return 1
        close_table()
        print("  control PASS: eight ingots and a dirt make nothing")
        return 0

    for row in rows:
        craft(row)
        if failures:
            break
    close_table()
    if failures:
        for line in failures:
            print(f"FAIL: {line}", file=sys.stderr)
        return 1
    print(f"craft PASS: all {len(rows)} carried storage-block and nugget recipes craft with their "
          "declared id and count, their grid empties and their ingredients are consumed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
