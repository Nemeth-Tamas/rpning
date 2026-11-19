#!/usr/bin/env python3
"""Exact, fail-closed repair for rpning's 2026-11-19 transcendental unit tests.

Only the #[cfg(test)] module is modified. No code outside the test module,
no TODO state, and no source file other than src/transcendental.rs is changed.
Original LF/CRLF line endings are preserved.
"""
from pathlib import Path

root = Path(__file__).resolve().parent.parent
path = root / "src" / "transcendental.rs"
raw = path.read_bytes()
newline = "\r\n" if b"\r\n" in raw else "\n"
source = raw.decode("utf-8").replace("\r\n", "\n")
marker = "#[cfg(test)]\nmod tests {\n"
if source.count(marker) != 1:
    raise SystemExit("ABORT: test module marker did not match exactly once; no changes written")
production, tests = source.split(marker, 1)
fixes = [
    ("r(1_000_000_000_000_000_000)", 'r("1000000000000000000")', 1),
    ("r(1_000_000_000)", 'r("1000000000")', 7),
    ("r(1)", 'r("1")', 9),
]
for old, new, expected_count in fixes:
    actual = tests.count(old)
    if actual != expected_count:
        raise SystemExit(
            f"ABORT: exact test FIND {old!r} matched {actual} times, "
            f"expected {expected_count}; no changes written"
        )
    tests = tests.replace(old, new)
updated = production + marker + tests
path.write_bytes(updated.replace("\n", newline).encode("utf-8"))
print("Fixed 17 test helper argument errors in src/transcendental.rs.")
print("No production code or TODO items changed. Run the full quality gates.")
