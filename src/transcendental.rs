//! Explicitly approximate real mathematics, using exact rational series internally.
//!
//! Basic + - * /, integer operations and existing rational math remain exact.
//! Transcendental results are finite dyadic approximations at the configured
//! precision. The series have guard bits but are NOT certified correctly rounded
//! for every input. No f32 or f64 arithmetic is used.

use malachite::base::num::arithmetic::traits::FloorLogBasePowerOf2;
use malachite::{Natural, Rational};

use crate::precision::{Rounding, round_binary};

pub const MAX_FUNCTION_BITS: u64 = 512;
const GUARD_BITS: u64 = 40;
const MAX_TERMS: u64 = 8_192;
const MAX_ANGLE: i64 = 1_000;
const MAX_EXP_ARG: i64 = 256;
const MAX_LOG_EXPONENT: i64 = 100_000;

type Real = Rational;

fn r(n: u64) -> Real {
    Real::from(n)
}

fn abs(value: &Real) -> Real {
    if value < &r(0) {
        -value.clone()
    } else {
        value.clone()
    }
}

fn tolerance(bits: u64) -> Result<Real, String> {
    if !(2..=MAX_FUNCTION_BITS).contains(&bits) {
        return Err(format!(
            "transcendental precision must be 2..={MAX_FUNCTION_BITS} bits; other operators still support up to 4096"
        ));
    }
    Ok(r(1) / Real::from(Natural::from(1u8) << (bits + GUARD_BITS)))
}

fn output(value: Real, bits: u64, rounding: Rounding) -> Result<Real, String> {
    round_binary(&value, bits, rounding).map_err(str::to_owned)
}

// atan(1/q) from an alternating series with bounded remainder.
fn atan_inverse(q: u64, epsilon: &Real) -> Result<Real, String> {
    let z = r(1) / r(q);
    let z2 = &z * &z;
    let mut term = z;
    let mut sum = r(0);
    for n in 0..MAX_TERMS {
        let next = &term / r(2 * n + 1);
        if n % 2 == 0 {
            sum += &next;
        } else {
            sum -= &next;
        }
        if next < *epsilon {
            return Ok(sum);
        }
        term *= &z2;
    }
    Err("pi series exceeded iteration budget".to_owned())
}

fn pi_raw(epsilon: &Real) -> Result<Real, String> {
    // Machin's identity: pi = 16 atan(1/5) - 4 atan(1/239).
    let first = atan_inverse(5, epsilon)?;
    let second = atan_inverse(239, epsilon)?;
    Ok(first * r(16) - second * r(4))
}

fn exp_raw(x: &Real, epsilon: &Real, bits: u64) -> Result<Real, String> {
    if abs(x) > Real::from(MAX_EXP_ARG) {
        return Err(format!("exp argument exceeds +/-{MAX_EXP_ARG}"));
    }
    let mut reduced = x.clone();
    let mut squares = 0u32;
    while abs(&reduced) > r(1) / r(2) {
        reduced /= r(2);
        squares += 1;
    }
    let mut sum = r(1);
    let mut term = r(1);
    for n in 1..=MAX_TERMS {
        term *= &reduced;
        term /= r(n);
        sum += &term;
        if abs(&term) < *epsilon {
            // Repeated squaring is rounded internally to bound intermediate size.
            for _ in 0..squares {
                sum = output(&sum * &sum, bits + GUARD_BITS, Rounding::NearestEven)?;
            }
            return Ok(sum);
        }
    }
    Err("exp series exceeded iteration budget".to_owned())
}

fn log_atanh(z: &Real, epsilon: &Real) -> Result<Real, String> {
    let z2 = z * z;
    let mut term = z.clone();
    let mut sum = r(0);
    for n in 0..MAX_TERMS {
        let add = &term / r(2 * n + 1);
        sum += &add;
        if abs(&add) < *epsilon {
            return Ok(sum * r(2));
        }
        term *= &z2;
    }
    Err("log series exceeded iteration budget".to_owned())
}

fn ln_raw(x: &Real, epsilon: &Real) -> Result<Real, String> {
    if x <= &r(0) {
        return Err("ln/log require a positive argument".to_owned());
    }
    if x == &r(1) {
        return Ok(r(0));
    }
    let exponent = x.floor_log_base_power_of_2(1);
    if !(-MAX_LOG_EXPONENT..=MAX_LOG_EXPONENT).contains(&exponent) {
        return Err("ln argument exponent exceeds safety limit".to_owned());
    }
    let magnitude = exponent.unsigned_abs();
    let power = Real::from(Natural::from(1u8) << magnitude);
    let normalized = if exponent >= 0 {
        x / &power
    } else {
        x * &power
    };
    let z = (&normalized - r(1)) / (&normalized + r(1));
    let ln_mantissa = log_atanh(&z, epsilon)?;
    let ln_two = log_atanh(&(r(1) / r(3)), epsilon)?;
    Ok(ln_mantissa + ln_two * Real::from(exponent))
}

fn sin_cos_raw(x: &Real, pi: &Real, epsilon: &Real) -> Result<(Real, Real), String> {
    if abs(x) > Real::from(MAX_ANGLE) {
        return Err(format!(
            "angle exceeds +/-{MAX_ANGLE} radians; reduce it first"
        ));
    }
    let mut angle = x.clone();
    let two_pi = pi * r(2);
    while angle > *pi {
        angle -= &two_pi;
    }
    while angle < -pi.clone() {
        angle += &two_pi;
    }
    let minus_x2 = -(&angle * &angle);
    let mut sin_term = angle.clone();
    let mut cos_term = r(1);
    let mut sin_sum = angle;
    let mut cos_sum = r(1);
    for n in 1..=MAX_TERMS {
        sin_term *= &minus_x2;
        sin_term /= r(2 * n) * r(2 * n + 1);
        cos_term *= &minus_x2;
        cos_term /= r(2 * n - 1) * r(2 * n);
        sin_sum += &sin_term;
        cos_sum += &cos_term;
        if abs(&sin_term) < *epsilon && abs(&cos_term) < *epsilon {
            return Ok((sin_sum, cos_sum));
        }
    }
    Err("trigonometric series exceeded iteration budget".to_owned())
}

/// Approximate the mathematical constants pi and e.
pub fn constant(name: &str, bits: u64, rounding: Rounding) -> Result<Real, String> {
    let epsilon = tolerance(bits)?;
    let raw = match name {
        "pi" => pi_raw(&epsilon)?,
        "e" => exp_raw(&r(1), &epsilon, bits)?,
        _ => return Err("unknown constant".to_owned()),
    };
    output(raw, bits, rounding)
}

/// Approximate a real function. All domain checking is done before committing
/// any stack mutations in the engine.
pub fn unary(op: &str, x: &Real, bits: u64, rounding: Rounding) -> Result<Real, String> {
    let epsilon = tolerance(bits)?;
    // Exact identities avoid creating rounding noise where the result is known.
    if x == &r(0) {
        match op {
            "sin" | "tan" | "rad" | "deg" => return Ok(r(0)),
            "cos" | "exp" => return Ok(r(1)),
            _ => {}
        }
    }
    if x == &r(1) && matches!(op, "ln" | "log10" | "log2") {
        return Ok(r(0));
    }
    let raw = match op {
        "exp" => exp_raw(x, &epsilon, bits)?,
        "ln" => ln_raw(x, &epsilon)?,
        "log2" => ln_raw(x, &epsilon)? / ln_raw(&r(2), &epsilon)?,
        "log10" => ln_raw(x, &epsilon)? / ln_raw(&r(10), &epsilon)?,
        "sin" | "cos" | "tan" => {
            let pi = pi_raw(&epsilon)?;
            let (sin, cos) = sin_cos_raw(x, &pi, &epsilon)?;
            match op {
                "sin" => sin,
                "cos" => cos,
                _ => {
                    if abs(&cos) < (epsilon.clone() * r(4)) {
                        return Err(
                            "tan is undefined near this angle at the selected precision".to_owned()
                        );
                    }
                    sin / cos
                }
            }
        }
        "rad" => x * pi_raw(&epsilon)? / r(180),
        "deg" => x * r(180) / pi_raw(&epsilon)?,
        _ => return Err(format!("unknown function '{op}'")),
    };
    output(raw, bits, rounding)
}

#[cfg(test)]
mod tests {
    use super::{constant, unary};
    use crate::precision::Rounding;
    use malachite::Rational;

    fn r(s: &str) -> Rational {
        s.parse().unwrap()
    }
    fn approximate(a: &Rational, b: &Rational, tolerance: &Rational) {
        let diff = if a >= b { a - b } else { b - a };
        assert!(diff <= *tolerance, "actual={a} expected={b} diff={diff}");
    }

    #[test]
    fn identities_are_exact() {
        let rounding = Rounding::NearestEven;
        for (op, input, expected) in [
            ("exp", "0", "1"),
            ("sin", "0", "0"),
            ("cos", "0", "1"),
            ("tan", "0", "0"),
            ("ln", "1", "0"),
            ("log10", "1", "0"),
            ("log2", "1", "0"),
            ("rad", "0", "0"),
            ("deg", "0", "0"),
        ] {
            assert_eq!(unary(op, &r(input), 53, rounding).unwrap(), r(expected));
        }
    }

    #[test]
    fn constants_and_conversions() {
        let mode = Rounding::NearestEven;
        let pi = constant("pi", 90, mode).unwrap();
        let reference =
            crate::number::parse_literal("3.1415926535897932384626433832795028841971").unwrap();
        approximate(&pi, &reference, &(r("1") / r("1000000000000000000")));
        let converted = unary("rad", &r("180"), 90, mode).unwrap();
        approximate(&converted, &pi, &(r("1") / r("1000000000")));
        let degrees = unary("deg", &pi, 90, mode).unwrap();
        approximate(&degrees, &r("180"), &(r("1") / r("1000000000")));
    }

    #[test]
    fn logarithms_and_trigonometry() {
        let mode = Rounding::NearestEven;
        let ln_e = unary("ln", &constant("e", 90, mode).unwrap(), 90, mode).unwrap();
        approximate(&ln_e, &r("1"), &(r("1") / r("1000000000")));
        approximate(
            &unary("log2", &r("8"), 90, mode).unwrap(),
            &r("3"),
            &(r("1") / r("1000000000")),
        );
        approximate(
            &unary("log10", &r("100"), 90, mode).unwrap(),
            &r("2"),
            &(r("1") / r("1000000000")),
        );
        let half_pi = constant("pi", 90, mode).unwrap() / r("2");
        approximate(
            &unary("sin", &half_pi, 90, mode).unwrap(),
            &r("1"),
            &(r("1") / r("1000000000")),
        );
        approximate(
            &unary("cos", &half_pi, 90, mode).unwrap(),
            &r("0"),
            &(r("1") / r("1000000000")),
        );
    }

    #[test]
    fn invalid_domains_are_errors() {
        let mode = Rounding::NearestEven;
        assert!(unary("ln", &r("0"), 53, mode).is_err());
        assert!(unary("log10", &r("-5"), 53, mode).is_err());
        assert!(unary("exp", &r("1000"), 53, mode).is_err());
        assert!(unary("sin", &r("1001"), 53, mode).is_err());
        assert!(constant("pi", 513, mode).is_err());
    }
}
