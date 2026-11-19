# rpning — Comprehensive Work-mode handoff

## Current development checkpoint

The first M7 batch is implemented on main: alternate-base literals/display,
bitwise operations, fixed-width interpretations, counted statistics and exact
binomial probability/combinatorics. See docs/EXTENDED.md. Five original M7
tasks are checked only after 127 Rust tests, strict Clippy and both independent
numeric oracles passed. Remaining M7 tasks and both M2 tasks stay open. M9 is
the next active implementation slice. The user asked to stop with a usage
reserve; the assistant cannot inspect the account meter directly.

## Published release and next work

The user explicitly added M9: a calculator TUI with a prominent result/entry
above a lower stack/history region. Wide terminals show lower panes side by
side; narrow terminals switch them. The new tasks are unchecked; no TUI
implementation is claimed. See docs/TUI.md for layout and behavior. Existing
M0-M8 entries are preserved. M2/M7 remain open and Linux/macOS hands-on checks
remain deferred.

rpning 0.1.0 is published (non-draft, non-prerelease) at
https://github.com/Nemeth-Tamas/rpning/releases/tag/v0.1.0 from commit
67d12a9ae9bd5fe1527d500cc0df7c1103b07262. Windows x64, Linux x64 and macOS
ARM64 binaries and SHA256SUMS.txt were verified uploaded. All three platform
build/test/Clippy/oracle jobs passed in release run 44508757824. Windows local
validation covered 123 Rust tests and five isolated console sessions.

The user wants to continue the remaining TODO. M2 still needs a stable numeric
abstraction and measured small-integer optimization; M7 is unimplemented.
Recommended next slice: finish those M2 foundations before adding complex and
other extended values. Do not mark any M7 task complete yet. The user deferred
hands-on Linux/macOS acceptance, so those two TODO boxes remain open despite
successful automated CI. Cross-platform release builds and initial publishing
are complete. INSTALL.txt now contains release installation steps instead of
obsolete slice-install instructions. Remaining sections are historical context.

## Initial release implementation (2026-11-19)

Current code includes the M4 compiled macro language, all M5 workflow features
and all M6 numpad features. The user's two diagnostic files verified the
CONNECT IT NumCALC hardware; its reported NumLock flag remains false, so nav
layers follow actual key events. See docs/LANGUAGE.md, docs/WORKFLOW.md and
docs/KEYPAD.md. Settings/macros persist in interactive sessions; pipes and
one-shot/script evaluation start fresh. No new runtime dependencies were added.

Windows validation covers 123 Rust tests, strict Clippy, 1140 exact Fraction
rounding checks, 477 Decimal function comparisons and five isolated console
sessions (including keypad layers, line editing, history and completion).
Startup, stack/macro throughput and sampled memory were measured before/after
removing redundant VM transaction-buffer clones; see docs/RELEASE_PERFORMANCE.md.

Three-platform GitHub Actions builds validate Windows/Linux/macOS and publish
binaries/checksums only after all gates pass for a release commit. Check the
actual workflow/release state before claiming a published release. The M2
numeric abstraction/small-integer optimization and M7 extended mathematical
environment remain open in the original TODO; do not rename or move tasks.
The snapshots below are historical and do not describe current feature status.

## Current continuation status (2026-11-19)

The snapshot below this section is historical. The test literal fix was
already pushed at `ac72dbb55ea1e5fbef04fa02453f1553ae6ce9d9`. Work continued
from that exact GitHub main in a writable working copy because this session
could read, but could not write, the original Desktop checkout.

The continuation fixes two strict-Clippy `cmp_owned` errors in
`src/transcendental.rs`, implements `--keypad` and `--keypad-diagnose` with no
new dependencies, and adds pure entry-state tests, Windows mapping/ABI tests,
CLI regressions and an isolated hidden-console synthetic integration test.
See `docs/KEYPAD.md` for keys, semantics, limits and testing commands.

Numeric quality gates pass, including 1140 exact Fraction rounding comparisons
and 477 expanded Decimal function comparisons. These sampled function checks
do not certify correct rounding. The five pre-existing numeric TODO items and
five implemented/verified M6 software items are checked. No TODO labels were
added, renamed or moved. M2 abstraction/optimization remain open. M4/M5 and
the remaining M6 mappings/layers/macros remain open.

The next user-facing action is physical acceptance: run
`cargo run -- --keypad-diagnose`, press each key with NumLock on/off, copy the
reported events, then try `cargo run -- --keypad` with
`12`, Enter, `3`, `+`, `2`, `*` (expected X=30). Hardware testing remains
unchecked until actual output is observed. This adapter reads focused console
events, not global keyboard hooks or per-device Raw Input.

## Historical handoff snapshot

**Snapshot:** 2026-11-19, 10:00 Budapest time.  **Repository:** https://github.com/Nemeth-Tamas/rpning  **Branch:** `main`  **GitHub HEAD inspected:** `774e7cd53124ac35814476f1a1c74d52a9341b4c`.

> **Start here:** The latest pushed slice is **BROKEN IN UNIT-TEST COMPILATION**, not green. Fix `src/transcendental.rs` first, run tests/oracles, then continue. **Always re-fetch current GitHub main**; this SHA is a handoff snapshot, not permission to overwrite newer commits.

## Ready-to-paste instruction for ChatGPT Work

Continue the Rust project **rpning** from this handoff and connected GitHub repository on `main`. Your first task is to repair the pushed test compilation issue in `src/transcendental.rs` (see the exact fix and script below). Then run formatting, `cargo check`, `cargo test`, strict Clippy, and precision oracles; preserve every correct existing feature. On success check only existing completed TODO.md checkboxes. Proceed through remaining M2, M3, M4, M5, M6, M7, and M8 milestones in substantial but testable slices, with tests, documentation, and commit/push of every slice, **including broken/WIP slices**. User wants Windows-first, minimal dependencies, exact/arbitrary-precision RPN, powerful macros, and a dedicated physical USB numpad mode. The numpad **has arrived** and can be tested with the user. Do not invent keycodes or hardware behavior. Do not shift TODO goalposts: **never add, rename, split, or move existing TODO entries**; only toggle a checklist box when verified. Pull literal source from GitHub before every proposed change; use exact text replacements or directly edit genuine checked-out files, never fuzzy patches or imaginary snippets. Prefer files delivered individually, not ZIPs. Don't waste turns on trivial one-line slices where multiple relevant edits can be safely combined. After each successful slice, run quality gates, `git add .`, commit and push to `main`.

## The immediate broken state and exact solution

User ran `python tools/verify_and_tick_slice10.py`. `cargo fmt`, `cargo fmt --check`, and `cargo check` passed. `cargo test` failed with **17 E0308 mismatched-type errors**, all in the test section of `src/transcendental.rs`, approximately lines 266–297. The nested test helper is `fn r(s: &str) -> Rational { s.parse().unwrap() }`, but tests call `r(1)`, `r(1_000_000_000)`, and `r(1_000_000_000_000_000_000)`. The main code compiles; these are unit-test-only errors.

The attached `fix_transcendental_tests.py` (save as `tools/fix_transcendental_tests.py`) performs **exact, count-validated replacements only inside `#[cfg(test)] mod tests`**:

- `r(1_000_000_000_000_000_000)` → `r("1000000000000000000")` (**1 occurrence**)
- `r(1_000_000_000)` → `r("1000000000")` (**7 occurrences**)
- `r(1)` → `r("1")` (**9 occurrences**)

The input sources, test names, tolerances, and production arithmetic remain unchanged. It checks those exact occurrence counts and aborts before writing if they differ. It preserves CRLF/LF. **This script has had Python syntax validation but not been run against the actual checkout; use on the current GitHub state only if the patterns still match.**

Run in Windows PowerShell, from repository root:

```powershell
python tools/fix_transcendental_tests.py
cargo fmt
cargo fmt --check
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
cargo build --bin rpn
python tools/precision_oracle.py target/debug/rpn.exe
python tools/transcendental_oracle.py target/debug/rpn.exe
```

Alternatively `python tools/verify_and_tick_slice10.py` runs those gates and toggles **five pre-existing TODO items only after every check passes**. **Do not assert that it will pass without testing**; identify/fix any subsequent runtime, precision, or Clippy errors.

After a successful fix, `git add .`, `git commit -m "fix: correct transcendental unit-test rational literals"`, and `git push origin main`. If broken, **still commit/push the WIP** and explicitly report the failing command. Avoid amending/rebasing away the existing record without the user's approval.

## User-defined non-negotiable workflow

1. **Repository truth first:** Use connected GitHub to read the *latest* `main` sources before each change. User frequently pushes between messages. Check `TODO.md` live rather than using this snapshot as authoritative.
2. **Edits:** Exact literal **FIND → REPLACE**, including indentation and intentional blank lines. No vague “insert below,” fuzzy snippets, patches or guessed code. With direct repository workspace write access, edit real files in place and validate the diff against the fetched source; if no direct access, deliver entire file replacements or a verified exact-anchor installer, not invented instructions.
3. **Tests/gates:** `cargo fmt`; `cargo fmt --check`; `cargo check`; `cargo test`; `cargo clippy --all-targets --all-features -- -D warnings`. For changes to numeric accuracy, also rebuild debug binary and run oracles. The user is on Windows PowerShell and tests locally; you may use your own environment if Work provides one, but don't pretend it can access their PC unless it can.
4. **Git:** **Every development slice gets `git add .`, commit, and push to `origin main`, even broken/error-state slices**. Do not inspect unnecessary git status/log-style output repeatedly. Keep the next change visible on GitHub. Use meaningful commit messages.
5. **TODO discipline:** **NEVER add/rename/replace/move TODO lines or change the task list without explicit user request**. Check only existing tasks genuinely verified. No inflated progress or goalpost moving. Update TODO during each slice only by status checkbox toggles justified by tests.
6. **Artifact delivery:** User now permits downloadable files, **each file separately**; no ZIP. Preserve correct paths and don't overwrite `src/main.rs` accidentally (this previously happened when a math module was saved as main and caused missing-`main()` E0601). Always include git commands and copy-pastable, individually named paths. No extra stray newline within FIND/REPLACE blocks.
7. **Pace:** Prefer useful multi-file increments over dozens of trivial one-line edits. Implement and test enough that each slice has an end-to-end milestone. If the user's instruction to continue is clear, proceed without waiting for unnecessary confirmations. For hardware-specific behavior, seek observed test output rather than guessing.
8. **Dependencies:** Prefer lean Rust. Keep `Cargo.lock` committed. Runtime backend is Malachite. Rug/GMP are **benchmark-only** in a separate crate, best built under WSL/Linux (Windows/MSVC concerns).

## Product and architectural scope

**rpning** (`Cargo.toml`: package `rpning`, binary `rpn`, Rust edition **2024**, version **0.1.0**) is a Windows-first fast, lightweight, exact, programmable **RPN terminal calculator**. Use cross-platform architecture where practical; Windows first, then Linux and macOS. Future ambitions include very powerful macros, configurable terminal workflow, physical numpad quick entry, and expanded mathematics. Avoid turning it into a bloated GUI app.

Current external Rust dependency: `malachite = 0.14.0` with `default-features = false`, features `rationals`, `std`. `src/number.rs` exposes `pub type Number = Rational`, `parse_literal`, and `format_number` (up to **128** exact terminating decimal places; otherwise exact fraction; no f64). The current type alias is provisional; **stable numeric abstraction is still a pending M2 task**.

**Source paths verified in GitHub HEAD:**

- `src/main.rs` — interactive/piped REPL and CLI (`--eval`, `--help`, `--version`), `:stack`, `:registers`, `:undo`, `:redo`, `:save`, `:load`, numeric settings (`:mode`, `:precision`, `:round`, `:display`, `:settings`), output formatting.
- `src/lib.rs` — exports engine, math, number, precision, scientific, storage, transcendental modules.
- `src/engine.rs` — per-line atomic RPN evaluation, exact arithmetic, stack and memory registers, capped **128** undo/redo snapshots, integer division/mod, advanced exact math, explicit approximate math, functions and constants. **Engine clones full stack+registers per transaction and snapshots; consider profiling before optimizing.**
- `src/number.rs` — Malachite Rational parsing and exact decimal output.
- `src/math.rs` — exact `pow`, `sqrt`, `root`, `fact`, `gcd`, `lcm`; domain/size guards.
- `src/precision.rs` — bounded binary dyadic rational rounding (2–4096 bits), nearest-even/zero/up/down/away; unbounded exponent, **not IEEE754**.
- `src/scientific.rs` — lossless scientific notation for terminating rationals.
- `src/storage.rs` — explicitly persisted named registers via `:save`/`:load`, versioned UTF-8 file, size bounds, error-safe load, intended atomic writes; validate behavior on Windows replacement paths.
- `src/transcendental.rs` — current **new, not-yet-green** approximate high-precision `pi`, `e`, `exp`, `ln`, `log2`, `log10`, `sin`, `cos`, `tan`, `rad`, `deg`, series-based rational internals, 40 guard bits, 512-bit function cap, input/domain limits. **Not guaranteed correctly rounded across all inputs**. No f64 operations. Errors must leave stack/registers unchanged.
- `tests/cli.rs`, `tests/math_cli.rs`, `tests/transcendental_cli.rs` — executable integration regressions.
- `tools/precision_oracle.py`, `tools/transcendental_oracle.py` — independent testing scripts; run against newly built `target/debug/rpn.exe` (not a stale executable). `tools/verify_and_tick_slice10.py` gates and checks five existing TODO lines if green.
- `benchmarks/{Cargo.toml,src/main.rs,src/cases.rs,src/data.rs,src/timing.rs,summarize.py,README.md}` — isolated bench crate for Malachite vs Rug.
- `docs/M2-BENCHMARK.md`, `docs/M2-M3-REAL-MATH.md`, `README.md`, `TODO.md`, `comparison.csv`, `comparison-report.md`.

**Current RPN semantics:** Stack top X, next Y. `3 4 +`=7, `5` Enter and `3+` Enter=8. Exact `+ - * /`; integer truncating `div` and dividend-signed `mod`; `dup drop swap over rot nip tuck clear pick roll depth`; `sto:name`, `rcl:name`, `del:name`, `clearmem`. Register names are 1–64 ASCII identifier characters, case sensitive. `:registers` lists sorted names. Transactions are atomic per input line; failed token rolls back entire line. Undo/redo restores stack **and** registers; writing after undo invalidates redo. Historical snapshots are in-memory. `:save` and `:load` explicitly persist registers; **settings are session-only and do not participate in undo/redo**. `--eval` has a fresh calculator per invocation.

**Benchmark conclusions:** User ran real WSL2 comparison with **27** operation/size cases, 400 iterations and 7 repeats; **Malachite won 17**, Rug won 10. Measurements were operation-level medians including 2 operand clones, excluding parse/format, **not** full REPL throughput. Malachite wins common small integer/rational work, Rug some larger multiplications; **Malachite chosen for Windows-first runtime**, Rug only as optional reference/benchmark. `comparison-report.md` in repo contains authoritative actual numbers.

## Current existing TODO status (from `main`, 2026-11-19)

**M0:** all checked. **M1:** all checked.

**M2:** checked: Benchmark Malachite against Rug; Finalize arithmetic backend selection; Add configurable binary floating-point precision; Add exact-to-approximate conversion rules; Add configurable rounding; Verify exact terminating-decimal formatting and display limits; Add scientific number formatting; Benchmark integer, rational and real arithmetic. **Still unchecked, exact existing labels:**

- `Define stable internal numeric abstraction`
- `Optimize small integer operations`
- `Add independent precision reference tests` (scripts exist, but **the full verification failed before running them**; do not check until they pass)

**M3:** checked: extended stack ops, atomic rollback, indexed ops, bounded history, named registers, register deletion/clearing, persistent register storage, integer div/mod, exponentiation and roots, factorial/GCD/LCM. **Still unchecked, exact existing labels:**

- `Add logarithmic and exponential functions`
- `Add trigonometric functions`
- `Add mathematical constants`
- `Add degree/radian conversion`

The current code implements attempts at these last four, but their tests are still broken. `verify_and_tick_slice10.py` checks the four M3 lines **and** M2 independent precision reference tests only when every gate/oracle passes. Revisit accuracy/robustness even after a green build; passing sampled tests does not constitute mathematical proof.

**M4 — Macro language:** *all unchecked*: Finalize macro syntax; Implement tokenizer and parser; Compile macros to internal instructions; Implement macro execution engine; Support nested macros; Add aliases and namespaces; Add macro definitions and descriptions; Persist macros between sessions; Add macro import/export; Add recursion and execution limits; Add macro debugging and tracing; Add conditionals, loops and variables.

**M5 — Terminal workflow:** *all unchecked*: Add persistent command history; Add history search; Add tab completion; Add configurable number display; Add persistent application settings; Add script-file execution; Add helpful built-in documentation; Add programmable command shortcuts.

**M6 — Physical numpad mode:** checked: Select affordable standalone USB numpad; Order CONNECT IT NumCALC CKB-0060-BK. The user bought a **CONNECT IT NumCALC CKB-0060-BK wired USB numpad** from Alza for **2,330 HUF** Nov 18, 2026; **USER NOW CONFIRMS IT HAS ARRIVED Nov 19**. All other original M6 tasks are still open: Test hardware and emitted key events; Design dedicated keypad state machine; Implement immediate operator execution; Define Enter and pending-number behavior; Implement backspace and decimal entry; Add configurable key mappings; Add alternate keypad layers; Add programmable macro shortcuts; Verify normal terminal mode remains unaffected.

Hardware acceptance: Investigate emitted keyboard events with user on Windows (NumLock on/off, decimal separator, Enter, +/-/*// keys, scan codes and layout, focus behavior), **do not claim numpad has a distinct device identity or specific scancodes without testing**. Standard terminals may present a standalone numpad exactly like integrated keypad; dedicated device separation may need Windows Raw Input and additional APIs. Avoid capturing global keyboard input without explicit intentional design. Preserve ordinary REPL and offer opt-in `--keypad`/similar mode. Define predictable pending-number/Enter state machine, immediate operators, backspace/clear, macro layers and escape. Automated state-machine tests must run without hardware, then user verifies on actual device. Do not infer hardware event output from this handoff.

**M7 — Extended mathematical environment:** *all unchecked*: Implement complex numbers; Add binary, octal and hexadecimal modes; Add bitwise arithmetic; Add fixed-width integer interpretations; Add statistical functions; Add probability and combinatorics; Add vectors and matrices; Add unit conversions; Add physical constants; Add financial mathematics; Add numerical integration and differentiation; Add equation solving; Add custom mathematical extensions.

**M8 — Performance and releases:** *all unchecked*: Establish startup benchmarks; Benchmark stack-operation throughput; Optimize atomic execution overhead; Benchmark compiled macro execution; Profile memory usage; Verify Windows compatibility; Verify Linux compatibility; Verify macOS compatibility; Add cross-platform release builds; Complete user documentation; Publish initial stable release.

## Work-mode execution plan and acceptance criteria

1. **Repair and stabilize (immediate):** Pull fresh `main`; fix the 17 test E0308s, run full gates + oracles. Fix ensuing failures without regressions. Consider a test on the `exp`, trig and log limits, error atomicity, real precision and rounding. Only then mark existing M2/M3 items verified. Commit+push, even WIP if stuck.
2. **Finish M2 properly:** Stabilize a backend-independent `Number` abstraction *without* sacrificing exact Rational semantics, measure/optimize common small-integer hot paths with actual before/after benchmarks, and pass independent numerical reference checks. Avoid claiming a native arbitrary-precision `Float` type: current binary mode produces rounded exact rationals with unbounded exponent, as stated in docs. Preserve users' exact calculations. No premature checkbox checks.
3. **Finish M3 rigorously:** Domain/precision tests for logarithms, trig and constants; calibrated reference comparisons across multiple input magnitudes and precisions; angle reduction/performance stress, cancellation and tan singularities. Respect explicit approximation and 512-bit current cap unless deliberately improved/tested. Persistence round trips and failure atomicity on Windows.
4. **M4 compiler/macros:** Agree a deterministic syntax; implement multiple source modules from start (lexer/parser/AST or bytecode/execution); make nested macros, namespaces, variables, control flow, persistence, import/export, debugging and recursion/gas limits safe and testable. Retain transactional evaluation/undo semantics; avoid arbitrary OS shell execution as a default macro feature. Check only existing TODO items.
5. **M5 terminal productivity:** Readline-style history/search/completion, stable configuration, script-file execution and shortcuts. Keep piped input/noninteractive commands reliable and with nonzero exit on failure. Separate key handling from pure engine and don't require an interactive tty for `--eval`.
6. **M6 numpad:** Because hardware has arrived, prioritize event diagnostics and a Windows opt-in keypad mode without upsetting normal terminal operation. Include instrumented key-event test tool (with explicit user consent for device input handling), a testable key-state model, mapping/layers and macro shortcuts. Don't guess hardware details or demand them repeatedly; present a focused test only when needed.
7. **M7/M8:** Implement expanded math, benchmarks and release gates in substantial tested slices. Memory/runtime performance is particularly important for large rationals, series and per-line cloning. Publish a stable release only after cross-platform verification and docs.

Every slice should include test cases, documentation updates, an accurate progress note, and git add/commit/push. **No invented completed status, no unsupported performance claims, and no untested dependency API guesses.** For user-facing handoff during work, keep messages concise; deliver files separately (no ZIP) if needing manual application.

## Important history / known pitfalls

- During an earlier artifact handoff, the user accidentally placed the `math.rs` module into `src/main.rs`, producing `E0601: main function not found`. A repair installer recovered the original CLI. Current `src/main.rs` **does** have `fn main()` at inspected HEAD. Do not repeat that mistake.
- Earlier installer failed because `src/math.rs` wasn't downloaded to its expected destination; source was not modified by that failed run. Later a self-contained installer fixed this. New installers must preflight all inputs, fail closed, and preserve both file paths and line endings.
- Strict Clippy repeatedly flagged empty lines after Rust `///` doc comments and a nested collapsible `if`. Treat `cargo clippy ... -D warnings` as a required gate, not optional.
- `tools/precision_oracle.py` must run against **freshly rebuilt** `rpn.exe`; an old binary gave bogus “unknown token ':mode'” errors when installer had failed.
- Current test E0308 happens because production `r(n: u64)` and nested test-module `r(s: &str)` are **different functions**. Do not change production `r(n: u64)` or introduce f64 to repair tests.
- Benchmarks measured real operations but not full calculator throughput; do not overinterpret 17–10 as universal backend superiority.
- Arithmetic operations and rational formatting are exact; transcendental series are approximations and must be labeled as such.

**Latest authoritative source is always GitHub `main` at the time Work begins.**
