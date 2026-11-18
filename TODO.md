
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

- [ ] Benchmark Malachite against Rug
- [ ] Finalize arithmetic backend selection
- [ ] Define stable internal numeric abstraction
- [ ] Optimize small integer operations
- [ ] Add configurable binary floating-point precision
- [ ] Add exact-to-approximate conversion rules
- [ ] Add configurable rounding
- [x] Verify exact terminating-decimal formatting and display limits
- [ ] Add scientific number formatting
- [ ] Add independent precision reference tests
- [ ] Benchmark integer, rational and real arithmetic

## M3 — Advanced stack and mathematics

- [x] Verify over, rot, nip and tuck against unit tests
- [x] Verify atomic rollback for extended stack operations
- [x] Verify pick, roll and depth with valid and invalid indices
- [x] Verify bounded stack snapshots, undo/redo, failed-line history and redo branching
- [x] Verify named memory registers, atomic writes and register-aware undo/redo
- [ ] Add register deletion and management operations
- [ ] Add persistent register storage
- [ ] Add integer division and remainder
- [ ] Add exponentiation and roots
- [ ] Add factorial, GCD and LCM
- [ ] Add logarithmic and exponential functions
- [ ] Add trigonometric functions
- [ ] Add mathematical constants
- [ ] Add degree/radian conversion

## M4 — Macro language

- [ ] Finalize macro syntax
- [ ] Implement tokenizer and parser
- [ ] Compile macros to internal instructions
- [ ] Implement macro execution engine
- [ ] Support nested macros
- [ ] Add aliases and namespaces
- [ ] Add macro definitions and descriptions
- [ ] Persist macros between sessions
- [ ] Add macro import/export
- [ ] Add recursion and execution limits
- [ ] Add macro debugging and tracing
- [ ] Add conditionals, loops and variables

## M5 — Terminal workflow

- [ ] Add persistent command history
- [ ] Add history search
- [ ] Add tab completion
- [ ] Add configurable number display
- [ ] Add persistent application settings
- [ ] Add script-file execution
- [ ] Add helpful built-in documentation
- [ ] Add programmable command shortcuts

## M6 — Physical numpad mode

- [x] Select affordable standalone USB numpad
- [x] Order CONNECT IT NumCALC CKB-0060-BK
- [ ] Test hardware and emitted key events
- [ ] Design dedicated keypad state machine
- [ ] Implement immediate operator execution
- [ ] Define Enter and pending-number behavior
- [ ] Implement backspace and decimal entry
- [ ] Add configurable key mappings
- [ ] Add alternate keypad layers
- [ ] Add programmable macro shortcuts
- [ ] Verify normal terminal mode remains unaffected

## M7 — Extended mathematical environment

- [ ] Implement complex numbers
- [ ] Add binary, octal and hexadecimal modes
- [ ] Add bitwise arithmetic
- [ ] Add fixed-width integer interpretations
- [ ] Add statistical functions
- [ ] Add probability and combinatorics
- [ ] Add vectors and matrices
- [ ] Add unit conversions
- [ ] Add physical constants
- [ ] Add financial mathematics
- [ ] Add numerical integration and differentiation
- [ ] Add equation solving
- [ ] Add custom mathematical extensions

## M8 — Performance and releases

- [ ] Establish startup benchmarks
- [ ] Benchmark stack-operation throughput
- [ ] Optimize atomic execution overhead
- [ ] Benchmark compiled macro execution
- [ ] Profile memory usage
- [ ] Verify Windows compatibility
- [ ] Verify Linux compatibility
- [ ] Verify macOS compatibility
- [ ] Add cross-platform release builds
- [ ] Complete user documentation
- [ ] Publish initial stable release
