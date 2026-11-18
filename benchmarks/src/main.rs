//! Benchmark-only executable, isolated from the rpning application crate.
mod cases;
mod data;
mod timing;

use data::{FLOAT_PRECISIONS, INTEGER_DIGITS, RATIONAL_DIGITS};
use timing::Config;

fn main() {
    let config = parse_args();
    eprintln!(
        "rpning backend benchmark | Malachite 0.14.0{} | {} iterations x {} repeats",
        if cfg!(feature = "compare-rug") {
            " vs Rug 1.30.0"
        } else {
            " (Rug disabled)"
        },
        config.iterations,
        config.repeats
    );
    eprintln!("Prepared inputs and cross-backend checks are outside the timed loops.");
    eprintln!("The timed loops INCLUDE cloning both operands and creating the result.");

    let mut rows = Vec::new();

    for digits in INTEGER_DIGITS {
        cases::integer_cases(&mut rows, config, digits);
    }
    for digits in RATIONAL_DIGITS {
        cases::rational_cases(&mut rows, config, digits);
    }
    for bits in FLOAT_PRECISIONS {
        cases::float_cases(&mut rows, config, bits);
    }

    if config.validate_only {
        eprintln!(
            "Input preparation complete. Cross-library exact checks run only with --features compare-rug."
        );
        return;
    }

    println!(
        "family,operation,size,unit,backend,iterations,repeats,median_ns_per_op,min_ns_per_op,max_ns_per_op"
    );
    for row in &rows {
        println!(
            "{},{},{},{},{},{},{},{:.2},{:.2},{:.2}",
            row.family,
            row.operation,
            row.size,
            row.unit,
            row.backend,
            row.iterations,
            row.repeats,
            row.median_ns,
            row.min_ns,
            row.max_ns
        );
    }
    eprintln!("Wrote {} measurement rows to stdout (CSV).", rows.len());
}

fn parse_args() -> Config {
    let mut config = Config {
        iterations: 400,
        repeats: 5,
        validate_only: false,
    };
    let mut args = std::env::args().skip(1);

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--iterations" => {
                config.iterations = parse_positive(args.next(), "--iterations");
            }
            "--repeats" => {
                config.repeats = parse_positive(args.next(), "--repeats");
            }
            "--validate-only" => config.validate_only = true,
            "-h" | "--help" => {
                println!("rpning backend benchmark (standalone, not part of normal builds)");
                println!("  --iterations N   Operations per repeat (default 400)");
                println!("  --repeats N      Samples per case (default 5)");
                println!("  --validate-only Run setup and correctness checks without timing");
                println!("  --help          Show this help");
                std::process::exit(0);
            }
            _ => {
                eprintln!("Unknown argument: {argument}");
                std::process::exit(2);
            }
        }
    }
    config
}

fn parse_positive(value: Option<String>, flag: &str) -> usize {
    match value.and_then(|text| text.parse::<usize>().ok()) {
        Some(n) if n > 0 => n,
        _ => {
            eprintln!("{flag} must be followed by an integer greater than zero");
            std::process::exit(2);
        }
    }
}
