#!/usr/bin/env python3
"""Drive every carried injecting row through real Chemical Injection Chambers.

The machine's recipe table is data (`tools/gen_injecting.py` writes it into
`src/runtime.rs` from the jar's `mekanism:injecting` recipes); this script is the
proof that the machine *does* each row. It is called by `tools/e2e.sh` with the
open game's control socket and the rows as JSON (`gen_injecting.py --rows`), so
the scenario and the table cannot drift apart.

Each row is one operation: a Basic Chemical Tank beside a chamber holds the row's
gas, the chamber's input slot holds the row's item count, the chamber is charged
to its 160,000 FE buffer, and 225 ticks pass (an operation is 200). Then the
container is read back and asserted to be empty in its input slot and to hold
exactly the declared output stack in slot 2; the tank is read **through the
capability** (`/mekanism chemical read`) and must hold exactly the gas the
operation spent — 200 ticks x the row's per-tick amount, no more (the machine
pulls one tick's gas per tick, so it leaves none behind).

Six rows run in parallel per batch: a machine only ever works on one input, and
each pair owns its own tank, so they do not interact.

Controls (`--control`): a chamber fed raw iron — which its table injects — with a
tank of **hydrogen** beside it, a chemical the port knows but no raw-iron recipe
takes, must make no progress, spend no FE and draw no gas.

The answers are read from the game's own `latest.log` (the log line count is the
wait), because `ctl messages` trails the server under load.

    HORIZON_LOG=<home>/logs/latest.log \
        python3 tools/inject_sweep.py SOCK CTL ROWS_JSON X0 Y Z0 [--batch N]
    HORIZON_LOG=... python3 tools/inject_sweep.py SOCK CTL --control X Y Z
"""
import json
import os
import re
import subprocess
import sys
import time

CHAMBER = "mekanism:chemical_injection_chamber"
TANK = "mekanism:basic_chemical_tank"
DIMENSION = "minecraft:the_end"
BUFFER = 160_000
TICKS = 225
TICKS_PER_OPERATION = 200
# Enough gas for the longest operation (200 ticks x 5 units) with room to spare,
# so the assertion is about the operation's draw, not about an empty tank.
GAS = 5_000


def main() -> int:
    args = sys.argv[1:]
    if len(args) < 4:
        raise SystemExit(__doc__)
    sock, ctl = args[0], args[1]
    rest = args[2:]
    log_path = os.environ.get("HORIZON_LOG", "")

    def run(*command):
        out = subprocess.run([ctl, "--socket", sock, *command], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"ctl {' '.join(command)}: {out.stdout}{out.stderr}")

    def end(command):
        return run("command", f"execute in {DIMENSION} run {command}")

    def say(command):
        run("chat", f"/{command}")

    def chat_lines(prefix):
        try:
            with open(log_path, encoding="utf-8", errors="replace") as log:
                return [line.split("[CHAT] ", 1)[1].strip() for line in log if f"[CHAT] {prefix}" in line]
        except FileNotFoundError:
            return []

    def answer(prefix, command):
        """Send one command once and return the answer line it produced."""
        before = len(chat_lines(prefix))
        say(command)
        for _ in range(12):
            run("wait", "4")
            lines = chat_lines(prefix)
            if len(lines) > before:
                time.sleep(0.2)
                lines = chat_lines(prefix)
                return lines[-1]
        raise SystemExit(f"no {prefix!r} answer after /{command}")

    def slots(x, y, z):
        reply = run("container", str(x), str(y), str(z))
        if "slots" not in reply:
            raise SystemExit(f"container {x},{y},{z} answered without slots: {reply}")
        return reply["slots"]

    def place(x, y, z, block):
        end(f"setblock {x} {y} {z} {block}")
        run("wait", "3")
        got = run("block", str(x), str(y), str(z)).get("block") or ""
        if got != block:
            raise SystemExit(f"expected {block} at {x},{y},{z}, got {got!r}")

    def insert(x, y, z, item, count):
        reply = run("container", str(x), str(y), str(z), item, str(count))
        if "slots" not in reply:
            raise SystemExit(f"could not put {item} into {x},{y},{z}: {reply}")

    def charge(x, y, z):
        say(f"mekanism energy {BUFFER} {x} {y} {z}")

    def fill_tank(x, y, z, chemical, amount):
        answer("chemical: ", f"mekanism chemical insert {amount} {chemical} {x} {y} {z}")

    def tank_read(x, y, z):
        return answer("chemical: ", f"mekanism chemical read {x} {y} {z}")

    def energy(x, y, z):
        text = answer("mekanism:chemical_injection_chamber: ", f"mekanism info {x} {y} {z}")
        return int(re.search(r":\s*(\d+)/\d+ FE", text).group(1))

    def clear(x, y, z):
        end(f"setblock {x} {y} {z} minecraft:air")

    if rest and rest[0] == "--control":
        x, y, z = (int(value) for value in rest[1:4])
        print("== control: a tank of a chemical the recipe does not take moves nothing ==")
        place(x, y, z, TANK)
        fill_tank(x, y, z, "mekanism:hydrogen", GAS)
        place(x + 1, y, z, CHAMBER)
        insert(x + 1, y, z, "minecraft:raw_iron", 3)
        charge(x + 1, y, z)
        run("wait", str(TICKS))
        held = slots(x + 1, y, z)
        gas = tank_read(x, y, z)
        fe = energy(x + 1, y, z)
        print(f"  gas: {gas}")
        print(f"  machine: {json.dumps(held)}, {fe} FE")
        bad = []
        input_left = held[0] if held else None
        output = held[2] if len(held) > 2 else None
        if not input_left or input_left.get("item") != "minecraft:raw_iron":
            bad.append("the raw iron left the input slot")
        if output:
            bad.append(f"an output appeared: {json.dumps(output)}")
        if f"mekanism:hydrogen {GAS}/64000" not in gas:
            bad.append(f"the wrong-gas tank was drawn from: {gas}")
        if fe != BUFFER:
            bad.append(f"the machine spent energy on a recipe it cannot run: {fe} FE")
        clear(x, y, z)
        clear(x + 1, y, z)
        if bad:
            for line in bad:
                print(f"FAIL: {line}", file=sys.stderr)
            return 1
        print("  control PASS: the raw iron is still there, nothing was made and no gas was drawn")
        return 0

    rows_path = rest[0]
    x0, y, z0 = (int(value) for value in rest[1:4])
    batch = 6
    if "--batch" in rest:
        batch = int(rest[rest.index("--batch") + 1])
    rows = json.load(open(rows_path))
    failures = []
    done = 0
    for start in range(0, len(rows), batch):
        chunk = rows[start : start + batch]
        placement = []
        for offset, row in enumerate(chunk):
            x = x0 + 3 * offset
            # The tank first: the machine must find it when its first tick runs.
            place(x, y, z0, TANK)
            fill_tank(x, y, z0, row["chemicals"][0], GAS)
            place(x + 1, y, z0, CHAMBER)
            insert(x + 1, y, z0, row["input"], row["consumed"])
            charge(x + 1, y, z0)
            placement.append((x, row))
        run("wait", str(TICKS))
        for x, row in placement:
            held = slots(x + 1, y, z0)
            want = {"item": row["output"], "count": row["count"]}
            input_left = held[0] if held else None
            output = held[2] if len(held) > 2 else None
            gas = tank_read(x, y, z0)
            spent = TICKS_PER_OPERATION * row["per_tick"]
            left = GAS - spent
            print(
                f"  {row['input']} x{row['consumed']} + {row['chemicals'][0]} "
                f"x{row['per_tick']}/t -> {json.dumps(output)} ({gas})"
            )
            if input_left:
                failures.append(f"{row['input']}: {input_left} was not consumed")
            if output != want:
                failures.append(
                    f"{row['input']}: expected {json.dumps(want)}, got {json.dumps(output)}"
                )
            if f"{row['chemicals'][0]} {left}/64000" not in gas:
                failures.append(
                    f"{row['input']}: the tank should hold {left} after {spent} spent, it says {gas}"
                )
            if len(held) > 1 and held[1]:
                failures.append(f"{row['input']}: the unused middle slot holds {held[1]}")
            clear(x, y, z0)
            clear(x + 1, y, z0)
            done += 1
    print(f"rows injected: {done} of {len(rows)}")
    if failures:
        for line in failures:
            print(f"FAIL: {line}", file=sys.stderr)
        return 1
    print(
        f"sweep PASS: all {done} carried injecting recipes ran in a real chamber with their "
        "declared input count and output stack, the input was consumed, and the neighbour tank "
        "lost exactly the gas the operation spent"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
