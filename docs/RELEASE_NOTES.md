# Initial release: rpning 0.1.0

A Windows-first, exact, programmable RPN terminal calculator. Download the
binary for your platform; no Rust installation is needed to run it. Windows
includes immediate keypad mode and focused key diagnostics. Ordinary REPL,
pipes, scripts and one-shot evaluation are available on all three platforms.

- Exact arbitrary-precision integers/rationals; exact decimals and scientific input.
- Advanced exact math, named registers and bounded stack/register/macro undo/redo.
- Explicit approximate real functions, 2..4096-bit binary rounding controls,
  2..512-bit transcendental functions and multiple display modes.
- Compiled macros, nested calls, recursion limits, conditions, loops, variables,
  namespaces, aliases, descriptions, persistence, import/export and tracing.
- Saved settings, command shortcuts, history/search, Windows line editing and
  Tab completion, script-file execution, configurable keypad bindings/layers.

Quick start:

```text
rpn --eval "0.1 0.2 +"
rpn --run examples/invoice.rpn
rpn --keypad
```

Run without arguments for the REPL; `:help` lists commands. See README and
`docs/LANGUAGE.md`, `docs/WORKFLOW.md`, `docs/KEYPAD.md` for full instructions.

Transcendental functions are sampled/tested approximations, not guaranteed
correctly rounded for every input. Irrational roots are explicitly rejected.
`~` marks rounded fixed-decimal display. Keypad/advanced line editing currently
require a focused Windows console; there is no global or per-device input
capture. Numeric undo and command history are separate. Macro execution limits
bound instructions, not wall-clock time for large exact arithmetic.

The existing roadmap remains in TODO.md. Numeric abstraction/small-integer
optimization and the M7 extended mathematical environment are future work;
this release does not claim that the entire roadmap is complete.

Release binaries are built and tested on Windows, Linux and macOS. The release
is published only after every platform's quality gates and independent numeric
oracles pass. SHA256SUMS.txt contains checksums for the downloadable binaries.
