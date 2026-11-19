#!/usr/bin/env python3
"""Exercise the actual Win32 console adapter in a NEW HIDDEN console.

Synthetic records go only to that isolated console, never the user's terminal
or global keyboard. Verifies input, key releases/repeats, errors, diagnostic
fields, Esc/Ctrl+C exits and console-mode restoration. No dependencies.
"""
from __future__ import annotations

import ctypes
from contextlib import contextmanager
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


class Rect(ctypes.Structure):
    _fields_ = [("left", w.SHORT), ("top", w.SHORT), ("right", w.SHORT), ("bottom", w.SHORT)]


class ScreenInfo(ctypes.Structure):
    _fields_ = [("size", Coord), ("cursor", Coord), ("attributes", w.WORD), ("window", Rect), ("maximum", Coord)]


class CursorInfo(ctypes.Structure):
    _fields_ = [("size", w.DWORD), ("visible", w.BOOL)]


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
    kernel.GetConsoleScreenBufferInfo.argtypes = [w.HANDLE, ctypes.POINTER(ScreenInfo)]
    kernel.GetConsoleScreenBufferInfo.restype = w.BOOL
    kernel.GetConsoleCursorInfo.argtypes = [w.HANDLE, ctypes.POINTER(CursorInfo)]
    kernel.GetConsoleCursorInfo.restype = w.BOOL
    kernel.SetConsoleWindowInfo.argtypes = [w.HANDLE, w.BOOL, ctypes.POINTER(Rect)]
    kernel.SetConsoleWindowInfo.restype = w.BOOL
    kernel.SetConsoleScreenBufferSize.argtypes = [w.HANDLE, Coord]
    kernel.SetConsoleScreenBufferSize.restype = w.BOOL
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
        exercise("--repl", text(": square dup * ;") + [enter]
                 + text("13") + [record(0x25, flags=0x100)] + text("2")
                 + [record(0x23, flags=0x100), record(0x08, "\b"),
                    record(0x24, flags=0x100), record(0x2e, flags=0x100), enter]
                 + text("9 squa") + [record(0x09, "\t", flags=0), enter]
                 + [record(0x26, flags=0x100), enter]
                 + text("square") + [record(0x52, "\x12", flags=0x08), enter]
                 + text(":stack") + [enter]
                 + text(":history square") + [enter]
                 + text(":quit") + [enter],
                 ["[0] 81", "[2] 81", "[3] 2", "9 square", ": square dup * ;"])
        # Input can be a console while stdout is redirected. No-argument
        # startup must then use plain line input, without VT or editor modes.
        redirected = subprocess.Popen([str(executable)], stdin=console_in, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        try:
            events = text("3 4 +") + [enter] + text(":quit") + [enter]
            array = (InputRecord * len(events))(*events)
            count = w.DWORD()
            assert kernel.WriteConsoleInputW(input_handle, array, len(array), ctypes.byref(count))
            assert count.value == len(events)
            output, errors = redirected.communicate(timeout=5)
            assert redirected.returncode == 0, errors.decode(errors="replace")
            assert output.decode().strip() == "7", output
            assert mode() == original_mode
        finally:
            if redirected.poll() is None:
                redirected.kill()
                redirected.wait()
        # TUI assertions run while its alternate buffer is active. After exit
        # the original screen must be back, so post-exit text alone cannot
        # validate rendering. Each read opens the currently active CONOUT$.
        @contextmanager
        def active_output():
            descriptor = os.open("CONOUT$", os.O_RDWR | os.O_BINARY)
            try:
                yield msvcrt.get_osfhandle(descriptor)
            finally:
                os.close(descriptor)

        def active() -> tuple[str, ScreenInfo]:
            with active_output() as handle:
                info = ScreenInfo()
                if not kernel.GetConsoleScreenBufferInfo(handle, ctypes.byref(info)):
                    raise ctypes.WinError(ctypes.get_last_error())
                buffer = ctypes.create_unicode_buffer(32768)
                count = w.DWORD()
                if not kernel.ReadConsoleOutputCharacterW(handle, buffer, len(buffer), Coord(0, 0), ctypes.byref(count)):
                    raise ctypes.WinError(ctypes.get_last_error())
                return buffer[:count.value], info

        with active_output() as handle:
            info = ScreenInfo()
            assert kernel.GetConsoleScreenBufferInfo(handle, ctypes.byref(info))
            small = Rect(0, 0, min(119, info.window.right-info.window.left), min(29, info.window.bottom-info.window.top))
            assert kernel.SetConsoleWindowInfo(handle, True, ctypes.byref(small))
            assert kernel.SetConsoleScreenBufferSize(handle, Coord(120, max(30, info.size.y)))
            wide = Rect(0, 0, 119, 29)
            assert kernel.SetConsoleWindowInfo(handle, True, ctypes.byref(wide))

        old_output_mode = w.DWORD()
        assert kernel.GetConsoleMode(output_handle, ctypes.byref(old_output_mode))
        old_cursor = CursorInfo()
        assert kernel.GetConsoleCursorInfo(output_handle, ctypes.byref(old_cursor))
        child = subprocess.Popen([str(executable)], stdin=console_in, stdout=console_out, stderr=subprocess.PIPE)
        try:
            def wait_frame(check) -> tuple[str, ScreenInfo]:
                deadline = time.monotonic() + 8
                latest = ""
                while time.monotonic() < deadline:
                    if child.poll() is not None:
                        raise AssertionError(f"TUI exited early: {child.communicate()[1]!r}")
                    latest, info = active()
                    if check(latest, info):
                        return latest, info
                    time.sleep(0.01)
                raise AssertionError(f"TUI frame timed out: {latest[:4000]!r}")

            def send(events: list[InputRecord]) -> None:
                array = (InputRecord * len(events))(*events)
                written = w.DWORD()
                assert kernel.WriteConsoleInputW(input_handle, array, len(array), ctypes.byref(written))
                assert written.value == len(events)

            def result_is(text: str, info: ScreenInfo, expected: str) -> bool:
                return text[3 * info.size.x:4 * info.size.x].strip() == expected

            wait_frame(lambda text, info: "RESULT X" in text and "STACK" in text and "HISTORY" in text)
            send(text("12") + [enter] + text("3+2*"))
            wait_frame(lambda text, info: result_is(text, info, "30"))
            send(text("0/"))
            wait_frame(lambda text, info: result_is(text, info, "30") and "ERROR:" in text and "division by zero" in text)
            send([record(0x08, "\b")] + text("2/"))
            wait_frame(lambda text, info: result_is(text, info, "15"))
            send([record(0x74)] + text("5 squa") + [record(0x09, "\t"), enter])
            wait_frame(lambda text, info: result_is(text, info, "25"))
            send(text(":precision 90") + [enter])
            wait_frame(lambda text, info: "90 bits" in text)
            send([record(0x72), record(0x72), record(0x28, flags=0x100), enter])
            wait_frame(lambda text, info: result_is(text, info, "25") and "5 square" in text[4 * info.size.x:5 * info.size.x])
            send([enter])
            wait_frame(lambda text, info: "STACK [3]" in text)
            send([record(0x5a, "\x1a", flags=0x08)])
            wait_frame(lambda text, info: "STACK [2]" in text)
            send([record(0x59, "\x19", flags=0x08)])
            wait_frame(lambda text, info: "STACK [3]" in text)
            # Bare settings are read-only queries; typed hints and argument
            # completion must be visible without overwriting the result.
            send(text(":display"))
            wait_frame(lambda text, info: "current = auto" in text and "fixed:N" in text)
            send([enter])
            wait_frame(lambda text, info: "OUTPUT:" in text and "Allowed:" in text and result_is(text, info, "25"))
            send(text(":display h") + [record(0x09, "\t"), enter])
            wait_frame(lambda text, info: result_is(text, info, "0x19"))
            send(text(":display auto") + [enter])
            wait_frame(lambda text, info: result_is(text, info, "25"))
            send(text(":display") + [record(0x09, "\t"), record(0x75), record(0x23, flags=0x100)])
            wait_frame(lambda text, info: "fixed:128" in text and "FOCUS: Output" in text)
            # Completion leaves the command editable; clear it using the
            # documented CMD Home/Delete keys, then inspect the actual guide.
            send([record(0x70), record(0x24, flags=0x100)]
                 + [record(0x2e, flags=0x100)] * len(":display ")
                 + text(":help layers") + [enter])
            wait_frame(lambda text, info: "Keypad layers" in text and "OUTPUT:" in text)
            send([record(0x75), record(0x22, flags=0x100)])
            wait_frame(lambda text, info: "Square X" in text and "FOCUS: Output" in text)
            send([record(0x73)])
            wait_frame(lambda text, info: "STACK [3]" in text and "OUTPUT:" not in text)
            send([record(0x73)])
            wait_frame(lambda text, info: "FOCUS: History" in text)
            # Inject the documented resize record into this isolated console.
            # Classic size-changing APIs cannot resize every hidden VT host.
            resized = InputRecord(0x0004, Event(raw=(w.DWORD * 4)(60 | (20 << 16), 0, 0, 0)))
            send([resized])
            wait_frame(lambda text, info: "HISTORY" in text[7 * info.size.x:8 * info.size.x] and "STACK" not in text[7 * info.size.x:8 * info.size.x])
            send([record(0x73)])
            wait_frame(lambda text, info: "STACK" in text[7 * info.size.x:8 * info.size.x])
            send([record(0x1b, "\x1b")])
            _, stderr = child.communicate(timeout=5)
            assert child.returncode == 0, stderr.decode(errors="replace")
            assert mode() == original_mode
            restored = w.DWORD()
            assert kernel.GetConsoleMode(output_handle, ctypes.byref(restored))
            assert restored.value == old_output_mode.value
            cursor = CursorInfo()
            assert kernel.GetConsoleCursorInfo(output_handle, ctypes.byref(cursor))
            assert (cursor.visible, cursor.size) == (old_cursor.visible, old_cursor.size)
            assert "RESULT X" not in active()[0], "alternate screen was not restored"
        finally:
            if child.poll() is None:
                child.kill()
                child.wait()
    print("PASS: 7 isolated Windows console sessions; explicit REPL/default TUI, redirected output, live frames, setting discovery, full completion, layer help, errors, recall, undo, resize and restoration")


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
            env=dict(os.environ, RPNING_HOME=state, PYTHONIOENCODING="utf-8"),
            stdout=log, stderr=subprocess.STDOUT, timeout=30, check=False,
        )
        log.seek(0)
        sys.stdout.buffer.write(log.read())
        sys.stdout.buffer.flush()
        if result.returncode:
            raise SystemExit(result.returncode)


if __name__ == "__main__":
    main()
