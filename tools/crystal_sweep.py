#!/usr/bin/env python3
"""Drive every carried crystallizing row through real Chemical Crystallizers.

The crystallizer's recipe table is data (`tools/gen_crystallizing.py` writes it
into `src/runtime.rs` from the jar's `mekanism:crystallizing` recipes); this
script is the proof that the machine *does* each row. It is called by
`tools/e2e.sh` with the open game's control socket and the rows as JSON
(`gen_crystallizing.py --rows`), so the scenario and the table cannot drift
apart.

Each row is one operation:
- place a crystallizer;
- insert the row's 200 units of its clean slurry through `mekanism:chemical`
  (`/mekanism chemical insert`, the machine's own tank);
- charge the 160,000 FE buffer;
- let 225 ticks pass (an operation is 200).

Then the machine is read back: `/mekanism info` names the crystal and its count,
the tank's saved `amount` is 0 (the operation consumed it) and the saved
`energy` is short exactly 80,000 FE (200 ticks x 400 FE). Three rows run in one
batch: a machine only works on one slurry, so they do not interact.

    python3 tools/crystal_sweep.py SOCK CTL ROWS_JSON X0 Y Z0 [--batch N]
"""
import json
import os
import re
import subprocess
import sys
import time

CRYSTALLIZER = "mekanism:chemical_crystallizer"
DIMENSION = "minecraft:the_end"
BUFFER = 160_000
OPERATION_FE = 80_000
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

    log_path = os.environ.get("HORIZON_LOG", "")

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
        """Send one `/mekanism ...` command once and return the answer line.

        The game's own `latest.log` is read, not the control client's chat
        buffer, which trails the server under load. A command that moves a
        chemical is never re-sent.
        """
        if not log_path:
            raise SystemExit("set HORIZON_LOG to the game's latest.log")
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

    def place(x, y, z):
        end(f"setblock {x} {y} {z} {CRYSTALLIZER}")
        run("wait", "3")
        got = run("block", str(x), str(y), str(z)).get("block") or ""
        if got != CRYSTALLIZER:
            raise SystemExit(f"expected {CRYSTALLIZER} at {x},{y},{z}, got {got!r}")

    def number(x, y, z, path):
        """One `/data get block` integer."""
        reply = run("command", f"data get block {x} {y} {z} {path}")
        line = (reply.get("chat") or [""])[0]
        marker = "block data: "
        value = line.split(marker, 1)[1] if marker in line else line.rsplit(":", 1)[-1]
        found = re.search(r"-?\d+", value)
        if not found:
            raise SystemExit(f"data get block {x} {y} {z} {path} is not a number: {value!r}")
        return int(found.group(0))

    rows_path = rest[0]
    x0, y, z0 = (int(value) for value in rest[1:4])
    batch = 3
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
            line = answer(
                f"mekanism chemical insert {row['amount']} {row['slurry']} {x} {y} {z0}",
                "chemical: ",
            )
            if f"inserted {row['amount']} {row['slurry']} into {CRYSTALLIZER}" not in line:
                raise SystemExit(f"chemical insert at {x},{y},{z0}: got {line!r}")
            answer(f"mekanism energy {BUFFER} {x} {y} {z0}", "Energy set")
            placement.append((x, row))
        run("wait", str(TICKS))
        for x, row in placement:
            info = answer(f"mekanism info {x} {y} {z0}", f"{CRYSTALLIZER}: ")
            amount = number(x, y, z0, "amount")
            energy = number(x, y, z0, "energy")
            print(
                f"  {row['slurry']} x{row['amount']} -> {row['crystal']} x{row['count']} "
                f"(tank {amount}, energy {energy})"
            )
            if f"output {row['crystal']} x{row['count']}" not in info:
                failures.append(f"{row['slurry']}: expected output {row['crystal']}, got {info!r}")
            if amount != 0:
                failures.append(f"{row['slurry']}: {amount} units were not consumed")
            if energy != BUFFER - OPERATION_FE:
                failures.append(
                    f"{row['slurry']}: {energy} FE left, expected {BUFFER - OPERATION_FE}"
                )
            end(f"setblock {x} {y} {z0} minecraft:air")
            done += 1
    print(f"rows crystallized: {done} of {len(rows)}")
    if failures:
        for line in failures:
            print(f"FAIL: {line}", file=sys.stderr)
        return 1
    print(
        f"sweep PASS: all {done} carried crystallizing recipes ran in a real Chemical "
        "Crystallizer, the clean slurry consumed and its crystal made, for exactly one "
        "operation's FE"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
