//! Timing with deterministic workloads and optimizer barriers.
use std::hint::black_box;
use std::time::Instant;

#[derive(Clone, Copy)]
pub(crate) struct Config {
    pub(crate) iterations: usize,
    pub(crate) repeats: usize,
    pub(crate) validate_only: bool,
}

#[derive(Clone, Copy)]
pub(crate) struct Case {
    pub(crate) family: &'static str,
    pub(crate) operation: &'static str,
    pub(crate) size: usize,
    pub(crate) unit: &'static str,
    pub(crate) backend: &'static str,
}

pub(crate) struct Row {
    pub(crate) family: &'static str,
    pub(crate) operation: &'static str,
    pub(crate) size: usize,
    pub(crate) unit: &'static str,
    pub(crate) backend: &'static str,
    pub(crate) iterations: usize,
    pub(crate) repeats: usize,
    pub(crate) median_ns: f64,
    pub(crate) min_ns: f64,
    pub(crate) max_ns: f64,
}

fn measure<T, F>(inputs: &[(T, T)], config: Config, operation: F) -> (f64, f64, f64)
where
    T: Clone,
    F: Fn(T, T) -> T,
{
    assert!(!inputs.is_empty());

    // Warm up the same operation on the same set of prepared operands.
    for i in 0..config.iterations.min(100) {
        let (a, b) = &inputs[i % inputs.len()];
        let a = black_box(a).clone();
        let b = black_box(b).clone();
        black_box(operation(a, b));
    }

    let mut timings = Vec::with_capacity(config.repeats);
    for _ in 0..config.repeats {
        let start = Instant::now();
        for i in 0..config.iterations {
            let (a, b) = &inputs[i % inputs.len()];
            let a = black_box(a).clone();
            let b = black_box(b).clone();
            black_box(operation(a, b));
        }
        let elapsed = start.elapsed();
        timings.push(elapsed.as_secs_f64() * 1_000_000_000.0 / config.iterations as f64);
    }

    timings.sort_by(f64::total_cmp);
    (
        timings[timings.len() / 2],
        timings[0],
        timings[timings.len() - 1],
    )
}

pub(crate) fn bench<T, F>(
    rows: &mut Vec<Row>,
    case: Case,
    inputs: &[(T, T)],
    config: Config,
    calculation: F,
) where
    T: Clone,
    F: Fn(T, T) -> T,
{
    if config.validate_only {
        return;
    }
    let (median_ns, min_ns, max_ns) = measure(inputs, config, calculation);
    rows.push(Row {
        family: case.family,
        operation: case.operation,
        size: case.size,
        unit: case.unit,
        backend: case.backend,
        iterations: config.iterations,
        repeats: config.repeats,
        median_ns,
        min_ns,
        max_ns,
    });
}
