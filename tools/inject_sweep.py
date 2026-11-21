#!/usr/bin/env python3
"""Slice 5's name for the injecting sweep: `tools/gas_sweep.py --kind injecting`.

The Chemical Injection Chamber and the Purification Chamber run the same
`ItemStackChemicalToItemStack` machine, so slice 6 generalised this driver and
kept this name as the entry point slice 5's own command and evidence used. The
arguments are unchanged; see `gas_sweep.py` for the scenario.

    HORIZON_LOG=<home>/logs/latest.log \
        python3 tools/inject_sweep.py SOCK CTL ROWS_JSON X0 Y Z0 [--batch N]
    HORIZON_LOG=... python3 tools/inject_sweep.py SOCK CTL --control X Y Z
"""
import os
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
os.execv(
    sys.executable,
    [sys.executable, str(HERE / "gas_sweep.py"), *sys.argv[1:], "--kind", "injecting"],
)
