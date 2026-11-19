# rpning 0.2.0 - TUI and extended mathematics

A new calculator screen, discoverable settings and a complete embedded manual.
Download the binary for your platform; Rust is not needed to run it.

## What's new

- **Windows TUI by default:** committed result and editable input above a live
  stack and calculation journal. Wide terminals show both lower panes; narrow
  terminals switch them. Explicit `--repl` keeps the ordinary line editor.
- **Discoverable settings:** type `:display`, `:mode`, `:round` or `:precision`
  for hints, press Enter without arguments for current/allowed values, or Tab
  for argument completion.
- **Scrollable command output and help:** F6 opens full lists, bindings, macro
  inspection and guides without replacing the result. F4 returns to stack/history.
- **Complete embedded documentation:** `:help tui`, `:help program`,
  `:help layers` and topic chapters including advanced macro programming.
- **Extended exact mathematics:** binary/octal/hex literals and display; bitwise
  operations; signed/unsigned fixed-width conversions; counted statistics;
  combinations, permutations and exact binomial probabilities.
- Existing exact rational arithmetic, numeric precision/rounding, named registers,
  compiled macros, loops/conditions/locals, scripts and saved workflow settings
  remain available. Pipes, redirected output and one-shot evaluation retain
  plain CLI behavior.

## Download and run

| Platform | Asset |
| --- | --- |
| Windows x64 | `rpn-Windows-X64.exe` |
| Linux x64 | `rpn-Linux-X64` |
| macOS Apple Silicon | `rpn-macOS-ARM64` |

SHA256SUMS.txt contains SHA-256 checksums for the three binaries.

On Windows, optionally rename the executable to rpn.exe, then run:

```powershell
.\rpn.exe
.\rpn.exe --repl
.\rpn.exe --eval "0.1 0.2 +"
```

In the TUI: F2 toggles Base/Alt; F3 changes focus; F4 switches lower panes;
F5 selects numeric/command entry; F6 browses output/help; Esc exits.
Try `12`, Enter, `3`, `+`, `2`, `*` in Base/NUM: X becomes 30.

On Linux/macOS, make the binary executable (`chmod +x ./rpn`) and run it.
These platforms retain their ordinary REPL; TUI/keypad adapters are Windows-only.

## Guides and upgrading

- [TUI user guide and full Base/Alt/Nav layer tables](https://github.com/Nemeth-Tamas/rpning/blob/v0.2.0/docs/TUI.md)
- [Complete program guide: mathematics, settings, registers, macros, advanced programming, scripts and persistence](https://github.com/Nemeth-Tamas/rpning/blob/v0.2.0/docs/PROGRAM_GUIDE.md)
- [Installation instructions](https://github.com/Nemeth-Tamas/rpning/blob/v0.2.0/INSTALL.txt)

Replace the old executable with the new binary. Existing settings, mappings,
macros and explicit register files use the same formats and state directory.
Windows no-argument interactive startup now opens the TUI; use `--repl` to
keep the previous line editor. The numeric stack and TUI journal are session-only.
The v0.1.0 release remains available under its original tag.

## Validation and current limits

Local Windows validation passed 137 Rust tests, strict Clippy, 1140 independent
Fraction rounding checks, 477 independent Decimal comparisons and seven isolated
console sessions, including live TUI frames, settings, guides, redirected output,
resize handling and terminal restoration. The release pipeline repeats checks
and numeric oracles on Windows/Linux/macOS before publishing these binaries.

Linux/macOS hands-on acceptance remains deferred. Approximate transcendental
functions support 2..512 bits and are not certified correctly rounded for every
input. Irrational roots are rejected. Fixed decimal rounding is marked with ~.
The TUI assumes single-column glyphs and restores terminal state on ordinary
exit/errors/unwinding. Remaining M2/M7 items are still tracked in TODO.md;
this release does not claim that the entire mathematical roadmap is complete.
