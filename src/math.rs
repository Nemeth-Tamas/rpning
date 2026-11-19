//! Exact mathematical operations for the rational calculator.
//!
//! Domain errors never modify the caller's stack. Operations are deliberately
//! exact: an irrational root is rejected rather than silently rounded.

use malachite::base::num::arithmetic::traits::{CheckedRoot, CheckedSqrt, Gcd, Lcm, UnsignedAbs};
use malachite::{Integer, Natural, Rational};
use std::fmt;

const MAX_POWER: u64 = 100_000;
const MAX_FACTORIAL: u64 = 10_000;
const MAX_ROOT: u64 = 1_024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MathError {
    IntegerRequired(&'static str),
    Range(&'static str),
    Domain(&'static str),
    IrrationalRoot,
}

impl fmt::Display for MathError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IntegerRequired(op) => write!(f, "'{op}' requires integer operands"),
            Self::Range(reason) => write!(f, "number out of supported range: {reason}"),
            Self::Domain(reason) => write!(f, "math domain error: {reason}"),
            Self::IrrationalRoot => write!(f, "root is not rational (no rounding was performed)"),
        }
    }
}

impl std::error::Error for MathError {}

fn exact_integer(value: &Rational, op: &'static str) -> Result<Integer, MathError> {
    Integer::try_from(value.clone()).map_err(|_| MathError::IntegerRequired(op))
}

fn bounded_integer(value: &Rational, op: &'static str) -> Result<i64, MathError> {
    let integer = exact_integer(value, op)?;
    integer
        .to_string()
        .parse::<i64>()
        .map_err(|_| MathError::Range("integer operand does not fit i64"))
}

/// Evaluate one exact unary mathematical operation.
pub fn unary(op: &str, x: &Rational) -> Result<Rational, MathError> {
    match op {
        "sqrt" => {
            if x < &Rational::from(0) {
                return Err(MathError::Domain("square root of a negative value"));
            }
            x.clone().checked_sqrt().ok_or(MathError::IrrationalRoot)
        }
        "fact" => {
            let count = bounded_integer(x, "fact")?;
            if count < 0 {
                return Err(MathError::Domain("factorial of a negative value"));
            }
            let n = count as u64;
            if n > MAX_FACTORIAL {
                return Err(MathError::Range("factorial exceeds 10000"));
            }
            let mut result = Natural::from(1u8);
            for i in 2..=n {
                result *= Natural::from(i);
            }
            Ok(Rational::from(result))
        }
        _ => unreachable!("the engine dispatches only known unary operations"),
    }
}

/// Evaluate one exact binary mathematical operation.
/// The left operand is Y and the right operand is X.
pub fn binary(op: &str, y: &Rational, x: &Rational) -> Result<Rational, MathError> {
    match op {
        "pow" => {
            let exponent = bounded_integer(x, "pow")?;
            let magnitude = exponent.unsigned_abs();
            if magnitude > MAX_POWER {
                return Err(MathError::Range("absolute exponent exceeds 100000"));
            }
            if exponent < 0 && y == &Rational::from(0) {
                return Err(MathError::Domain("zero to a negative power"));
            }
            let mut result = Rational::from(1);
            let mut factor = y.clone();
            let mut remaining = magnitude;
            while remaining > 0 {
                if remaining & 1 == 1 {
                    result *= &factor;
                }
                remaining >>= 1;
                if remaining != 0 {
                    factor = &factor * &factor;
                }
            }
            if exponent < 0 {
                Ok(Rational::from(1) / result)
            } else {
                Ok(result)
            }
        }
        "root" => {
            let degree = bounded_integer(x, "root")?;
            if degree <= 0 || degree as u64 > MAX_ROOT {
                return Err(MathError::Range("root degree must be 1..=1024"));
            }
            if degree % 2 == 0 && y < &Rational::from(0) {
                return Err(MathError::Domain("even root of a negative value"));
            }
            y.clone()
                .checked_root(degree)
                .ok_or(MathError::IrrationalRoot)
        }
        "gcd" | "lcm" => {
            let a = exact_integer(y, if op == "gcd" { "gcd" } else { "lcm" })?;
            let b = exact_integer(x, if op == "gcd" { "gcd" } else { "lcm" })?;
            let a = a.unsigned_abs();
            let b = b.unsigned_abs();
            let result = if op == "gcd" { a.gcd(b) } else { a.lcm(b) };
            Ok(Rational::from(result))
        }
        _ => unreachable!("the engine dispatches only known binary operations"),
    }
}

#[cfg(test)]
mod tests {
    use super::{MathError, binary, unary};
    use malachite::Rational;

    fn rational(value: &str) -> Rational {
        value.parse().expect("valid rational test input")
    }

    #[test]
    fn exact_powers_and_inverse_powers() {
        let cases = [
            ("2", "10", "1024"),
            ("2", "-3", "1/8"),
            ("-2", "3", "-8"),
            ("0", "0", "1"),
            ("3/2", "3", "27/8"),
        ];
        for (base, exponent, expected) in cases {
            assert_eq!(
                binary("pow", &rational(base), &rational(exponent)).unwrap(),
                rational(expected)
            );
        }
        assert!(matches!(
            binary("pow", &rational("0"), &rational("-1")),
            Err(MathError::Domain(_))
        ));
        assert!(matches!(
            binary("pow", &rational("2"), &rational("1/2")),
            Err(MathError::IntegerRequired(_))
        ));
    }

    #[test]
    fn roots_are_exact_or_explicitly_rejected() {
        assert_eq!(unary("sqrt", &rational("81/16")).unwrap(), rational("9/4"));
        assert_eq!(
            binary("root", &rational("-27/8"), &rational("3")).unwrap(),
            rational("-3/2")
        );
        assert_eq!(
            binary("root", &rational("16"), &rational("4")).unwrap(),
            rational("2")
        );
        assert!(matches!(
            unary("sqrt", &rational("2")),
            Err(MathError::IrrationalRoot)
        ));
        assert!(matches!(
            binary("root", &rational("-16"), &rational("2")),
            Err(MathError::Domain(_))
        ));
    }

    #[test]
    fn factorial_and_number_theory() {
        assert_eq!(unary("fact", &rational("0")).unwrap(), rational("1"));
        assert_eq!(unary("fact", &rational("5")).unwrap(), rational("120"));
        assert_eq!(
            binary("gcd", &rational("-48"), &rational("18")).unwrap(),
            rational("6")
        );
        assert_eq!(
            binary("lcm", &rational("-12"), &rational("18")).unwrap(),
            rational("36")
        );
        assert_eq!(
            binary("gcd", &rational("0"), &rational("0")).unwrap(),
            rational("0")
        );
        assert!(matches!(
            unary("fact", &rational("-1")),
            Err(MathError::Domain(_))
        ));
        assert!(matches!(
            binary("gcd", &rational("1/2"), &rational("3")),
            Err(MathError::IntegerRequired(_))
        ));
    }
}
