#!/usr/bin/env python3
"""Exact-anchor, fail-closed rpning slice 10 integration.

Read the current local files and require every FIND block to match exactly once
before modifying anything. Preserves CRLF/LF style. Does not rewrite task names
or move TODO goalposts. No network, git operations, or dependencies.
"""
from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent.parent
pending: dict[Path, tuple[str, str]] = {}


def exact(relative: str, find: str, replace: str) -> None:
    path = ROOT / relative
    if path not in pending:
        raw = path.read_bytes()
        newline = "\r\n" if b"\r\n" in raw else "\n"
        pending[path] = (raw.decode("utf-8").replace("\r\n", "\n"), newline)
    content, newline = pending[path]
    count = content.count(find)
    if count != 1:
        raise RuntimeError(f"{relative}: literal FIND occurred {count} times, not 1:\n{find}")
    pending[path] = (content.replace(find, replace, 1), newline)


def main() -> None:
    for relative in [
        "src/transcendental.rs",
        "tests/transcendental_cli.rs",
        "tools/transcendental_oracle.py",
        "docs/M2-M3-REAL-MATH.md",
    ]:
        if not (ROOT / relative).is_file():
            raise RuntimeError(f"Missing downloaded file: {relative}")

    if "fn main()" not in (ROOT / "src/main.rs").read_text(encoding="utf-8"):
        raise RuntimeError("src/main.rs has no CLI main function; fix the CLI before applying")

    exact(
        "src/lib.rs",
        "pub mod storage;",
        "pub mod storage;\npub mod transcendental;",
    )
    exact(
        "src/engine.rs",
        "use crate::precision::{MAX_BITS, MIN_BITS, NumericSettings, Rounding, round_binary};",
        "use crate::precision::{MAX_BITS, MIN_BITS, NumericSettings, Rounding, round_binary};\nuse crate::transcendental;",
    )
    exact(
        "src/engine.rs",
        '                "pow" | "root" | "gcd" | "lcm" => {',
        '''                "pi" | "e" => {
                    let value = transcendental::constant(
                        token,
                        self.settings.bits,
                        self.settings.rounding,
                    )
                    .map_err(CalcError::Math)?;
                    working.push(value);
                }

                "exp" | "ln" | "log2" | "log10" | "sin" | "cos" | "tan" | "rad"
                | "deg" => {
                    require_depth(&working, 1, token)?;
                    let len = working.len();
                    let value = transcendental::unary(
                        token,
                        &working[len - 1],
                        self.settings.bits,
                        self.settings.rounding,
                    )
                    .map_err(CalcError::Math)?;
                    working.pop();
                    working.push(value);
                }

                "pow" | "root" | "gcd" | "lcm" => {''',
    )
    exact(
        "src/main.rs",
        '    println!("  pow sqrt root fact gcd lcm Exact advanced mathematics");',
        '''    println!("  pow sqrt root fact gcd lcm Exact advanced mathematics");
    println!("  pi e                      Approximate mathematical constants");
    println!("  exp ln log2 log10         Approximate exponential/logarithms");
    println!("  sin cos tan               Approximate trigonometry (radians)");
    println!("  rad deg                   Convert degrees/radians");''',
    )
    exact(
        "README.md",
        "## Roadmap\n\nSee `TODO.md`.",
        '''## Approximate real functions (M3)

The new `pi`, `e`, `exp`, `ln`, `log2`, `log10`, `sin`, `cos`, `tan`, `rad`,
`deg` words are computed from exact rational series and then rounded to the
configured binary precision. **They are approximate even in `:mode exact`.**
The ordinary rational arithmetic operators remain exact in exact mode.

The implementation avoids f32/f64 conversion and uses 40 guard bits, but
is **not certified correctly rounded for every possible input**. It limits
transcendental precision to 512 bits for responsiveness. Basic binary
arithmetic still supports 4096 bits. Input-domain/range errors roll back
atomically. `sin`, `cos` and `tan` accept radians; `rad` converts degrees to
radians and `deg` converts radians to degrees.

See `docs/M2-M3-REAL-MATH.md` for accuracy, limits, examples and oracle tests.

## Roadmap

See `TODO.md`.''',
    )
    # The code and unit tests already exist for these pre-existing milestones.
    # Tick *only* literal checkbox lines. New functions stay pending until gates.
    previously_implemented = [
        "Add configurable binary floating-point precision",
        "Add exact-to-approximate conversion rules",
        "Add configurable rounding",
        "Add scientific number formatting",
        "Add persistent register storage",
        "Add exponentiation and roots",
        "Add factorial, GCD and LCM",
    ]
    for label in previously_implemented:
        exact("TODO.md", f"- [ ] {label}", f"- [x] {label}")

    # Every anchor has been verified; write only after successful preflight.
    for path, (content, newline) in pending.items():
        path.write_bytes(content.replace("\n", newline).encode("utf-8"))
        print(f"updated {path.relative_to(ROOT)}")
    print("All exact anchors matched. New M3 real-math checkboxes remain open until tested.")


if __name__ == "__main__":
    try:
        main()
    except (OSError, UnicodeError, RuntimeError) as error:
        sys.exit(f"No changes written: {error}")
