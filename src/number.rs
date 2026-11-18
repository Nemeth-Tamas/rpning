use malachite::Rational;
use malachite::base::num::conversion::traits::FromSciString;

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

#[cfg(test)]
mod tests {
    use super::parse_literal;

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
}
