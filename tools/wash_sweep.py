#!/usr/bin/env python3
"""Drive every carried washing row through real Chemical Washers, in game.

The Washer's recipe table is data (`tools/gen_washing.py` writes it into
`src/runtime.rs` from the jar's `mekanism:washing` recipes); this script is the
proof that the machine *does* each row, and that its water is a **fluid** in the
game's own fluid capability while its slurries are **chemicals** in this port's.
It is called by `tools/e2e.sh` with the open game's control socket and the rows
as JSON (`gen_washing.py --rows`), so the scenario and the table cannot drift
apart.

Each row is one operation:
- place a washer;
- insert 500 mB of water through `horizon:fluids` (`/mekanism fluid insert`);
- insert one unit of the row's dirty slurry through `mekanism:chemical`
  (`/mekanism chemical insert`, tank 0, the machine's input);
- charge the 80,000 FE buffer;
- let 225 ticks pass (an operation is 200).

Then every tank is read back from the block entity's **saved data**
(`/data get block`): the water tank is short exactly the row's 5 mB, the input
tank is empty (the operation consumed it), the output tank holds exactly the
row's clean slurry and amount, and the buffer is short exactly 40,000 FE
(200 ticks x 200 FE). Seven rows run in parallel in one batch: a machine only
ever works on one slurry, so they do not interact.

    python3 tools/wash_sweep.py SOCK CTL ROWS_JSON X0 Y Z0 [--batch N]
"""
import json
import os
import re
import subprocess
import sys
import time

WASHER = "mekanism:chemical_washer"
DIMENSION = "minecraft:the_end"
BUFFER = 80_000
OPERATION_FE = 40_000
TICKS = 225
WATER = 500


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
        buffer, which trails the server under load (the e2e reads it too). A
        command that moves a fluid or a chemical is never re-sent.
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
        end(f"setblock {x} {y} {z} {WASHER}")
        run("wait", "3")
        got = run("block", str(x), str(y), str(z)).get("block") or ""
        if got != WASHER:
            raise SystemExit(f"expected {WASHER} at {x},{y},{z}, got {got!r}")

    def data(x, y, z, path):
        """One `/data get block` value: the number, or the quoted string."""
        reply = run("command", f"data get block {x} {y} {z} {path}")
        line = (reply.get("chat") or [""])[0]
        # `<pos> has the following block data: <value>` — a value can itself hold
        # a colon (`"mekanism:clean_iron"`), so split on the marker, not the last
        # colon.
        marker = "block data: "
        value = line.split(marker, 1)[1] if marker in line else line.rsplit(":", 1)[-1]
        return value.strip().strip('"')

    def number(x, y, z, path):
        value = data(x, y, z, path)
        found = re.search(r"-?\d+", value)
        if not found:
            raise SystemExit(f"data get block {x} {y} {z} {path} is not a number: {value!r}")
        return int(found.group(0))

    rows_path = rest[0]
    x0, y, z0 = (int(value) for value in rest[1:4])
    batch = 7
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
            place(x, y, z0)
            wanted = f"inserted {WATER} {row['fluid']} into {WASHER} ({WATER}/10000)"
            line = answer(f"mekanism fluid insert {WATER} {x} {y} {z0}", "fluid: ")
            if wanted not in line:
                raise SystemExit(f"fluid insert at {x},{y},{z0}: expected {wanted!r}, got {line!r}")
            line = answer(
                f"mekanism chemical insert {row['dirty_amount']} {row['dirty']} {x} {y} {z0}",
                "chemical: ",
            )
            if f"inserted {row['dirty_amount']} {row['dirty']} into {WASHER}" not in line:
                raise SystemExit(f"chemical insert at {x},{y},{z0}: got {line!r}")
            answer(f"mekanism energy {BUFFER} {x} {y} {z0}", "Energy set")
            placement.append((x, row))
        run("wait", str(TICKS))
        for x, row in placement:
            water_left = number(x, y, z0, "fluids[0].Amount")
            input_amount = number(x, y, z0, "input_amount")
            output_chemical = data(x, y, z0, "output_chemical")
            output_amount = number(x, y, z0, "output_amount")
            energy = number(x, y, z0, "energy")
            print(
                f"  {row['dirty']} x{row['dirty_amount']} + {row['water']} mB water -> "
                f"{{{output_chemical!r}: {output_amount}}} "
                f"(water {water_left}, input {input_amount}, energy {energy})"
            )
            if water_left != WATER - row["water"]:
                failures.append(
                    f"{row['dirty']}: the water tank is {water_left}, "
                    f"expected {WATER - row['water']}"
                )
            if input_amount != 0:
                failures.append(f"{row['dirty']}: {input_amount} units were not consumed")
            if output_chemical != row["clean"] or output_amount != row["clean_amount"]:
                failures.append(
                    f"{row['dirty']}: expected {row['clean']} x{row['clean_amount']}, "
                    f"got {output_chemical!r} x{output_amount}"
                )
            if energy != BUFFER - OPERATION_FE:
                failures.append(
                    f"{row['dirty']}: {energy} FE left, expected {BUFFER - OPERATION_FE}"
                )
            end(f"setblock {x} {y} {z0} minecraft:air")
            done += 1
    print(f"rows washed: {done} of {len(rows)}")
    if failures:
        for line in failures:
            print(f"FAIL: {line}", file=sys.stderr)
        return 1
    print(
        f"sweep PASS: all {done} carried washing recipes ran in a real Chemical Washer with their "
        "declared water and slurry, the dirty slurry consumed, the clean slurry made, and exactly "
        "one operation's water and FE spent"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
