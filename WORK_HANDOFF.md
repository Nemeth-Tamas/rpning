# rpning development checkpoint

## Release 0.2.0

The package/lockfile and user-facing download instructions now target v0.2.0.
This release packages the implemented M7 batch, default Windows TUI, setting
discovery, scrollable output and complete embedded TUI/program/layer guides.
The release: commit triggers the existing three-platform verification/build
pipeline, then creates a new v0.2.0 release with three binaries and checksums.
Verify the run and uploaded assets before claiming publication. Preserve v0.1.0.
The pre-release feature commit 3f871ef passed Windows/Linux/macOS CI in run
44526691980. The remaining M2/M7 tasks and manual Linux/macOS checks stay open.

## Latest TUI follow-up

Windows terminal startup now defaults to the TUI; --repl explicitly selects
the ordinary editor. Pipes/redirected output use plain line input, and
Linux/macOS retain their ordinary REPL. The TUI adds setting hints, contextual
argument completion and read-only bare :display/:mode/:round/:precision queries.
F6 focuses a wrapped, scrollable Output pane for complete query/list/help output.
F3 now cycles Input, Stack, History, Result, Output; F4 returns to stack/history.
TUI :registers is available alongside macro inspection and history search.

docs/TUI.md is the full interface/layer guide. Its attached docs/PROGRAM_GUIDE.md
contains the complete segmented program reference, including advanced macro
programming, all math, registers, scripts, command index, persistence and limits.
Both guides are embedded via :help tui / :help program; topic help includes
layers, settings, math, stack, registers, macros, scripts and persistence.
No external documentation files are needed beside the built executable.

The M9 follow-up requirements are implemented and checked after final gates:
137 Rust tests, strict Clippy, 1140 Fraction rounding checks, 477 Decimal
comparisons, seven isolated Windows console sessions, and guide examples/links.
The prior implementation details below
provide historical context; the latest behavior above takes precedence.

The original v0.1.0 contains the initial calculator core. Version v0.2.0 packages
the extended features above; do not replace the original v0.1.0 tag.

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

M2 still needs a stable numeric abstraction
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
