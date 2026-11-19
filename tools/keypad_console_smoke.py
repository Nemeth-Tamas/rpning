#!/usr/bin/env python3
"""Exercise the actual Win32 console adapter in a NEW HIDDEN console.

Synthetic records go only to that isolated console, never the user's terminal
or global keyboard. Verifies input, key releases/repeats, errors, diagnostic
fields, Esc/Ctrl+C exits and console-mode restoration. No dependencies.
"""
from __future__ import annotations

import ctypes
from ctypes import wintypes as w
import msvcrt
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time


class KeyEvent(ctypes.Structure):
    _fields_ = [("down", w.BOOL), ("repeat", w.WORD), ("vk", w.WORD),
                ("scan", w.WORD), ("char", w.WORD), ("flags", w.DWORD)]


class Event(ctypes.Union):
    _fields_ = [("key", KeyEvent), ("raw", w.DWORD * 4)]


class InputRecord(ctypes.Structure):
    _fields_ = [("kind", w.WORD), ("event", Event)]


class Coord(ctypes.Structure):
    _fields_ = [("x", w.SHORT), ("y", w.SHORT)]


def record(vk: int, char: str = "\0", flags: int = 0x20,
           repeat: int = 1, down: bool = True) -> InputRecord:
    return InputRecord(1, Event(key=KeyEvent(down, repeat, vk, 0x53, ord(char), flags)))


def worker(executable: Path) -> None:
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.GetConsoleMode.argtypes = [w.HANDLE, ctypes.POINTER(w.DWORD)]
    kernel.GetConsoleMode.restype = w.BOOL
    kernel.WriteConsoleInputW.argtypes = [w.HANDLE, ctypes.POINTER(InputRecord), w.DWORD, ctypes.POINTER(w.DWORD)]
    kernel.WriteConsoleInputW.restype = w.BOOL
    kernel.ReadConsoleOutputCharacterW.argtypes = [w.HANDLE, w.LPWSTR, w.DWORD, Coord, ctypes.POINTER(w.DWORD)]
    kernel.ReadConsoleOutputCharacterW.restype = w.BOOL
    assert ctypes.sizeof(InputRecord) == 20

    with open("CONIN$", "r") as console_in, open("CONOUT$", "w") as console_out:
        input_handle = msvcrt.get_osfhandle(console_in.fileno())
        output_handle = msvcrt.get_osfhandle(console_out.fileno())

        def mode() -> int:
            value = w.DWORD()
            if not kernel.GetConsoleMode(input_handle, ctypes.byref(value)):
                raise ctypes.WinError(ctypes.get_last_error())
            return value.value

        def screen() -> str:
            text = ctypes.create_unicode_buffer(32768)
            count = w.DWORD()
            if not kernel.ReadConsoleOutputCharacterW(output_handle, text, len(text), Coord(0, 0), ctypes.byref(count)):
                raise ctypes.WinError(ctypes.get_last_error())
            return text[:count.value]

        original_mode = mode()

        def exercise(flag: str, events: list[InputRecord], expected: list[str]) -> None:
            child = subprocess.Popen([str(executable)] + ([flag] if flag else []), stdin=console_in, stdout=console_out, stderr=subprocess.PIPE)
            try:
                deadline = time.monotonic() + 5
                while mode() == original_mode:
                    if child.poll() is not None or time.monotonic() > deadline:
                        raise AssertionError(f"{flag}: console did not enter input mode; exit={child.poll()}")
                    time.sleep(0.01)
                array = (InputRecord * len(events))(*events)
                count = w.DWORD()
                if not kernel.WriteConsoleInputW(input_handle, array, len(array), ctypes.byref(count)):
                    raise ctypes.WinError(ctypes.get_last_error())
                assert count.value == len(events)
                _, stderr = child.communicate(timeout=5)
                assert child.returncode == 0, stderr.decode(errors="replace")
                assert mode() == original_mode, "console input mode was not restored"
                output = screen()
                for value in expected:
                    assert value in output, f"missing {value!r}: {output!r}"
            finally:
                if child.poll() is None:
                    child.kill()
                    child.wait()

        exercise("--keypad", [
            record(0x61, "1"), record(0x61, "1", down=False), record(0x62, "2"),
            record(0x0d, "\r", flags=0x120), record(0x63, "3"), record(0x6b, "+"),
            record(0x60, "0"), record(0x6f, "/"), record(0x08, "\b"),
            record(0x62, "2"), record(0x6f, "/"),
            record(0x5a, "\u001a", flags=0x08), record(0x59, "\u0019", flags=0x08),
            record(0x1b, "\x1b"),
        ], ["depth=1  X=12", "depth=1  X=15", "division by zero", "X=15  entry=0", "X=7.5"])
        exercise("--keypad", [
            record(0x6e, ","), record(0x65, "5", repeat=2),
            record(0x0d, "\r"), record(0x43, "\x03", flags=0x08),
        ], ["depth=1  X=0.55"])
        exercise("--keypad-diagnose", [
            record(0x6e, ",", repeat=3), record(0x6e, ",", down=False),
            record(0x23, flags=0), record(0x0d, "\r", flags=0x120),
            record(0x1b, "\x1b"),
        ], ["repeat=3 vk=0x006E scan=0x0053 unicode=U+002C", "up  ", "numlock=false", "enhanced=true", "Some(Enter)"])
        exercise("--keypad", [
            record(0x34, "4", flags=0), record(0x0d, "\r"),
            record(0x71), record(0x37, "7", flags=0),
            record(0x23, flags=0x100), record(0x43, "\x03", flags=0x08),
        ], ["X=16", "layer=alt", "X=4"])

        def text(source: str) -> list[InputRecord]:
            return [record(0, char, flags=0) for char in source]

        enter = record(0x0d, "\r", flags=0)
        exercise("", text(": square dup * ;") + [enter]
                 + text("9 squa") + [record(0x09, "\t", flags=0), enter]
                 + [record(0x26, flags=0x100), enter]
                 + text("square") + [record(0x52, "\x12", flags=0x08), enter]
                 + text(":history square") + [enter]
                 + text(":quit") + [enter],
                 ["81", "9 square", ": square dup * ;"])
    print("PASS: 5 isolated Windows console sessions; keypad layers, line editing, history, completion and mode restoration")


def main() -> None:
    if len(sys.argv) == 3 and sys.argv[1] == "--worker":
        worker(Path(sys.argv[2]).resolve())
        return
    if len(sys.argv) != 2:
        raise SystemExit("Usage: python tools/keypad_console_smoke.py target/debug/rpn.exe")
    executable = Path(sys.argv[1]).resolve()
    startup = subprocess.STARTUPINFO()
    startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
    startup.wShowWindow = subprocess.SW_HIDE
    with tempfile.TemporaryFile(mode="w+b") as log, tempfile.TemporaryDirectory(prefix="rpning-console-state-") as state:
        result = subprocess.run(
            [sys.executable, str(Path(__file__).resolve()), "--worker", str(executable)],
            creationflags=subprocess.CREATE_NEW_CONSOLE, startupinfo=startup,
            env=dict(os.environ, RPNING_HOME=state),
            stdout=log, stderr=subprocess.STDOUT, timeout=30, check=False,
        )
        log.seek(0)
        print(log.read().decode(errors="replace"), end="")
        if result.returncode:
            raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
