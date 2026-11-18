
# rpning

A fast, exact, programmable Reverse Polish Notation calculator written in Rust.

The goal is to combine traditional stack-based calculation with arbitrary-precision mathematics, a powerful macro language, and a lightweight terminal interface.

## Current implementation

Early functional proof of concept.

- Exact arbitrary-precision integers and rational numbers
- Exact decimal and scientific-notation input
- Numeric literals with attached trailing arithmetic operators
- Addition, subtraction, multiplication and division
- Basic and extended stack manipulation
- Atomic evaluation of input lines
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
3/10
```

Exact fractional output is used for now. Decimal formatting will be implemented separately.

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
| `/` | Divide Y by X |

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

### Interactive commands

| Command | Description |
|---------|-------------|
| `:stack` | Display stack, top first |
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
