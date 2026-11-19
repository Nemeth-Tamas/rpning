
# rpning

A fast, exact, programmable Reverse Polish Notation calculator written in Rust.

The goal is to combine traditional stack-based calculation with arbitrary-precision mathematics, a powerful macro language, and a lightweight terminal interface.

## Current implementation

Early functional proof of concept.

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
- Unit and CLI integration tests

Initial arithmetic CLI smoke tests have been confirmed. Full quality-gate verification for the latest slice is pending.

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

Operators execute when the input line is submitted with Enter. Immediate execution on keypress will be implemented separately for dedicated keypad mode.

For now, only a single trailing operator can be attached to a numeric literal. General whitespace-free expressions such as `3+4*` are not yet supported.


## Commands

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

Registers are session-local and are not yet saved to disk.

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

Malachite is the provisional numerical backend.

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
only the top value even in exact mode. Settings are session-local and are not
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

## Roadmap

See `TODO.md`.

Future development includes arbitrary-precision real functions, stack registers, programmable macros, script execution, configurable key mappings, physical numpad mode, advanced mathematics and performance optimization.

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
