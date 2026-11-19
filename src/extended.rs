//! Exact integer tools, counted statistics and combinatorics.
use crate::number::Number;
pub const WORDS: &str = "band bor bxor bnot shl shr asuint asint u8 i8 u16 i16 u32 i32 u64 i64 u128 i128 sum mean median min max var svar stdev sstdev choose perm binomp";
use malachite::base::num::conversion::traits::{FromStringBase, ToStringBase};
use malachite::{Integer, Natural};

fn integer(x: &Number) -> Result<Integer, String> {
    Integer::try_from(x.clone()).map_err(|_| "integer operand required".to_owned())
}

pub fn parse_base(text: &str) -> Option<Number> {
    let (negative, text) = if let Some(rest) = text.strip_prefix('-') {
        (true, rest)
    } else {
        (false, text.strip_prefix('+').unwrap_or(text))
    };
    let (base, digits) = match text.get(..2)? {
        "0b" | "0B" => (2, &text[2..]),
        "0o" | "0O" => (8, &text[2..]),
        "0x" | "0X" => (16, &text[2..]),
        _ => return None,
    };
    if digits.is_empty() || digits.len() > 100_000 {
        return None;
    }
    if !digits.chars().all(|c| c.is_digit(base)) {
        return None;
    }
    let mut value = Integer::from_string_base(base as u8, digits)?;
    if negative {
        value = -value;
    }
    Some(Number::from(value))
}

pub fn format_base(value: &Number, base: u32) -> Option<String> {
    if !matches!(base, 2 | 8 | 16) {
        return None;
    }
    let mut value = integer(value).ok()?;
    let negative = value < 0;
    if negative {
        value = -value;
    }
    let digits = value.to_string_base(base as u8);
    let prefix = match base {
        2 => "0b",
        8 => "0o",
        16 => "0x",
        _ => return None,
    };
    Some(format!(
        "{}{prefix}{}",
        if negative { "-" } else { "" },
        digits
    ))
}

pub fn binary(op: &str, y: &Number, x: &Number) -> Result<Number, String> {
    let a = integer(y)?;
    let b = integer(x)?;
    let result = match op {
        "band" => a & b,
        "bor" => a | b,
        "bxor" => a ^ b,
        "shl" | "shr" => {
            let shift = b
                .to_string()
                .parse::<u64>()
                .map_err(|_| "shift count must be nonnegative")?;
            if shift > 100_000 {
                return Err("shift count exceeds 100000".to_owned());
            }
            if op == "shl" { a << shift } else { a >> shift }
        }
        "asuint" | "asint" => {
            let bits = b
                .to_string()
                .parse::<u64>()
                .map_err(|_| "width must be 1..4096")?;
            return fixed(y, bits, op == "asint");
        }
        "choose" | "perm" => {
            let n = a
                .to_string()
                .parse::<u64>()
                .map_err(|_| "n must be a nonnegative integer")?;
            let k = b
                .to_string()
                .parse::<u64>()
                .map_err(|_| "k must be a nonnegative integer")?;
            if n > 10_000 || k > n {
                return Err("require 0 <= k <= n <= 10000".to_owned());
            }
            let count = if op == "choose" { k.min(n - k) } else { k };
            let mut result = Natural::from(1u8);
            for i in 0..count {
                result *= Natural::from(n - i);
                if op == "choose" {
                    result /= Natural::from(i + 1);
                }
            }
            return Ok(Number::from(result));
        }
        _ => return Err("unknown integer operation".to_owned()),
    };
    Ok(Number::from(result))
}

pub fn fixed(value: &Number, bits: u64, signed: bool) -> Result<Number, String> {
    if !(1..=4096).contains(&bits) {
        return Err("width must be 1..4096".to_owned());
    }
    let modulus = Integer::from(Natural::from(1u8) << bits);
    let mut result = integer(value)? % &modulus;
    if result < 0 {
        result += &modulus;
    }
    if signed && result >= (&modulus >> 1u64) {
        result -= modulus;
    }
    Ok(Number::from(result))
}

pub fn unary(op: &str, value: &Number) -> Result<Number, String> {
    if op == "bnot" {
        return Ok(Number::from(!integer(value)?));
    }
    let signed = op.starts_with('i');
    let bits = op[1..]
        .parse::<u64>()
        .map_err(|_| "invalid fixed-width operation")?;
    fixed(value, bits, signed)
}

pub fn binomial(n: &Number, k: &Number, probability: &Number) -> Result<Number, String> {
    if probability < &Number::from(0) || probability > &Number::from(1) {
        return Err("probability must be between 0 and 1".to_owned());
    }
    let combinations = binary("choose", n, k)?;
    let exponent = n - k;
    let power = |value: &Number, exponent: &Number| {
        if exponent == &Number::from(0) {
            Ok(Number::from(1))
        } else {
            crate::math::binary("pow", value, exponent).map_err(|error| error.to_string())
        }
    };
    Ok(combinations * power(probability, k)? * power(&(Number::from(1) - probability), &exponent)?)
}

pub fn aggregate(op: &str, values: &[Number]) -> Result<Number, String> {
    if values.is_empty() {
        return if op == "sum" {
            Ok(Number::from(0))
        } else {
            Err("statistic requires at least one value".to_owned())
        };
    }
    let count = Number::from(values.len());
    let sum = values
        .iter()
        .fold(Number::from(0), |total, value| total + value);
    let mean = &sum / &count;
    match op {
        "sum" => Ok(sum),
        "mean" => Ok(mean),
        "median" | "min" | "max" => {
            let mut sorted = values.to_vec();
            sorted.sort();
            Ok(match op {
                "min" => sorted[0].clone(),
                "max" => sorted[sorted.len() - 1].clone(),
                _ if sorted.len().is_multiple_of(2) => {
                    (&sorted[sorted.len() / 2 - 1] + &sorted[sorted.len() / 2]) / Number::from(2)
                }
                _ => sorted[sorted.len() / 2].clone(),
            })
        }
        "var" | "svar" | "stdev" | "sstdev" => {
            let sample = matches!(op, "svar" | "sstdev");
            if sample && values.len() < 2 {
                return Err("sample statistic requires two values".to_owned());
            }
            let squares = values.iter().fold(Number::from(0), |total, value| {
                let delta = value - &mean;
                total + &delta * &delta
            });
            let variance = squares / Number::from(values.len() - usize::from(sample));
            if matches!(op, "stdev" | "sstdev") {
                crate::math::unary("sqrt", &variance).map_err(|e| e.to_string())
            } else {
                Ok(variance)
            }
        }
        _ => Err("unknown statistic".to_owned()),
    }
}
