# Initial release performance measurements

Measured on the same Windows 11 host with Rust 1.97.1 and optimized release
builds. Raw results and binary SHA-256 hashes are in
`benchmarks/release-before.json` and `benchmarks/release-after.json`.

The VM originally cloned its full working stack and registers for each
primitive instruction, despite already running inside a disposable outer
transaction. It now moves those buffers through primitive instructions. A
failed VM invocation is still discarded in full; a successful invocation
creates one history checkpoint. Direct non-VM line transactions are unchanged.

| Measurement | Before | After |
| --- | ---: | ---: |
| Startup: `--eval 1`, median of 30 processes | 4.755 ms | 4.585 ms |
| Compiled dup/drop, 1000-item stack, 2000 operations | 9.854 ms | 0.056 ms |
| Plain dup/drop, 1000-item stack, 2000 operations | 4.119 ms | 4.150 ms |
| Compiled increment macro, 2000 calls | 0.403 ms | 0.229 ms |
| Sampled process peak working set, fixed VM workload | 4.50 MiB | 3.98 MiB |

Operation medians use nine samples. The deep-stack compiled case improved
about 177 times in this synthetic workload; ordinary direct transactions did
not improve. The smaller startup difference can be normal process/cache noise.
Memory comes from Windows process counters sampled every 1 ms during a fixed
1000-item/10000-loop workload; it is not a universal memory limit or leak test.
Parsing, process creation and exact operation sizes affect other workloads.
No claim is made about large arbitrary-precision arithmetic or all machines.

Reproduce:

```powershell
cargo build --release --bin rpn --example performance
python tools/performance.py target/release/rpn.exe target/release/examples/performance.exe local benchmarks/release-local.json
```

The operation benchmark compiles programs once. Windows process startup is
measured separately. The numerical backend comparison remains in
`docs/M2-BENCHMARK.md`; these measurements do not compare Malachite with Rug.
