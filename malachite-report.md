# rpning numerical-backend benchmark results

- Generated (UTC): 2026-11-18T21:07:42+00:00
- Platform: Windows-11-10.0.26300-SP0
- Python runtime: 3.14.4 (report generator only)
- Source CSV: `malachite.csv`
- Iterations per repeat: 400
- Repeats: 5

All timings are median nanoseconds per arithmetic operation, including two operand clones.
Inputs are generated and parsed before timing. Smaller is faster.

| Family | Operation | Size | Malachite (ns) | Rug (ns) | Faster backend / ratio |
|---|---|---:|---:|---:|---|
| float | add | 53 precision_bits | 15.25 | — | Not compared |
| float | add | 256 precision_bits | 62.75 | — | Not compared |
| float | add | 1024 precision_bits | 76.50 | — | Not compared |
| float | mul | 53 precision_bits | 18.00 | — | Not compared |
| float | mul | 256 precision_bits | 131.75 | — | Not compared |
| float | mul | 1024 precision_bits | 377.25 | — | Not compared |
| integer | add | 20 decimal_digits | 30.75 | — | Not compared |
| integer | add | 78 decimal_digits | 56.00 | — | Not compared |
| integer | add | 310 decimal_digits | 98.00 | — | Not compared |
| integer | div | 20 decimal_digits | 32.25 | — | Not compared |
| integer | div | 78 decimal_digits | 101.50 | — | Not compared |
| integer | div | 310 decimal_digits | 217.75 | — | Not compared |
| integer | mul | 20 decimal_digits | 31.25 | — | Not compared |
| integer | mul | 78 decimal_digits | 90.75 | — | Not compared |
| integer | mul | 310 decimal_digits | 213.25 | — | Not compared |
| integer | rem | 20 decimal_digits | 38.50 | — | Not compared |
| integer | rem | 78 decimal_digits | 109.00 | — | Not compared |
| integer | rem | 310 decimal_digits | 210.25 | — | Not compared |
| rational | add | 10 decimal_digits_per_part | 88.50 | — | Not compared |
| rational | add | 40 decimal_digits_per_part | 1,030.50 | — | Not compared |
| rational | add | 155 decimal_digits_per_part | 1,307.75 | — | Not compared |
| rational | div | 10 decimal_digits_per_part | 112.75 | — | Not compared |
| rational | div | 40 decimal_digits_per_part | 743.75 | — | Not compared |
| rational | div | 155 decimal_digits_per_part | 2,034.75 | — | Not compared |
| rational | mul | 10 decimal_digits_per_part | 103.00 | — | Not compared |
| rational | mul | 40 decimal_digits_per_part | 752.25 | — | Not compared |
| rational | mul | 155 decimal_digits_per_part | 1,998.50 | — | Not compared |

Compared operation/size pairs: **0**

## Interpretation and limitations

- Results apply to this host, toolchain, workload, and numerical-library versions.
- This is an operation-level microbenchmark, not end-to-end rpning calculator throughput.
- The benchmark includes operand cloning; parsing, input generation, and formatting are excluded.
- Integer/rational equality is cross-checked against Rug when `compare-rug` is enabled.
- Floating-point runs measure speed, **not** proof of identical rounding or ULP accuracy.
- Native Windows MSVC normally cannot build Rug's bundled GMP/MPFR; compare in WSL/Linux.
- Re-run after changing CPU governor, power state, compiler flags, or dependencies.
- Do not mark M2 backend selection finalized from one noisy run alone.
