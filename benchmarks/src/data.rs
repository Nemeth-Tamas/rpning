//! Deterministic shared input datasets for each numerical backend.

pub(crate) const INPUT_SAMPLES: usize = 8;
pub(crate) const INTEGER_DIGITS: [usize; 3] = [20, 78, 310];
pub(crate) const RATIONAL_DIGITS: [usize; 3] = [10, 40, 155];
pub(crate) const FLOAT_PRECISIONS: [u32; 3] = [53, 256, 1024];

/// Deterministically generate a nonzero decimal number of exactly `len` digits.
/// Every backend receives precisely the same decimal operand strings.
pub(crate) fn decimal_digits(len: usize, seed: u64) -> String {
    let mut state = seed;
    let mut out = String::with_capacity(len);
    for i in 0..len {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let digit = ((state >> 32) % 10) as u8;
        let digit = if i == 0 { 1 + digit % 9 } else { digit };
        out.push(char::from(b'0' + digit));
    }
    out
}

pub(crate) fn integer_input_strings(digits: usize) -> Vec<(String, String)> {
    (0..INPUT_SAMPLES)
        .map(|i| {
            (
                decimal_digits(digits, 31 + 2 * i as u64),
                decimal_digits(digits.div_ceil(2), 32 + 2 * i as u64),
            )
        })
        .collect()
}

pub(crate) fn rational_input_strings(digits: usize) -> Vec<(String, String)> {
    (0..INPUT_SAMPLES)
        .map(|i| {
            let n = 100 + 4 * i as u64;
            (
                format!(
                    "{}/{}",
                    decimal_digits(digits, n),
                    decimal_digits(digits, n + 1)
                ),
                format!(
                    "{}/{}",
                    decimal_digits(digits, n + 2),
                    decimal_digits(digits, n + 3)
                ),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{INPUT_SAMPLES, decimal_digits, integer_input_strings, rational_input_strings};

    #[test]
    fn generated_operands_are_nonzero_and_deterministic() {
        assert_eq!(decimal_digits(20, 42), decimal_digits(20, 42));
        assert_ne!(decimal_digits(20, 42), decimal_digits(20, 43));
        assert_ne!(decimal_digits(20, 42).as_bytes()[0], b'0');
        assert_eq!(integer_input_strings(20).len(), INPUT_SAMPLES);
        assert_eq!(rational_input_strings(20).len(), INPUT_SAMPLES);
    }
}
