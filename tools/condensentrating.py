#!/usr/bin/env python3
"""Prove the rotary Condensentrator's gas → fluid half in a real game.

The machine is charged with exactly one operation's FE (50) after a known amount
of `mekanism:water_vapor` has been put into its gas tank *through the chemical
capability* in condensentrating mode. One operation spends 1 gas unit and 50 FE
and makes 1 mB of water, which stays in the machine's `horizon:fluids` tank and
is reported by `/mekanism info`. A no-gas control spends nothing, and
`--selftest` flips the water amount so the assertion can fail.

    python3 tools/condensentrating.py SOCK CTL X Y Z [--control] [--selftest]
"""
import json
import os
import re
import subprocess
import sys
import time

MACHINE = "mekanism:rotary_condensentrator"
DIMENSION = "minecraft:the_end"
WATER = "minecraft:water"
VAPOUR = "mekanism:water_vapor"
CHARGE = 50          # exactly one operation: 1 tick x 50 FE
GAS_SOURCE = 5       # a known amount inserted through mekanism:chemical
GAS_COST = 1
WATER_GAIN = 1


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
        chat throttles a chat line under load."""
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
        return answer(f"mekanism info {at[0]} {at[1]} {at[2]}", f"{MACHINE}: ")

    def fluid(at):
        return answer(f"mekanism fluid read {at[0]} {at[1]} {at[2]}", "fluid: ")

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

    machine = (x, y, z)

    if control:
        cx = x + 4
        c_machine = (cx, y, z)
        print("== control: no gas, so a condensentrating machine spends nothing ==")
        place(c_machine, MACHINE)
        reply = mutate(f"mekanism mode {cx} {y} {z}", "Mode set")
        print(f"  mode{c_machine}: {reply}")
        if "Mode set to condensentrating." not in reply:
            raise SystemExit(f"FAIL: mode toggle: {reply!r}")
        if number(c_machine, "mode") != 0:
            raise SystemExit(f"FAIL: saved mode byte is not 0 (condensentrating): {data(c_machine, 'mode')!r}")
        mutate(f"mekanism energy {CHARGE} {cx} {y} {z}", "Energy set")
        run("wait", "30")
        line = info(c_machine)
        print(f"  info{c_machine}: {line}")
        for want in (f"{CHARGE}/20000 FE", "water 0/10000 mB", "gas empty 0/10000"):
            if want not in line:
                raise SystemExit(f"FAIL: a gasless condensentrator: expected {want!r}, got {line!r}")
        if number(c_machine, "amount") != 0:
            raise SystemExit(f"FAIL: a gasless condensentrator wrote gas: {data(c_machine, 'amount')!r}")
        print("  control PASS: no gas, no water and no energy spent")
        return 0

    failures = []
    print("== S20: the gas → fluid row: 1 water vapour -> 1 mB water for exactly 50 FE ==")
    place(machine, MACHINE)

    # The handler gate is the product: in the default (fluid → gas) mode a
    # foreign chemical insert is refused. This is the differential for the mode.
    refused = mutate(f"mekanism chemical insert {GAS_SOURCE} {VAPOUR} {x} {y} {z}", "chemical: ")
    print(f"  chemical insert before toggle: {refused}")
    if "inserted 0" not in refused:
        failures.append(f"the gas tank accepted an insert before the toggle: {refused!r}")
    before = info(machine)
    print(f"  info{machine} before toggle: {before}")
    if "mode decondensentrating" not in before:
        failures.append(f"a fresh machine did not start in fluid -> gas mode: {before!r}")

    reply = mutate(f"mekanism mode {x} {y} {z}", "Mode set")
    print(f"  mode{machine}: {reply}")
    if "Mode set to condensentrating." not in reply:
        failures.append(f"mode toggle: {reply!r}")
    saved_mode = number(machine, "mode")
    print(f"  data get block {machine} mode: {saved_mode}")
    if saved_mode != 0:
        failures.append(f"saved mode byte: expected 0, got {saved_mode}")

    fill = mutate(f"mekanism chemical insert {GAS_SOURCE} {VAPOUR} {x} {y} {z}", "chemical: ")
    print(f"  chemical insert {GAS_SOURCE} {machine}: {fill}")
    expected_fill = f"inserted {GAS_SOURCE} {VAPOUR} into {MACHINE} ({GAS_SOURCE}/10000)"
    if expected_fill not in fill:
        failures.append(f"gas fill: expected {expected_fill!r}, got {fill!r}")

    mutate(f"mekanism energy {CHARGE} {x} {y} {z}", "Energy set")
    run("wait", "30")

    line = info(machine)
    print(f"  info{machine}: {line}")
    want_water = WATER_GAIN + 1 if selftest else WATER_GAIN  # the fail-capable flip
    wants = (
        f"{CHARGE - CHARGE}/20000 FE",
        f"water {want_water}/10000 mB",
        f"gas {VAPOUR} {GAS_SOURCE - GAS_COST}/10000",
        "mode condensentrating",
    )
    for want in wants:
        if want not in line:
            failures.append(f"condensentrating: expected {want!r} in {line!r}")

    saved_gas = number(machine, "amount")
    saved_water = number(machine, "fluids[0].Amount")
    print(f"  data get block {machine} amount (gas): {saved_gas}")
    print(f"  data get block {machine} fluids[0].Amount (water): {saved_water}")
    if saved_gas != GAS_SOURCE - GAS_COST:
        failures.append(f"saved gas amount: expected {GAS_SOURCE - GAS_COST}, got {saved_gas}")
    if saved_water != want_water:
        failures.append(f"saved water amount: expected {want_water}, got {saved_water}")

    tank_line = fluid(machine)
    print(f"  fluid read {machine}: {tank_line}")
    if f"holds {WATER} {want_water}/10000" not in tank_line:
        failures.append(f"the water output tank: expected {want_water} mB, got {tank_line!r}")

    if failures:
        for item in failures:
            print(f"FAIL: {item}", file=sys.stderr)
        return 1
    print(f"PASS: one condensentrating operation spent exactly {CHARGE} FE and one "
          f"{VAPOUR} unit and made {WATER_GAIN} mB of water, saved and readable "
          "through the fluid capability; the insert was refused before the mode toggle")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
