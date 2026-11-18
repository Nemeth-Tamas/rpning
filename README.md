
# rpning

A fast, exact, programmable Reverse Polish Notation calculator written in Rust.

The goal is to combine traditional stack-based calculation with arbitrary-precision mathematics, a powerful macro language, and a lightweight terminal interface.

## Current implementation

Early functional proof of concept.

- Exact arbitrary-precision integers and rational numbers
- Exact decimal and scientific-notation input
- Addition, subtraction, multiplication and division
- Basic stack manipulation
- Atomic evaluation of input lines
- Interactive terminal
- One-shot CLI evaluation
- Standard-input support
- Unit and CLI integration tests

The first build and quality-gate results are pending verification.

## Building

Requires a recent stable Rust toolchain.

```powershell
cargo build --release
```

The resulting Windows executable is:

```text
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

## Commands

Arithmetic: `+`, `-`, `*`, `/`

Stack: `dup`, `drop`, `swap`, `clear`

Interactive: `:stack`, `:help`, `:quit`

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

```powershell
cargo fmt
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --release
```

Keep `Cargo.lock` committed for reproducible application builds.
