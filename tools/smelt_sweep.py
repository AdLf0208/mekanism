#!/usr/bin/env python3
"""Drive every carried smelting row through real Energized Smelters, in game.

The Energized Smelter's recipe table is data: the 163 vanilla `minecraft:smelting`
rows `tools/gen_smelting.py` expands from the prepared 26.3 resources, plus the
jar's own seven dust -> ingot rows (`processing/<material>/ingot/
from_dust_smelting.json`) the same machine serves. This script is the proof that
the machine *does* each row — the parity rule's debt for a table the port has
carried since slice 1. It is called by `tools/e2e.sh` with the open game's
control socket and the rows as JSON (`gen_smelting.py --rows` /
`--mekanism-rows`), so the scenario and the tables cannot drift apart.

Each row is one operation: place a smelter, put the row's input in its input
slot, charge it to the 20,000 FE buffer, let `TICKS` ticks pass (an operation is
200), then read the container back and assert that the input slot is empty (the
recipe consumed it), the output slot holds exactly the declared item and count,
and the middle slot stayed empty. `--batch` rows run in parallel on one shared
row of smelters: a machine only ever works on one input, so they do not
interact. The default batch is 20, which keeps 163 rows to nine waits — the
ctl calls per row, not the ticks, are what a sweep of this size costs.

A failure names the row: its 1-based index, its input id and what the machine
answered, so a defect in one of 163 rows is not reported as "1 of 163 failed".

Controls (`--control`): a smelter fed `minecraft:dirt`, which the table has no
row for, must still hold the dirt and produce nothing after the same wait.

    python3 tools/smelt_sweep.py SOCK CTL ROWS_JSON X0 Y Z0 [--batch N] [--label TEXT]
    python3 tools/smelt_sweep.py SOCK CTL --control X Y Z
"""
import json
import subprocess
import sys

SMELTER = "mekanism:energized_smelter"
DIMENSION = "minecraft:the_end"
BUFFER = 20_000
TICKS = 225


def main() -> int:
    args = sys.argv[1:]
    if len(args) < 4:
        raise SystemExit(__doc__)
    sock, ctl = args[0], args[1]
    rest = args[2:]

    def run(*command):
        out = subprocess.run([ctl, "--socket", sock, *command], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"ctl {' '.join(command)}: {out.stdout}{out.stderr}")

    def end(command):
        return run("command", f"execute in {DIMENSION} run {command}")

    def slots(x, y, z):
        reply = run("container", str(x), str(y), str(z))
        if "slots" not in reply:
            raise SystemExit(f"container {x},{y},{z} answered without slots: {reply}")
        return reply["slots"]

    def place(x, y, z):
        end(f"setblock {x} {y} {z} {SMELTER}")
        run("wait", "3")
        got = run("block", str(x), str(y), str(z)).get("block") or ""
        if got != SMELTER:
            raise SystemExit(f"expected {SMELTER} at {x},{y},{z}, got {got!r}")

    def insert(x, y, z, item, count):
        reply = run("container", str(x), str(y), str(z), item, str(count))
        if "slots" not in reply:
            raise SystemExit(f"could not put {item} into {x},{y},{z}: {reply}")

    def charge(x, y, z):
        run("chat", f"/mekanism energy {BUFFER} {x} {y} {z}")

    def clear(x, y, z):
        end(f"setblock {x} {y} {z} minecraft:air")

    if rest and rest[0] == "--control":
        x, y, z = (int(value) for value in rest[1:4])
        print("== control: a smelter with no recipe for its input smelts nothing ==")
        place(x, y, z)
        insert(x, y, z, "minecraft:dirt", 1)
        charge(x, y, z)
        run("wait", str(TICKS))
        held = slots(x, y, z)
        print(f"  smelted dirt: {json.dumps(held)}")
        bad = []
        if not held or not held[0] or held[0].get("item") != "minecraft:dirt":
            bad.append("the dirt left the input slot")
        if len(held) < 3 or held[2]:
            bad.append(f"an output appeared: {held[2] if len(held) > 2 else None}")
        clear(x, y, z)
        if bad:
            for line in bad:
                print(f"FAIL: {line}", file=sys.stderr)
            return 1
        print("  control PASS: the dirt is still in the input and nothing was made")
        return 0

    rows_path = rest[0]
    x0, y, z0 = (int(value) for value in rest[1:4])
    batch = 20
    if "--batch" in rest:
        batch = int(rest[rest.index("--batch") + 1])
    label = ""
    if "--label" in rest:
        label = rest[rest.index("--label") + 1] + " "
    rows = json.load(open(rows_path))
    failures = []
    done = 0
    for start in range(0, len(rows), batch):
        chunk = rows[start : start + batch]
        placement = []
        for offset, row in enumerate(chunk):
            x = x0 + 2 * offset
            place(x, y, z0)
            insert(x, y, z0, row["input"], 1)
            charge(x, y, z0)
            placement.append((x, row, start + offset + 1))
        run("wait", str(TICKS))
        for x, row, number in placement:
            held = slots(x, y, z0)
            want = {"item": row["output"], "count": row["count"]}
            input_left = held[0] if held else None
            output = held[2] if len(held) > 2 else None
            print(f"  row {number} {row['input']} x1 -> {json.dumps(output)}")
            if input_left:
                failures.append(
                    f"row {number} {row['input']}: {json.dumps(input_left)} was not consumed"
                )
            if output != want:
                failures.append(
                    f"row {number} {row['input']}: expected {json.dumps(want)}, "
                    f"got {json.dumps(output)}"
                )
            if len(held) > 1 and held[1]:
                failures.append(f"row {number} {row['input']}: the unused middle slot holds {held[1]}")
            clear(x, y, z0)
            done += 1
    print(f"{label}rows smelted: {done} of {len(rows)}")
    if failures:
        for line in failures:
            print(f"FAIL: {line}", file=sys.stderr)
        return 1
    print(
        f"sweep PASS: all {done} carried {label}smelting recipes ran in a real energized smelter "
        "with their declared output stack, and the input was consumed"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
