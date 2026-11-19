# M9 - Calculator terminal interface

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

Windows is the first acceptance target. Choose the terminal backend after
checking compatibility, cleanup behavior, binary size and startup impact;
this planning milestone does not select a new dependency. Linux/macOS manual
acceptance remains deferred as requested. Before checking any M9 item, verify
layout/navigation and real-console behavior, along with all existing CLI and
numeric regressions.

M9 is planned, not implemented by this roadmap addition. Existing M0-M8 task
names/order/status are preserved; completion is tracked in TODO.md.
