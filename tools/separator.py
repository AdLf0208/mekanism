#!/usr/bin/env python3
"""Prove the Electrolytic Separator in a real game, through `horizon-ctl`.

The separator is charged with exactly one operation's FE (400) after being
filled with 1,000 mB of water, so the number of operations cannot depend on how
long the leg waits: one operation spends 2 mB and 400 FE and pushes 2 hydrogen
plus 1 oxygen into the two neighbouring Basic Chemical Tanks. An empty-water
control spends nothing, and `--selftest` flips one gas amount so the assertion
can fail.

    python3 tools/separator.py SOCK CTL X Y Z [--control] [--selftest]
"""
import json
import os
import re
import subprocess
import sys
import time

SEPARATOR = "mekanism:electrolytic_separator"
TANK = "mekanism:basic_chemical_tank"
DIMENSION = "minecraft:the_end"
WATER = "minecraft:water"
HYDROGEN = "mekanism:hydrogen"
OXYGEN = "mekanism:oxygen"
SOURCE = 1_000
CHARGE = 400          # exactly one operation: 1 tick x 400 FE
SPEND = 400
WATER_COST = 2
H2 = 2
O2 = 1


def main() -> int:
    args = sys.argv[1:]
    if len(args) < 5:
        raise SystemExit(__doc__)
    sock, ctl = args[0], args[1]
    x, y, z = (int(value) for value in args[2:5])
    selftest = "--selftest" in args
    control = "--control" in args

    def run(*command):
        out = subprocess.run([ctl, "--socket", sock, *command], text=True, capture_output=True)
        try:
            return json.loads(out.stdout)
        except json.JSONDecodeError:
            raise SystemExit(f"ctl {' '.join(command)}: {out.stdout}{out.stderr}")

    def end(command):
        return run("command", f"execute in {DIMENSION} run {command}")

    log_path = os.environ.get("HORIZON_LOG", "")
    if not log_path:
        raise SystemExit("set HORIZON_LOG to the game's latest.log")

    def chat_lines(prefix):
        try:
            with open(log_path, encoding="utf-8", errors="replace") as log:
                return [
                    line.split("[CHAT] ", 1)[1].strip()
                    for line in log
                    if f"[CHAT] {prefix}" in line
                ]
        except FileNotFoundError:
            return []

    def answer(command, prefix, attempts=3):
        """Wait for a NEW logged answer; a read-only command may be re-sent if
        the game throttles a chat line under load."""
        for _ in range(attempts):
            before = len(chat_lines(prefix))
            run("chat", f"/{command}")
            for _ in range(15):
                run("wait", "4")
                if len(chat_lines(prefix)) > before:
                    time.sleep(0.2)
                    return chat_lines(prefix)[-1]
        raise SystemExit(f"no {prefix!r} answer after /{command}")

    def mutate(command, prefix):
        """A command that CHANGES state is sent exactly once. Re-sending it when
        chat throttles the reply would repeat the mutation (the slice-4 defect:
        a retried drain moved 250 three times)."""
        return answer(command, prefix, attempts=1)

    def info(at):
        return answer(f"mekanism info {at[0]} {at[1]} {at[2]}", f"{SEPARATOR}: ")

    def chemical(at):
        return answer(f"mekanism chemical read {at[0]} {at[1]} {at[2]}", "chemical: ")

    def place(at, block):
        px, py, pz = at
        end(f"setblock {px} {py} {pz} {block}")
        run("wait", "3")
        got = run("block", str(px), str(py), str(pz)).get("block") or ""
        if got != block:
            raise SystemExit(f"expected {block} at {at}, got {got!r}")

    def data(at, path):
        for _ in range(3):
            reply = run("command", f"data get block {at[0]} {at[1]} {at[2]} {path}")
            line = (reply.get("chat") or [""])[0]
            if line.strip():
                marker = "block data: "
                return (line.split(marker, 1)[1] if marker in line else line.rsplit(":", 1)[-1]).strip().strip('"')
        raise SystemExit(f"no data get answer at {at} {path}")

    def number(at, path):
        value = data(at, path)
        if "Found no elements matching" in value:
            return 0
        found = re.search(r"-?\d+", value)
        if not found:
            raise SystemExit(f"data get block {at} {path} is not a number: {value!r}")
        return int(found.group(0))

    sep = (x, y, z)
    left_tank = (x - 1, y, z)
    right_tank = (x + 1, y, z)
    place(left_tank, TANK)
    place(right_tank, TANK)
    place(sep, SEPARATOR)

    if control:
        # A *different* block entity: setblock on the main machine's position
        # would keep its saved 400 FE, and the main phase would then run two
        # operations instead of the one it is charged for.
        cx = x + 4
        c_sep = (cx, y, z)
        c_left = (cx - 1, y, z)
        c_right = (cx + 1, y, z)
        print("== control: an empty-water separator spends nothing and makes no gas ==")
        place(c_left, TANK)
        place(c_right, TANK)
        place(c_sep, SEPARATOR)
        mutate(f"mekanism energy {CHARGE} {cx} {y} {z}", "Energy set")
        run("wait", "30")
        line = info(c_sep)
        print(f"  info{c_sep}: {line}")
        for want in (f"{CHARGE}/160000 FE", f"water 0/24000 mB"):
            if want not in line:
                raise SystemExit(f"FAIL: a dry separator: expected {want!r}, got {line!r}")
        for at in (c_left, c_right):
            tank_line = chemical(at)
            print(f"  chemical read {at}: {tank_line}")
            if "holds nothing 0/64000" not in tank_line:
                raise SystemExit(f"FAIL: a dry separator made gas: {tank_line!r}")
            if number(at, "amount") != 0:
                raise SystemExit(f"FAIL: a dry separator wrote gas to {at}")
        print("  control PASS: no water, no gas and no energy spent")
        return 0

    failures = []
    print("== the water row: 2 mB -> 2 hydrogen + 1 oxygen for exactly 400 FE ==")
    fill = mutate(f"mekanism fluid insert {SOURCE} {x} {y} {z}", "fluid: ")
    print(f"  fluid insert {SOURCE} {sep}: {fill}")
    expected_fill = f"inserted {SOURCE} {WATER} into {SEPARATOR} ({SOURCE}/24000)"
    if expected_fill not in fill:
        raise SystemExit(f"FAIL: water fill: expected {expected_fill!r}, got {fill!r}")
    mutate(f"mekanism energy {CHARGE} {x} {y} {z}", "Energy set")
    run("wait", "30")

    line = info(sep)
    print(f"  info{sep}: {line}")
    for want in (
        f"{CHARGE - SPEND}/160000 FE",
        f"water {SOURCE - WATER_COST}/24000 mB",
        "left empty 0/2400",
        "right empty 0/2400",
    ):
        if want not in line:
            failures.append(f"separator: expected {want!r} in {line!r}")

    # The two products are pushed to neighbours; which neighbour gets which is
    # decided by the game's face order, so assert the exact pair of gases and
    # amounts across the two tanks rather than a guessed assignment.
    seen = []
    for at in (left_tank, right_tank):
        tank_line = chemical(at)
        print(f"  chemical read {at}: {tank_line}")
        found = re.search(r"holds (mekanism:\w+) (\d+)/64000", tank_line)
        if not found:
            failures.append(f"neighbour tank {at} holds no parsed gas: {tank_line!r}")
            continue
        gas, amount = found.group(1), int(found.group(2))
        saved = number(at, "amount")
        print(f"  data get block {at} amount: {saved}")
        if saved != amount:
            failures.append(f"{at}: handler says {amount}, saved compound says {saved}")
        seen.append((gas, amount))

    want_h2 = H2 + 1 if selftest else H2  # the fail-capable flip
    expected = sorted([(HYDROGEN, want_h2), (OXYGEN, O2)])
    if sorted(seen) != expected:
        failures.append(f"neighbour gases: expected {expected}, got {sorted(seen)}")

    if failures:
        for item in failures:
            print(f"FAIL: {item}", file=sys.stderr)
        return 1
    print("PASS: one separator operation spent exactly 400 FE and 2 mB of water, "
          "and the two neighbour tanks gained exactly 2 hydrogen and 1 oxygen; "
          "the dry control spent nothing")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
