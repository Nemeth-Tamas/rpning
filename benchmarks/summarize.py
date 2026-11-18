#!/usr/bin/env python3
"""Summarize actual rpning backend-benchmark CSV output. Standard library only."""

import argparse
import csv
from datetime import datetime, timezone
from pathlib import Path
import platform


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("csv_file", type=Path, help="CSV produced by rpning-backend-bench")
    parser.add_argument("--output", type=Path, help="Write the Markdown report to this file")
    args = parser.parse_args()

    with args.csv_file.open(newline="", encoding="utf-8-sig") as handle:
        reader = csv.DictReader(handle)
        required = {
            "family", "operation", "size", "unit", "backend",
            "iterations", "repeats", "median_ns_per_op",
        }
        if not required.issubset(reader.fieldnames or []):
            parser.error("Input is not a valid rpning benchmark CSV")
        rows = list(reader)

    if not rows:
        parser.error("Benchmark CSV has no measurements")

    groups = {}
    for row in rows:
        key = (row["family"], row["operation"], int(row["size"]), row["unit"])
        groups.setdefault(key, {})[row["backend"]] = float(row["median_ns_per_op"])

    lines = [
        "# rpning numerical-backend benchmark results",
        "",
        f"- Generated (UTC): {datetime.now(timezone.utc).isoformat(timespec='seconds')}",
        f"- Platform: {platform.platform()}",
        f"- Python runtime: {platform.python_version()} (report generator only)",
        f"- Source CSV: `{args.csv_file.name}`",
        f"- Iterations per repeat: {rows[0]['iterations']}",
        f"- Repeats: {rows[0]['repeats']}",
        "",
        "All timings are median nanoseconds per arithmetic operation, including two operand clones.",
        "Inputs are generated and parsed before timing. Smaller is faster.",
        "",
        "| Family | Operation | Size | Malachite (ns) | Rug (ns) | Faster backend / ratio |",
        "|---|---|---:|---:|---:|---|",
    ]

    paired = 0
    for (family, op, size, unit), backends in sorted(groups.items()):
        m = backends.get("malachite")
        r = backends.get("rug")
        if m is None:
            continue
        if r is None:
            comparison = "Not compared"
        elif m == 0 or r == 0:
            comparison = "Measurement too small"
        else:
            paired += 1
            if m < r:
                comparison = f"Malachite {r / m:.2f}x"
            elif r < m:
                comparison = f"Rug {m / r:.2f}x"
            else:
                comparison = "Tie"
        lines.append(
            f"| {family} | {op} | {size} {unit} | "
            f"{m:,.2f} | {r:,.2f} | {comparison} |" if r is not None else
            f"| {family} | {op} | {size} {unit} | {m:,.2f} | — | {comparison} |"
        )

    lines.extend([
        "",
        f"Compared operation/size pairs: **{paired}**",
        "",
        "## Interpretation and limitations",
        "",
        "- Results apply to this host, toolchain, workload, and numerical-library versions.",
        "- This is an operation-level microbenchmark, not end-to-end rpning calculator throughput.",
        "- The benchmark includes operand cloning; parsing, input generation, and formatting are excluded.",
        "- Integer/rational equality is cross-checked against Rug when `compare-rug` is enabled.",
        "- Floating-point runs measure speed, **not** proof of identical rounding or ULP accuracy.",
        "- Native Windows MSVC normally cannot build Rug's bundled GMP/MPFR; compare in WSL/Linux.",
        "- Re-run after changing CPU governor, power state, compiler flags, or dependencies.",
        "- Do not mark M2 backend selection finalized from one noisy run alone.",
        "",
    ])

    report = "\n".join(lines)
    if args.output:
        args.output.write_text(report, encoding="utf-8")
        print(f"Wrote {args.output}")
    else:
        print(report)


if __name__ == "__main__":
    main()
