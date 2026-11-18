use std::env;
use std::io::{self, IsTerminal, Write};
use std::process;

use rpning::Calculator;

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

        [flag] if flag == "--help" || flag == "-h" => {
            print_help();
            Ok(())
        }

        [flag] if flag == "--version" || flag == "-V" => {
            println!("rpn {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }

        [flag, expression] if flag == "--eval" || flag == "-e" => evaluate_once(expression),

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
        println!("{top}");
    }

    Ok(())
}

fn repl() -> io::Result<()> {
    let stdin = io::stdin();
    let interactive = stdin.is_terminal();

    let mut calc = Calculator::new();
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

            ":stack" => {
                if calc.stack().is_empty() {
                    println!("(empty)");
                } else {
                    for (index, value) in calc.stack().iter().rev().enumerate() {
                        println!("[{index}] {value}");
                    }
                }
            }

            expression => match calc.evaluate(expression) {
                Ok(()) => {
                    if let Some(top) = calc.top() {
                        println!("{top}");
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

fn print_help() {
    println!("rpning - exact RPN calculator");
    println!();
    println!("Usage:");
    println!("  rpn                       Interactive calculator");
    println!("  rpn --eval \"3 4 +\"          Evaluate an expression");
    println!("  rpn --help                Show help");
    println!("  rpn --version             Show version");
    println!();
    println!("Operators:");
    println!("  +  -  *  /                Arithmetic");
    println!("  dup drop swap clear       Stack operations");
    println!();
    println!("Interactive commands:");
    println!("  :stack                    Display stack (X at index 0)");
    println!("  :help                     Show help");
    println!("  :quit                     Exit");
    println!();
    println!("Numbers are exact. Integers, fractions and decimals work.");
}
