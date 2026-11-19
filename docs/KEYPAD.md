# Keypad mode: first usable Windows slice

## Guided physical-key report

If a physical key invokes the wrong layer action, run this read-only wizard
from a Windows terminal (Python 3.10+, standard library only; no Rust rebuild):

```powershell
python tools/keypad_guided_diagnostic.py
```

Set NumLock ON and press main-keyboard F10 to start. Press/release each prompted
numpad key once, then wait for the next prompt. It captures /, *, Backspace,
7, 8, 9, -, 4, 5, 6, +, 1, 2, 3, Enter, 0, 00 and decimal, in that order.
Repeat after setting NumLock OFF and pressing F10. A single 00 press is captured
as one burst, retaining all emitted zero events. Use main-keyboard F8 to skip
a silent key; it does not silently advance on a timeout.
Esc/Ctrl+C aborts with a partial report. Console input mode is restored.

The report includes physical prompt labels, timing, keydown/up/repeat, virtual
key, scan code, Unicode and flags, plus saved binding records. Requested
NumLock pass labels are separate from the reported NumLock flag. It changes
no mappings/settings and performs no calculations. Copy the final report
between its BEGIN-style heading and END marker; a timestamped copy is also
saved as keypad-diagnostic-*.txt in the working directory and ignored by Git.

Self-test in an isolated hidden console:
`python tools/keypad_guided_diagnostic.py --self-test`.

The CONNECT IT NumCALC CKB-0060-BK's actual emitted events were checked on
2026-11-19, with both numeric and navigation modes. The calculation
`12 Enter 3 + 2 *` produced X=30. This implementation uses console-translated
events, without assuming a particular USB identity.

The complete guided capture on 2026-11-19 confirmed standard numpad virtual
keys and character digits with NumLock ON. Decimal reports comma. With NumLock
OFF the positions emit Home/Up/PageUp/Left/Clear/Right/End/Down/PageDown/Insert/
Delete, with NUL characters and no enhanced flag; the reported NumLock flag
tracks the requested states in this capture. The 00 key emits two 0 events
with NumLock ON or two Insert events with it OFF. Earlier captures used a
different console translation and remain covered by existing regressions.

Alt/custom layers now resolve those measured non-enhanced navigation events
back to physical digit/decimal positions before looking up bindings. Thus
Alt-7 squares and Alt-6 drops in both states, and a removed Alt binding falls
back to ordinary entry. Base retains Nav shortcuts with NumLock OFF; dedicated
enhanced navigation keys, CMD editing, pane browsing and raw diagnostics are
unchanged. The capture identifies console events, not a particular USB device.

## First hardware check

Run directly in PowerShell or Windows Terminal, with the terminal focused:

```powershell
cargo run -- --keypad-diagnose
```

Press every numpad key once with NumLock on, then off, including decimal,
Enter, Backspace, the arithmetic operators and any extra keys. Test a held
digit briefly. Esc or Ctrl+C exits and restores the console input mode.
Copy the emitted lines after exiting. Do not redirect or pipe diagnostic
stdout: both stdin and stdout must be interactive console handles.

Output includes down/up, repeat count, virtual-key code, scan code, UTF-16
character, modifier flags, NumLock, enhanced-key flag and the proposed action.
These are the events delivered by the console host. Terminal software can
translate hardware input. This is not a USB Raw Input logger; it cannot
distinguish the separate numpad from the main keyboard or identify its device.
Keys that emit no event remain unverified rather than being assigned guessed
codes. Only the focused console supplies input; there are no global hooks.

## Calculation

```powershell
cargo run -- --keypad
```

Use NumLock on. The display shows stack depth, X and pending entry after each
accepted event. Each value is clipped to 56 characters on screen, with `...`
when clipped; the stored exact value is unchanged. This intentionally simple
first interface prints a fresh status line instead of requiring ANSI support.
Normal `cargo run` starts the TUI on Windows when both streams are terminals.
Use `cargo run -- --repl` for the command-based line editor. The
[TUI layer guide](TUI.md#keypad-layers) lists every Base/Alt/Nav mapping.

| Input | Behavior |
| --- | --- |
| Digits | Append to a separate pending number, maximum 4096 characters |
| `.` or `,` | Decimal point; leading separator enters `0.`; duplicate separator ignored |
| Enter with pending number | Push the number exactly once and empty entry |
| Enter with empty entry | Duplicate X; empty stack reports an error |
| `+ - * /` | Commit pending number and execute operator in one atomic transaction |
| Backspace | Remove one pending character; empty entry leaves stack untouched |
| Enhanced Delete | Cancel pending entry, including this numpad's navigation-mode decimal |
| `S` | Toggle pending sign, or negate X if entry is empty |
| `C` | Clear stack and pending entry; committed stack clear is undoable |
| Tab | Commit pending number and swap X/Y atomically |
| Ctrl+Z | Cancel pending entry first; otherwise undo a committed transaction |
| Ctrl+Y | Redo; reports an error if an entry is pending |
| Esc or Ctrl+C | Exit, discarding any pending entry |
| Other unmapped input | Ignore |

A trailing decimal point is accepted as an integer. A sign-only pending entry
must be completed or cancelled. Minus is always subtraction; `S` supplies sign
entry. There is no implicit repetition of the last binary operator.

Example: `12`, Enter, `3`, `+`, `2`, `*` gives `30`. Undo restores `15`,
because `2 *` was a single transaction. A failed `0 /` preserves both the
stack and pending `0`; Backspace then `2 /` retries with a corrected operand.
Failed operations preserve registers and redo history. Key-up records do not
execute actions; repeat counts are honored. Ctrl/Alt combinations are ignored
except the documented Ctrl shortcuts. Mappings, alternate layers and macro
shortcuts are configurable with `:bind`, `:unbind`, `:shortcut` and `:bindings`;
see [the workflow guide](WORKFLOW.md). F2 toggles base/alt; navigation events
select nav independently of the reported NumLock flag.

The console input mode is restored on normal return, I/O failure and Rust
panic unwinding. Process termination without unwinding cannot run cleanup.
Linux/macOS keypad console adapters are pending; their ordinary REPL and
one-shot calculator interfaces do not depend on the Windows module.

## Verification

```powershell
python tools/verify_and_tick_slice10.py
python tools/keypad_console_smoke.py target/debug/rpn.exe
```

The first runs formatting, check, Rust tests, strict Clippy, rebuild and both
numerical oracles before checking the five existing numeric milestone boxes.
The second creates a new hidden console, injects synthetic input only there,
and verifies the real Win32 adapter, decimal comma, repeats, releases,
arithmetic errors, undo/redo, diagnostic fields, keypad layers, Windows line
editing, history recall/search, Tab completion, exits and mode restoration
across five sessions.
It never injects input into the user's active console. This supplements the
pure state-machine and CLI tests, and does not replace physical numpad tests.

The expanded transcendental oracle makes 477 Decimal comparisons across
16/53/128/256/512-bit settings and all five output rounding modes (each mode
at 128 bits). It covers nonzero trig, negative/large angles, near-quarter-turn
input, exp range boundaries, logs and conversions. Pi uses independent
Gauss-Legendre iteration. The allowed relative error is `2^(4-bits)`;
passing sampled checks is not a correct-rounding guarantee. The separate
Fraction oracle checks 1140 binary rounding results exactly.

Win32 bindings follow Microsoft's [KEY_EVENT_RECORD], [INPUT_RECORD],
[ReadConsoleInput] and [SetConsoleMode] contracts. ABI size, alignment and
field offsets are checked in Rust tests.

[KEY_EVENT_RECORD]: https://learn.microsoft.com/en-us/windows/console/key-event-record-str
[INPUT_RECORD]: https://learn.microsoft.com/en-us/windows/console/input-record-str
[ReadConsoleInput]: https://learn.microsoft.com/en-us/windows/console/readconsoleinput
[SetConsoleMode]: https://learn.microsoft.com/en-us/windows/console/setconsolemode
