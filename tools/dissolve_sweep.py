#!/usr/bin/env python3
"""Drive every carried dissolution row through real Chemical Dissolution Chambers, in game.

The chamber's recipe table is data (`tools/gen_dissolving.py` writes it into
`src/runtime.rs` from the jar's `mekanism:dissolution` recipes); this script is
the proof that the machine *does* each row. It is called by `tools/e2e.sh` with
the open game's control socket and the rows as JSON (`gen_dissolving.py --rows`),
so the scenario and the table cannot drift apart.

Each row is one operation, in a tank + chamber pair:

- place a Basic Chemical Tank and fill it to 1000 units of sulfuric acid through
  `mekanism:chemical` (the chamber then draws 100 x acid_per_tick from it — an
  ore 100, a raw block 200 — so the source reads 1000 minus that spend);
- place the chamber, put the row's declared item count in its one input slot, and
  assert `/mekanism info` sees the input **before** charging (an id that does not
  resolve is reported, never silently skipped);
- charge the 160,000 FE buffer, let 120 ticks pass (an operation is 100), then
  assert the chamber's parsed `/mekanism info` says `input empty x0`,
  `output <slurry> <amount>/10000` and `120000/160000 FE`, and that the source
  tank holds exactly `1000 - 100 x acid_per_tick`;
- cross-check the output chemical and amount from the block entity's saved data.

Six pairs run in parallel per batch: a chamber only ever works on one row, so
they do not interact. `--selftest` flips the deepslate-iron row's expected
output to another real slurry, so the run fails on the slurry the machine really
made — the fail-capable counterfactual for this family. `--control` runs a
no-row dirt input: it must keep the item, spend no FE and siphon no acid.

    python3 tools/dissolve_sweep.py SOCK CTL ROWS_JSON X0 Y Z0 [--batch N] [--selftest]
    python3 tools/dissolve_sweep.py SOCK CTL --control X Y Z
"""
import json
import os
import re
import subprocess
import sys
import time

CHAMBER = "mekanism:chemical_dissolution_chamber"
TANK = "mekanism:basic_chemical_tank"
DIMENSION = "minecraft:the_end"
ACID = "mekanism:sulfuric_acid"
BUFFER = 160_000
SOURCE_ACID = 1000
OPERATION_FE = 40_000
TICKS = 120
# The one row whose expected output the fail-capable run inverts.
FLIP_INPUT = "minecraft:deepslate_iron_ore"
FLIP_OUTPUT = "mekanism:dirty_copper"


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

    def answer(command, prefix):
        """Send one mod command once and return the answer line the game logged.

        A command that moves a chemical or energy is never re-sent by a retry
        loop (slice 4's lesson). The game's own latest.log is read, not the
        control client's trailing chat buffer.
        """
        before = len(chat_lines(prefix))
        run("chat", f"/{command}")
        for _ in range(12):
            run("wait", "4")
            lines = chat_lines(prefix)
            if len(lines) > before:
                time.sleep(0.2)
                lines = chat_lines(prefix)
                return lines[-1]
        raise SystemExit(f"no {prefix!r} answer after /{command}")

    def info(x, y, z):
        """The chamber's parsed `/mekanism info` line, read once and by prefix."""
        return answer(f"mekanism info {x} {y} {z}", f"{CHAMBER}: ")

    def place(x, y, z, block):
        end(f"setblock {x} {y} {z} {block}")
        run("wait", "3")
        got = run("block", str(x), str(y), str(z)).get("block") or ""
        if got != block:
            raise SystemExit(f"expected {block} at {x},{y},{z}, got {got!r}")

    def data(x, y, z, path):
        reply = run("command", f"data get block {x} {y} {z} {path}")
        line = (reply.get("chat") or [""])[0]
        marker = "block data: "
        value = line.split(marker, 1)[1] if marker in line else line.rsplit(":", 1)[-1]
        return value.strip().strip('"')

    def number(x, y, z, path):
        value = data(x, y, z, path)
        found = re.search(r"-?\d+", value)
        if not found:
            raise SystemExit(f"data get block {x},{y},{z} {path} is not a number: {value!r}")
        return int(found.group(0))

    def clear(x, y, z):
        end(f"setblock {x} {y} {z} minecraft:air")

    # ---- the no-row control: a dirt input makes nothing and draws no acid ----
    if rest and rest[0] == "--control":
        x, y, z = (int(value) for value in rest[1:4])
        print("== control: a dirt in the chamber makes nothing and draws no acid ==")
        place(x, y, z, TANK)
        expect = f"inserted 1000 {ACID} into {TANK} (1000/64000)"
        line = answer(f"mekanism chemical insert 1000 {ACID} {x} {y} {z}", "chemical: ")
        if expect not in line:
            raise SystemExit(f"control chemical insert at {x},{y},{z}: expected {expect!r}, got {line!r}")
        place(x + 1, y, z, CHAMBER)
        end(f"item replace block {x + 1} {y} {z} container.0 with minecraft:dirt 1")
        run("wait", "4")
        answer(f"mekanism energy {BUFFER} {x + 1} {y} {z}", "Energy set")
        run("wait", str(TICKS))
        line = info(x + 1, y, z)
        print(f"  info({x + 1},{y},{z}): {line}")
        left = number(x, y, z, "amount")
        print(f"  data get block {x} {y} {z} amount: {left}")
        bad = []
        for want in (f"{BUFFER}/{BUFFER} FE", "input minecraft:dirt x1", "output empty 0/10000"):
            if want not in line:
                bad.append(f"expected {want!r} in {line!r}")
        if left != 1000:
            bad.append(f"the source tank holds {left}, expected 1000 (unchanged)")
        clear(x, y, z)
        clear(x + 1, y, z)
        if bad:
            for item in bad:
                print(f"FAIL: {item}", file=sys.stderr)
            return 1
        print("  control PASS: the dirt kept its slot, spent no FE and siphoned no acid")
        return 0

    rows_path = rest[0]
    x0, y, z0 = (int(value) for value in rest[1:4])
    batch = 6
    if "--batch" in rest:
        batch = int(rest[rest.index("--batch") + 1])
    selftest = "--selftest" in rest
    rows = json.load(open(rows_path))
    failures = []
    done = 0
    for start in range(0, len(rows), batch):
        chunk = rows[start : start + batch]
        pairs = []
        for offset, row in enumerate(chunk):
            x = x0 + 3 * offset
            tx, cx = x, x + 1
            place(tx, y, z0, TANK)
            acid = 100 * int(row["acid_per_tick"])
            expect = f"inserted {SOURCE_ACID} {ACID} into {TANK} ({SOURCE_ACID}/64000)"
            line = answer(f"mekanism chemical insert {SOURCE_ACID} {ACID} {tx} {y} {z0}", "chemical: ")
            if expect not in line:
                raise SystemExit(
                    f"{row['input']}: chemical insert at {tx},{y},{z0}: expected {expect!r}, got {line!r}"
                )
            place(cx, y, z0, CHAMBER)
            end(f"item replace block {cx} {y} {z0} container.0 with {row['input']} {row['count']}")
            run("wait", "4")
            # Resolve the input BEFORE spending anything: an unknown item id
            # leaves the slot empty and must be reported, not skipped.
            before = info(cx, y, z0)
            want_input = f"input {row['input']} x{row['count']}"
            if want_input not in before:
                raise SystemExit(
                    f"{row['input']}: the input did not resolve — expected {want_input!r} "
                    f"in {before!r} (row silently failed to resolve)"
                )
            answer(f"mekanism energy {BUFFER} {cx} {y} {z0}", "Energy set")
            pairs.append((tx, cx, row))
        run("wait", str(TICKS))
        for tx, cx, row in pairs:
            want_out = row["output"]
            if selftest and row["input"] == FLIP_INPUT:
                want_out = FLIP_OUTPUT
            line = info(cx, y, z0)
            left = number(tx, y, z0, "amount")
            acid = 100 * int(row["acid_per_tick"])
            want_left = SOURCE_ACID - acid
            saved_chem = data(cx, y, z0, "output_chemical")
            saved_amount = number(cx, y, z0, "output_amount")
            print(
                f"  {row['input']} x{row['count']}: {line} (source tank {left}, "
                f"saved {saved_chem} {saved_amount})"
            )
            for want in (
                f"{BUFFER - OPERATION_FE}/{BUFFER} FE",
                "input empty x0",
                f"output {want_out} {row['output_amount']}/10000",
            ):
                if want not in line:
                    failures.append(f"{row['input']}: expected {want!r}, got {line!r}")
            if left != want_left:
                failures.append(
                    f"{row['input']}: the source tank holds {left}, expected {want_left}"
                )
            if row["output"] not in saved_chem or saved_amount != int(row["output_amount"]):
                failures.append(
                    f"{row['input']}: saved tank is {saved_chem} {saved_amount}, "
                    f"expected {row['output']} {row['output_amount']}"
                )
            clear(tx, y, z0)
            clear(cx, y, z0)
            done += 1
    print(f"rows dissolved: {done} of {len(rows)}")
    if failures:
        for item in failures:
            print(f"FAIL: {item}", file=sys.stderr)
        return 1
    print(
        f"sweep PASS: all {done} carried dissolution recipes ran in a real Chemical Dissolution "
        "Chamber with their declared item count, per-tick acid, slurry and amount — the input was "
        "consumed, exactly 40,000 FE was spent and the source tank lost exactly the operation's acid"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
