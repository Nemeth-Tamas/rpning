#!/usr/bin/env python3
"""Fail-closed, exact-anchor update for rpning main cf4262affb6c.

Place this file at tools/apply_m2_m3.py, other downloadable .rs files at
src/{math,precision,scientific,storage}.rs and tests/math_cli.rs, then run:
    python tools/apply_m2_m3.py

Every FIND anchor is literal text from the inspected GitHub main snapshot.
The program validates every anchor before changing any file. It does not
add, rename, or rewrite TODO goals: only existing checkboxes may change.
"""
from __future__ import annotations

from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parent.parent
pending: dict[Path, tuple[str, str]] = {}


def edit(relative: str, old: str, new: str) -> None:
    path = ROOT / relative
    if path not in pending:
        raw = path.read_bytes()
        newline = "\r\n" if b"\r\n" in raw else "\n"
        pending[path] = (raw.decode("utf-8").replace("\r\n", "\n"), newline)
    content, newline = pending[path]
    occurrences = content.count(old)
    if occurrences != 1:
        raise RuntimeError(f"{relative}: exact FIND matched {occurrences} times (expected 1):\n{old}")
    pending[path] = (content.replace(old, new, 1), newline)


def main() -> None:
    for name in ["math", "precision", "scientific", "storage"]:
        if not (ROOT / "src" / f"{name}.rs").is_file():
            raise RuntimeError(f"Missing downloaded file: src/{name}.rs")
    if not (ROOT / "tests/math_cli.rs").is_file():
        raise RuntimeError("Missing downloaded file: tests/math_cli.rs")

    edit("src/lib.rs", "pub mod engine;\npub mod number;", "pub mod engine;\npub mod math;\npub mod number;\npub mod precision;\npub mod scientific;\npub mod storage;")

    edit("src/engine.rs", "use crate::number::{Number, parse_literal};", "use crate::math;\nuse crate::number::{Number, parse_literal};\nuse crate::precision::{MAX_BITS, MIN_BITS, NumericSettings, Rounding, round_binary};")
    edit("src/engine.rs", "    IntegerOperandsRequired(String),\n}", "    IntegerOperandsRequired(String),\n    Math(String),\n}")
    edit("src/engine.rs", """            Self::IntegerOperandsRequired(operator) => {
                write!(f, "'{operator}' requires integer operands")
            }
""", """            Self::IntegerOperandsRequired(operator) => {
                write!(f, "'{operator}' requires integer operands")
            }
            Self::Math(message) => write!(f, "{message}"),
""")
    edit("src/engine.rs", """pub struct Calculator {
    stack: Vec<Number>,
    registers: BTreeMap<String, Number>,
    undo_history: Vec<Snapshot>,
    redo_history: Vec<Snapshot>,
}""", """pub struct Calculator {
    stack: Vec<Number>,
    registers: BTreeMap<String, Number>,
    settings: NumericSettings,
    undo_history: Vec<Snapshot>,
    redo_history: Vec<Snapshot>,
}""")
    edit("src/engine.rs", """    /// Restore the previous successful calculator state.
""", """    /// Current binary approximation and display settings.
    pub fn settings(&self) -> NumericSettings {
        self.settings
    }

    pub fn set_binary_mode(&mut self, enabled: bool) {
        self.settings.binary_mode = enabled;
    }

    pub fn set_precision_bits(&mut self, bits: u64) -> Result<(), CalcError> {
        if !(MIN_BITS..=MAX_BITS).contains(&bits) {
            return Err(CalcError::Math(format!(
                "precision must be between {MIN_BITS} and {MAX_BITS} bits"
            )));
        }
        self.settings.bits = bits;
        Ok(())
    }

    pub fn set_rounding(&mut self, rounding: Rounding) {
        self.settings.rounding = rounding;
    }

    pub fn set_scientific_display(&mut self, enabled: bool) {
        self.settings.scientific_display = enabled;
    }

    /// Replace persisted registers as one undoable transaction.
    pub fn restore_registers(&mut self, loaded: BTreeMap<String, Number>) -> Result<(), CalcError> {
        for name in loaded.keys() {
            validate_register_name(name)?;
        }
        if loaded != self.registers {
            let previous = Snapshot {
                stack: self.stack.clone(),
                registers: std::mem::replace(&mut self.registers, loaded),
            };
            push_bounded(&mut self.undo_history, previous);
            self.redo_history.clear();
        }
        Ok(())
    }

    fn quantize(&self, value: Number) -> Result<Number, CalcError> {
        if self.settings.binary_mode {
            round_binary(&value, self.settings.bits, self.settings.rounding)
                .map_err(|message| CalcError::Math(message.to_owned()))
        } else {
            Ok(value)
        }
    }

    /// Restore the previous successful calculator state.
""")
    edit("src/engine.rs", """                    let x = working.remove(len - 1);
                    let y = working.remove(len - 2);

                    if token == "/" && x == 0 {""", """                    let x = self.quantize(working.remove(len - 1))?;
                    let y = self.quantize(working.remove(len - 2))?;

                    if token == "/" && x == 0 {""")
    edit("src/engine.rs", """                    working.push(result);
                }

                "div" | "mod" => {""", """                    working.push(self.quantize(result)?);
                }

                "div" | "mod" => {""")
    edit("src/engine.rs", """                _ if token.starts_with("sto:") => {""", """                "pow" | "root" | "gcd" | "lcm" => {
                    require_depth(&working, 2, token)?;
                    let len = working.len();
                    let result = math::binary(token, &working[len - 2], &working[len - 1])
                        .map_err(|error| CalcError::Math(error.to_string()))?;
                    working.truncate(len - 2);
                    working.push(self.quantize(result)?);
                }

                "sqrt" | "fact" => {
                    require_depth(&working, 1, token)?;
                    let len = working.len();
                    let result = math::unary(token, &working[len - 1])
                        .map_err(|error| CalcError::Math(error.to_string()))?;
                    working.pop();
                    working.push(self.quantize(result)?);
                }

                "approx" => {
                    require_depth(&working, 1, token)?;
                    let len = working.len();
                    working[len - 1] = round_binary(
                        &working[len - 1], self.settings.bits, self.settings.rounding
                    ).map_err(|message| CalcError::Math(message.to_owned()))?;
                }

                _ if token.starts_with("sto:") => {""")
    edit("src/engine.rs", """                    working.push(value);
                }

                _ if token.starts_with("del:") => {""", """                    working.push(self.quantize(value)?);
                }

                _ if token.starts_with("del:") => {""")
    edit("src/engine.rs", """                    working.push(value);
                }
            }
        }

        if working != self.stack || working_registers != self.registers {""", """                    working.push(self.quantize(value)?);
                }
            }
        }

        if working != self.stack || working_registers != self.registers {""")
    edit("src/engine.rs", """    #[test]
    fn failed_lines_do_not_mutate_the_stack() {""", """    #[test]
    fn exact_advanced_math_preserves_atomicity_and_history() {
        let mut calc = Calculator::new();
        calc.evaluate("2 10 pow").unwrap();
        assert_stack(&calc, &["1024"]);
        calc.evaluate("9/4 sqrt").unwrap();
        assert_stack(&calc, &["1024", "3/2"]);
        let before = calc.stack().to_vec();
        assert!(calc.evaluate("2 sqrt").is_err());
        assert_eq!(calc.stack(), before);
        assert!(calc.undo());
        assert_stack(&calc, &["1024"]);
        calc.evaluate("clear 48 18 gcd 5 fact +").unwrap();
        assert_stack(&calc, &["126"]);
    }

    #[test]
    fn binary_mode_rounds_results_without_f64() {
        let mut calc = Calculator::new();
        calc.set_precision_bits(4).unwrap();
        calc.set_binary_mode(true);
        calc.evaluate("1 3 /").unwrap();
        assert_stack(&calc, &["11/32"]);
        calc.set_rounding(crate::precision::Rounding::TowardZero);
        calc.evaluate("clear 1 3 /").unwrap();
        assert_stack(&calc, &["5/16"]);
        calc.set_binary_mode(false);
        calc.evaluate("clear 1 3 /").unwrap();
        assert_stack(&calc, &["1/3"]);
    }

    #[test]
    fn restore_registers_is_undoable() {
        let mut calc = Calculator::new();
        calc.evaluate("5 sto:old").unwrap();
        let loaded = std::collections::BTreeMap::from([
            ("new".to_owned(), crate::number::parse_literal("1/3").unwrap()),
        ]);
        calc.restore_registers(loaded).unwrap();
        assert!(calc.register("old").is_none());
        assert_eq!(calc.register("new").unwrap().to_string(), "1/3");
        assert!(calc.undo());
        assert_eq!(calc.register("old").unwrap().to_string(), "5");
    }

    #[test]
    fn failed_lines_do_not_mutate_the_stack() {""")

    edit("src/main.rs", """use rpning::Calculator;
use rpning::number::format_number;""", """use rpning::Calculator;
use rpning::number::{Number, format_number};
use rpning::precision::Rounding;
use rpning::scientific::format_scientific;
use rpning::storage;""")
    edit("src/main.rs", """    if let Some(top) = calc.top() {
        println!("{}", format_number(top));
    }

    Ok(())
}""", """    if let Some(top) = calc.top() {
        println!("{}", format_for_calc(&calc, top));
    }

    Ok(())
}""")
    edit("src/main.rs", """                        println!("[{index}] {}", format_number(value));""", """                        println!("[{index}] {}", format_for_calc(&calc, value));""")
    edit("src/main.rs", """                        println!("{name} = {}", format_number(value));""", """                        println!("{name} = {}", format_for_calc(&calc, value));""")
    edit("src/main.rs", """                if changed {
                    if let Some(top) = calc.top() {
                        println!("{}", format_number(top));""", """                if changed {
                    if let Some(top) = calc.top() {
                        println!("{}", format_for_calc(&calc, top));""")
    edit("src/main.rs", """            expression => match calc.evaluate(expression) {
                Ok(()) => {
                    if let Some(top) = calc.top() {
                        println!("{}", format_number(top));""", """            expression => match calc.evaluate(expression) {
                Ok(()) => {
                    if let Some(top) = calc.top() {
                        println!("{}", format_for_calc(&calc, top));""")
    edit("src/main.rs", """            command @ (":undo" | ":redo") => {""", """            ":save" => {
                let path = storage::default_register_path();
                match storage::save(&path, calc.registers()) {
                    Ok(()) => println!("saved registers: {}", path.display()),
                    Err(error) => {
                        eprintln!("error: could not save registers: {error}");
                        if !interactive {
                            had_error = true;
                        }
                    }
                }
            }

            ":load" => {
                let path = storage::default_register_path();
                let result = storage::load(&path)
                    .and_then(|values| calc.restore_registers(values).map_err(io::Error::other));
                match result {
                    Ok(()) => println!("loaded registers: {}", path.display()),
                    Err(error) => {
                        eprintln!("error: could not load registers: {error}");
                        if !interactive {
                            had_error = true;
                        }
                    }
                }
            }

            command if command == ":settings"
                || command.starts_with(":precision ")
                || command.starts_with(":round ")
                || command.starts_with(":mode ")
                || command.starts_with(":display ") =>
            {
                match configure(&mut calc, command) {
                    Ok(message) => println!("{message}"),
                    Err(message) => {
                        eprintln!("error: {message}");
                        if !interactive {
                            had_error = true;
                        }
                    }
                }
            }

            command @ (":undo" | ":redo") => {""")
    edit("src/main.rs", """fn print_help() {""", """fn format_for_calc(calc: &Calculator, value: &Number) -> String {
    if calc.settings().scientific_display {
        format_scientific(value)
    } else {
        format_number(value)
    }
}

fn configure(calc: &mut Calculator, command: &str) -> Result<String, String> {
    let words: Vec<&str> = command.split_whitespace().collect();
    match words.as_slice() {
        [":settings"] => {
            let settings = calc.settings();
            Ok(format!(
                "mode={} precision={} bits rounding={} display={}",
                if settings.binary_mode { "binary" } else { "exact" },
                settings.bits,
                settings.rounding.name(),
                if settings.scientific_display { "scientific" } else { "auto" },
            ))
        }
        [":mode", mode @ ("exact" | "binary")] => {
            calc.set_binary_mode(*mode == "binary");
            Ok(format!("numeric mode: {mode}"))
        }
        [":precision", bits] => {
            let bits = bits.parse::<u64>().map_err(|_| "invalid bit count".to_owned())?;
            calc.set_precision_bits(bits).map_err(|error| error.to_string())?;
            Ok(format!("precision: {bits} bits"))
        }
        [":round", mode] => {
            let rounding = Rounding::parse(mode).ok_or("rounding: nearest|zero|up|down|away")?;
            calc.set_rounding(rounding);
            Ok(format!("rounding: {}", rounding.name()))
        }
        [":display", mode @ ("auto" | "scientific")] => {
            calc.set_scientific_display(*mode == "scientific");
            Ok(format!("display: {mode}"))
        }
        _ => Err("invalid settings command; see :help".to_owned()),
    }
}

fn print_help() {""")
    edit("src/main.rs", """    println!("  div mod                   Integer quotient and remainder");""", """    println!("  div mod                   Integer quotient and remainder");
    println!("  pow sqrt root fact gcd lcm Exact advanced mathematics");
    println!("  approx                    Round X to current binary precision");""")
    edit("src/main.rs", """    println!("  :registers                List named memory registers");""", """    println!("  :registers                List named memory registers");
    println!("  :save / :load             Save/load exact registers from disk");
    println!("  :mode exact|binary        Select exact or rounded arithmetic");
    println!("  :precision BITS           Binary precision (2..4096)");
    println!("  :round nearest|zero|up|down|away  Binary rounding mode");
    println!("  :display auto|scientific  Numeric display style");
    println!("  :settings                 Show numeric settings");""")

    edit("README.md", """## Roadmap

See `TODO.md`.""", """## Exact advanced mathematics and numeric controls

Exact operations: `pow` (integer exponent), `sqrt` (exact square root),
`root` (integer degree), `fact`, `gcd` and `lcm`. Irrational roots are
explicitly rejected rather than silently approximated. Exponent, factorial,
and root-degree limits protect interactive responsiveness.

The calculator starts in exact mode. To intentionally perform finite-precision
binary arithmetic, use `:mode binary`, `:precision 53` and
`:round nearest|zero|up|down|away`. This mode rounds the operands and result
of the four basic arithmetic operators to a dyadic rational with the specified
significant-bit precision. It has an unbounded exponent range and is **not**
IEEE 754 floating point (no infinities, NaNs or signed zeros). `approx` rounds
only the top value even in exact mode. Settings are session-local and are not
part of undo/redo history. `:display scientific` selects lossless
scientific notation for terminating decimal values; repeating values retain
exact fractional output. `:settings` shows the active controls.

Named registers persist only on explicit `:save` and `:load`. Default path:
Windows `%APPDATA%/rpning/registers.txt`; Linux `$XDG_DATA_HOME/rpning/`
`registers.txt` or `~/.local/share/rpning/registers.txt`. Loading is undoable.
A corrupted register file is rejected without altering memory.

Backend selection and benchmark methodology: `docs/M2-BENCHMARK.md`.

## Roadmap

See `TODO.md`.""")

    # TODO: preserve task names literally, check only completed/grounded goals.
    for old in [
        "- [ ] Benchmark Malachite against Rug",
        "- [ ] Finalize arithmetic backend selection",
        "- [ ] Benchmark integer, rational and real arithmetic",
        "- [ ] Add integer division and remainder",
    ]:
        edit("TODO.md", old, old.replace("[ ]", "[x]"))

    # All edits validated; only now write modified files.
    for path, (content, newline) in pending.items():
        path.write_bytes(content.replace("\n", newline).encode("utf-8"))
        print(f"updated {path.relative_to(ROOT)}")
    print("All exact anchors matched. Run cargo fmt/check/test/Clippy before checking further TODO tasks.")


if __name__ == "__main__":
    try:
        main()
    except (OSError, RuntimeError) as error:
        sys.exit(f"No changes written: {error}")
