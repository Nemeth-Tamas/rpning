# rpning numerical backend benchmark

This is a **standalone Cargo crate** in `benchmarks/`. It deliberately does not change
rpning's normal runtime dependencies or its build times.

It benchmarks Malachite 0.14.0 and (optionally) Rug 1.30.0 with an identical set of
deterministically generated operands. Head-to-head comparison is best performed in
WSL2/Ubuntu, Linux, or another suitable GNU build environment.

## Files and repository placement

| Downloaded file | Put in the rpning repository at |
|---|---|
| `Cargo.toml` | `benchmarks/Cargo.toml` |
| `main.rs` | `benchmarks/src/main.rs` |
| `cases.rs` | `benchmarks/src/cases.rs` |
| `data.rs` | `benchmarks/src/data.rs` |
| `timing.rs` | `benchmarks/src/timing.rs` |
| `summarize.py` | `benchmarks/summarize.py` |
| `README.md` | `benchmarks/README.md` |

These files do **not** replace your repository's top-level Cargo.toml or README.md.

## 1. Native Windows: Malachite baseline only

Run from the repository root in PowerShell:

```powershell
cargo fmt --manifest-path benchmarks/Cargo.toml
cargo test --manifest-path benchmarks/Cargo.toml
cargo clippy --manifest-path benchmarks/Cargo.toml --all-targets -- -D warnings
cargo run --release --manifest-path benchmarks/Cargo.toml -- --iterations 400 --repeats 5 > malachite.csv
python benchmarks/summarize.py malachite.csv --output malachite-report.md
```

The `--release` flag is **only** for benchmarking. Optimized code is essential for
meaningful performance measurements; it is not part of the everyday rpning gates.

## 2. WSL2/Ubuntu: Malachite vs Rug (GMP/MPFR)

Use a normal Ubuntu/WSL terminal, with Rust/Cargo installed. Before the first build:

```bash
sudo apt update
sudo apt install -y build-essential m4 pkg-config
```

From the rpning repository root in the WSL/Linux shell:

```bash
cargo test --manifest-path benchmarks/Cargo.toml --features compare-rug
cargo clippy --manifest-path benchmarks/Cargo.toml --all-targets --all-features -- -D warnings
cargo run --release --manifest-path benchmarks/Cargo.toml --features compare-rug -- --validate-only
cargo run --release --manifest-path benchmarks/Cargo.toml --features compare-rug -- --iterations 400 --repeats 7 > comparison.csv
python3 benchmarks/summarize.py comparison.csv --output comparison-report.md
```

Rug uses GMP/MPFR native libraries, and its first GNU build may be substantially slower
than routine Rust builds. On Windows, building it with the MSVC target is not supported
by the upstream bundled GMP build system. A Rust GNU target and MSYS2 also work, but
WSL2 is usually simpler. Avoid paths containing spaces for Rug/GMP builds.

## What gets measured

- **Integers**: addition, multiplication, division and remainder with 20-, 78-,
  and 310-digit first operands; second operands are roughly half as long.
- **Exact rationals**: addition, multiplication and division with 10-, 40-,
  and 155-digit numerator/denominator parts.
- **Binary floating point**: addition and multiplication at 53, 256 and 1024
  bits of precision, with the same rational inputs rounded into each backend.

For every case:

1. Generate eight deterministic operand pairs for integer/rational cases and four
   common exact rational input pairs for floating-point cases.
2. Parse/prepare both implementations **outside** the measured region.
3. With `compare-rug`, check integer and rational operation outputs against each
   other **before** timing (differences cause a hard failure).
4. Warm up, then time each operation with two operand clones and one new result.
5. Use `std::hint::black_box` to prevent trivial compiler elimination.
6. Print the median, minimum and maximum nanoseconds per operation in CSV form.

The CSV is written to stdout so you can redirect it to a file. Progress and
configuration messages go to stderr.

**Important:** Integer/rational arithmetic cross-checks are exact. Floating-point
runs are **performance-only**; a separate precision reference suite is necessary
to validate rounding behavior and ULP differences. This tool does not measure
parsing, output formatting, RPN tokenization, startup time or end-to-end macros.

## Choosing a backend

Do not decide based on one microbenchmark run. Repeat the head-to-head run at least
three times with a plugged-in, idle machine, then compare medians by workload.

The current application already uses Malachite for exact rationals and is Windows
first. Rug may be faster for some arithmetic but incurs native GMP/MPFR tooling,
distribution and maintenance tradeoffs. Keep the app on Malachite until you have
actual measurements and a good reason to migrate.

## References

- Malachite performance: https://www.malachite.rs/performance/
- Rug documentation: https://docs.rs/rug/1.30.0/rug/
- GMP/MPFR Windows build notes: https://docs.rs/gmp-mpfr-sys/latest/gmp_mpfr_sys/#building-on-windows
