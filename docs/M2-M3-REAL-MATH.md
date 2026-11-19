# M2/M3 numerical completion work — real functions

## Status and scope

This slice introduces approximate transcendental functions alongside the exact rational calculator. It does **not** change exactness of existing `+`, `-`, `*`, `/`, `pow`, `sqrt`, `root`, `gcd`, `lcm`, `fact` or register values except where their existing binary mode explicitly rounds.

New postfix words: `pi`, `e`, `exp`, `ln`, `log2`, `log10`, `sin`, `cos`, `tan`, `rad`, `deg`.

The new functions compute with Malachite `Rational` series with **40 guard bits** and return a finite-precision dyadic rational using the current `:precision` and `:round` settings. This is *not* a proof of correct rounding for all inputs; it is **not** an IEEE float. `pi` and `e` are similarly approximated.

Safety limits (explicit errors, atomic rollback): trans functions allow 2–512 bits; `exp` inputs in [-256,+256]; trig arguments in [-1000,+1000] radians; logarithm normalized binary exponent magnitude <=100000. The existing basic binary rational rounding remains available at 2–4096 bits.

## Accuracy

- π uses Machin's identity and alternating arctan series.
- e and exp use a rational Taylor series with argument reduction.
- ln uses base-two normalization and the convergent atanh series.
- trig uses a rational Taylor series after angle reduction by a guarded π.
- degree/radian conversions use the same guarded π.
- Outputs are rounded to configured significant-bit precision; finite-precision inputs can therefore accumulate errors.
- 512-bit limit is for responsiveness; engineering improvements can raise it in future without hiding computational cost.

## Verification

```powershell
cargo fmt
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --bin rpn
python tools/precision_oracle.py target/debug/rpn.exe
python tools/transcendental_oracle.py target/debug/rpn.exe
```

The Python oracle uses independent `decimal.Decimal` and a 60-digit π reference for sample points; it is *not* an exhaustive numerical proof.

## Manual examples

```text
> :precision 80
precision: 80 bits
> pi
...approximation of π...
> clear 180 rad
...approximation of π...
> clear 2 ln
...approximation of ln(2)...
> clear 1 exp
...approximation of e...
> clear 0 sin
0
```

## Roadmap bookkeeping

The installation tool marks **only existing TODO checkbox names**, including prior implemented numeric and exact math features. It intentionally keeps unimplemented small-integer optimization pending and leaves this new real-math slice unchecked until quality gates and its oracle pass. After testing, tick the four existing M3 items (logs/exp, trig, constants, degree/radian conversion) without renaming them.
