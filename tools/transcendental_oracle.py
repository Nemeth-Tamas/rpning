#!/usr/bin/env python3
"""Independent 50-digit reference check for rpning real functions.

Only Python standard-library Decimal/Fraction is required. No f64 or third-party
numerical backend. Tests absolute error of representative inputs, NOT certified
correct rounding on every real input.
"""
from __future__ import annotations
import subprocess
import sys
from decimal import Decimal, localcontext
from fractions import Fraction
from pathlib import Path

PI_60 = Decimal("3.141592653589793238462643383279502884197169399375105820974945")

def decimal_from_rational(text: str) -> Decimal:
    value = Fraction(text)
    with localcontext() as ctx:
        ctx.prec = 85
        return Decimal(value.numerator) / Decimal(value.denominator)


def exercise(executable: Path) -> None:
    with localcontext() as ctx:
        ctx.prec = 85
        inputs = [
            ("pi", PI_60),
            ("e", Decimal(1).exp()),
            ("1 exp", Decimal(1).exp()),
            ("2 ln", Decimal(2).ln()),
            ("10 log10", Decimal(1)),
            ("8 log2", Decimal(3)),
            ("180 rad", PI_60),
            ("0 sin", Decimal(0)),
            ("0 cos", Decimal(1)),
            ("0 tan", Decimal(0)),
            ("0 exp", Decimal(1)),
            ("1 ln", Decimal(0)),
        ]
        commands = [":precision 128"] + [f"clear {expr}" for expr, _ in inputs]
        process = subprocess.run(
            [str(executable)], input="\n".join(commands) + "\n",
            text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False,
        )
        if process.returncode:
            raise SystemExit(f"rpn returned error: {process.stderr}")
        lines = process.stdout.splitlines()
        if lines[0] != "precision: 128 bits" or len(lines) != 1 + len(inputs):
            raise SystemExit(f"unexpected output format: {lines}")
        for (expr, expected), actual_text in zip(inputs, lines[1:]):
            actual = decimal_from_rational(actual_text)
            if abs(actual - expected) > Decimal("1e-34"):
                raise SystemExit(
                    f"FAIL {expr}: actual={actual} expected={expected} "
                    f"difference={abs(actual-expected)}"
                )
        print(f"PASS: {len(inputs)} independent Decimal/reference comparisons")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: python tools/transcendental_oracle.py path/to/rpn[.exe]")
    exercise(Path(sys.argv[1]).resolve())
