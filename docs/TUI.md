# M9 - Calculator terminal interface

The first Windows implementation is available on development main:

```powershell
git pull --ff-only
cargo run --release -- --tui
```

The v0.1.0 download predates this feature; build from current source. It adds
no runtime dependencies. Normal REPL, pipes, scripts and --eval remain available.

## Controls

| Key | Action |
| --- | --- |
| Digits / decimal / operators | Existing immediate keypad entry while input is focused |
| F1 | Focus input |
| F2 | Toggle base/alt keypad layer |
| F3 | Cycle Input, Stack, History, Result focus |
| F4 | Toggle the lower pane/focus (both remain visible on wide terminals) |
| F5 | Toggle numeric entry and command editor; focus input |
| Arrows / PageUp / PageDown | Browse the focused lower pane or scroll a long result |
| Home / End | First/last item in a browsing pane; edit positions in command mode |
| Enter in history | Recall selected expression into command input without executing it |
| Enter in command input | Execute RPN, macro definitions or application commands |
| Tab in command input | Complete a word, macro, register or command |
| Ctrl+Z / Ctrl+Y | Undo / redo committed calculator operations |
| Esc / Ctrl+C | Exit and restore the original terminal screen/modes/cursor/colors |

Commands include existing numeric controls, macros, bindings, registers and
file persistence. For example, F5 then `:display hex` changes integer output;
`: square dup * ;` defines a macro. Command editing supports cursor movement,
insertion/deletion, UTF-16 input and a bounded 4096-character line. RPN command
expressions include a pending numeric entry; application commands leave it
pending. Recalling a journal expression replaces pending input explicitly.

The calculation journal keeps 256 session entries, including failed operations;
it is distinct from saved command history and the engine's undo snapshots.
Browsing never executes an operation. Committed result X stays visible on
error, and the editable pending number is preserved. Numeric mode describes
ordinary arithmetic; transcendental words remain approximate as documented.

Below 90 columns the lower panes switch. Below 32 columns or 13 rows, a small
recovery view asks for a larger window while preserving calculator state.
Result focus supports horizontal scrolling with explicit clipping markers;
stored values are never truncated. Alignment assumes single-column glyphs.

The requested interface is a full-screen terminal calculator with a prominent
result at the top and calculation history/stack below. It redraws its panes
in place rather than printing a fresh status line on every keypress.

## Layout

| Region | Content |
| --- | --- |
| Upper result pane | Current result X, clearly distinguished from pending entry; exact/approximate presentation follows existing display settings |
| Entry line below result | Editable pending number or command; operators still execute immediately in keypad mode |
| Lower left pane | Live stack, X first with X/Y/Z labels and indexed deeper values; scrolling for long stacks |
| Lower right pane | Scrollable calculation history showing the entered expression/action and its result or error |
| Footer/status | Numeric mode, precision, rounding, active keypad layer, focused pane and concise key hints |

On a wide terminal the lower panes appear side by side. On a narrow terminal
the lower region switches between stack and history so both remain readable.
The upper result/entry stay visible while either lower pane scrolls. Long
exact values need an explicit expansion/horizontal-scroll interaction; visual
clipping must never change the stored value or silently imply rounding.

Errors occupy a dedicated message area without replacing the last successful
result. Colors and borders should support quick scanning with a restrained
default palette. Meaning must remain clear without color alone.

## Behavior

- Reuse the existing calculator engine, transaction boundaries and undo/redo.
- Reuse pending-entry rules, immediate operators, configured layers and macros.
- Keep pane navigation separate from calculator input so browsing history
  cannot accidentally execute a keypad operation or modify the stack.
- Recall an expression into the input editor explicitly; browsing alone does
  not reevaluate it. Distinguish command history from numeric undo snapshots.
- Preserve normal REPL, pipes, scripts and one-shot --eval. Introduce the TUI
  as an explicit mode before deciding whether it should become the default.
- Restore terminal modes/cursor/screen on normal exit, errors and unwinding.
  Handle resizing and small windows without panics or corrupting input.

## Implementation boundaries

Separate a testable view model/layout from rendering and keyboard input. The
renderer reads calculator state and a calculation journal; it does not own a
second arithmetic implementation. Journal entries must reflect committed
operations and failures accurately, including macro and undo/redo actions.
Reuse the observed numpad events rather than relying on its NumLock flag or
assuming per-device identification.

Windows uses the existing focused Win32 input adapter with VT alternate-screen
output, saved terminal state and bounded viewport rendering. Active screen size
is polled and resize records are handled. Linux/macOS TUI adapters and manual
platform acceptance remain deferred; their ordinary CLI remains available.

Verification covers 132 Rust tests, including responsive layouts, long-value
scrolling, command/macro integration, journal bounds and atomic error behavior.
Six isolated Windows-console sessions cover actual TUI frame contents,
calculation/errors, completion, recall, undo/redo, synthetic resize events,
narrow-pane switching and restoration of the original screen/modes/cursor.
The tests inject input only into a new hidden console, never the user's active
terminal. Existing M0-M8 task names/order remain unchanged.
