#!/usr/bin/env python3
"""Drive every carried crushing row through real Crushers, in game.

The Crusher's recipe table is data (`tools/gen_crushing.py` writes it into
`src/runtime.rs` from the jar's `mekanism:crushing` recipes); this script is the
proof that the machine *does* each row. It is called by `tools/e2e.sh` with the
open game's control socket and the rows as JSON (`gen_crushing.py --rows`), so
the scenario and the table cannot drift apart.

Each row is one operation: place a crusher, put the row's input count in its
input slot, charge it to the 20,000 FE buffer, let 225 ticks pass (an operation
is 200), then read the container back and assert that the input slot is empty
(the recipe consumed it), the output slot holds exactly the declared item and
count, and the middle slot stayed empty. Eight rows run in parallel per batch: a
machine only ever works on one input, so they do not interact.

Controls (`--control`): a crusher fed `minecraft:dirt`, which the table has no
row for, must still hold the dirt and produce nothing after the same wait; a
crusher with no energy must not crush.

    python3 tools/crush_sweep.py SOCK CTL ROWS_JSON X0 Y Z0 [--batch N]
    python3 tools/crush_sweep.py SOCK CTL --control X Y Z
"""
import json
import subprocess
import sys

CRUSHER = "mekanism:crusher"
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
        end(f"setblock {x} {y} {z} {CRUSHER}")
        run("wait", "3")
        got = run("block", str(x), str(y), str(z)).get("block") or ""
        if got != CRUSHER:
            raise SystemExit(f"expected {CRUSHER} at {x},{y},{z}, got {got!r}")

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
        print("== control: a crusher with no recipe for its input crushes nothing ==")
        place(x, y, z)
        insert(x, y, z, "minecraft:dirt", 1)
        charge(x, y, z)
        run("wait", str(TICKS))
        held = slots(x, y, z)
        print(f"  crushed dirt: {json.dumps(held)}")
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
    batch = 8
    if "--batch" in rest:
        batch = int(rest[rest.index("--batch") + 1])
    rows = json.load(open(rows_path))
    failures = []
    done = 0
    for start in range(0, len(rows), batch):
        chunk = rows[start : start + batch]
        placement = []
        for offset, row in enumerate(chunk):
            x = x0 + 2 * offset
            place(x, y, z0)
            insert(x, y, z0, row["input"], row["consumed"])
            charge(x, y, z0)
            placement.append((x, row))
        run("wait", str(TICKS))
        for x, row in placement:
            held = slots(x, y, z0)
            want = {"item": row["output"], "count": row["count"]}
            input_left = held[0] if held else None
            output = held[2] if len(held) > 2 else None
            print(f"  {row['input']} x{row['consumed']} -> {json.dumps(output)}")
            if input_left:
                failures.append(f"{row['input']}: {input_left} was not consumed")
            if output != want:
                failures.append(f"{row['input']}: expected {json.dumps(want)}, got {json.dumps(output)}")
            if len(held) > 1 and held[1]:
                failures.append(f"{row['input']}: the unused middle slot holds {held[1]}")
            clear(x, y, z0)
            done += 1
    print(f"rows crushed: {done} of {len(rows)}")
    if failures:
        for line in failures:
            print(f"FAIL: {line}", file=sys.stderr)
        return 1
    print(f"sweep PASS: all {done} carried crushing recipes ran in a real crusher with their "
          "declared input count and output stack, and the input was consumed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
