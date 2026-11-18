use malachite::base::num::arithmetic::traits::Sign;
use malachite::base::num::conversion::traits::FromSciString;
use malachite::{Natural, Rational};

/// The numerical backend is isolated here so it can evolve later.
pub type Number = Rational;

/// Parse an exact integer, fraction, decimal or scietific literal.
///
/// No conversion through f32 or f64 is ever performed.
pub fn parse_literal(token: &str) -> Option<Number> {
    token
        .parse::<Number>()
        .ok()
        .or_else(|| Number::from_sci_string(token))
}

/// Maximum number of fractional decimal digits displayed automatically.
///
/// Longer terminating expansions and recurring decimals retain their
/// exact rational representation rather than being rounded.
const MAX_DECIMAL_PLACES: u64 = 128;

/// Format an exact rational as a decimal when reasonably short.
///
/// This function never uses floating-point arithmetic or rounding.
/// The stored rational value is not modified.
pub fn format_number(value: &Number) -> String {
    let Some(places) = value.length_after_point_in_small_base(10) else {
        return value.to_string();
    };

    if places == 0 || places > MAX_DECIMAL_PLACES {
        return value.to_string();
    }

    // A terminating base-10 rational satisfies:
    // numerator * 10^places / denominator = an exact integer.
    let ten = Natural::from(10u32);
    let mut scaled = value.numerator_ref().clone();

    for _ in 0..places {
        scaled *= &ten;
    }

    let digits = (scaled / value.denominator_ref()).to_string();
    let places = places as usize;

    let mut result = String::with_capacity(digits.len() + 3);

    if value.sign().is_lt() {
        result.push('-');
    }

    if digits.len() <= places {
        result.push_str("0.");
        result.extend(std::iter::repeat_n('0', places - digits.len()));
        result.push_str(&digits);
    } else {
        let split = digits.len() - places;
        let (integer, fractional) = digits.split_at(split);

        result.push_str(integer);
        result.push('.');
        result.push_str(fractional);
    }

    result
}

#[cfg(test)]
mod tests {
    use super::{format_number, parse_literal};

    #[test]
    fn parses_exact_decimal_literals() {
        assert_eq!(parse_literal("0.1").unwrap().to_string(), "1/10");
        assert_eq!(parse_literal("-0.125").unwrap().to_string(), "-1/8");
        assert_eq!(parse_literal("1.2e5").unwrap().to_string(), "120000");
        assert_eq!(parse_literal("1.2e-5").unwrap().to_string(), "3/250000");
    }

    #[test]
    fn parses_and_normalizes_fractions() {
        assert_eq!(parse_literal("10/20").unwrap().to_string(), "1/2");
        assert_eq!(parse_literal("-6/9").unwrap().to_string(), "-2/3");
    }

    #[test]
    fn rejects_invalid_literals() {
        assert!(parse_literal("hello").is_none());
        assert!(parse_literal("1/0").is_none());
        assert!(parse_literal("1.2.3").is_none());
    }

    #[test]
    fn formats_terminating_decimals_exactly() {
        let cases = [
            ("0", "0"),
            ("42", "42"),
            ("1/2", "0.5"),
            ("5/2", "2.5"),
            ("1/8", "0.125"),
            ("1/20", "0.05"),
            ("1/1000", "0.001"),
            ("-5/2", "-2.5"),
            ("-1/8", "-0.125"),
            ("100000000000000000001/2", "50000000000000000000.5"),
        ];

        for (input, expected) in cases {
            let number = parse_literal(input).unwrap();
            let formatted = format_number(&number);

            assert_eq!(formatted, expected, "input: {input}");
            assert_eq!(
                parse_literal(&formatted),
                Some(number),
                "decimal output must preserve the original value"
            );
        }
    }

    #[test]
    fn recurring_decimals_remain_exact_fractions() {
        for input in ["1/3", "1/6", "2/7", "-5/11"] {
            let number = parse_literal(input).unwrap();

            assert_eq!(format_number(&number), number.to_string());
        }
    }

    #[test]
    fn long_decimal_expansions_remain_exact_fractions() {
        let number = parse_literal("1e-129").unwrap();

        assert_eq!(format_number(&number), number.to_string());
    }

    #[test]
    fn handles_decimal_display_limit_boundary() {
        let number = parse_literal("1e-128").unwrap();
        let formatted = format_number(&number);

        assert!(formatted.starts_with("0."));
        assert_eq!(formatted.len(), 130);
        assert_eq!(parse_literal(&formatted), Some(number));
    }
}
