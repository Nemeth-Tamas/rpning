//! Reproducible operation-level release benchmarks; not an arithmetic-backend race.
use rpning::{Calculator, language::compile};
use std::hint::black_box;
use std::time::Instant;

fn median(mut samples: Vec<u128>) -> u128 {
    samples.sort_unstable();
    samples[samples.len() / 2]
}

fn main() {
    println!("case,operations,median_ns");
    let mut calc = Calculator::new();
    calc.evaluate("1000 times 1 end").unwrap();
    calc.clear_history();
    let compiled = compile("1000 times dup drop end").unwrap();
    let mut samples = Vec::new();
    for _ in 0..9 {
        let start = Instant::now();
        calc.execute_program(black_box(&compiled), false).unwrap();
        samples.push(start.elapsed().as_nanos());
    }
    println!("compiled_stack_1000,2000,{}", median(samples));

    let mut samples = Vec::new();
    for _ in 0..9 {
        let start = Instant::now();
        for _ in 0..1000 {
            calc.evaluate(black_box("dup drop")).unwrap();
        }
        samples.push(start.elapsed().as_nanos());
    }
    println!("plain_stack_1000,2000,{}", median(samples));

    let mut calc = Calculator::new();
    calc.evaluate(": increment 1+ ; 0").unwrap();
    calc.clear_history();
    let compiled = compile("2000 times increment end").unwrap();
    let mut samples = Vec::new();
    for _ in 0..9 {
        let start = Instant::now();
        calc.execute_program(black_box(&compiled), false).unwrap();
        samples.push(start.elapsed().as_nanos());
    }
    println!("compiled_macro,2000,{}", median(samples));
}
