# rpning numerical-backend benchmark results

- Generated (UTC): 2026-11-18T21:12:50+00:00
- Platform: Linux-6.18.58.1-microsoft-standard-WSL2-x86_64-with-glibc2.39
- Python runtime: 3.12.3 (report generator only)
- Source CSV: `comparison.csv`
- Iterations per repeat: 400
- Repeats: 7

All timings are median nanoseconds per arithmetic operation, including two operand clones.
Inputs are generated and parsed before timing. Smaller is faster.

| Family | Operation | Size | Malachite (ns) | Rug (ns) | Faster backend / ratio |
|---|---|---:|---:|---:|---|
| float | add | 53 precision_bits | 15.68 | 32.97 | Malachite 2.10x |
| float | add | 256 precision_bits | 35.70 | 38.87 | Malachite 1.09x |
| float | add | 1024 precision_bits | 46.20 | 41.59 | Rug 1.11x |
| float | mul | 53 precision_bits | 20.34 | 33.12 | Malachite 1.63x |
| float | mul | 256 precision_bits | 77.21 | 50.54 | Rug 1.53x |
| float | mul | 1024 precision_bits | 249.38 | 172.07 | Rug 1.45x |
| integer | add | 20 decimal_digits | 16.49 | 30.32 | Malachite 1.84x |
| integer | add | 78 decimal_digits | 32.22 | 37.10 | Malachite 1.15x |
| integer | add | 310 decimal_digits | 35.62 | 45.33 | Malachite 1.27x |
| integer | div | 20 decimal_digits | 27.73 | 52.94 | Malachite 1.91x |
| integer | div | 78 decimal_digits | 52.69 | 55.05 | Malachite 1.04x |
| integer | div | 310 decimal_digits | 111.04 | 106.05 | Rug 1.05x |
| integer | mul | 20 decimal_digits | 23.97 | 43.11 | Malachite 1.80x |
| integer | mul | 78 decimal_digits | 47.37 | 42.65 | Rug 1.11x |
| integer | mul | 310 decimal_digits | 137.06 | 97.36 | Rug 1.41x |
| integer | rem | 20 decimal_digits | 37.33 | 66.00 | Malachite 1.77x |
| integer | rem | 78 decimal_digits | 56.87 | 61.41 | Malachite 1.08x |
| integer | rem | 310 decimal_digits | 148.10 | 126.92 | Rug 1.17x |
| rational | add | 10 decimal_digits_per_part | 81.60 | 120.02 | Malachite 1.47x |
| rational | add | 40 decimal_digits_per_part | 384.80 | 245.20 | Rug 1.57x |
| rational | add | 155 decimal_digits_per_part | 1,151.38 | 1,256.15 | Malachite 1.09x |
| rational | div | 10 decimal_digits_per_part | 67.58 | 91.61 | Malachite 1.36x |
| rational | div | 40 decimal_digits_per_part | 596.27 | 381.68 | Rug 1.56x |
| rational | div | 155 decimal_digits_per_part | 1,930.14 | 1,976.93 | Malachite 1.02x |
| rational | mul | 10 decimal_digits_per_part | 64.43 | 95.39 | Malachite 1.48x |
| rational | mul | 40 decimal_digits_per_part | 624.99 | 393.57 | Rug 1.59x |
| rational | mul | 155 decimal_digits_per_part | 1,927.32 | 1,959.41 | Malachite 1.02x |

Compared operation/size pairs: **27**

## Interpretation and limitations

- Results apply to this host, toolchain, workload, and numerical-library versions.
- This is an operation-level microbenchmark, not end-to-end rpning calculator throughput.
- The benchmark includes operand cloning; parsing, input generation, and formatting are excluded.
- Integer/rational equality is cross-checked against Rug when `compare-rug` is enabled.
- Floating-point runs measure speed, **not** proof of identical rounding or ULP accuracy.
- Native Windows MSVC normally cannot build Rug's bundled GMP/MPFR; compare in WSL/Linux.
- Re-run after changing CPU governor, power state, compiler flags, or dependencies.
- Do not mark M2 backend selection finalized from one noisy run alone.
