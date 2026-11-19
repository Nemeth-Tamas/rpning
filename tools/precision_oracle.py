#!/usr/bin/env python3
"""Independent exact Python Fraction oracle for rpning binary rounding.

Usage on Windows:
    python tools/precision_oracle.py target/debug/rpn.exe
On Linux:
    python3 tools/precision_oracle.py target/debug/rpn

No Rust/Rug/Python third-party dependencies are required. Verifies every
supported rounding direction, tiny values, huge values and midpoint ties.
"""
from __future__ import annotations

import random
import subprocess
import sys
from fractions import Fraction
from pathlib import Path


def reference(value: Fraction, bits: int, mode: str) -> Fraction:
    if value == 0:
        return Fraction(0)
    negative = value < 0
    positive = abs(value)
    numerator, denominator = positive.numerator, positive.denominator
    exponent = numerator.bit_length() - denominator.bit_length()
    if exponent >= 0:
        if numerator < (denominator << exponent):
            exponent -= 1
    elif (numerator << -exponent) < denominator:
        exponent -= 1
    shift = bits - 1 - exponent
    if shift >= 0:
        numerator <<= shift
    else:
        denominator <<= -shift
    quotient, rem = divmod(numerator, denominator)
    if mode == "nearest":
        inc = 2 * rem > denominator or (2 * rem == denominator and quotient % 2 != 0)
    elif mode == "up":
        inc = bool(rem) and not negative
    elif mode == "down":
        inc = bool(rem) and negative
    elif mode == "away":
        inc = bool(rem)
    elif mode == "zero":
        inc = False
    else:
        raise ValueError(mode)
    quotient += int(inc)
    result = Fraction(quotient, 1 << shift) if shift >= 0 else Fraction(quotient << -shift)
    return -result if negative else result


def test_vectors() -> list[Fraction]:
    samples = [
        Fraction(0), Fraction(1, 3), Fraction(-1, 3), Fraction(5, 4),
        Fraction(7, 4), Fraction(-7, 4), Fraction(1, 2**150),
        Fraction(2**180), Fraction((2**200) + 123, 3),
        Fraction(-((2**200) + 123), 3), Fraction(1, 10),
        Fraction(-25, 16), Fraction(2**200, 2**155),
    ]
    rng = random.Random(20261118)
    for _ in range(25):
        num = rng.getrandbits(rng.randint(1, 120))
        den = max(rng.getrandbits(rng.randint(1, 120)), 1)
        samples.append(Fraction(num * (-1 if rng.getrandbits(1) else 1), den))
    return samples


def main(executable: Path) -> None:
    if not executable.is_file():
        raise SystemExit(f"Calculator executable not found: {executable}")
    cases = test_vectors()
    passed = 0
    for bits in [2, 3, 4, 8, 53, 256]:
        for mode in ["nearest", "zero", "up", "down", "away"]:
            commands = [":mode binary", f":precision {bits}", f":round {mode}"]
            commands.extend(f"clear {value.numerator}/{value.denominator}" for value in cases)
            output = subprocess.run(
                [str(executable)], input="\n".join(commands) + "\n", text=True,
                capture_output=True, check=False,
            )
            if output.returncode:
                raise SystemExit(f"rpn failed at {bits=} {mode=}: {output.stderr}")
            lines = output.stdout.splitlines()
            if len(lines) != len(cases) + 3:
                raise SystemExit(f"unexpected output count for {bits=} {mode=}: {lines[:8]}")
            for value, actual_text in zip(cases, lines[3:]):
                expected = reference(value, bits, mode)
                actual = Fraction(actual_text)
                if actual != expected:
                    raise SystemExit(
                        f"ROUNDING MISMATCH {bits=} {mode=} {value=} "
                        f"expected={expected} actual={actual} printed={actual_text}"
                    )
                passed += 1
    print(f"PASS: {passed} independently checked binary-rounding results")


if __name__ == "__main__":
    if len(sys.argv) != 2:
        raise SystemExit("Usage: python tools/precision_oracle.py path/to/rpn[.exe]")
    main(Path(sys.argv[1]).resolve())
