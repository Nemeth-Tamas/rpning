#!/usr/bin/env python3
"""Record release startup/operation medians and sampled Windows memory usage."""
from __future__ import annotations
import csv
import hashlib
import io
import json
import platform
from pathlib import Path
import statistics
import subprocess
import sys
import time


def profile_memory(executable: Path) -> int | None:
    if sys.platform != "win32":
        return None
    import ctypes
    from ctypes import wintypes as w
    class Counters(ctypes.Structure):
        _fields_ = [("cb", w.DWORD), ("faults", w.DWORD)] + [(name, ctypes.c_size_t) for name in [
            "peak_working_set", "working_set", "peak_paged", "paged", "peak_nonpaged", "nonpaged", "pagefile", "peak_pagefile"]]
    api = ctypes.WinDLL("psapi", use_last_error=True).GetProcessMemoryInfo
    api.argtypes = [w.HANDLE, ctypes.POINTER(Counters), w.DWORD]
    api.restype = w.BOOL
    process = subprocess.Popen([str(executable), "--eval", "1000 times 1 end 10000 times dup drop end"], stdout=subprocess.DEVNULL, stderr=subprocess.PIPE)
    peak = 0
    while process.poll() is None:
        counters = Counters()
        counters.cb = ctypes.sizeof(counters)
        if api(int(process._handle), ctypes.byref(counters), counters.cb):
            peak = max(peak, counters.peak_working_set)
        time.sleep(0.001)
    _, stderr = process.communicate()
    if process.returncode:
        raise SystemExit(stderr.decode(errors="replace"))
    return peak or None


def main() -> None:
    if len(sys.argv) != 5:
        raise SystemExit("Usage: python tools/performance.py rpn.exe performance.exe LABEL OUTPUT.json")
    executable, benchmark = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    startup = []
    for _ in range(30):
        started = time.perf_counter_ns()
        subprocess.run([str(executable), "--eval", "1"], stdout=subprocess.DEVNULL, check=True)
        startup.append(time.perf_counter_ns() - started)
    output = subprocess.check_output([str(benchmark)], text=True)
    operations = [{"case": row["case"], "operations": int(row["operations"]), "median_ns": int(row["median_ns"])} for row in csv.DictReader(io.StringIO(output))]
    data = {"label": sys.argv[3], "platform": platform.platform(), "rust": subprocess.check_output(["rustc", "--version"], text=True).strip(),
            "binary_sha256": hashlib.sha256(executable.read_bytes()).hexdigest(), "startup_samples": 30,
            "startup_median_ns": int(statistics.median(startup)), "operation_samples": 9, "operations": operations,
            "sampled_peak_working_set_bytes": profile_memory(executable)}
    destination = Path(sys.argv[4])
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(json.dumps(data, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(data, indent=2))


if __name__ == "__main__":
    main()
