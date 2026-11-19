# M7 - First extended mathematics batch

This batch adds exact integer/base tools, counted statistics, and discrete
probability/combinatorics. Complex values, vectors/matrices, units/constants,
finance, integration/differentiation, solvers and extensions remain open.

## Bases and bits

Use signed prefixed integer literals: `0b101`, `0o17`, `0xff`, `-0x80`.
`:display binary|octal|hex` selects integer output, including a base prefix.
Unprefixed input remains decimal. Fractions retain lossless ordinary output.
The existing `:mode binary` still means significant-bit approximation;
`:display binary` only changes integer presentation.

`band`, `bor`, `bxor` consume Y/X; `bnot` consumes X. Signed bitwise arithmetic
uses unbounded two's-complement semantics. `shl`/`shr` consume VALUE COUNT;
right shift of negative values is arithmetic. Counts must be 0..100000.

`u8 i8 u16 i16 u32 i32 u64 i64 u128 i128` wrap and interpret the low bits,
without changing the calculator's general arithmetic type. Examples:
`255 i8` is -1; `-1 u8` is 255. `VALUE BITS asuint|asint` supports widths
1..4096. These are explicit conversions, not a persistent machine-word mode.

## Counted statistics

Push values, then their count N, then the statistic:

```text
1 2 3 3 mean
4 1 3 2 4 median
1 2 3 3 svar
```

Results are 2, 2.5, and 1 respectively. Items below the dataset are preserved.
Available words: sum, mean, median, min, max, var (population variance), svar
(sample variance), stdev and sstdev. Dataset limit is 10000. Empty sum is zero;
other empty statistics error. Sample statistics require two values. Standard
deviation follows the existing exact-root policy: irrational results are
explicitly rejected. Mean/variance never convert through floating point.

## Discrete probability and combinatorics

`N K choose` and `N K perm` require 0 <= K <= N <= 10000.
`N K P binomp` gives the exact binomial probability
`choose(N,K) * P^K * (1-P)^(N-K)`, with 0 <= P <= 1.
For example `2 1 1/2 binomp` is exactly 1/2. Endpoint probabilities and N=0
are supported. Large exact results remain subject to the usual arithmetic
resource costs; no correctly-rounded floating-point approximation is implied.

All failures preserve the complete input transaction, named registers and
redo history. The words are available inside compiled macros and scripts.
