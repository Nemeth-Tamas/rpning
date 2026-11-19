use std::env;
use std::io::{self, IsTerminal, Write};
use std::process;

use rpning::Calculator;
use rpning::number::{Number, format_number};
use rpning::precision::Rounding;
use rpning::scientific::format_scientific;
use rpning::storage;
use rpning::{persistence, programming};

mod keypad_terminal;

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

fn run() -> io::Result<()> {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.as_slice() {
        [] => repl(),

        [flag] if flag == "--keypad" => keypad_terminal::run(false),

        [flag] if flag == "--keypad-diagnose" => keypad_terminal::run(true),

        [flag] if flag == "--help" || flag == "-h" => {
            print_help();
            Ok(())
        }

        [flag] if flag == "--version" || flag == "-V" => {
            println!("rpn {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }

        [flag, expression] if flag == "--eval" || flag == "-e" => evaluate_once(expression),

        [flag, path] if flag == "--run" || flag == "-f" => {
            let source = persistence::read_text(std::path::Path::new(path))?;
            evaluate_once(&source)
        }

        _ => {
            print_help();
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid command-line arguments",
            ))
        }
    }
}

fn evaluate_once(expression: &str) -> io::Result<()> {
    let mut calc = Calculator::new();

    calc.evaluate(expression).map_err(io::Error::other)?;

    if let Some(top) = calc.top() {
        println!("{}", format_for_calc(&calc, top));
    }

    Ok(())
}

fn repl() -> io::Result<()> {
    let stdin = io::stdin();
    let interactive = stdin.is_terminal();

    let mut calc = Calculator::new();
    if interactive && programming::macro_path().exists() {
        programming::load(&mut calc, &programming::macro_path()).map_err(io::Error::other)?;
    }
    let mut line = String::new();
    let mut had_error = false;

    loop {
        if interactive {
            print!("> ");
            io::stdout().flush()?;
        }

        line.clear();

        if stdin.read_line(&mut line)? == 0 {
            break;
        }

        match line.trim() {
            "" => continue,

            ":quit" | ":exit" => break,

            ":help" => {
                print_help();
            }

            command if programming::is_command(command) => {
                match programming::command(&mut calc, command) {
                    Ok(lines) => {
                        for line in lines {
                            println!("{line}");
                        }
                        if interactive {
                            programming::save(&calc, &programming::macro_path())
                                .map_err(io::Error::other)?;
                        }
                    }
                    Err(error) => {
                        eprintln!("error: {error}");
                        if !interactive {
                            had_error = true;
                        }
                    }
                }
            }

            ":stack" => {
                if calc.stack().is_empty() {
                    println!("(empty)");
                } else {
                    for (index, value) in calc.stack().iter().rev().enumerate() {
                        println!("[{index}] {}", format_for_calc(&calc, value));
                    }
                }
            }

            ":registers" => {
                if calc.registers().next().is_none() {
                    println!("(no registers)");
                } else {
                    for (name, value) in calc.registers() {
                        println!("{name} = {}", format_for_calc(&calc, value));
                    }
                }
            }

            ":save" => {
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

            command
                if command == ":settings"
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

            command @ (":undo" | ":redo") => {
                let changed = if command == ":undo" {
                    calc.undo()
                } else {
                    calc.redo()
                };

                if changed {
                    if let Some(top) = calc.top() {
                        println!("{}", format_for_calc(&calc, top));
                    } else {
                        println!("(empty)");
                    }
                } else {
                    eprintln!("error: nothing to {}", &command[1..]);

                    if !interactive {
                        had_error = true;
                    }
                }
            }

            expression => match calc.evaluate(expression) {
                Ok(()) => {
                    if interactive && !calc.macros().is_empty() {
                        programming::save(&calc, &programming::macro_path())
                            .map_err(io::Error::other)?;
                    }
                    if let Some(top) = calc.top() {
                        println!("{}", format_for_calc(&calc, top));
                    } else {
                        println!("(empty)");
                    }
                }

                Err(error) => {
                    eprintln!("error: {error}");

                    if !interactive {
                        had_error = true;
                    }
                }
            },
        }
    }

    if had_error {
        Err(io::Error::other("one or more input lines failed"))
    } else {
        Ok(())
    }
}

fn format_for_calc(calc: &Calculator, value: &Number) -> String {
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
                if settings.binary_mode {
                    "binary"
                } else {
                    "exact"
                },
                settings.bits,
                settings.rounding.name(),
                if settings.scientific_display {
                    "scientific"
                } else {
                    "auto"
                },
            ))
        }
        [":mode", mode @ ("exact" | "binary")] => {
            calc.set_binary_mode(*mode == "binary");
            Ok(format!("numeric mode: {mode}"))
        }
        [":precision", bits] => {
            let bits = bits
                .parse::<u64>()
                .map_err(|_| "invalid bit count".to_owned())?;
            calc.set_precision_bits(bits)
                .map_err(|error| error.to_string())?;
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

fn print_help() {
    println!("rpning - exact RPN calculator");
    println!();
    println!("Usage:");
    println!("  rpn                       Interactive calculator");
    println!("  rpn --keypad              Immediate keypad input (Windows console)");
    println!("  rpn --keypad-diagnose     Inspect focused console key events (Windows)");
    println!("  rpn --run FILE            Execute an RPN script atomically");
    println!("  rpn --eval \"3 4 +\"          Evaluate an expression");
    println!("  rpn --help                Show help");
    println!("  rpn --version             Show version");
    println!();
    println!("Operators:");
    println!("  +  -  *  /                Exact rational arithmetic");
    println!("  div mod                   Integer quotient and remainder");
    println!("  pow sqrt root fact gcd lcm Exact advanced mathematics");
    println!("  pi e                      Approximate mathematical constants");
    println!("  exp ln log2 log10         Approximate exponential/logarithms");
    println!("  sin cos tan               Approximate trigonometry (radians)");
    println!("  rad deg                   Convert degrees/radians");
    println!("  approx                    Round X to current binary precision");
    println!("  dup drop swap clear       Basic stack operations");
    println!("  over                      Duplicate second stack item");
    println!("  rot                       Rotate top three items left");
    println!("  nip                       Remove second stack item");
    println!("  tuck                      Copy top item behind second");
    println!("  pick                      Copy indexed item (0 = top)");
    println!("  roll                      Move indexed item to top");
    println!("  depth                     Push current stack depth");
    println!("  sto:name                  Store X without consuming it");
    println!("  rcl:name                  Push a stored register value");
    println!("  del:name                  Delete a named register");
    println!("  clearmem                  Delete all named registers");
    println!();
    println!("Interactive commands:");
    println!("  :stack                    Display stack (X at index 0)");
    println!("  :registers                List named memory registers");
    println!("  :save / :load             Save/load exact registers from disk");
    println!("  :mode exact|binary        Select exact or rounded arithmetic");
    println!("  :precision BITS           Binary precision (2..4096)");
    println!("  :round nearest|zero|up|down|away  Binary rounding mode");
    println!("  :display auto|scientific  Numeric display style");
    println!("  :settings                 Show numeric settings");
    println!("  :undo                     Restore previous stack");
    println!("  :redo                     Reapply undone calculation");
    println!("  :help                     Show help");
    println!("  :quit                     Exit");
    println!("  : NAME \"description\" BODY ;  Define a compiled macro (namespaces: math.square)");
    println!("  :macros / :see NAME       List macros / inspect compiled instructions");
    println!("  :alias NAME TARGET_WORD   Define an alias");
    println!("  :trace EXPRESSION         Execute with a bounded instruction trace");
    println!("  :import FILE / :export FILE   Merge/export macro definitions");
    println!("  :macros-save / :macros-load   Explicit macro persistence");
    println!("  if ... else ... then     Conditional (consume flag; zero=false)");
    println!("  N times ... end          Counted loop; while CONDITION do BODY end");
    println!("  let:NAME / get:NAME       Macro-local variables; = != < <= > >= not");
    println!();
    println!("Numbers are exact. Integers, fractions and decimals work.");
}
