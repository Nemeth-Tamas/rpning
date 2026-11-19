//! Explicit, correctly rounded binary approximation of exact rational values.
//!
//! All intermediate calculations use Malachite integers and rationals. No
//! f32/f64 conversion occurs. The output is a dyadic rational carrying the
//! precise value of the selected finite-precision binary result.

use malachite::base::num::arithmetic::traits::{DivRem, FloorLogBasePowerOf2};
use malachite::{Natural, Rational};

pub const MIN_BITS: u64 = 2;
pub const MAX_BITS: u64 = 4096;
const MAX_SHIFT: u64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Rounding {
    #[default]
    NearestEven,
    TowardZero,
    Up,
    Down,
    AwayFromZero,
}

impl Rounding {
    pub fn parse(input: &str) -> Option<Self> {
        match input {
            "nearest" => Some(Self::NearestEven),
            "zero" => Some(Self::TowardZero),
            "up" => Some(Self::Up),
            "down" => Some(Self::Down),
            "away" => Some(Self::AwayFromZero),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::NearestEven => "nearest",
            Self::TowardZero => "zero",
            Self::Up => "up",
            Self::Down => "down",
            Self::AwayFromZero => "away",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NumericSettings {
    pub binary_mode: bool,
    pub bits: u64,
    pub rounding: Rounding,
    pub scientific_display: bool,
}

impl Default for NumericSettings {
    fn default() -> Self {
        Self {
            binary_mode: false,
            bits: 53,
            rounding: Rounding::NearestEven,
            scientific_display: false,
        }
    }
}

/// Exactly round a nonzero rational to the nearest representable binary number
/// with the chosen count of significant bits and unlimited exponent range.
///
/// Returns an error if requested precision or intermediate shifts are too
/// large. The mode uses nearest-even ties, not nearest-away.
pub fn round_binary(value: &Rational, bits: u64, mode: Rounding) -> Result<Rational, &'static str> {
    if !(MIN_BITS..=MAX_BITS).contains(&bits) {
        return Err("binary precision must be between 2 and 4096 bits");
    }
    if value == &Rational::from(0) {
        return Ok(Rational::from(0));
    }

    let negative = value < &Rational::from(0);
    let positive = if negative {
        -value.clone()
    } else {
        value.clone()
    };
    let exponent = (&positive).floor_log_base_power_of_2(1);
    let shift = i64::try_from(bits - 1)
        .map_err(|_| "precision is too large")?
        .checked_sub(exponent)
        .ok_or("binary exponent is out of range")?;
    let shift_magnitude = shift.unsigned_abs();
    if shift_magnitude > MAX_SHIFT {
        return Err("binary exponent shift exceeds the safety limit");
    }

    let mut numerator = positive.numerator_ref().clone();
    let mut denominator = positive.denominator_ref().clone();
    if shift >= 0 {
        numerator <<= shift_magnitude;
    } else {
        denominator <<= shift_magnitude;
    }
    let (mut quotient, remainder) = numerator.div_rem(denominator.clone());
    let rem_nonzero = remainder != 0;
    let round_up = match mode {
        Rounding::TowardZero => false,
        Rounding::AwayFromZero => rem_nonzero,
        Rounding::Up => !negative && rem_nonzero,
        Rounding::Down => negative && rem_nonzero,
        Rounding::NearestEven => {
            let twice = &remainder + &remainder;
            twice > denominator || (twice == denominator && (&quotient % Natural::from(2u8)) != 0)
        }
    };
    if round_up {
        quotient += Natural::from(1u8);
    }

    let output = if shift >= 0 {
        Rational::from(quotient) / Rational::from(Natural::from(1u8) << shift_magnitude)
    } else {
        Rational::from(quotient << shift_magnitude)
    };
    Ok(if negative { -output } else { output })
}

#[cfg(test)]
mod tests {
    use super::{Rounding, round_binary};
    use malachite::Rational;

    fn r(s: &str) -> Rational {
        s.parse().unwrap()
    }

    #[test]
    fn rounding_is_exact_and_configurable() {
        assert_eq!(
            round_binary(&r("1/3"), 4, Rounding::NearestEven).unwrap(),
            r("11/32")
        );
        assert_eq!(
            round_binary(&r("1/3"), 4, Rounding::TowardZero).unwrap(),
            r("5/16")
        );
        assert_eq!(
            round_binary(&r("1/3"), 4, Rounding::Up).unwrap(),
            r("11/32")
        );
        assert_eq!(
            round_binary(&r("-1/3"), 4, Rounding::Down).unwrap(),
            r("-11/32")
        );
        assert_eq!(
            round_binary(&r("-1/3"), 4, Rounding::Up).unwrap(),
            r("-5/16")
        );
    }

    #[test]
    fn ties_are_even_including_negative_values() {
        assert_eq!(
            round_binary(&r("5/4"), 2, Rounding::NearestEven).unwrap(),
            r("1")
        );
        assert_eq!(
            round_binary(&r("7/4"), 2, Rounding::NearestEven).unwrap(),
            r("2")
        );
        assert_eq!(
            round_binary(&r("-7/4"), 2, Rounding::NearestEven).unwrap(),
            r("-2")
        );
    }

    #[test]
    fn exact_values_survive_approximation() {
        for input in ["0", "1", "-1", "3/2", "1024", "1/256"] {
            let value = r(input);
            assert_eq!(
                round_binary(&value, 53, Rounding::NearestEven).unwrap(),
                value
            );
        }
    }

    #[test]
    fn invalid_precision_is_rejected() {
        assert!(round_binary(&r("1"), 1, Rounding::NearestEven).is_err());
        assert!(round_binary(&r("1"), 4097, Rounding::NearestEven).is_err());
    }
}
