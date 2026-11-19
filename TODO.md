
# rpning — Development Roadmap

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

- [ ] Design a split TUI layout with results above stack and history
- [ ] Add full-screen TUI mode with safe terminal restoration
- [ ] Show the current result and editable pending input in a persistent upper pane
- [ ] Add a scrollable lower stack pane with X/Y/Z labels
- [ ] Add a scrollable calculation history pane with expressions and results
- [ ] Support side-by-side lower panes and narrow-terminal switching
- [ ] Integrate numpad input, configurable layers and macro shortcuts
- [ ] Add pane focus, history recall, completion and undo/redo controls
- [ ] Show numeric settings, active layer and errors without overwriting results
- [ ] Handle terminal resizing, long values and readable color styling
- [ ] Verify TUI behavior while preserving REPL, pipes, scripts and one-shot evaluation
