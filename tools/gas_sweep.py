#!/usr/bin/env python3
"""Drive every carried gas-machine row through real machines, in game.

The Chemical Injection Chamber and the Purification Chamber run the same
`ItemStackChemicalToItemStack` recipe type, so one sweep proves both: the rows
come from their generators (`gen_injecting.py`/`gen_purifying.py --rows`), the
machine, its buffer and its control come from `--kind`, and nothing in the
scenario is written twice.

Each row is one operation: a Basic Chemical Tank beside the machine holds the
row's gas, the machine's input slot holds the row's item count, the machine is
charged to its buffer, and 225 ticks pass (an operation is 200). Then the
container is read back and asserted to be empty in its input slot and to hold
exactly the declared output stack in slot 2; the tank is read **through the
capability** (`/mekanism chemical read`) and must hold exactly the gas the
operation spent — 200 ticks x the row's per-tick amount, no more (the machine
pulls one tick's gas per tick, so it leaves none behind).

Six rows run in parallel per batch: a machine only ever works on one input, and
each pair owns its own tank, so they do not interact.

Controls (`--control`): the machine fed an input its table does process, with a
tank of a chemical the port knows but no recipe for that input takes, must make
no progress, spend no FE and draw no gas. For the injection chamber that gas is
hydrogen; for the purification chamber it is the injection chamber's own
hydrogen chloride, so the control is also the cross-machine differential.

The answers are read from the game's own `latest.log` (the log line count is the
wait), because `ctl messages` trails the server under load.

    HORIZON_LOG=<home>/logs/latest.log \
        python3 tools/gas_sweep.py SOCK CTL ROWS_JSON X0 Y Z0 --kind purifying [--batch N]
    HORIZON_LOG=... python3 tools/gas_sweep.py SOCK CTL --control X Y Z --kind purifying
"""
import argparse
import json
import os
import re
import subprocess
import sys
import time

TANK = "mekanism:basic_chemical_tank"
DIMENSION = "minecraft:the_end"
TICKS = 225
TICKS_PER_OPERATION = 200
# Enough gas for the longest operation (200 ticks x 2 units here) with room to
# spare, so the assertion is about the operation's draw, not about an empty tank.
GAS = 5_000

KINDS = {
    "injecting": {
        "block": "mekanism:chemical_injection_chamber",
        "buffer": 160_000,
        "input": "minecraft:raw_iron",
        "consumed": 3,
        "chemical": "mekanism:hydrogen",
    },
    "purifying": {
        "block": "mekanism:purification_chamber",
        "buffer": 80_000,
        "input": "minecraft:iron_ore",
        "consumed": 1,
        "chemical": "mekanism:hydrogen_chloride",
    },
}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("sock")
    parser.add_argument("ctl")
    parser.add_argument("rest", nargs="*", help="ROWS_JSON X0 Y Z0")
    parser.add_argument("--control", nargs=3, type=int, metavar=("X", "Y", "Z"))
    parser.add_argument("--kind", choices=sorted(KINDS), required=True)
    parser.add_argument("--batch", type=int, default=6)
    args = parser.parse_args()
    kind = KINDS[args.kind]
    machine = kind["block"]
    buffer = kind["buffer"]
    sock, ctl = args.sock, args.ctl
    rest = args.rest
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
        say(f"mekanism energy {buffer} {x} {y} {z}")

    def fill_tank(x, y, z, chemical, amount):
        answer("chemical: ", f"mekanism chemical insert {amount} {chemical} {x} {y} {z}")

    def tank_read(x, y, z):
        return answer("chemical: ", f"mekanism chemical read {x} {y} {z}")

    def energy(x, y, z):
        text = answer(f"{machine}: ", f"mekanism info {x} {y} {z}")
        return int(re.search(r":\s*(\d+)/\d+ FE", text).group(1))

    def clear(x, y, z):
        end(f"setblock {x} {y} {z} minecraft:air")

    if args.control:
        x, y, z = args.control
        print(f"== control: a tank of a gas the {args.kind} recipe does not take moves nothing ==")
        place(x, y, z, TANK)
        fill_tank(x, y, z, kind["chemical"], GAS)
        place(x + 1, y, z, machine)
        insert(x + 1, y, z, kind["input"], kind["consumed"])
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
        if not input_left or input_left.get("item") != kind["input"]:
            bad.append(f"the {kind['input']} left the input slot")
        if output:
            bad.append(f"an output appeared: {json.dumps(output)}")
        if f"{kind['chemical']} {GAS}/64000" not in gas:
            bad.append(f"the wrong-gas tank was drawn from: {gas}")
        if fe != buffer:
            bad.append(f"the machine spent energy on a recipe it cannot run: {fe} FE")
        clear(x, y, z)
        clear(x + 1, y, z)
        if bad:
            for line in bad:
                print(f"FAIL: {line}", file=sys.stderr)
            return 1
        print("  control PASS: the input is still there, nothing was made and no gas was drawn")
        return 0

    rows_path = rest[0]
    x0, y, z0 = (int(value) for value in rest[1:4])
    rows = json.load(open(rows_path))
    failures = []
    done = 0
    for start in range(0, len(rows), args.batch):
        chunk = rows[start : start + args.batch]
        placement = []
        for offset, row in enumerate(chunk):
            x = x0 + 3 * offset
            # The tank first: the machine must find it when its first tick runs.
            place(x, y, z0, TANK)
            fill_tank(x, y, z0, row["chemicals"][0], GAS)
            place(x + 1, y, z0, machine)
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
    print(f"rows {args.kind}: {done} of {len(rows)}")
    if failures:
        for line in failures:
            print(f"FAIL: {line}", file=sys.stderr)
        return 1
    print(
        f"sweep PASS: all {done} carried {args.kind} recipes ran in a real machine with their "
        "declared input count and output stack, the input was consumed, and the neighbour tank "
        "lost exactly the gas the operation spent"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
