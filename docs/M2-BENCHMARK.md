# M2 numerical backend — benchmark and decision

Source: user-run `comparison.csv` and `comparison-report.md`, 2026-11-18, WSL2 Linux x86_64. The benchmark ran 400 iterations × 7 repeats per case; reported timings are median nanoseconds per arithmetic operation **including two operand clones**, excluding parsing/input generation/formatting.

## Results

27 size/operation comparisons were available. Malachite won **17**, Rug **10**. By category:

| Category | Malachite wins | Rug wins | Noteworthy result |
|---|---:|---:|---|
| Integer | 8 | 4 | At 20 decimal digits, Malachite was 1.80× faster for multiplication and 1.91× for division. At 310 digits, Rug was 1.41× faster at multiplication. |
| Rational | 6 | 3 | Malachite won the smaller cases; Rug won all three 40-digits-per-part cases by ~1.56–1.59×; the largest cases were close. |
| Binary float | 3 | 3 | Malachite led 53-bit addition (2.10×), while Rug led 256-bit multiplication (1.53×) and 1024-bit multiplication (1.45×). |

The geometric mean of per-case Malachite/Rug execution-time ratios was approximately **0.91** (a modest average Malachite advantage across this workload). This is *not* an end-to-end speedup, and the choice of cases/weights matters.

## Decision: Keep Malachite as the runtime backend

- Existing exact integer and rational operations already use Malachite.
- The Windows-first `rpning` executable remains pure Rust and does not require shipping GMP/MPFR.
- The measured small-number results favor Malachite, an important interactive-calculator workload.
- Rug remains a **benchmark/reference dependency only**, in the separately compiled `benchmarks` crate. Its advantage in some larger operations is worth revisiting after profiling real-world use, not enough to justify a backend migration now.
- Binary approximation is initially implemented by rounding exact rational arithmetic to a configurable number of binary significant bits. This is a **dyadic rational approximation**, not full IEEE 754 or Malachite `Float` semantics. The separate benchmark's float cases do not validate this calculator mode.

## Limitations and follow-ups

1. One WSL2 host and one benchmark configuration; the samples may include scheduling and power-state noise.
2. The benchmark measures isolated operations and clones, not `Calculator::evaluate`, allocation behavior, undo/redo snapshots, CLI parsing or formatting.
3. Integer and rational cross-checks were enabled. Floating cases were performance-only and do **not** prove rounding accuracy.
4. A future M2/M8 run should cover startup, end-to-end stack throughput, parsing/formatting, larger operand distributions and correctness against independent precision references.
5. Binary approximation uses unlimited exponent range, so NaN, infinity, signed zero, exact MPFR transcendental rounding and special IEEE behavior remain outside the current implementation.

**TODO policy:** The two existing benchmark checkboxes and the backend-selection checkbox may be checked. Do not check M2 or M3 features that are not implemented and verified merely to declare a whole milestone complete.
