# rpning development checkpoint

The published v0.1.0 contains the original calculator core. Development main
also contains the first M7 batch and the first Windows M9 TUI. Build current
source to use them; do not replace the v0.1.0 tag.

    git pull --ff-only
    cargo run --release -- --tui

M7 implements alternate-base literals/display, bitwise arithmetic, fixed-width
integer interpretations, counted statistics, and exact binomial probability
and combinatorics. See docs/EXTENDED.md for syntax and limits.

M9 shows result and editable input above scrollable stack and calculation
history. Wide terminals show both lower panes; narrow terminals switch them.
F1 focuses input; F2 changes layer; F3 cycles focus; F4 switches the lower pane;
F5 toggles command editing. History Enter recalls without executing. Ctrl+Z/Y
undo/redo; Esc exits. See docs/TUI.md for the full controls and boundaries.

The TUI shares the engine, numeric settings, keypad mappings, macros,
completion and persistence. src/tui.rs contains the view model and layout;
src/keypad_terminal.rs supplies the Windows adapter. The session journal is
capped at 256 entries. Long values are clipped only on display and can be
scrolled. Errors preserve the committed result and editable input. Terminal
modes, cursor, attributes and main screen restore on exit/errors/unwinding.

## Verification

Windows gates passed: 132 Rust tests, formatting, strict Clippy, 1140 Fraction
rounding checks, 477 Decimal comparisons, and six isolated console sessions.
Console tests cover live frames, calculation errors, completion, recall,
undo/redo, synthetic resize events, narrow panes and terminal restoration.
Regression coverage for REPL, pipes, scripts and one-shot evaluation passed.

    python tools/verify_and_tick_slice10.py
    cargo build --offline --release --bin rpn
    python tools/keypad_console_smoke.py target/release/rpn.exe

The generic verifier checks only its original TODO tasks. Update other boxes
separately after relevant checks pass. Preserve existing task labels/order.

## Remaining work

Stop at this tested checkpoint. M2 still needs a stable numeric abstraction
and measured small-integer optimization. Eight M7 tasks remain: complex
numbers, vectors/matrices, unit conversions, physical constants, financial
math, numerical calculus, equation solving and custom math extensions.
Complete the numeric abstraction before adding new value types.

The first Windows M9 checklist is complete. Linux/macOS manual acceptance and
TUI adapters remain deferred; their ordinary CLI remains available. Original
v0.1.0 automated builds passed on all three platforms. Those builds do not
constitute hands-on acceptance of the new TUI.

Continue in small tested commits. Read current source first, update only
verified TODO entries, and keep the handoff focused on project information.
