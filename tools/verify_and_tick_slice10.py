#!/usr/bin/env python3
"""Run rpning quality gates, then check off only existing verified TODO lines.

No TODO edits occur unless all tests, Clippy, and both numeric oracles pass.
No new task names, no task deletions, and no renamed milestones.
"""
from __future__ import annotations
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EXECUTABLE = "target/debug/rpn.exe" if sys.platform == "win32" else "target/debug/rpn"
CHECKS = [
    ["cargo", "fmt"],
    ["cargo", "fmt", "--check"],
    ["cargo", "check"],
    ["cargo", "test"],
    ["cargo", "clippy", "--all-targets", "--all-features", "--", "-D", "warnings"],
    ["cargo", "build", "--bin", "rpn"],
    [sys.executable, "tools/precision_oracle.py", EXECUTABLE],
    [sys.executable, "tools/transcendental_oracle.py", EXECUTABLE],
]
FINISHED_MILESTONES = [
    "Add independent precision reference tests",
    "Add logarithmic and exponential functions",
    "Add trigonometric functions",
    "Add mathematical constants",
    "Add degree/radian conversion",
]


def main() -> None:
    for argv in CHECKS:
        print("\n+ " + " ".join(argv), flush=True)
        code = subprocess.call(argv, cwd=ROOT)
        if code:
            raise SystemExit(f"Quality gate failed (exit {code}). TODO.md is unchanged.")
    todo_path = ROOT / "TODO.md"
    raw = todo_path.read_bytes()
    newline = "\r\n" if b"\r\n" in raw else "\n"
    text = raw.decode("utf-8").replace("\r\n", "\n")
    changed = []
    for name in FINISHED_MILESTONES:
        old = f"- [ ] {name}"
        new = f"- [x] {name}"
        if text.count(old) == 1:
            text = text.replace(old, new, 1)
            changed.append(name)
        elif text.count(new) == 1:
            print(f"already checked: {name}")
        else:
            raise SystemExit(
                f"TODO task missing/ambiguous: {name}. Quality gates passed; TODO not edited."
            )
    todo_path.write_bytes(text.replace("\n", newline).encode("utf-8"))
    print(f"\nAll gates green. Checked {len(changed)} existing TODO tasks.")
    print("M2 stable numeric abstraction and small-integer optimization remain pending.")


if __name__ == "__main__":
    main()
