//! Presentation modes never modify the exact stored number.

use crate::number::{Number, format_number};
use crate::precision::Rounding;
use crate::scientific::format_scientific;
use malachite::Natural;
use malachite::base::num::arithmetic::traits::DivRem;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum DisplayMode {
    #[default]
    Auto,
    Scientific,
    Fraction,
    Binary,
    Octal,
    Hex,
    Fixed(u8),
}

impl DisplayMode {
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "auto" => Some(Self::Auto),
            "scientific" => Some(Self::Scientific),
            "fraction" => Some(Self::Fraction),
            "binary" => Some(Self::Binary),
            "octal" => Some(Self::Octal),
            "hex" => Some(Self::Hex),
            _ => text
                .strip_prefix("fixed:")?
                .parse::<u8>()
                .ok()
                .filter(|n| *n <= 128)
                .map(Self::Fixed),
        }
    }
    pub fn name(self) -> String {
        match self {
            Self::Auto => "auto".to_owned(),
            Self::Scientific => "scientific".to_owned(),
            Self::Fraction => "fraction".to_owned(),
            Self::Binary => "binary".to_owned(),
            Self::Octal => "octal".to_owned(),
            Self::Hex => "hex".to_owned(),
            Self::Fixed(places) => format!("fixed:{places}"),
        }
    }
    pub fn format(self, value: &Number, rounding: Rounding) -> String {
        match self {
            Self::Auto => format_number(value),
            Self::Scientific => format_scientific(value),
            Self::Fraction => value.to_string(),
            Self::Binary => {
                crate::extended::format_base(value, 2).unwrap_or_else(|| format_number(value))
            }
            Self::Octal => {
                crate::extended::format_base(value, 8).unwrap_or_else(|| format_number(value))
            }
            Self::Hex => {
                crate::extended::format_base(value, 16).unwrap_or_else(|| format_number(value))
            }
            Self::Fixed(places) => fixed(value, places, rounding),
        }
    }
}

fn fixed(value: &Number, places: u8, rounding: Rounding) -> String {
    let mut scaled = value.numerator_ref().clone();
    for _ in 0..places {
        scaled *= Natural::from(10u8);
    }
    let denominator = value.denominator_ref().clone();
    let (mut quotient, rem) = scaled.div_rem(denominator.clone());
    let inexact = rem != 0;
    let negative = value < &Number::from(0u8);
    let increment = match rounding {
        Rounding::TowardZero => false,
        Rounding::AwayFromZero => inexact,
        Rounding::Up => inexact && !negative,
        Rounding::Down => inexact && negative,
        Rounding::NearestEven => {
            let twice = &rem + &rem;
            twice > denominator || (twice == denominator && &quotient % Natural::from(2u8) != 0)
        }
    };
    if increment {
        quotient += Natural::from(1u8);
    }
    let sign = if negative && quotient != 0 { "-" } else { "" };
    let mut digits = quotient.to_string();
    let places = usize::from(places);
    if places > 0 {
        if digits.len() <= places {
            digits = format!("{}{}", "0".repeat(places + 1 - digits.len()), digits);
        }
        digits.insert(digits.len() - places, '.');
    }
    format!("{}{sign}{digits}", if inexact { "~" } else { "" })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::number::parse_literal;
    #[test]
    fn fixed_display_rounds_exactly_marks_approximation_and_preserves_value() {
        for (input, mode, expected) in [
            ("1/3", Rounding::NearestEven, "~0.33"),
            ("-1/3", Rounding::Down, "~-0.34"),
            ("-1/3", Rounding::Up, "~-0.33"),
            ("1/8", Rounding::NearestEven, "~0.12"),
            ("3/8", Rounding::NearestEven, "~0.38"),
            ("1/2", Rounding::NearestEven, "0.50"),
        ] {
            let value = parse_literal(input).unwrap();
            assert_eq!(DisplayMode::Fixed(2).format(&value, mode), expected);
            assert_eq!(value, parse_literal(input).unwrap());
        }
    }
}
