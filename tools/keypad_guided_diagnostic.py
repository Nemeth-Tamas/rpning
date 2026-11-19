#!/usr/bin/env python3
"""Read-only guided Windows keypad capture; no calculator mappings are changed."""
from __future__ import annotations
import ctypes
from ctypes import wintypes as w
from datetime import datetime
import json
import msvcrt
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

KEYS = ("/", "*", "Backspace", "7", "8", "9", "-", "4", "5", "6", "+",
        "1", "2", "3", "Enter", "0", "00", ".")


class KeyEvent(ctypes.Structure):
    _fields_ = [("down", w.BOOL), ("repeat", w.WORD), ("vk", w.WORD),
                ("scan", w.WORD), ("char", w.WORD), ("flags", w.DWORD)]


class Event(ctypes.Union):
    _fields_ = [("key", KeyEvent), ("raw", w.DWORD * 4)]


class InputRecord(ctypes.Structure):
    _fields_ = [("kind", w.WORD), ("event", Event)]


class Console:
    def __init__(self, source, destination):
        if not source.isatty() or not destination.isatty():
            raise RuntimeError("Run directly in a Windows terminal, without pipes or redirection.")
        self.k = ctypes.WinDLL("kernel32", use_last_error=True)
        self.k.GetConsoleMode.argtypes = [w.HANDLE, ctypes.POINTER(w.DWORD)]
        self.k.GetConsoleMode.restype = w.BOOL
        self.k.SetConsoleMode.argtypes = [w.HANDLE, w.DWORD]
        self.k.SetConsoleMode.restype = w.BOOL
        self.k.WaitForSingleObject.argtypes = [w.HANDLE, w.DWORD]
        self.k.WaitForSingleObject.restype = w.DWORD
        self.k.ReadConsoleInputW.argtypes = [w.HANDLE, ctypes.POINTER(InputRecord), w.DWORD, ctypes.POINTER(w.DWORD)]
        self.k.ReadConsoleInputW.restype = w.BOOL
        self.handle = msvcrt.get_osfhandle(source.fileno())
        self.original = w.DWORD()
        if not self.k.GetConsoleMode(self.handle, ctypes.byref(self.original)):
            raise ctypes.WinError(ctypes.get_last_error())

    def __enter__(self):
        # Same raw-input flags as rpning; only this focused console is read.
        mode = (self.original.value | 0x80) & ~(1 | 2 | 4 | 8 | 16 | 64 | 512)
        if not self.k.SetConsoleMode(self.handle, mode):
            raise ctypes.WinError(ctypes.get_last_error())
        return self

    def __exit__(self, *_):
        if not self.k.SetConsoleMode(self.handle, self.original):
            raise ctypes.WinError(ctypes.get_last_error())

    def read(self):
        ready = self.k.WaitForSingleObject(self.handle, 50)
        if ready == 258:
            return None
        if ready != 0:
            raise ctypes.WinError(ctypes.get_last_error())
        record, count = InputRecord(), w.DWORD()
        if not self.k.ReadConsoleInputW(self.handle, ctypes.byref(record), 1, ctypes.byref(count)):
            raise ctypes.WinError(ctypes.get_last_error())
        if count.value == 1 and record.kind == 1:
            return KeyEvent.from_buffer_copy(record.event.key)
        return None


def aborted(event):
    return event.down and (event.vk == 0x1B or (
        event.vk == 0x43 and event.flags & 0xC and not event.flags & 3))


def collect(console, quiet=0.6):
    events, started, last = [], None, None
    while True:
        event = console.read()
        now = time.monotonic()
        if event is not None:
            if aborted(event):
                return "aborted", events
            if event.down and event.vk == 0x77:
                return ("manual-finish" if events else "no-event-skipped"), events
            if event.vk in (0x77, 0x79):  # Ignore F8/F10 release tails.
                continue
            if not events and not event.down:
                continue
            if started is None:
                started = now
            events.append((now - started, event))
            last = now
            if len(events) >= 512:
                return "too-many-events-aborted", events
        if last is not None and now - last >= quiet:
            return "captured", events


def format_event(elapsed, event):
    return (f"+{elapsed*1000:8.1f}ms {'down' if event.down else 'up  '} repeat={event.repeat} "
            f"vk=0x{event.vk:04X} scan=0x{event.scan:04X} unicode=U+{event.char:04X} "
            f"char={json.dumps(chr(event.char), ensure_ascii=True)} flags=0x{event.flags:04X} "
            f"numlock={bool(event.flags & 0x20)} enhanced={bool(event.flags & 0x100)}")


def saved_bindings():
    if "RPNING_HOME" in os.environ:
        path = Path(os.environ["RPNING_HOME"]) / "settings.txt"
    elif os.environ.get("APPDATA"):
        path = Path(os.environ["APPDATA"]) / "rpning/settings.txt"
    elif os.environ.get("XDG_DATA_HOME"):
        path = Path(os.environ["XDG_DATA_HOME"]) / "rpning/settings.txt"
    elif os.environ.get("HOME"):
        path = Path(os.environ["HOME"]) / ".local/share/rpning/settings.txt"
    else:
        path = Path("settings.txt")
    try:
        if not path.exists():
            return ["Saved settings absent: calculator defaults apply."]
        if path.stat().st_size > 1024*1024:
            return ["Saved settings exceed 1 MiB: not read."]
        return ["Saved binding records (read only):"] + [
            json.dumps(line.split("\t"), ensure_ascii=True)
            for line in path.read_text(encoding="utf-8").splitlines() if line.startswith("bind\t")]
    except (OSError, UnicodeError) as error:
        return [f"Settings could not be read: {type(error).__name__}"]


def report(captures, settings, complete):
    lines = ["RPNING GUIDED KEYPAD REPORT v1", f"Complete: {complete}",
             "Pass labels are requested physical NumLock states, not inferred from event flags.",
             "Physical labels come from the prompts; no key remapping is applied.",
             "Order: / * Backspace | 7 8 9 - | 4 5 6 + | 1 2 3 Enter | 0 00 .", *settings, ""]
    for phase, label, status, events in captures:
        lines.append(f"PASS={phase} PHYSICAL_KEY={label} STATUS={status} EVENTS={len(events)}")
        lines.extend(format_event(elapsed, event) for elapsed, event in events)
        lines.append("")
    return "\n".join(lines + ["END RPNING GUIDED KEYPAD REPORT", ""])


def wizard():
    captures, complete = [], False
    settings = saved_bindings()
    path = Path.cwd() / ("keypad-diagnostic-" + datetime.now().strftime("%Y%m%d-%H%M%S-%f") + ".txt")
    print("Guided keypad diagnostic: settings and mappings are not changed.")
    print("Press/release each NUMPAD key ONCE, then WAIT for the next prompt.")
    print("00 is ONE physical press; its entire event burst is captured together.")
    print("F8 on the main keyboard skips a silent key (often NumLock-OFF 5), or finishes a capture.")
    print("Esc/Ctrl+C aborts and produces a partial report.")
    try:
        with Console(sys.stdin, sys.stdout) as console:
            for phase in ("NUMLOCK_ON", "NUMLOCK_OFF"):
                print(f"\nSet NumLock {'ON' if phase == 'NUMLOCK_ON' else 'OFF'} physically, then press F10 on the main keyboard.", flush=True)
                while True:
                    event = console.read()
                    if event is not None:
                        if aborted(event):
                            return
                        if event.down and event.vk == 0x79:
                            break
                for index, label in enumerate(KEYS, 1):
                    print(f"[{phase} {index:02}/{len(KEYS)}] Press NUMPAD {label} once, release, then wait...", flush=True)
                    status, events = collect(console)
                    captures.append((phase, label, status, events))
                    print(f"  {status}: {len(events)} events", flush=True)
                    if status.endswith("aborted"):
                        return
            complete = True
    finally:
        text = report(captures, settings, complete)
        print("\n" + text, end="")
        try:
            path.write_text(text, encoding="utf-8")
            print(f"Report saved: {path}")
        except OSError as error:
            print(f"Could not save ({type(error).__name__}); copy the report above.")
        print("Paste everything from RPNING GUIDED KEYPAD REPORT through END RPNING GUIDED KEYPAD REPORT.")


def test_worker():
    # This is called ONLY in the new hidden console created by --self-test.
    assert ctypes.sizeof(KeyEvent) == 16 and ctypes.sizeof(InputRecord) == 20
    with open("CONIN$", "r") as source, open("CONOUT$", "w") as destination:
        console = Console(source, destination)
        console.k.WriteConsoleInputW.argtypes = [w.HANDLE, ctypes.POINTER(InputRecord), w.DWORD, ctypes.POINTER(w.DWORD)]
        console.k.WriteConsoleInputW.restype = w.BOOL

        def inject(events):
            records = (InputRecord * len(events))(*(InputRecord(1, Event(key=e)) for e in events))
            count = w.DWORD()
            assert console.k.WriteConsoleInputW(console.handle, records, len(records), ctypes.byref(count))
            assert count.value == len(records)

        with console:
            inject([KeyEvent(1, 1, 0x24, 0x47, ord('7'), 0x100), KeyEvent(0, 1, 0x24, 0x47, ord('7'), 0x100)])
            status, seven = collect(console, 0.01)
            assert status == "captured" and len(seven) == 2
            assert seven[0][1].vk == 0x24 and seven[0][1].char == ord('7')
            inject([KeyEvent(1, 1, 0x60, 0x52, ord('0'), 0), KeyEvent(0, 1, 0x60, 0x52, ord('0'), 0)] * 2)
            status, zeros = collect(console, 0.01)
            assert status == "captured" and len(zeros) == 4
            inject([KeyEvent(1, 1, 0x77, 0, 0, 0)])
            assert collect(console) == ("no-event-skipped", [])
            inject([KeyEvent(1, 1, 0x1B, 0, 27, 0)])
            assert collect(console)[0] == "aborted"
            text = report([("NUMLOCK_ON", "7", "captured", seven), ("NUMLOCK_OFF", "00", "captured", zeros)], [], False)
            assert 'PHYSICAL_KEY=7' in text and 'vk=0x0024' in text and 'char="7"' in text
            assert 'PHYSICAL_KEY=00 STATUS=captured EVENTS=4' in text
        restored = w.DWORD()
        assert console.k.GetConsoleMode(console.handle, ctypes.byref(restored))
        assert restored.value == console.original.value
    assert len(KEYS) == 18 and KEYS[-3:] == ("0", "00", ".")
    print("PASS: raw identities, grouped 00 events, silent-key skip, abort and console-mode restoration")


def main():
    if sys.platform != "win32":
        raise SystemExit("This focused keypad diagnostic requires Windows.")
    if sys.argv[1:] == ["--test-worker"]:
        test_worker()
    elif sys.argv[1:] == ["--self-test"]:
        startup = subprocess.STARTUPINFO()
        startup.dwFlags |= subprocess.STARTF_USESHOWWINDOW
        startup.wShowWindow = subprocess.SW_HIDE
        with tempfile.TemporaryFile(mode="w+b") as log:
            result = subprocess.run([sys.executable, str(Path(__file__).resolve()), "--test-worker"],
                                    creationflags=subprocess.CREATE_NEW_CONSOLE, startupinfo=startup,
                                    stdout=log, stderr=subprocess.STDOUT,
                                    env=dict(os.environ, PYTHONIOENCODING="utf-8"), timeout=15)
            log.seek(0)
            sys.stdout.buffer.write(log.read())
        raise SystemExit(result.returncode)
    elif sys.argv[1:]:
        raise SystemExit("Usage: python tools/keypad_guided_diagnostic.py [--self-test]")
    else:
        wizard()


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError) as error:
        raise SystemExit(str(error))
