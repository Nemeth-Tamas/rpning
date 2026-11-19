use std::env;
use std::io::{self, IsTerminal};
use std::process;

use rpning::Calculator;
use rpning::number::Number;
use rpning::storage;
use rpning::{config::Config, history::History, workflow};
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
    let config = Config::default();

    calc.evaluate(expression).map_err(io::Error::other)?;

    if let Some(top) = calc.top() {
        println!("{}", format_for_calc(&calc, &config, top));
    }

    Ok(())
}

fn repl() -> io::Result<()> {
    let stdin = io::stdin();
    let interactive = stdin.is_terminal();

    let mut calc = Calculator::new();
    let mut config = if interactive {
        Config::load().map_err(io::Error::other)?
    } else {
        Config::default()
    };
    config.apply(&mut calc).map_err(io::Error::other)?;
    let mut history = if interactive {
        History::load().map_err(io::Error::other)?
    } else {
        History::default()
    };
    if interactive && programming::macro_path().exists() {
        programming::load(&mut calc, &programming::macro_path()).map_err(io::Error::other)?;
    }
    calc.clear_history();
    let mut line = String::new();
    let mut had_error = false;

    loop {
        line.clear();
        if interactive {
            match keypad_terminal::read_line(
                "> ",
                history.entries(),
                &workflow::completions(&calc),
            )? {
                Some(input) => line = input,
                None => break,
            }
        } else if stdin.read_line(&mut line)? == 0 {
            break;
        }
        if !line.trim().starts_with(":history") {
            history.record(line.trim());
        }

        let previous_macros = calc.macros().clone();
        let previous_config = config.clone();

        match line.trim() {
            "" => continue,

            ":quit" | ":exit" => break,

            ":help" => {
                print_help();
            }

            command if workflow::is_command(command) => {
                match workflow::command(&mut calc, &mut config, &mut history, command) {
                    Ok(lines) => {
                        for line in lines {
                            println!("{line}");
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

            command if programming::is_command(command) => {
                match programming::command(&mut calc, command) {
                    Ok(lines) => {
                        for line in lines {
                            println!("{line}");
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
                        println!("[{index}] {}", format_for_calc(&calc, &config, value));
                    }
                }
            }

            ":registers" => {
                if calc.registers().next().is_none() {
                    println!("(no registers)");
                } else {
                    for (name, value) in calc.registers() {
                        println!("{name} = {}", format_for_calc(&calc, &config, value));
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
                match workflow::configure_numeric(&mut calc, &mut config, command) {
                    Ok(message) => {
                        println!("{message}");
                        config.capture(&calc);
                    }
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
                        println!("{}", format_for_calc(&calc, &config, top));
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
                    if let Some(top) = calc.top() {
                        println!("{}", format_for_calc(&calc, &config, top));
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
        if interactive && calc.macros() != &previous_macros {
            programming::save(&calc, &programming::macro_path()).map_err(io::Error::other)?;
        }
        config.capture(&calc);
        if interactive && config != previous_config {
            config.save().map_err(io::Error::other)?;
        }
    }

    if interactive {
        history.save().map_err(io::Error::other)?;
    }
    if had_error {
        Err(io::Error::other("one or more input lines failed"))
    } else {
        Ok(())
    }
}

fn format_for_calc(calc: &Calculator, config: &Config, value: &Number) -> String {
    config.display.format(value, calc.settings().rounding)
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
    println!("  0b/0o/0x literals        Exact binary, octal and hexadecimal integers");
    println!("  band bor bxor bnot shl shr  Integer bitwise operations");
    println!(
        "  u8/i8 .. u128/i128       Wrap/interpret fixed-width integers; VALUE BITS asuint/asint"
    );
    println!("  sum mean median min max var svar stdev sstdev  Consume N values followed by N");
    println!("  choose perm / N K P binomp  Combinatorics / exact binomial probability");
    println!("  :display binary|octal|hex  Integer base display (fractions remain lossless)");
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
    println!("  :display fraction|fixed:N Exact fraction / rounded decimal (marked ~)");
    println!("  :settings-save / :settings-load  Persist/restore numeric settings and mappings");
    println!(
        "  :history [TEXT]           Search saved command history; Up/Down, Ctrl+R in Windows REPL"
    );
    println!("  :history-save / :history-load    Explicit history persistence");
    println!(
        "  Tab                       Complete a command, word, macro or register (Windows REPL)"
    );
    println!("  :complete PREFIX          List completions in any terminal");
    println!("  :shortcut NAME EXPRESSION / :do NAME   Define/use a command shortcut");
    println!("  :bind LAYER KEY ACTION    Bind a keypad key to RPN or @undo/@layer:NAME/etc.");
    println!(
        "  :unbind LAYER KEY / :bindings  Remove/list mappings; F2 toggles base/alt in keypad"
    );
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
