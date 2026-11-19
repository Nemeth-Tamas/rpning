//! Lossless base-10 scientific display of terminating rational values.
//!
//! Recurring rationals remain exact fractions. We never manufacture a
//! finite decimal approximation merely to make the output look scientific.

use crate::number::{Number, format_number};

pub fn format_scientific(value: &Number) -> String {
    let decimal = format_number(value);
    if decimal.contains('/') || decimal == "0" {
        return decimal;
    }
    let (negative, unsigned) = match decimal.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", decimal.as_str()),
    };
    let before_dot = unsigned.split('.').next().unwrap_or(unsigned);
    let digits_before_dot = before_dot.len();
    let all_digits: String = unsigned.chars().filter(|ch| *ch != '.').collect();
    let Some(first_nonzero) = all_digits.bytes().position(|byte| byte != b'0') else {
        return "0".to_owned();
    };
    let exponent = digits_before_dot as i64 - first_nonzero as i64 - 1;
    let mut significand = all_digits[first_nonzero..].trim_end_matches('0').to_owned();
    if significand.len() > 1 {
        significand.insert(1, '.');
    }
    if exponent == 0 {
        format!("{negative}{significand}")
    } else {
        format!("{negative}{significand}e{exponent}")
    }
}

#[cfg(test)]
mod tests {
    use super::format_scientific;
    use crate::number::parse_literal;

    #[test]
    fn formats_terminating_numbers_losslessly() {
        let cases = [
            ("0", "0"),
            ("1", "1"),
            ("12345", "1.2345e4"),
            ("-500", "-5e2"),
            ("0.00125", "1.25e-3"),
            ("-0.5", "-5e-1"),
            ("10/4", "2.5"),
            ("1/3", "1/3"),
        ];
        for (input, expected) in cases {
            let value = parse_literal(input).unwrap();
            let output = format_scientific(&value);
            assert_eq!(output, expected);
            assert_eq!(parse_literal(&output), Some(value));
        }
    }
}
