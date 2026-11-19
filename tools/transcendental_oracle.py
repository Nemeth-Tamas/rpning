#!/usr/bin/env python3
"""Independent Decimal references, including nonzero trig and range boundaries.

Uses only Python's standard library. Pi uses Gauss-Legendre iteration, distinct
from rpning's Machin series. Decimal exp/ln and trig Taylor references run at
210 decimal digits. Sampled error checks are not a proof of correct rounding.
"""
from __future__ import annotations

import subprocess
import sys
from decimal import Decimal, localcontext
from fractions import Fraction
from pathlib import Path


def pi_reference() -> Decimal:
    a, b, t, p = Decimal(1), Decimal(1) / Decimal(2).sqrt(), Decimal("0.25"), Decimal(1)
    for _ in range(11):
        next_a = (a + b) / 2
        b = (a * b).sqrt()
        t -= p * (a - next_a) ** 2
        a, p = next_a, p * 2
    return (a + b) ** 2 / (4 * t)


def sin_cos_reference(x: Decimal, pi: Decimal) -> tuple[Decimal, Decimal]:
    x %= 2 * pi
    if x > pi:
        x -= 2 * pi
    if x < -pi:
        x += 2 * pi
    sin_term, cos_term = x, Decimal(1)
    sine, cosine = sin_term, cos_term
    for n in range(1, 1000):
        sin_term *= -(x * x) / ((2 * n) * (2 * n + 1))
        cos_term *= -(x * x) / ((2 * n - 1) * (2 * n))
        sine += sin_term
        cosine += cos_term
        if max(abs(sin_term), abs(cos_term)) < Decimal("1e-205"):
            return sine, cosine
    raise AssertionError("reference trig did not converge")


def exercise(executable: Path) -> None:
    with localcontext() as ctx:
        ctx.prec = 210
        pi = pi_reference()
        assert str(pi).startswith("3.14159265358979323846264338327950288419716939937510")
        inputs = [("pi", pi), ("e", Decimal(1).exp()), ("180 rad", pi), ("1 deg", 180 / pi)]
        for text in ["-256", "-10", "-0.5", "0", "1", "10", "256"]:
            inputs.append((f"{text} exp", Decimal(text).exp()))
        for text in ["0.0001", "0.5", "1", "1.000001", "2", "10000"]:
            x = Decimal(text)
            inputs.extend([
                (f"{text} ln", x.ln()),
                (f"{text} log2", x.ln() / Decimal(2).ln()),
                (f"{text} log10", x.ln() / Decimal(10).ln()),
            ])
        for text in ["-1000", "-3", "-0.5", "0", "0.5", "1", "1.5707", "1000"]:
            sine, cosine = sin_cos_reference(Decimal(text), pi)
            inputs.extend([(f"{text} sin", sine), (f"{text} cos", cosine), (f"{text} tan", sine / cosine)])

        total = 0
        for bits, rounding in [(16, "nearest"), (53, "nearest"), (128, "nearest"),
                               (256, "nearest"), (512, "nearest"),
                               (128, "zero"), (128, "up"), (128, "down"), (128, "away")]:
            commands = [f":precision {bits}", f":round {rounding}"]
            commands += [f"clear {expression}" for expression, _ in inputs]
            process = subprocess.run(
                [str(executable)], input="\n".join(commands) + "\n", text=True,
                stdout=subprocess.PIPE, stderr=subprocess.PIPE, check=False, timeout=60,
            )
            if process.returncode:
                raise SystemExit(f"rpn returned error ({bits}/{rounding}): {process.stderr}")
            lines = process.stdout.splitlines()
            headers = [f"precision: {bits} bits", f"rounding: {rounding}"]
            if lines[:2] != headers or len(lines) != 2 + len(inputs):
                raise SystemExit(f"unexpected output format: {lines}")
            for (expression, expected), actual_text in zip(inputs, lines[2:], strict=True):
                rational = Fraction(actual_text)
                actual = Decimal(rational.numerator) / Decimal(rational.denominator)
                # Eight relative significant-bit units accommodate directed
                # output rounding and the documented non-certified series.
                tolerance = abs(expected) * Decimal(2) ** (4 - bits)
                if abs(actual - expected) > tolerance:
                    raise SystemExit(
                        f"FAIL {bits}/{rounding} {expression}: actual={actual} "
                        f"expected={expected} difference={abs(actual-expected)} limit={tolerance}"
                    )
                total += 1
        print(f"PASS: {total} independent Decimal comparisons (16..512 bits, all rounding modes)")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: python tools/transcendental_oracle.py path/to/rpn[.exe]")
    exercise(Path(sys.argv[1]).resolve())
