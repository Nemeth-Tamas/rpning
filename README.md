
# rpning

A fast, exact, programmable Reverse Polish Notation calculator written in Rust.

**[Download rpning 0.1.0](https://github.com/Nemeth-Tamas/rpning/releases/tag/v0.1.0)**
for Windows x64, Linux x64 or macOS ARM64. Prebuilt binaries need no Rust
installation. See [INSTALL.txt](INSTALL.txt) for first-run/source-build steps;
SHA256SUMS.txt accompanies the downloads. The release passed all three
platforms' build, test, Clippy and independent numerical-oracle gates.

The goal is to combine traditional stack-based calculation with arbitrary-precision mathematics, a powerful macro language, and a lightweight terminal interface.

## Current implementation

Development `main` additionally includes the first M7 batch: base-prefixed
integer input/output, bitwise and fixed-width conversions, counted exact
statistics, combinations/permutations and binomial probabilities. See
[extended mathematics](docs/EXTENDED.md). Build from source for features added
after the v0.1.0 downloadable binaries.

Initial release implementation, with the extended roadmap retained in TODO.md.

- Exact arbitrary-precision integers and rational numbers
- Exact decimal and scientific-notation input
- Exact terminating-decimal output (up to 128 fractional digits)
- Rational output for recurring or excessively long decimal expansions
- Numeric literals with attached trailing arithmetic operators
- Addition, subtraction, multiplication and division
- Basic and extended stack manipulation
- Atomic evaluation of input lines
- 128 levels of stack and register undo/redo history
- Named memory registers with exact values
- Interactive terminal
- One-shot CLI evaluation
- Standard-input support
- Opt-in Windows console keypad input and key-event diagnostics
- Compiled macros, namespaces, aliases, conditions, loops and local variables
- Macro persistence/import/export, instruction tracing and script files
- Saved settings, command history/search, Windows line editing and completion
- Configurable keypad layers, key bindings and programmable macro shortcuts
- Unit and CLI integration tests

Windows quality gates, independent numerical oracles and synthetic console
input tests pass. Physical numpad events and the first calculation have also
been verified with the CONNECT IT NumCALC CKB-0060-BK.

## Building

Requires a recent stable Rust toolchain.

For development:

```powershell
cargo build
```

For an optimized release executable when needed:

```powershell
cargo build --release
```

The Windows executables are:

```text
target/debug/rpn.exe
target/release/rpn.exe
```

## Usage

Start the interactive calculator:

```powershell
cargo run
```

Example session:

```text
> 3 4 +
7
> 10 dup *
100
> :stack
[0] 100
[1] 7
> :quit
```

One-shot evaluation:

```powershell
cargo run -- --eval "15 7 + 3 *"
```

Result:

```text
66
```

Exact fractions:

```powershell
cargo run -- --eval "1 3 / 3 *"
```

Result:

```text
1
```

Exact decimals:

```powershell
cargo run -- --eval "0.1 0.2 +"
```

Result:

```text
0.3
```

### Exact decimal formatting

Rational values with terminating decimal expansions are displayed in decimal notation automatically.

Examples:

| Value | Display |
|-------|---------|
| `3/10` | `0.3` |
| `5/2` | `2.5` |
| `1/8` | `0.125` |
| `-5/2` | `-2.5` |
| `1/3` | `1/3` |
| `1/6` | `1/6` |

The formatter never rounds or converts through floating-point arithmetic.

Terminating decimals requiring more than 128 fractional digits are displayed as exact fractions instead. Future formatting modes will allow the user to choose between decimal, fraction and scientific representations.

The underlying calculator always retains the exact rational value.


### Calculator-style Enter workflow

Interactive mode preserves the stack between input lines.

A trailing arithmetic operator can be attached directly to a valid numeric literal, without a separating space:

```text
> 5
5
> 3+
8
> 2*
16
```

This also works with exact fractions (`1/3+`), negative numbers (`-3+`), and scientific notation (`2e-3+`).

In the ordinary REPL, operators execute when the input line is submitted with
Enter. The separate `--keypad` mode executes operators immediately on keypress.

For now, only a single trailing operator can be attached to a numeric literal. General whitespace-free expressions such as `3+4*` are not yet supported.

### Physical numpad mode (Windows)

```powershell
cargo run -- --keypad-diagnose
cargo run -- --keypad
```

The first command reports focused console key events for hardware testing.
The second starts immediate keypad calculation. With NumLock on, type
`12`, Enter, `3`, `+`, `2`, `*` to get `30` without pressing Enter after an
operator. Both `.` and `,` enter a decimal point. Enter commits a pending
number; Enter with no pending number duplicates X. Backspace edits the pending
entry. Esc exits; `S` changes sign, `C` clears the stack, Tab swaps, and
Ctrl+Z/Ctrl+Y undo/redo. An enhanced navigation Delete cancels the pending entry.

This version reads the active console, has no per-device separation, and
does not use global keyboard hooks. Navigation events select configurable
shortcuts; F2 toggles an alternate digit-key layer. Bind a key with
`:bind nav home math.square` and inspect mappings with `:bindings`.
It requires interactive stdin and stdout; ordinary REPL/pipes/`--eval` retain
their existing behavior. See [the keypad guide](docs/KEYPAD.md) for exact entry,
error, undo, display and hardware-test behavior.


## Commands

## Programming

```text
: square "Square X" dup * ;
: gross "27 percent VAT" 1.27 * ;
10000 gross
```

Macros compile to bounded instructions, support nested/recursive calls,
`if ... else ... then`, counted/while loops and local variables. Definitions
participate in atomic rollback and undo/redo. Use `:macros`, `:see NAME`,
`:trace EXPRESSION`, `:alias NAME TARGET`, `:import FILE` and `:export FILE`.
Interactive sessions save/restore macros automatically; pipes can explicitly
use `:macros-save`/`:macros-load`. `rpn --run FILE` executes a whole RPN script.
See [the language guide](docs/LANGUAGE.md) for syntax, examples and limits.


### Arithmetic

| Command | Description |
|---------|-------------|
| `+` | Add Y and X |
| `-` | Subtract X from Y |
| `*` | Multiply Y and X |
| `/` | Divide Y by X exactly, preserving fractions |
| `div` | Integer quotient Y / X, truncated toward zero |
| `mod` | Integer remainder of Y divided by X |

### Stack operations

The stack is described from bottom to top.

| Command | Before | After |
|---------|--------|-------|
| `dup` | `a` | `a a` |
| `drop` | `a b` | `a` |
| `swap` | `a b` | `b a` |
| `over` | `a b` | `a b a` |
| `rot` | `a b c` | `b c a` |
| `nip` | `a b` | `b` |
| `tuck` | `a b` | `b a b` |
| `pick` | `a b c 1` | `a b c b` |
| `roll` | `a b c 2` | `b c a` |
| `depth` | `a b c` | `a b c 3` |
| `clear` | Any stack | Empty stack |


For `rot`, only the top three elements are rotated. Elements farther down the stack are unchanged.

For `pick` and `roll`, the top stack element supplies the zero-based index. Index 0 selects the item immediately below the index itself.

- `pick` copies the selected item to the top.
- `roll` moves the selected item to the top.
- `depth` pushes the stack size before the operation.

Indices must be nonnegative integers that fit the platform's `usize` type and refer to an existing stack item. Invalid indices and out-of-range accesses produce errors without changing the original stack.

Each operation checks that enough operands are available.


A failed input line leaves the original stack unchanged.

### Named memory registers

The calculator supports case-sensitive named registers for storing exact numeric values.

- `sto:name` stores the top stack value in the named register without consuming it.
- `rcl:name` pushes a copy of the register's value onto the stack.
- `del:name` deletes an existing named register without changing the stack.
- `clearmem` deletes all named registers without changing the stack.
- `:registers` lists all currently defined registers alphabetically.
- `clear` clears the stack but preserves registers.
- Undefined registers and invalid names produce explicit errors.
- Deletion and clearing are atomic and can be undone or redone.

Register identifiers may contain 1-64 ASCII letters, digits and underscores. They must begin with a letter or underscore.

Example:

```text
> 10000 sto:net
10000
> clear
(empty)
> rcl:net 1.27 *
12700
> :registers
net = 10000
```

Register changes participate in atomic input-line evaluation and undo/redo. A failed line cannot partially modify registers.

Registers are session-local unless explicitly saved with `:save` and restored
with `:load`; see the storage paths below.

### Undo and redo

Interactive mode maintains up to 128 previous calculator states, including both stack values and memory registers.

```text
> 5
5
> 3+
8
> 2*
16
> :undo
8
> :undo
5
> :redo
8
```

Each successfully completed input line that changes the stack or registers creates one checkpoint.

- `:undo` restores the previous stack and register state.
- `:redo` restores a previously undone state.
- Failed operations do not create checkpoints.
- Calculations that leave the stack unchanged do not create checkpoints.
- A new stack-changing calculation after undo discards redo history.
- `clear` can be undone.
- History is stored in memory for the current session only.

History stores full stack snapshots. A future optimization may reduce memory usage for very large stacks.

### Interactive commands

| Command | Description |
|---------|-------------|
| `:stack` | Display stack, top first |
| `:registers` | List stored register values |
| `:undo` | Undo the last state-changing input line |
| `:redo` | Reapply the last undone input line |
| `:help` | Show help |
| `:quit` | Exit |
| `:exit` | Exit |

## Architecture

- `src/main.rs` - terminal interface and CLI
- `src/lib.rs` - public library interface
- `src/engine.rs` - calculator state, stack operations and evaluation
- `src/number.rs` - numerical backend and parsing
- `tests/cli.rs` - CLI integration tests

Malachite is the selected runtime numerical backend. A stable backend-independent
number abstraction remains on the roadmap.

## Exact advanced mathematics and numeric controls

Exact operations: `pow` (integer exponent), `sqrt` (exact square root),
`root` (integer degree), `fact`, `gcd` and `lcm`. Irrational roots are
explicitly rejected rather than silently approximated. Exponent, factorial,
and root-degree limits protect interactive responsiveness.

The calculator starts in exact mode. To intentionally perform finite-precision
binary arithmetic, use `:mode binary`, `:precision 53` and
`:round nearest|zero|up|down|away`. This mode rounds the operands and result
of the four basic arithmetic operators to a dyadic rational with the specified
significant-bit precision. It has an unbounded exponent range and is **not**
IEEE 754 floating point (no infinities, NaNs or signed zeros). `approx` rounds
only the top value even in exact mode. Interactive settings are saved and are not
part of undo/redo history. `:display scientific` selects lossless
scientific notation for terminating decimal values; repeating values retain
exact fractional output. `:settings` shows the active controls.

Named registers persist only on explicit `:save` and `:load`. Default path:
Windows `%APPDATA%/rpning/registers.txt`; Linux `$XDG_DATA_HOME/rpning/`
`registers.txt` or `~/.local/share/rpning/registers.txt`. Loading is undoable.
A corrupted register file is rejected without altering memory.

Backend selection and benchmark methodology: `docs/M2-BENCHMARK.md`.

## Approximate real functions (M3)

The new `pi`, `e`, `exp`, `ln`, `log2`, `log10`, `sin`, `cos`, `tan`, `rad`,
`deg` words are computed from exact rational series and then rounded to the
configured binary precision. **They are approximate even in `:mode exact`.**
The ordinary rational arithmetic operators remain exact in exact mode.

The implementation avoids f32/f64 conversion and uses 40 guard bits, but
is **not certified correctly rounded for every possible input**. It limits
transcendental precision to 512 bits for responsiveness. Basic binary
arithmetic still supports 4096 bits. Input-domain/range errors roll back
atomically. `sin`, `cos` and `tan` accept radians; `rad` converts degrees to
radians and `deg` converts radians to degrees.

See `docs/M2-M3-REAL-MATH.md` for accuracy, limits, examples and oracle tests.

## Saved workflow

Windows REPL supports Up/Down history, Ctrl+R reverse search and Tab completion.
`:history [TEXT]` and `:complete PREFIX` work in all terminals. Settings and
macros persist automatically in interactive sessions; history saves on exit.
`:settings-save`/`:settings-load` explicitly persist numeric settings, bindings
and command shortcuts. Pipes, scripts and `--eval` begin with fresh defaults.
Use `:display auto|scientific|fraction|fixed:N` to choose presentation; `~`
marks rounded fixed output without changing the exact number. See the full
[workflow guide](docs/WORKFLOW.md) for files, keys, layers and examples.

## Roadmap

See `TODO.md`.

Future work includes the remaining numeric abstraction/optimization tasks and
the M7 extended mathematical environment: complex numbers, alternate bases,
statistics, vectors, units, finance and numerical methods. Existing roadmap
entries remain unchanged in TODO.md.

## Development

Routine development gates:

```powershell
cargo fmt
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Release builds are reserved for milestones, benchmarking and release preparation:

```powershell
cargo build --release
```

Keep `Cargo.lock` committed for reproducible application builds.
