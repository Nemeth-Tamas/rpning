# Keypad mode: first usable Windows slice

The CONNECT IT NumCALC CKB-0060-BK's actual emitted events were checked on
2026-11-19, with both numeric and navigation modes. The calculation
`12 Enter 3 + 2 *` produced X=30. This implementation uses console-translated
events, without assuming a particular USB identity.

Observed numeric-mode keys are ordinary character/number-row events. Multiply
and add include synthetic Shift events. Decimal emits a period. The two
bottom-row zero keys emitted three zero presses total, consistent with 0 and
00. In navigation mode, digits become Home/Up/PageUp/Left/Right/End/Down/
PageDown/Insert; 5 emitted no event in the capture. Decimal emits enhanced
Delete and cancels pending entry. Every event reported NumLock=false, including
numeric mode: mappings must use emitted events rather than that flag. The
capture does not establish whether the device or host performs translation.

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
Normal `cargo run` still starts the full command-based REPL.

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
