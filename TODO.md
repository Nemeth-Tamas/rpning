
# rpning — Development Roadmap

Hotfix checkpoint: v0.2.1 corrects physical Alt/custom numpad routing with
NumLock OFF, with measured-event and live-console regressions. Existing task
labels and unchecked M2/M7/manual-platform work remain unchanged.

Release checkpoint: v0.2.0 packages the implemented M7 batch and Windows M9
TUI, setting discovery and full embedded guides. Its release pipeline builds
Windows/Linux/macOS binaries and checksums after all gates pass. The original
v0.1.0 release and all remaining unchecked tasks are preserved.

Keypad follow-up: the complete guided report confirmed that NumLock-off Home/
Right events overrode Alt-7/6. Alt/custom routing now follows measured physical
numpad positions in both states; Base/Nav, dedicated navigation, CMD editing
and pane browsing retain their behavior. Raw diagnostics and saved bindings
are preserved. Regression checks replay the measured events.

## M0 — Project foundation

- [x] Define project scope and architecture
- [x] Select Windows-first terminal application design
- [x] Define GitHub and development workflow
- [x] Verify Cargo package and modular source layout
- [x] Verify initial README and build instructions
- [x] Pass initial formatting, testing and Clippy gates
- [x] Verify strict Clippy warnings are resolved


## M1 — Functional RPN engine

- [x] Verify exact integer and rational value support
- [x] Verify numeric literal parsing
- [x] Verify addition, subtraction, multiplication and division
- [x] Verify dup, drop, swap and clear
- [x] Verify correct RPN operand order
- [x] Verify atomic input-line evaluation
- [x] Verify stack underflow and division-by-zero handling
- [x] Verify minimal interactive REPL
- [x] Verify one-shot CLI evaluation
- [x] Verify stdin and pipe handling
- [x] Pass unit and integration tests
- [x] Verify attached numeric operators and Enter-separated calculations



## M2 — Arbitrary-precision foundation

- [x] Benchmark Malachite against Rug
- [x] Finalize arithmetic backend selection
- [ ] Define stable internal numeric abstraction
- [ ] Optimize small integer operations
- [x] Add configurable binary floating-point precision
- [x] Add exact-to-approximate conversion rules
- [x] Add configurable rounding
- [x] Verify exact terminating-decimal formatting and display limits
- [x] Add scientific number formatting
- [x] Add independent precision reference tests
- [x] Benchmark integer, rational and real arithmetic

## M3 — Advanced stack and mathematics

- [x] Verify over, rot, nip and tuck against unit tests
- [x] Verify atomic rollback for extended stack operations
- [x] Verify pick, roll and depth with valid and invalid indices
- [x] Verify bounded stack snapshots, undo/redo, failed-line history and redo branching
- [x] Verify named memory registers, atomic writes and register-aware undo/redo
- [x] Verify register deletion, clearing, atomic rollback and undo/redo
- [x] Add persistent register storage
- [x] Add integer division and remainder
- [x] Add exponentiation and roots
- [x] Add factorial, GCD and LCM
- [x] Add logarithmic and exponential functions
- [x] Add trigonometric functions
- [x] Add mathematical constants
- [x] Add degree/radian conversion

## M4 — Macro language

- [x] Finalize macro syntax
- [x] Implement tokenizer and parser
- [x] Compile macros to internal instructions
- [x] Implement macro execution engine
- [x] Support nested macros
- [x] Add aliases and namespaces
- [x] Add macro definitions and descriptions
- [x] Persist macros between sessions
- [x] Add macro import/export
- [x] Add recursion and execution limits
- [x] Add macro debugging and tracing
- [x] Add conditionals, loops and variables

## M5 — Terminal workflow

- [x] Add persistent command history
- [x] Add history search
- [x] Add tab completion
- [x] Add configurable number display
- [x] Add persistent application settings
- [x] Add script-file execution
- [x] Add helpful built-in documentation
- [x] Add programmable command shortcuts

## M6 — Physical numpad mode

- [x] Select affordable standalone USB numpad
- [x] Order CONNECT IT NumCALC CKB-0060-BK
- [x] Test hardware and emitted key events
- [x] Design dedicated keypad state machine
- [x] Implement immediate operator execution
- [x] Define Enter and pending-number behavior
- [x] Implement backspace and decimal entry
- [x] Add configurable key mappings
- [x] Add alternate keypad layers
- [x] Add programmable macro shortcuts
- [x] Verify normal terminal mode remains unaffected

## M7 — Extended mathematical environment

- [ ] Implement complex numbers
- [x] Add binary, octal and hexadecimal modes
- [x] Add bitwise arithmetic
- [x] Add fixed-width integer interpretations
- [x] Add statistical functions
- [x] Add probability and combinatorics
- [ ] Add vectors and matrices
- [ ] Add unit conversions
- [ ] Add physical constants
- [ ] Add financial mathematics
- [ ] Add numerical integration and differentiation
- [ ] Add equation solving
- [ ] Add custom mathematical extensions

## M8 — Performance and releases

- [x] Establish startup benchmarks
- [x] Benchmark stack-operation throughput
- [x] Optimize atomic execution overhead
- [x] Benchmark compiled macro execution
- [x] Profile memory usage
- [x] Verify Windows compatibility
- [ ] Verify Linux compatibility
- [ ] Verify macOS compatibility
- [x] Add cross-platform release builds
- [x] Complete user documentation
- [x] Publish initial stable release

## M9 — Calculator terminal interface

- [x] Design a split TUI layout with results above stack and history
- [x] Add full-screen TUI mode with safe terminal restoration
- [x] Show the current result and editable pending input in a persistent upper pane
- [x] Add a scrollable lower stack pane with X/Y/Z labels
- [x] Add a scrollable calculation history pane with expressions and results
- [x] Support side-by-side lower panes and narrow-terminal switching
- [x] Integrate numpad input, configurable layers and macro shortcuts
- [x] Add pane focus, history recall, completion and undo/redo controls
- [x] Show numeric settings, active layer and errors without overwriting results
- [x] Handle terminal resizing, long values and readable color styling
- [x] Verify TUI behavior while preserving REPL, pipes, scripts and one-shot evaluation

### M9 follow-up requirements

- [x] Make TUI setting commands discoverable: for example, :display should offer argument completion with all valid options below the input, or Enter without arguments should show the current value and all allowed values without changing the setting. Apply this behavior consistently to similar setting commands.
- [x] Expand docs/TUI.md into a comprehensive user guide covering everything available inside the TUI: controls, numeric and command entry, settings and valid options, mathematics, stack/history, registers, macros, shortcuts/layers, persistence, errors and worked examples.
- [x] Make the TUI the default interactive startup mode for now; preserve explicit access to the ordinary REPL and existing pipe/script/one-shot behavior.

These follow-ups are implemented and verified; use the guides for current behavior.

- [x] Add a clear keypad layer guide to the full TUI user guide: explain base versus Alt, list every default key/action mapping for both layers (especially Alt), show how to switch layers and recognize the active layer, and explain how to inspect/customize mappings and macro shortcuts.
- [x] Attach a complete segmented program guide to the TUI guide, including all implemented commands, mathematics, settings, registers, macros, advanced programming, scripts, persistence and limits.
