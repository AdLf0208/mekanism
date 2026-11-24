#!/usr/bin/env python3
"""Prove the acid chain end to end in a real game, through `horizon-ctl`.

This is the in-game half of slice 17. It drives the jar's own three steps and
feeds the product to the dissolution chamber:

  sulfur dust --[Chemical Oxidizer]--> SO2 --[Chemical Infuser + O2]--> SO3
      --[Chemical Infuser + water vapour]--> sulfuric acid
      --[Chemical Dissolution Chamber + iron ore]--> dirty iron slurry

The machines' rules and recipe tables live in `src/runtime.rs` (generated from
the jar by `tools/gen_oxidizing.py` and `tools/gen_infusing.py`); this script is
the fail-capable proof that the game really runs them. Each assertion parses the
`/mekanism info` or `/mekanism chemical read` line, reads the saved compound
where chat can be throttled, and a no-ingredient control plus `--selftest` prove
the assertions can fail.

    python3 tools/acid_chain.py SOCK CTL X Y Z [--selftest]
    python3 tools/acid_chain.py SOCK CTL X Y Z --control
"""
import json
import os
import re
import subprocess
import sys
import time

OXIDIZER = "mekanism:chemical_oxidizer"
INFUSER = "mekanism:chemical_infuser"
DISSOLUTION = "mekanism:chemical_dissolution_chamber"
TANK = "mekanism:basic_chemical_tank"
DIMENSION = "minecraft:the_end"
SO2 = "mekanism:sulfur_dioxide"
OXYGEN = "mekanism:oxygen"
VAPOR = "mekanism:water_vapor"
ACID = "mekanism:sulfuric_acid"

OXIDIZER_FE = 80_000
OXIDIZER_TICKS = 100
OXIDIZER_SPEND = 20_000
INFUSER_FE = 80_000
CHAMBER_FE = 160_000
SOURCE = 1000


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

    def answer(command, prefix, attempts=1):
        """Wait for a NEW logged answer. Move commands are sent once; read-only
        readouts may be re-sent when the game throttles a chat line under load."""
        for attempt in range(attempts):
            before = len(chat_lines(prefix))
            run("chat", f"/{command}")
            for _ in range(15):
                run("wait", "4")
                if len(chat_lines(prefix)) > before:
                    time.sleep(0.2)
                    return chat_lines(prefix)[-1]
        raise SystemExit(f"no {prefix!r} answer after /{command}")

    def info(at, block):
        return answer(f"mekanism info {at[0]} {at[1]} {at[2]}", f"{block}: ", attempts=3)

    def chemical(at):
        return answer(f"mekanism chemical read {at[0]} {at[1]} {at[2]}", "chemical: ", attempts=3)

    def place(at, block):
        px, py, pz = at
        end(f"setblock {px} {py} {pz} {block}")
        run("wait", "3")
        got = run("block", str(px), str(py), str(pz)).get("block") or ""
        if got != block:
            raise SystemExit(f"expected {block} at {at}, got {got!r}")

    def item_slot(at, item, count=1):
        end(f"item replace block {at[0]} {at[1]} {at[2]} container.0 with {item} {count}")

    def charge(at, amount):
        answer(f"mekanism energy {amount} {at[0]} {at[1]} {at[2]}", "Energy set")

    def insert(at, amount, chemical_id):
        line = answer(
            f"mekanism chemical insert {amount} {chemical_id} {at[0]} {at[1]} {at[2]}",
            "chemical: ",
        )
        expected = f"inserted {amount} {chemical_id} into {TANK}"
        if expected not in line:
            raise SystemExit(f"chemical insert at {at}: expected {expected!r}, got {line!r}")

    def data(at, path):
        # `data get` is read-only, so a throttled empty reply can be re-asked.
        for _ in range(3):
            reply = run("command", f"data get block {at[0]} {at[1]} {at[2]} {path}")
            line = (reply.get("chat") or [""])[0]
            if line.strip():
                marker = "block data: "
                return (line.split(marker, 1)[1] if marker in line else line.rsplit(":", 1)[-1]).strip().strip('"')
        raise SystemExit(f"no data get answer at {at} {path}")

    def number(at, path):
        value = data(at, path)
        # A tank that has never been written has no saved keys at all: that is
        # an empty tank, i.e. amount 0, not a failed assertion.
        if "Found no elements matching" in value:
            return 0
        found = re.search(r"-?\d+", value)
        if not found:
            raise SystemExit(f"data get block {at} {path} is not a number: {value!r}")
        return int(found.group(0))

    def expect(text, line, what):
        if text not in line:
            raise SystemExit(f"FAIL: {what}: expected {text!r}, got {line!r}")

    p_oxidizer = (x, y, z)
    p_so2 = (x, y + 1, z)
    p_infuser1 = (x, y + 2, z)
    p_o2 = (x + 1, y + 2, z)
    p_infuser2 = (x, y + 3, z)
    p_vapor = (x + 1, y + 3, z)
    p_acid = (x - 1, y + 3, z)
    p_chamber = (x - 2, y + 3, z)

    if control:
        print("== control: a sulfurless oxidizer makes no gas and spends no FE ==")
        c_oxidizer = (x + 4, y, z)
        c_tank = (x + 4, y + 1, z)
        place(c_tank, TANK)
        place(c_oxidizer, OXIDIZER)
        item_slot(c_oxidizer, "minecraft:dirt")
        charge(c_oxidizer, OXIDIZER_FE)
        run("wait", str(OXIDIZER_TICKS + 20))
        line = info(c_oxidizer, OXIDIZER)
        print(f"  info{c_oxidizer}: {line}")
        expect(f"{OXIDIZER_FE}/{OXIDIZER_FE} FE", line, "a dirt control's buffer")
        expect("input minecraft:dirt x1", line, "a dirt control's slot")
        tank_amount = number(c_tank, "amount")
        print(f"  data get block {c_tank} amount: {tank_amount}")
        if tank_amount != 0:
            raise SystemExit(f"FAIL: a dirt control made {tank_amount} units of gas")
        print("  control PASS: no sulfur, no gas and no energy spent")
        return 0

    failures = []

    # --- A: the oxidizer makes SO2 from one sulfur dust, pushed into its tank ---
    print("== A: one sulfur dust oxidizes into 100 SO2 for 20,000 FE ==")
    place(p_so2, TANK)
    place(p_oxidizer, OXIDIZER)
    item_slot(p_oxidizer, "mekanism:dust_sulfur")
    before = info(p_oxidizer, OXIDIZER)
    expect("input mekanism:dust_sulfur x1", before, "the oxidizer's input before charging")
    charge(p_oxidizer, OXIDIZER_FE)
    run("wait", str(OXIDIZER_TICKS + 25))
    line = info(p_oxidizer, OXIDIZER)
    print(f"  info{p_oxidizer}: {line}")
    for want in (f"{OXIDIZER_FE - OXIDIZER_SPEND}/{OXIDIZER_FE} FE", "input empty x0"):
        if want not in line:
            failures.append(f"oxidizer: expected {want!r} in {line!r}")
    tank_line = chemical(p_so2)
    print(f"  chemical read {p_so2}: {tank_line}")
    expect(f"{SO2} 100/64000", tank_line, "the neighbour tank's SO2")
    if number(p_so2, "amount") != 100:
        failures.append(f"the SO2 tank's saved amount is not 100: {data(p_so2, 'amount')}")

    # --- B: the first infuser makes SO3 from that SO2 and oxygen ---
    print("== B: SO2 + oxygen infuses into SO3 for 200 FE a tick ==")
    place(p_o2, TANK)
    insert(p_o2, SOURCE, OXYGEN)
    place(p_infuser1, INFUSER)
    charge(p_infuser1, INFUSER_FE)
    run("wait", "60")
    line = info(p_infuser1, INFUSER)
    print(f"  info{p_infuser1}: {line}")
    so2_left = number(p_so2, "amount")
    o2_left = number(p_o2, "amount")
    print(f"  data get block {p_so2} amount: {so2_left}")
    print(f"  data get block {p_o2} amount: {o2_left}")
    expect(f"{INFUSER_FE - 50 * 200}/{INFUSER_FE} FE", line, "50 infuser ticks of FE")
    expect("output mekanism:sulfur_trioxide 100/10000", line, "the SO3 centre tank")
    # 50 operations spend 100 SO2 and 50 oxygen; the machine's own left tank
    # holds one more tick's oxygen at the read, so the neighbour is short 51.
    if so2_left != 0 or o2_left != SOURCE - 51:
        failures.append(f"the infuser drew {100 - so2_left} SO2 and {SOURCE - o2_left} O2, expected 100 and 51 (50 spent + 1 held)")

    # --- C: the second infuser makes the acid from SO3 and water vapour ---
    print("== C: SO3 + water vapour infuses into sulfuric acid ==")
    place(p_vapor, TANK)
    insert(p_vapor, SOURCE, VAPOR)
    place(p_acid, TANK)
    place(p_infuser2, INFUSER)
    charge(p_infuser2, INFUSER_FE)
    # 100 SO3 in the first infuser's centre tank is 100 one-tick operations;
    # wait long enough for all of it to become acid.
    run("wait", "110")
    line2 = info(p_infuser2, INFUSER)
    print(f"  info{p_infuser2}: {line2}")
    acid_line = chemical(p_acid)
    print(f"  chemical read {p_acid}: {acid_line}")
    acid_amount = number(p_acid, "amount")
    print(f"  data get block {p_acid} amount: {acid_amount}")
    want_acid = 999 if selftest else 100  # the fail-capable flip
    if f"{ACID} {want_acid}/64000" not in acid_line:
        failures.append(f"acid tank: expected {ACID} {want_acid}/64000 in {acid_line!r}")
    if acid_amount != want_acid:
        failures.append(f"acid tank saved amount {acid_amount}, expected {want_acid}")
    expect(f"{INFUSER_FE - 100 * 200}/{INFUSER_FE} FE", line2, "the acid infuser's buffer")
    expect("output empty 0/10000", line2, "the acid was pushed into the tank")
    vapor_left = number(p_vapor, "amount")
    print(f"  data get block {p_vapor} amount: {vapor_left}")
    # 100 operations spend 100 vapour and the infuser's right tank holds one
    # more at the read, so the neighbour is short 101.
    if vapor_left != SOURCE - 101:
        failures.append(f"the acid infuser drew {SOURCE - vapor_left} vapour, expected 101 (100 spent + 1 held)")

    # --- D: the locally made acid now runs the dissolution chamber ---
    print("== D: the locally made acid dissolves an iron ore into dirty iron ==")
    place(p_chamber, DISSOLUTION)
    item_slot(p_chamber, "minecraft:iron_ore")
    charge(p_chamber, CHAMBER_FE)
    run("wait", "130")
    chamber_line = info(p_chamber, DISSOLUTION)
    print(f"  info{p_chamber}: {chamber_line}")
    acid_left = number(p_acid, "amount")
    print(f"  data get block {p_acid} amount: {acid_left}")
    for want in ("input empty x0", "output mekanism:dirty_iron 1000/10000", f"{CHAMBER_FE - 40_000}/{CHAMBER_FE} FE"):
        if want not in chamber_line:
            failures.append(f"chamber: expected {want!r} in {chamber_line!r}")
    if acid_left != want_acid - 100:
        failures.append(f"the source acid tank holds {acid_left}, expected {want_acid - 100}")

    if failures:
        for item in failures:
            print(f"FAIL: {item}", file=sys.stderr)
        return 1
    print("PASS: sulfur dust oxidized to SO2, SO2 + oxygen infused to SO3, SO3 + vapour infused to locally made sulfuric acid, and that acid dissolved an iron ore into 1000 units of dirty iron")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
